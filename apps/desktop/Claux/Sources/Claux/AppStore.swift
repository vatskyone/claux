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
        ]
        keys.forEach { UserDefaults.standard.removeObject(forKey: $0) }
        monitor.invalidateCache()
        monitor.refresh()
        rateLimitMonitor.refresh()
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
}

// MARK: – SpendSummary zero value
extension SpendSummary {
    static let zero = SpendSummary(today: 0, thisWeek: 0, thisMonth: 0)
}
