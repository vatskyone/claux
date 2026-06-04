import SwiftUI
import Combine

/// Central state object — drives the popover and analytics UI.
/// Owns a `SessionMonitor` that reads live data from `~/.claude/projects/`.
final class AppStore: ObservableObject {

    // MARK: – Published data
    @Published var activeSession:    ClaudeSession?   = nil
    @Published var recentSessions:   [ClaudeSession]  = []
    @Published var spendSummary:     SpendSummary     = .zero
    @Published var dailySpend:       [DailySpend]     = []
    @Published var projectBreakdown: [ProjectSpend]   = []
    @Published var modelBreakdown:   [ModelSpend]     = []
    @Published var planLimits:       PlanLimitsSnapshot = .empty
    @Published var planLimitsDiagnostics: PlanLimitsDiagnostics = .defaultState

    // MARK: – Private
    private let monitor          = SessionMonitor()
    private let rateLimitMonitor = RateLimitMonitor()
    private var cancellables = Set<AnyCancellable>()

    // MARK: – Init

    init() {
        monitor.$sessions
            .receive(on: RunLoop.main)
            .sink { [weak self] sessions in
                self?.updateUI(from: sessions)
            }
            .store(in: &cancellables)

        rateLimitMonitor.$snapshot
            .receive(on: RunLoop.main)
            .sink { [weak self] snapshot in
                self?.planLimits = snapshot
            }
            .store(in: &cancellables)

        rateLimitMonitor.$diagnostics
            .receive(on: RunLoop.main)
            .sink { [weak self] diagnostics in
                self?.planLimitsDiagnostics = diagnostics
            }
            .store(in: &cancellables)

        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in
            guard let self else { return }
            NotificationManager.shared.observe(store: self)
        }
    }

    // MARK: – Actions

    /// Force an immediate re-scan of the monitored directory.
    func refreshNow() {
        monitor.refresh()
        rateLimitMonitor.refresh()
    }

    /// Wipe all app-level UserDefaults keys and force a fresh scan.
    func resetAllData() {
        let keys: [String] = [
            "costAlertThreshold", "contextHealthAlert", "showCostInMenuBar",
            "sessionRetentionDays", "enableNotifications", "alertOnSessionEnd",
            "showModelInMenuBar", "autoRefreshInterval", "includeCacheCost",
            "appTheme", "costProjectionPeriod",
            "claudemdAlertEnabled", "claudemdThreshold",
            "monthlyBudget", "dailySummaryEnabled", "dailySummaryHour",
            "weeklyRecapEnabled", "weeklyRecapWeekday", "summaryWeekdaysOnly",
            "notificationVerbosity", "notificationsQuietHoursEnabled",
            "notificationsQuietHoursStart", "notificationsQuietHoursEnd",
            "notificationSnoozedDayKey", "weeklyRecapLastSentWeekKey",
        ]
        keys.forEach { UserDefaults.standard.removeObject(forKey: $0) }
        monitor.invalidateCache()
        monitor.refresh()
        rateLimitMonitor.refresh()
    }

    func dailyRecap(for day: Date = Date()) -> DailyRecap? {
        let calendar = Calendar.current
        let targetDay = calendar.startOfDay(for: day)
        let sessions = trackedSessions().compactMap { session -> DailyRecapSession? in
            let dayCost = session.dailyCosts[targetDay] ?? 0
            let startedToday = calendar.isDate(session.startTime, inSameDayAs: targetDay)
            guard dayCost > 0 || startedToday else { return nil }

            return makeRecapSession(from: session, cost: dayCost)
        }
        .sorted { lhs, rhs in
            if lhs.dayCost != rhs.dayCost { return lhs.dayCost > rhs.dayCost }
            return lhs.qualityScore > rhs.qualityScore
        }

        guard !sessions.isEmpty else { return nil }

        let allTracked = trackedSessions()
        let sessionsByID = Dictionary(uniqueKeysWithValues: allTracked.map { ($0.id, $0) })

        var totalCost = 0.0
        var totalAcceptedEdits = 0
        var totalRejectedActions = 0
        var touchedFiles = Set<String>()
        var byProject: [String: Double] = [:]
        var byModel: [String: Double] = [:]

        for recapSession in sessions {
            totalCost += recapSession.dayCost
            totalAcceptedEdits += recapSession.acceptedEdits
            totalRejectedActions += recapSession.rejectedActions

            byProject[recapSession.projectDisplayPath, default: 0] += recapSession.dayCost
            byModel[recapSession.modelDisplayName, default: 0] += recapSession.dayCost

            if let sourceSession = sessionsByID[recapSession.id] {
                touchedFiles.formUnion(sourceSession.qualityMetrics.touchedFiles)
            }
        }

        let topProject = byProject.max { lhs, rhs in lhs.value < rhs.value }
        let topModel = byModel.max { lhs, rhs in lhs.value < rhs.value }
        let bestSession = sessions.max { lhs, rhs in
            if lhs.qualityScore != rhs.qualityScore { return lhs.qualityScore < rhs.qualityScore }
            return lhs.dayCost < rhs.dayCost
        }

        return DailyRecap(
            day: targetDay,
            totalCost: totalCost,
            sessionCount: sessions.count,
            totalAcceptedEdits: totalAcceptedEdits,
            totalRejectedActions: totalRejectedActions,
            totalTouchedFileCount: touchedFiles.count,
            topProjectDisplayPath: topProject?.key,
            topProjectCost: topProject?.value ?? 0,
            topModelDisplayName: topModel?.key,
            topModelCost: topModel?.value ?? 0,
            bestSession: bestSession,
            mostExpensiveSession: sessions.first,
            sessions: sessions
        )
    }

    func weeklyRecap(excluding day: Date = Date()) -> WeeklyRecap? {
        let calendar = Calendar.current
        let cutoffDay = calendar.startOfDay(for: day)
        let endDay = calendar.date(byAdding: .day, value: -1, to: cutoffDay) ?? cutoffDay
        let startDay = calendar.date(byAdding: .day, value: -6, to: endDay) ?? endDay

        let sessions = trackedSessions().compactMap { session -> DailyRecapSession? in
            let periodCost = session.dailyCosts.reduce(into: 0.0) { result, entry in
                if entry.key >= startDay && entry.key <= endDay {
                    result += entry.value
                }
            }
            let startedInRange = session.startTime >= startDay && session.startTime < cutoffDay
            guard periodCost > 0 || startedInRange else { return nil }
            return makeRecapSession(from: session, cost: periodCost)
        }
        .sorted { lhs, rhs in
            if lhs.dayCost != rhs.dayCost { return lhs.dayCost > rhs.dayCost }
            return lhs.qualityScore > rhs.qualityScore
        }

        guard !sessions.isEmpty else { return nil }

        let allTracked = trackedSessions()
        let sessionsByID = Dictionary(uniqueKeysWithValues: allTracked.map { ($0.id, $0) })

        var totalCost = 0.0
        var totalAcceptedEdits = 0
        var totalRejectedActions = 0
        var touchedFiles = Set<String>()
        var byProject: [String: Double] = [:]
        var byModel: [String: Double] = [:]

        for recapSession in sessions {
            totalCost += recapSession.dayCost
            totalAcceptedEdits += recapSession.acceptedEdits
            totalRejectedActions += recapSession.rejectedActions

            byProject[recapSession.projectDisplayPath, default: 0] += recapSession.dayCost
            byModel[recapSession.modelDisplayName, default: 0] += recapSession.dayCost

            if let sourceSession = sessionsByID[recapSession.id] {
                touchedFiles.formUnion(sourceSession.qualityMetrics.touchedFiles)
            }
        }

        let topProject = byProject.max { lhs, rhs in lhs.value < rhs.value }
        let topModel = byModel.max { lhs, rhs in lhs.value < rhs.value }
        let bestSession = sessions.max { lhs, rhs in
            if lhs.qualityScore != rhs.qualityScore { return lhs.qualityScore < rhs.qualityScore }
            return lhs.dayCost < rhs.dayCost
        }

        return WeeklyRecap(
            startDay: startDay,
            endDay: endDay,
            totalCost: totalCost,
            sessionCount: sessions.count,
            totalAcceptedEdits: totalAcceptedEdits,
            totalRejectedActions: totalRejectedActions,
            totalTouchedFileCount: touchedFiles.count,
            topProjectDisplayPath: topProject?.key,
            topProjectCost: topProject?.value ?? 0,
            topModelDisplayName: topModel?.key,
            topModelCost: topModel?.value ?? 0,
            bestSession: bestSession,
            mostExpensiveSession: sessions.first,
            sessions: sessions
        )
    }

    func dailyRecap(forDayKey dayKey: String) -> DailyRecap? {
        guard let date = Format.date(fromDayKey: dayKey) else { return nil }
        return dailyRecap(for: date)
    }

    func session(idString: String) -> ClaudeSession? {
        guard let id = UUID(uuidString: idString) else { return nil }
        return trackedSessions().first(where: { $0.id == id })
    }

    // MARK: – Private helpers

    private func updateUI(from sessions: [ClaudeSession]) {
        let retentionDays = UserDefaults.standard.integer(forKey: "sessionRetentionDays")
        let cutoff: Date = {
            let days = retentionDays > 0 ? retentionDays : 30
            return Calendar.current.date(byAdding: .day, value: -days, to: Date()) ?? .distantPast
        }()

        let visible = sessions.filter { $0.startTime >= cutoff || $0.isActive }

        activeSession  = visible.first(where: \.isActive)
        recentSessions = visible.filter { !$0.isActive }.prefix(50).map { $0 }
        spendSummary   = computeSpend(from: visible)
        dailySpend     = computeDailySpend(from: visible)
        projectBreakdown = computeProjectBreakdown(from: visible)
        modelBreakdown   = computeModelBreakdown(from: visible)
    }

    private func computeSpend(from sessions: [ClaudeSession]) -> SpendSummary {
        let calendar     = Calendar.current
        let startOfToday = calendar.startOfDay(for: Date())
        // All boundaries are at midnight local time so comparisons against
        // dailyCosts keys (which are also midnight-local) are exact.
        let startOfYesterday = calendar.date(byAdding: .day, value: -1,  to: startOfToday) ?? startOfToday
        let startOfWeek      = calendar.date(byAdding: .day, value: -7,  to: startOfToday) ?? startOfToday
        let startPrevWeek    = calendar.date(byAdding: .day, value: -14, to: startOfToday) ?? startOfToday
        let startOfMonth     = calendar.date(byAdding: .day, value: -30, to: startOfToday) ?? startOfToday
        let startPrevMonth   = calendar.date(byAdding: .day, value: -60, to: startOfToday) ?? startOfToday

        var today = 0.0, week = 0.0, month = 0.0
        var yesterday = 0.0, prevWeek = 0.0, prevMonth = 0.0

        for s in sessions {
            // Attribute cost turn-by-turn so multi-day sessions are split correctly
            // across the today / week / month buckets.
            for (day, cost) in s.dailyCosts {
                if day >= startOfToday                                { today     += cost }
                if day >= startOfWeek                                 { week      += cost }
                if day >= startOfMonth                                { month     += cost }
                if day >= startOfYesterday && day < startOfToday      { yesterday += cost }
                if day >= startPrevWeek    && day < startOfWeek       { prevWeek  += cost }
                if day >= startPrevMonth   && day < startOfMonth      { prevMonth += cost }
            }
        }
        return SpendSummary(today: today, thisWeek: week, thisMonth: month,
                            yesterday: yesterday, prevWeek: prevWeek, prevMonth: prevMonth)
    }

    private func computeDailySpend(from sessions: [ClaudeSession]) -> [DailySpend] {
        let calendar = Calendar.current
        let cutoff   = calendar.date(byAdding: .day, value: -30, to: Date()) ?? .distantPast

        var byDay: [Date: Double] = [:]
        for s in sessions {
            // Use per-turn day attribution so cross-midnight sessions show up
            // correctly in the analytics chart.
            for (day, cost) in s.dailyCosts where day >= cutoff {
                byDay[day, default: 0] += cost
            }
        }

        // Fill every calendar day in the range, including zeros
        var result: [DailySpend] = []
        var cursor = calendar.startOfDay(for: cutoff)
        let today  = calendar.startOfDay(for: Date())
        while cursor <= today {
            result.append(DailySpend(date: cursor, cost: byDay[cursor] ?? 0))
            cursor = calendar.date(byAdding: .day, value: 1, to: cursor) ?? cursor.addingTimeInterval(86_400)
        }
        return result
    }

    private func computeProjectBreakdown(from sessions: [ClaudeSession]) -> [ProjectSpend] {
        var byPath: [String: (cost: Double, count: Int)] = [:]
        for s in sessions {
            let existing = byPath[s.projectPath] ?? (cost: 0, count: 0)
            byPath[s.projectPath] = (cost: existing.cost + s.totalCost, count: existing.count + 1)
        }
        return byPath.map { path, data in
            ProjectSpend(
                path: path,
                displayPath: Format.projectPath(path),
                totalCost: data.cost,
                sessionCount: data.count
            )
        }
        .sorted { $0.totalCost > $1.totalCost }
    }

    private func computeModelBreakdown(from sessions: [ClaudeSession]) -> [ModelSpend] {
        var byModel: [String: (cost: Double, count: Int)] = [:]
        for s in sessions {
            let existing = byModel[s.model] ?? (cost: 0, count: 0)
            byModel[s.model] = (cost: existing.cost + s.totalCost, count: existing.count + 1)
        }
        return byModel.map { model, data in
            ModelSpend(
                model: model,
                displayName: ModelInfo.shortName(model),
                totalCost: data.cost,
                sessionCount: data.count
            )
        }
        .sorted { $0.totalCost > $1.totalCost }
    }

    private func makeRecapSession(from session: ClaudeSession, cost: Double) -> DailyRecapSession {
        let title = session.title?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false
            ? session.title!
            : session.displayPath

        return DailyRecapSession(
            id: session.id,
            title: title,
            subtitle: session.displayPath,
            modelDisplayName: ModelInfo.shortName(session.model),
            projectDisplayPath: session.displayPath,
            dayCost: cost,
            duration: session.duration,
            qualityScore: session.qualityMetrics.score,
            qualityLabel: session.qualityMetrics.scoreLabel,
            acceptedEdits: session.qualityMetrics.acceptedEdits,
            rejectedActions: session.qualityMetrics.rejectedToolResults,
            touchedFileCount: session.qualityMetrics.touchedFileCount
        )
    }

    private func trackedSessions() -> [ClaudeSession] {
        var sessions = recentSessions
        if let activeSession {
            sessions.append(activeSession)
        }
        return sessions
    }
}

// MARK: – SpendSummary zero value
extension SpendSummary {
    static let zero = SpendSummary(today: 0, thisWeek: 0, thisMonth: 0)
}
