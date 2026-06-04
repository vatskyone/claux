import UserNotifications
import Combine
import AppKit

extension Notification.Name {
    static let clauxOpenDailyRecap = Notification.Name("clauxOpenDailyRecap")
}

// MARK: – NotificationManager
// Owns all UNUserNotificationCenter interactions for Claux.
// Observes AppStore and fires alerts when thresholds are crossed.
final class NotificationManager: NSObject, ObservableObject {
    private enum NotificationPayloadKey {
        static let target = "clauxTarget"
        static let dayKey = "dayKey"
    }

    private enum NotificationTarget: String {
        case dailyRecap
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
        // Populate authStatus on startup
        refreshAuthStatus()
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
                    fire(
                        id: "session-ended-\(endedID)",
                        title: "Session Ended",
                        body: "\(ended.displayPath) finished · \(Format.cost(ended.totalCost)) · \(Format.duration(ended.duration))",
                        symbol: "checkmark.circle"
                    )
                }
            }
        }

        lastKnownActiveIDs = currentActiveIDs

        // ── Daily summary ───────────────────────────────────────────────────
        checkDailySummary(store: store)

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
            body = parts.isEmpty ? "Open Claux to review today’s sessions." : parts.joined(separator: " · ")
        } else {
            body = "No Claude sessions today"
        }

        fire(id: "daily-summary-\(todayKey)",
             title: "Claux Daily Summary",
             subtitle: subtitle,
             body: body,
             symbol: "chart.bar.fill",
             userInfo: [
                NotificationPayloadKey.target: NotificationTarget.dailyRecap.rawValue,
                NotificationPayloadKey.dayKey: todayKey
             ])

        UserDefaults.standard.set(todayKey, forKey: "dailySummaryLastSent")
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
            body: "\(session.displayPath) has spent \(Format.cost(session.totalCost)) this session.",
            symbol: "dollarsign.circle"
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
                body: "\(session.displayPath) context is \(Int(fraction * 100))% full. Consider starting a new session.",
                symbol: "exclamationmark.triangle"
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
                body: "\(session.displayPath) context is \(Int(fraction * 100))% full.",
                symbol: "exclamationmark.circle"
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
        userInfo: [AnyHashable: Any] = [:]
    ) {
        guard notificationsAvailable else { return }
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
            content.userInfo = userInfo
            UNUserNotificationCenter.current().add(
                UNNotificationRequest(identifier: id, content: content, trigger: nil)
            )
        }
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

        let userInfo = response.notification.request.content.userInfo
        guard let target = userInfo[NotificationPayloadKey.target] as? String else { return }

        switch NotificationTarget(rawValue: target) {
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
        case .none:
            break
        }
    }
}
