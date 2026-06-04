import UserNotifications
import Combine
import AppKit

enum NotificationVerbosity: String, CaseIterable, Identifiable {
    case minimal
    case standard
    case detailed

    var id: String { rawValue }

    var label: String {
        switch self {
        case .minimal: return "Minimal"
        case .standard: return "Standard"
        case .detailed: return "Detailed"
        }
    }
}

enum SummaryWeekday: Int, CaseIterable, Identifiable {
    case monday = 2
    case tuesday = 3
    case wednesday = 4
    case thursday = 5
    case friday = 6

    var id: Int { rawValue }

    var label: String {
        switch self {
        case .monday: return "Monday"
        case .tuesday: return "Tuesday"
        case .wednesday: return "Wednesday"
        case .thursday: return "Thursday"
        case .friday: return "Friday"
        }
    }
}

extension Notification.Name {
    static let clauxOpenDailyRecap = Notification.Name("clauxOpenDailyRecap")
    static let clauxOpenSession = Notification.Name("clauxOpenSession")
    static let clauxOpenDashboard = Notification.Name("clauxOpenDashboard")
}

// MARK: – NotificationManager
// Owns all UNUserNotificationCenter interactions for Claux.
// Observes AppStore and fires alerts when thresholds are crossed.
final class NotificationManager: NSObject, ObservableObject {
    private enum NotificationPayloadKey {
        static let target = "clauxTarget"
        static let dayKey = "dayKey"
        static let sessionID = "sessionID"
    }

    private enum NotificationTarget: String {
        case dailyRecap
        case session
        case dashboard
    }

    private enum NotificationCategoryIdentifier: String {
        case sessionAlert = "claux.session-alert"
        case summaryAlert = "claux.summary-alert"
    }

    private enum NotificationActionIdentifier: String {
        case openSession = "claux.open-session"
        case openDashboard = "claux.open-dashboard"
        case snoozeToday = "claux.snooze-today"
    }

    static let shared = NotificationManager()

    // Published so Settings can observe the current permission state live.
    @Published var authStatus: UNAuthorizationStatus = .notDetermined

    // MARK: – Dedup tracking (per session ID)
    // Prevents the same alert firing twice for the same session crossing the same threshold.
    private var firedCostAlert:   Set<UUID> = []
    private var firedCtxWarning:  Set<UUID> = []
    private var firedCtxCritical: Set<UUID> = []

    // Used to detect the active→inactive transition for "session ended" alerts.
    private var lastKnownActiveIDs: Set<UUID> = []

    private var cancellables = Set<AnyCancellable>()

    // MARK: – Setup

    /// `UserNotifications` requires a proper .app bundle with a bundle identifier.
    /// When running the raw SPM binary the framework crashes; we skip it gracefully.
    var notificationsAvailable: Bool {
        Bundle.main.bundleIdentifier != nil
    }

    private override init() {
        super.init()
        guard notificationsAvailable else { return }
        UNUserNotificationCenter.current().delegate = self
        registerCategories()
        // Populate authStatus on startup
        refreshAuthStatus()
    }

    private func registerCategories() {
        let openSession = UNNotificationAction(
            identifier: NotificationActionIdentifier.openSession.rawValue,
            title: "Open Session"
        )
        let openDashboard = UNNotificationAction(
            identifier: NotificationActionIdentifier.openDashboard.rawValue,
            title: "Open Dashboard"
        )
        let snoozeToday = UNNotificationAction(
            identifier: NotificationActionIdentifier.snoozeToday.rawValue,
            title: "Snooze Today"
        )

        let sessionCategory = UNNotificationCategory(
            identifier: NotificationCategoryIdentifier.sessionAlert.rawValue,
            actions: [openSession, openDashboard, snoozeToday],
            intentIdentifiers: [],
            options: []
        )
        let summaryCategory = UNNotificationCategory(
            identifier: NotificationCategoryIdentifier.summaryAlert.rawValue,
            actions: [openDashboard, snoozeToday],
            intentIdentifiers: [],
            options: []
        )

        UNUserNotificationCenter.current().setNotificationCategories([sessionCategory, summaryCategory])
    }

    /// Refresh the published `authStatus`. Call after any permission change.
    func refreshAuthStatus() {
        guard notificationsAvailable else { return }
        UNUserNotificationCenter.current().getNotificationSettings { [weak self] settings in
            DispatchQueue.main.async {
                self?.authStatus = settings.authorizationStatus
            }
        }
    }

    /// Request notification permission.
    /// - If status is `.notDetermined`, shows the system dialog.
    /// - If status is `.denied`, opens System Settings → Notifications.
    /// - If already `.authorized`, does nothing.
    func requestPermission(openSettingsIfDenied: Bool = false) {
        guard notificationsAvailable else { return }
        UNUserNotificationCenter.current().getNotificationSettings { [weak self] settings in
            guard let self else { return }
            switch settings.authorizationStatus {
            case .notDetermined:
                // Must be on the main thread with the app active; otherwise macOS
                // silently drops the permission dialog without showing anything.
                DispatchQueue.main.async {
                    NSApp.activate(ignoringOtherApps: true)
                    UNUserNotificationCenter.current().requestAuthorization(
                        options: [.alert, .sound, .badge]
                    ) { [weak self] _, _ in
                        self?.refreshAuthStatus()
                    }
                }
            case .denied:
                if openSettingsIfDenied {
                    DispatchQueue.main.async {
                        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.notifications") {
                            NSWorkspace.shared.open(url)
                        }
                    }
                }
            default:
                break
            }
            self.refreshAuthStatus()
        }
    }

    /// Fire a visible test notification immediately. Useful to verify the whole pipeline.
    func sendTestNotification() {
        guard notificationsAvailable else {
            DispatchQueue.main.async {
                let a = NSAlert()
                a.messageText     = "Notifications unavailable"
                a.informativeText = "Run Claux as a proper .app bundle (bash build_app.sh run) to enable notifications."
                a.runModal()
            }
            return
        }

        UNUserNotificationCenter.current().getNotificationSettings { [weak self] settings in
            guard let self else { return }
            DispatchQueue.main.async { self.authStatus = settings.authorizationStatus }

            switch settings.authorizationStatus {
            case .authorized, .provisional:
                let content   = UNMutableNotificationContent()
                content.title = "Claux Notifications Work ✓"
                content.body  = "Cost alerts, context warnings, and session-end alerts are active."
                content.sound = .default
                UNUserNotificationCenter.current().add(
                    UNNotificationRequest(identifier: "test-\(UUID())", content: content, trigger: nil)
                )
            case .notDetermined:
                // Must be on the main thread with the app active; otherwise macOS
                // silently drops the permission dialog without showing anything.
                DispatchQueue.main.async { [weak self] in
                    NSApp.activate(ignoringOtherApps: true)
                    UNUserNotificationCenter.current().requestAuthorization(
                        options: [.alert, .sound, .badge]
                    ) { [weak self] granted, _ in
                        self?.refreshAuthStatus()
                        if granted { self?.sendTestNotification() }
                    }
                }
            case .denied:
                DispatchQueue.main.async {
                    if let url = URL(string: "x-apple.systempreferences:com.apple.preference.notifications") {
                        NSWorkspace.shared.open(url)
                    }
                }
            @unknown default:
                break
            }
        }
    }

    /// Wire the manager to an AppStore.  Subscribes to session updates.
    func observe(store: AppStore) {
        store.$activeSession
            .combineLatest(store.$recentSessions)
            .receive(on: RunLoop.main)
            .sink { [weak self, weak store] active, recent in
                guard let self, let store else { return }
                // Use object(forKey:) so the registered default (true) is respected
                // even if the key was never explicitly written by the user.
                let enabled = (UserDefaults.standard.object(forKey: "enableNotifications") as? Bool) ?? true
                guard enabled else { return }
                self.evaluate(active: active, recent: recent, store: store)
            }
            .store(in: &cancellables)
    }

    // MARK: – Evaluation

    private func evaluate(active: ClaudeSession?,
                          recent: [ClaudeSession],
                          store: AppStore) {
        // ── Active session alerts ───────────────────────────────────────────
        if let session = active {
            checkCostAlert(session)
            checkContextAlert(session)
        }

        // ── Session-ended alert ─────────────────────────────────────────────
        let currentActiveIDs = active.map { Set([$0.id]) } ?? []
        let justEnded = lastKnownActiveIDs.subtracting(currentActiveIDs)

        if (UserDefaults.standard.object(forKey: "alertOnSessionEnd") as? Bool) ?? false {
            for endedID in justEnded {
                if let ended = recent.first(where: { $0.id == endedID }) {
                    let quality = "\(ended.qualityMetrics.score) \(ended.qualityMetrics.scoreLabel.lowercased())"
                    fire(
                        id: "session-ended-\(endedID)",
                        title: "Session Ended",
                        subtitle: "\(Format.cost(ended.totalCost)) · \(Format.duration(ended.duration)) · \(quality)",
                        body: sessionEndedBody(for: ended),
                        symbol: "checkmark.circle",
                        category: .sessionAlert,
                        userInfo: sessionUserInfo(for: ended)
                    )
                }
            }
        }

        lastKnownActiveIDs = currentActiveIDs

        // ── Daily summary ───────────────────────────────────────────────────
        checkDailySummary(store: store)
        checkWeeklyRecap(store: store)

        // ── Reset dedup when a session is no longer tracked ──
        let allIDs = Set(([active].compactMap { $0 } + recent).map { $0.id })
        firedCostAlert   = firedCostAlert.intersection(allIDs)
        firedCtxWarning  = firedCtxWarning.intersection(allIDs)
        firedCtxCritical = firedCtxCritical.intersection(allIDs)
    }

    // MARK: – Daily summary

    /// Fires a "daily summary" notification once per day after the user's configured
    /// hour.  Called on every session update; the guard ensures it fires at most once
    /// per calendar day even if many updates arrive.
    private func checkDailySummary(store: AppStore) {
        let enabled = (UserDefaults.standard.object(forKey: "dailySummaryEnabled") as? Bool) ?? false
        guard enabled, notificationsAvailable else { return }
        guard summarySchedulingAllowedToday() else { return }

        let summaryHour = UserDefaults.standard.integer(forKey: "dailySummaryHour")
        let currentHour = Calendar.current.component(.hour, from: Date())
        guard currentHour >= summaryHour else { return }

        let todayKey = Format.dayKey(Date())

        // Already sent today?
        if UserDefaults.standard.string(forKey: "dailySummaryLastSent") == todayKey { return }

        let recap = store.dailyRecap(for: Date())
        let sessionCount = recap?.sessionCount ?? 0
        let subtitle = sessionCount > 0
            ? "\(Format.cost(recap?.totalCost ?? 0)) across \(sessionCount) session\(sessionCount == 1 ? "" : "s")"
            : nil
        let body: String

        if let recap, recap.hasSessions {
            body = dailySummaryBody(for: recap)
        } else {
            body = "No Claude sessions today"
        }

        fire(id: "daily-summary-\(todayKey)",
             title: "Claux Daily Summary",
             subtitle: subtitle,
             body: body,
             symbol: "chart.bar.fill",
             category: .summaryAlert,
             userInfo: [
                NotificationPayloadKey.target: NotificationTarget.dailyRecap.rawValue,
                NotificationPayloadKey.dayKey: todayKey
             ])

        UserDefaults.standard.set(todayKey, forKey: "dailySummaryLastSent")
    }

    private func checkWeeklyRecap(store: AppStore) {
        let enabled = (UserDefaults.standard.object(forKey: "weeklyRecapEnabled") as? Bool) ?? false
        guard enabled, notificationsAvailable else { return }

        let summaryHour = UserDefaults.standard.integer(forKey: "dailySummaryHour")
        let currentHour = Calendar.current.component(.hour, from: Date())
        guard currentHour >= summaryHour else { return }

        let configuredWeekday = UserDefaults.standard.integer(forKey: "weeklyRecapWeekday")
        let weekday = Calendar.current.component(.weekday, from: Date())
        guard weekday == configuredWeekday else { return }

        let currentWeekKey = Format.weekKey(Date())
        if UserDefaults.standard.string(forKey: "weeklyRecapLastSentWeekKey") == currentWeekKey { return }

        guard let recap = store.weeklyRecap(excluding: Date()), recap.hasSessions else { return }

        let subtitle = "Last 7 days · \(Format.cost(recap.totalCost)) across \(recap.sessionCount) session\(recap.sessionCount == 1 ? "" : "s")"
        fire(
            id: "weekly-recap-\(currentWeekKey)",
            title: "Claux Weekly Recap",
            subtitle: subtitle,
            body: weeklySummaryBody(for: recap),
            symbol: "calendar",
            category: .summaryAlert,
            userInfo: [NotificationPayloadKey.target: NotificationTarget.dashboard.rawValue]
        )

        UserDefaults.standard.set(currentWeekKey, forKey: "weeklyRecapLastSentWeekKey")
    }

    // MARK: – Individual alert checks

    private func checkCostAlert(_ session: ClaudeSession) {
        var threshold = UserDefaults.standard.double(forKey: "costAlertThreshold")
        if threshold <= 0 { threshold = 5.0 }

        guard session.totalCost >= threshold,
              !firedCostAlert.contains(session.id) else { return }

        firedCostAlert.insert(session.id)
        fire(
            id: "cost-\(session.id)",
            title: "Cost Threshold Reached",
            subtitle: "\(Format.cost(session.totalCost)) spent · \(Format.cost(session.burnRatePerHour))/hr",
            body: costAlertBody(for: session),
            symbol: "dollarsign.circle",
            category: .sessionAlert,
            userInfo: sessionUserInfo(for: session)
        )
    }

    private func checkContextAlert(_ session: ClaudeSession) {
        let fraction = session.contextHealthFraction
        var alertPct = UserDefaults.standard.double(forKey: "contextHealthAlert")
        if alertPct <= 0 { alertPct = 80 }
        let warnThreshold = alertPct / 100.0

        // Critical (≥ 90%)
        if fraction >= 0.90, !firedCtxCritical.contains(session.id) {
            firedCtxCritical.insert(session.id)
            fire(
                id: "ctx-critical-\(session.id)",
                title: "Context Window Critical",
                subtitle: "\(ModelInfo.shortName(session.model)) · \(Int(fraction * 100))% full",
                body: contextAlertBody(for: session, critical: true),
                symbol: "exclamationmark.triangle",
                category: .sessionAlert,
                userInfo: sessionUserInfo(for: session)
            )
        }

        // Warning (≥ user threshold, < 90%)
        if fraction >= warnThreshold,
           fraction < 0.90,
           !firedCtxWarning.contains(session.id) {
            firedCtxWarning.insert(session.id)
            fire(
                id: "ctx-warning-\(session.id)",
                title: "Context Window Warning",
                subtitle: "\(ModelInfo.shortName(session.model)) · \(Int(fraction * 100))% full",
                body: contextAlertBody(for: session, critical: false),
                symbol: "exclamationmark.circle",
                category: .sessionAlert,
                userInfo: sessionUserInfo(for: session)
            )
        }
    }

    // MARK: – Delivery

    private func fire(
        id: String,
        title: String,
        subtitle: String? = nil,
        body: String,
        symbol: String,
        category: NotificationCategoryIdentifier,
        userInfo: [AnyHashable: Any] = [:]
    ) {
        guard notificationsAvailable else { return }
        guard !notificationsSnoozedToday() else { return }
        guard !isWithinQuietHours() else { return }
        UNUserNotificationCenter.current().getNotificationSettings { [weak self] settings in
            guard let self else { return }
            DispatchQueue.main.async { self.authStatus = settings.authorizationStatus }

            guard settings.authorizationStatus == .authorized ||
                  settings.authorizationStatus == .provisional
            else { return }

            let content   = UNMutableNotificationContent()
            content.title = title
            content.subtitle = subtitle ?? ""
            content.body  = body
            content.sound = .default
            content.categoryIdentifier = category.rawValue
            content.userInfo = userInfo
            UNUserNotificationCenter.current().add(
                UNNotificationRequest(identifier: id, content: content, trigger: nil)
            )
        }
    }

    private func sessionUserInfo(for session: ClaudeSession) -> [AnyHashable: Any] {
        [
            NotificationPayloadKey.target: NotificationTarget.session.rawValue,
            NotificationPayloadKey.sessionID: session.id.uuidString
        ]
    }

    private func currentVerbosity() -> NotificationVerbosity {
        let rawValue = UserDefaults.standard.string(forKey: "notificationVerbosity") ?? NotificationVerbosity.standard.rawValue
        return NotificationVerbosity(rawValue: rawValue) ?? .standard
    }

    private func sessionEndedBody(for session: ClaudeSession) -> String {
        let errors = session.errorCount
        switch currentVerbosity() {
        case .minimal:
            return "\(session.qualityMetrics.acceptedEdits) accepted edits · \(errors) error\(errors == 1 ? "" : "s")"
        case .standard:
            return "\(session.qualityMetrics.touchedFileCount) files touched · \(session.qualityMetrics.acceptedEdits) accepted edits · \(errors) error\(errors == 1 ? "" : "s")"
        case .detailed:
            return "\(session.displayPath) · \(session.qualityMetrics.touchedFileCount) files touched · \(session.qualityMetrics.acceptedEdits) accepted edits · \(errors) error\(errors == 1 ? "" : "s")"
        }
    }

    private func costAlertBody(for session: ClaudeSession) -> String {
        let projected = Format.cost(session.projectedCost)
        let burnRate = Format.cost(session.burnRatePerHour)
        switch currentVerbosity() {
        case .minimal:
            return "Projected 1h total: \(projected)"
        case .standard:
            return "\(session.displayPath) is burning \(burnRate)/hr · Projected 1h total: \(projected)"
        case .detailed:
            return "\(session.displayPath) · \(ModelInfo.shortName(session.model)) · Burn \(burnRate)/hr · Projected 1h total: \(projected)"
        }
    }

    private func contextAlertBody(for session: ClaudeSession, critical: Bool) -> String {
        let percentage = Int(session.contextHealthFraction * 100)
        let advice = "Consider compacting or starting a new session."
        switch currentVerbosity() {
        case .minimal:
            return advice
        case .standard:
            return "\(session.displayPath) is \(percentage)% full. \(advice)"
        case .detailed:
            let tokenCount = session.tokenUsage.contextWindowTokens > 0
                ? session.tokenUsage.contextWindowTokens
                : session.tokenUsage.totalContextTokens
            let prefix = critical ? "Critical" : "Warning"
            return "\(prefix): \(session.displayPath) is using \(Format.tokens(tokenCount)) context tokens. \(advice)"
        }
    }

    private func dailySummaryBody(for recap: DailyRecap) -> String {
        switch currentVerbosity() {
        case .minimal:
            if let bestSession = recap.bestSession {
                return "Best session: \(bestSession.qualityScore) \(bestSession.qualityLabel.lowercased())"
            }
            return "Open Claux to review today’s sessions."
        case .standard:
            var parts: [String] = []
            if let topProject = recap.topProjectDisplayPath {
                parts.append("Top project: \(topProject)")
            }
            if let bestSession = recap.bestSession {
                parts.append("Best session: \(bestSession.qualityScore) \(bestSession.qualityLabel.lowercased())")
            }
            if recap.totalAcceptedEdits > 0 || recap.totalRejectedActions > 0 {
                parts.append("Edits: \(recap.totalAcceptedEdits) accepted · \(recap.totalRejectedActions) rejected")
            }
            return parts.isEmpty ? "Open Claux to review today’s sessions." : parts.joined(separator: " · ")
        case .detailed:
            var parts: [String] = []
            if let topProject = recap.topProjectDisplayPath {
                parts.append("Top project: \(topProject)")
            }
            if let topModel = recap.topModelDisplayName {
                parts.append("Top model: \(topModel)")
            }
            if let bestSession = recap.bestSession {
                parts.append("Best: \(bestSession.qualityScore) \(bestSession.qualityLabel.lowercased())")
            }
            parts.append("Files touched: \(recap.totalTouchedFileCount)")
            parts.append("Edits: \(recap.totalAcceptedEdits) accepted · \(recap.totalRejectedActions) rejected")
            return parts.joined(separator: " · ")
        }
    }

    private func weeklySummaryBody(for recap: WeeklyRecap) -> String {
        switch currentVerbosity() {
        case .minimal:
            if let topProject = recap.topProjectDisplayPath {
                return "Top project: \(topProject)"
            }
            return "Open Claux to review the last 7 days."
        case .standard:
            var parts: [String] = []
            if let topProject = recap.topProjectDisplayPath {
                parts.append("Top project: \(topProject)")
            }
            if let bestSession = recap.bestSession {
                parts.append("Best session: \(bestSession.qualityScore) \(bestSession.qualityLabel.lowercased())")
            }
            parts.append("Edits: \(recap.totalAcceptedEdits) accepted · \(recap.totalRejectedActions) rejected")
            return parts.joined(separator: " · ")
        case .detailed:
            var parts: [String] = []
            if let topProject = recap.topProjectDisplayPath {
                parts.append("Top project: \(topProject)")
            }
            if let topModel = recap.topModelDisplayName {
                parts.append("Top model: \(topModel)")
            }
            if let highestSpend = recap.mostExpensiveSession {
                parts.append("Highest spend: \(Format.cost(highestSpend.dayCost))")
            }
            parts.append("Files touched: \(recap.totalTouchedFileCount)")
            parts.append("Edits: \(recap.totalAcceptedEdits) accepted · \(recap.totalRejectedActions) rejected")
            return parts.joined(separator: " · ")
        }
    }

    private func summarySchedulingAllowedToday() -> Bool {
        let weekdaysOnly = (UserDefaults.standard.object(forKey: "summaryWeekdaysOnly") as? Bool) ?? false
        guard weekdaysOnly else { return true }
        let weekday = Calendar.current.component(.weekday, from: Date())
        return (2...6).contains(weekday)
    }

    private func notificationsSnoozedToday() -> Bool {
        UserDefaults.standard.string(forKey: "notificationSnoozedDayKey") == Format.dayKey(Date())
    }

    private func isWithinQuietHours(now: Date = Date()) -> Bool {
        let enabled = (UserDefaults.standard.object(forKey: "notificationsQuietHoursEnabled") as? Bool) ?? false
        guard enabled else { return false }

        let startHour = UserDefaults.standard.integer(forKey: "notificationsQuietHoursStart")
        let endHour = UserDefaults.standard.integer(forKey: "notificationsQuietHoursEnd")
        let hour = Calendar.current.component(.hour, from: now)

        if startHour == endHour { return true }
        if startHour < endHour {
            return hour >= startHour && hour < endHour
        }
        return hour >= startHour || hour < endHour
    }
}

// MARK: – UNUserNotificationCenterDelegate
// Allows notifications to appear even while the app is in the foreground.
extension NotificationManager: UNUserNotificationCenterDelegate {
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler handler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        handler([.banner, .sound])
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        defer { completionHandler() }

        guard response.actionIdentifier != UNNotificationDismissActionIdentifier else { return }

        if response.actionIdentifier == NotificationActionIdentifier.snoozeToday.rawValue {
            UserDefaults.standard.set(Format.dayKey(Date()), forKey: "notificationSnoozedDayKey")
            return
        }

        if response.actionIdentifier == NotificationActionIdentifier.openDashboard.rawValue {
            DispatchQueue.main.async {
                NSApp.activate(ignoringOtherApps: true)
                NotificationCenter.default.post(name: .clauxOpenDashboard, object: nil)
            }
            return
        }

        let userInfo = response.notification.request.content.userInfo
        guard let target = userInfo[NotificationPayloadKey.target] as? String else { return }

        switch NotificationTarget(rawValue: target) {
        case .session:
            guard let sessionID = userInfo[NotificationPayloadKey.sessionID] as? String else { return }
            DispatchQueue.main.async {
                NSApp.activate(ignoringOtherApps: true)
                NotificationCenter.default.post(
                    name: .clauxOpenSession,
                    object: nil,
                    userInfo: [NotificationPayloadKey.sessionID: sessionID]
                )
            }
        case .dailyRecap:
            let dayKey = (userInfo[NotificationPayloadKey.dayKey] as? String) ?? Format.dayKey(Date())
            DispatchQueue.main.async {
                NSApp.activate(ignoringOtherApps: true)
                NotificationCenter.default.post(
                    name: .clauxOpenDailyRecap,
                    object: nil,
                    userInfo: [NotificationPayloadKey.dayKey: dayKey]
                )
            }
        case .dashboard:
            DispatchQueue.main.async {
                NSApp.activate(ignoringOtherApps: true)
                NotificationCenter.default.post(name: .clauxOpenDashboard, object: nil)
            }
        case .none:
            break
        }
    }
}
