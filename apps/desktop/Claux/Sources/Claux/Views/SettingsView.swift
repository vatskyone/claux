import SwiftUI
import AppKit
import ServiceManagement
import UserNotifications

// MARK: – Always-on-top + blur helper
// Reaches into the hosting NSWindow, pins its level to .floating, and makes it
// transparent so the NSVisualEffectView blur shows through.
private final class _FloatingBlurSetupView: NSView {
    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        guard let win = window else { return }
        DispatchQueue.main.async {
            win.level              = .floating
            win.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
            win.backgroundColor    = .clear
            win.isOpaque           = false
        }
    }
}

private struct WindowFloater: NSViewRepresentable {
    func makeNSView(context: Context) -> _FloatingBlurSetupView { _FloatingBlurSetupView() }
    func updateNSView(_ v: _FloatingBlurSetupView, context: Context) {}
}

// MARK: – SettingsView
struct SettingsView: View {

    @EnvironmentObject var store: AppStore
    @ObservedObject private var notifManager = NotificationManager.shared
    @ObservedObject private var statusLineManager = ClaudeStatusLineManager.shared

    // ── Appearance ───────────────────────────────────────────────────────────
    @AppStorage("appTheme")               private var appTheme:            String = "auto"
    @AppStorage("stateColorPreset")       private var stateColorPreset:    String = StateColorPreset.system.rawValue

    // ── General ─────────────────────────────────────────────────────────────
    @AppStorage("launchAtLogin")          private var launchAtLogin:       Bool   = false
    @AppStorage("showCostInMenuBar")      private var showCostInMenuBar:   Bool   = false
    @AppStorage("showModelInMenuBar")     private var showModelInMenuBar:  Bool   = false
    @AppStorage("sessionRetentionDays")   private var retentionDays:       Int    = 30
    @AppStorage("autoRefreshInterval")    private var refreshInterval:     Int    = 10
    @AppStorage("costProjectionPeriod")   private var projPeriod:          String = "monthly"

    // ── Budget ───────────────────────────────────────────────────────────────
    @AppStorage("monthlyBudget")          private var monthlyBudget:       Double = 0

    // ── Notifications ────────────────────────────────────────────────────────
    @AppStorage("enableNotifications")    private var notificationsOn:     Bool   = true
    @AppStorage("costAlertThreshold")     private var costThreshold:       Double = 5.0
    @AppStorage("contextHealthAlert")     private var contextAlert:        Double = 80.0
    @AppStorage("alertOnSessionEnd")      private var alertOnSessionEnd:   Bool   = false
    @AppStorage("notificationVerbosity")  private var notificationVerbosity: String = NotificationVerbosity.standard.rawValue
    @AppStorage("notificationsQuietHoursEnabled") private var quietHoursEnabled: Bool = false
    @AppStorage("notificationsQuietHoursStart") private var quietHoursStart: Int = 22
    @AppStorage("notificationsQuietHoursEnd") private var quietHoursEnd: Int = 8
    @AppStorage("dailySummaryEnabled")    private var dailySummaryEnabled: Bool   = false
    @AppStorage("dailySummaryHour")       private var dailySummaryHour:    Int    = 18
    @AppStorage("weeklyRecapEnabled")     private var weeklyRecapEnabled: Bool = false
    @AppStorage("weeklyRecapWeekday")     private var weeklyRecapWeekday: Int = SummaryWeekday.monday.rawValue
    @AppStorage("summaryWeekdaysOnly")    private var summaryWeekdaysOnly: Bool = false
    @AppStorage("claudemdAlertEnabled")   private var claudemdAlertEnabled: Bool  = true
    @AppStorage("claudemdThreshold")      private var claudemdThreshold:   Int    = 50

    // ── Data ─────────────────────────────────────────────────────────────────
    @AppStorage("monitoredDirectory")    private var watchDirectory:      String = "~/.claude"
    @AppStorage("includeCacheCost")      private var includeCacheCost:    Bool   = true

    // ── Local UI state ───────────────────────────────────────────────────────
    @State private var showResetConfirm   = false
    @State private var resetDone          = false
    @State private var loginItemError: String?

    /// True when SMAppService confirms the app is registered as a login item.
    private var isRegisteredAtLogin: Bool {
        guard Bundle.main.bundleIdentifier != nil else { return false }
        return SMAppService.mainApp.status == .enabled
    }

    var body: some View {
        VStack(spacing: 0) {
        Form {

            // ── General ──────────────────────────────────────────────────────
            Section {
                Toggle("Launch at login", isOn: Binding(
                    get: { isRegisteredAtLogin },
                    set: { setLaunchAtLogin($0) }
                ))

                Toggle("Show cost in menu bar", isOn: $showCostInMenuBar)

                Toggle("Show model badge in menu bar", isOn: $showModelInMenuBar)
                    .help("Show the current model (e.g. Sonnet 4.6) next to the menu bar icon during an active session")

                Picker("Session retention", selection: $retentionDays) {
                    Text("7 days").tag(7)
                    Text("14 days").tag(14)
                    Text("30 days").tag(30)
                    Text("60 days").tag(60)
                    Text("90 days").tag(90)
                    Text("1 year").tag(365)
                }

                Picker("Auto-refresh", selection: $refreshInterval) {
                    Text("1 s").tag(1)
                    Text("5 s").tag(5)
                    Text("10 s").tag(10)
                    Text("30 s").tag(30)
                    Text("60 s").tag(60)
                }
                .help("How often Claux re-scans Claude's session files")

                Picker("Cost projection", selection: $projPeriod) {
                    Text("Daily").tag("daily")
                    Text("Weekly").tag("weekly")
                    Text("Monthly").tag("monthly")
                }
                .help("Time period for the projected spend shown in the active session card")

                Picker("Monthly budget", selection: $monthlyBudget) {
                    Text("Off").tag(0.0)
                    Text("$25").tag(25.0)
                    Text("$50").tag(50.0)
                    Text("$100").tag(100.0)
                    Text("$200").tag(200.0)
                    Text("$500").tag(500.0)
                    Text("$1 000").tag(1000.0)
                }
                .help("Monthly spend cap. A progress bar appears in the popover once any spending is recorded.")

                Picker("Appearance", selection: $appTheme) {
                    Text("Light").tag("light")
                    Text("Dark").tag("dark")
                    Text("Auto").tag("auto")
                }
                .help("Choose Light, Dark, or Auto (follows macOS system appearance)")

                Picker("State colors", selection: $stateColorPreset) {
                    ForEach(StateColorPreset.allCases) { preset in
                        Text(preset.label).tag(preset.rawValue)
                    }
                }
                .help("Choose the color palette used for state badges, warnings, progress bars, and session stats. Accessibility-focused options include High Contrast and Colorblind Safe.")

                LabeledContent("Palette preview") {
                    HStack(spacing: 8) {
                        paletteSwatch("Blue", color: .clauxBlue)
                        paletteSwatch("Green", color: .clauxGreen)
                        paletteSwatch("Orange", color: .clauxOrange)
                        paletteSwatch("Red", color: .clauxRed)
                    }
                    .frame(maxWidth: .infinity, alignment: .trailing)
                }

            } header: {
                Label("General", systemImage: "gearshape")
            }
            .listRowBackground(Color.clear)

            // ── Notifications ─────────────────────────────────────────────────
            Section {
                // Live permission status — shows what macOS actually allows.
                // Must read through notifManager (the @ObservedObject) so SwiftUI
                // re-renders this row when authStatus changes.
                LabeledContent("System permission") {
                    HStack(spacing: 8) {
                        switch notifManager.authStatus {
                        case .authorized, .provisional:
                            Label("Enabled", systemImage: "checkmark.circle.fill")
                                .foregroundStyle(Color.clauxGreen)
                                .font(.system(size: 12))
                        case .denied:
                            Label("Denied", systemImage: "xmark.circle.fill")
                                .foregroundStyle(Color.clauxRed)
                                .font(.system(size: 12))
                            Button("Open System Settings") {
                                NotificationManager.shared.requestPermission(openSettingsIfDenied: true)
                            }
                            .controlSize(.small)
                        case .notDetermined:
                            Label("Not requested", systemImage: "questionmark.circle")
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                                .font(.system(size: 12))
                            Button("Allow") {
                                NotificationManager.shared.requestPermission()
                            }
                            .controlSize(.small)
                            .buttonStyle(.borderedProminent)
                        @unknown default:
                            EmptyView()
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .trailing)
                }

                Toggle("Enable notifications", isOn: $notificationsOn)

                Picker("Cost alert", selection: $costThreshold) {
                    Text("$0.50").tag(0.5)
                    Text("$1").tag(1.0)
                    Text("$2").tag(2.0)
                    Text("$5").tag(5.0)
                    Text("$10").tag(10.0)
                    Text("$20").tag(20.0)
                    Text("$50").tag(50.0)
                    Text("$100").tag(100.0)
                }
                .help("Notify when a session exceeds this cost")
                .disabled(!notificationsOn)

                LabeledContent("Context window alert") {
                    HStack(spacing: 8) {
                        Slider(value: $contextAlert, in: 50...95, step: 5)
                            .frame(width: 110)
                        Text("\(Int(contextAlert))%")
                            .monospacedDigit()
                            .foregroundStyle(.secondary)
                            .frame(width: 36, alignment: .trailing)
                    }
                    .frame(maxWidth: .infinity, alignment: .trailing)
                }
                .help("Warn when the context window exceeds this percentage")
                .disabled(!notificationsOn)

                Toggle("Notify when a session ends", isOn: $alertOnSessionEnd)
                    .disabled(!notificationsOn)

                Picker("Notification detail", selection: $notificationVerbosity) {
                    ForEach(NotificationVerbosity.allCases) { verbosity in
                        Text(verbosity.label).tag(verbosity.rawValue)
                    }
                }
                .help("Controls how much session and summary detail Claux includes in each notification.")
                .disabled(!notificationsOn)

                Toggle("Daily summary", isOn: $dailySummaryEnabled)
                    .disabled(!notificationsOn)
                    .help("Send a daily summary notification with today's total spend")

                Toggle("Weekly recap", isOn: $weeklyRecapEnabled)
                    .disabled(!notificationsOn)
                    .help("Send one recap notification for the last 7 completed days.")

                Toggle("Weekday-only summaries", isOn: $summaryWeekdaysOnly)
                    .disabled(!notificationsOn || (!dailySummaryEnabled && !weeklyRecapEnabled))
                    .help("Skip summary notifications on weekends.")

                if dailySummaryEnabled || weeklyRecapEnabled {
                    Picker("Send at", selection: $dailySummaryHour) {
                        ForEach(0..<24, id: \.self) { hour in
                            Text(hourLabel(hour)).tag(hour)
                        }
                    }
                    .disabled(!notificationsOn)
                }

                if weeklyRecapEnabled {
                    Picker("Weekly recap day", selection: $weeklyRecapWeekday) {
                        ForEach(SummaryWeekday.allCases) { weekday in
                            Text(weekday.label).tag(weekday.rawValue)
                        }
                    }
                    .disabled(!notificationsOn)
                }

                Toggle("Quiet hours", isOn: $quietHoursEnabled)
                    .disabled(!notificationsOn)
                    .help("Suppress Claux notifications during a blocked time window.")

                if quietHoursEnabled {
                    Picker("Quiet hours start", selection: $quietHoursStart) {
                        ForEach(0..<24, id: \.self) { hour in
                            Text(hourLabel(hour)).tag(hour)
                        }
                    }
                    .disabled(!notificationsOn)

                    Picker("Quiet hours end", selection: $quietHoursEnd) {
                        ForEach(0..<24, id: \.self) { hour in
                            Text(hourLabel(hour)).tag(hour)
                        }
                    }
                    .disabled(!notificationsOn)
                }

                Toggle("CLAUDE.md quality alert", isOn: $claudemdAlertEnabled)
                    .disabled(!notificationsOn)
                    .help("Show a warning in the active session card when CLAUDE.md quality is below the threshold")

                Picker("Quality threshold", selection: $claudemdThreshold) {
                    Text("30").tag(30)
                    Text("50").tag(50)
                    Text("70").tag(70)
                    Text("85").tag(85)
                }
                .help("Warn when the CLAUDE.md quality score drops below this value (0–100)")
                .disabled(!notificationsOn || !claudemdAlertEnabled)

            } header: {
                Label("Notifications", systemImage: "bell")
            }
            .listRowBackground(Color.clear)

            // ── Data Source ───────────────────────────────────────────────────
            Section {
                LabeledContent("Monitored directory") {
                    HStack(spacing: 8) {
                        // Show the expanded path so tilde values are never ambiguous
                        let expanded = NSString(string: watchDirectory).expandingTildeInPath as String
                        Text(expanded)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .frame(maxWidth: 120, alignment: .trailing)
                        Button("Change…") { chooseDirectory() }
                            .controlSize(.small)
                        if watchDirectory != "~/.claude" {
                                Button("Reset") { watchDirectory = "~/.claude" }
                                    .controlSize(.small)
                                    .foregroundStyle(Color.clauxOrange)
                        }
                    }
                }

                LabeledContent("Claude integration") {
                    VStack(alignment: .trailing, spacing: 6) {
                        Text(integrationSummary)
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(integrationSummaryColor)
                            .multilineTextAlignment(.trailing)

                        HStack(spacing: 8) {
                            Button(integrationActionTitle) {
                                if statusLineManager.installOrRepair() {
                                    store.refreshNow()
                                }
                            }
                            .controlSize(.small)

                            if case .managedReady = statusLineManager.inspection.state {
                                Button("Refresh") {
                                    statusLineManager.refresh()
                                    store.refreshNow()
                                }
                                .controlSize(.small)
                            }
                        }

                        if let lastOperationMessage = statusLineManager.lastOperationMessage {
                            Text(lastOperationMessage)
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                                .multilineTextAlignment(.trailing)
                        }
                    }
                }
                .help("Installs a Claux-managed Claude statusLine wrapper. Existing custom statusLine commands are preserved and wrapped.")

                Toggle("Include prompt-cache cost", isOn: $includeCacheCost)
                    .help("Count cache-read and cache-write tokens in session cost totals")

                LabeledContent("Usage data") {
                    HStack(spacing: 8) {
                        if resetDone {
                            Label("Erased!", systemImage: "checkmark.circle.fill")
                                .foregroundStyle(.green)
                                .font(.system(size: 12))
                                .transition(.opacity)
                        }

                        Button("Erase All Data…") {
                            showResetConfirm = true
                        }
                        .controlSize(.small)
                        .foregroundStyle(.red)
                    }
                }
                .help("Clear all in-memory session and usage data. Your settings and Claude session files are not affected.")

            } header: {
                Label("Data Source", systemImage: "folder")
            }
            .listRowBackground(Color.clear)

            // ── About ─────────────────────────────────────────────────────────
            Section {
                LabeledContent("Version") {
                    Text(AppVersion.current)
                        .foregroundStyle(.secondary)
                }

                LabeledContent("Claude Code") {
                    Link("claude.ai/code", destination: URL(string: "https://claude.ai/code")!)
                }

                LabeledContent("Feedback") {
                    Link("Open an issue", destination: URL(string: "https://github.com/vatskyone/claux/issues")!)
                }

                LabeledContent("") {
                    HStack(spacing: 12) {
                        Button {
                            store.refreshNow()
                        } label: {
                            Label("Refresh sessions and plan limits", systemImage: "arrow.clockwise")
                        }
                        .controlSize(.small)

                        Button {
                            NotificationManager.shared.sendTestNotification()
                        } label: {
                            Label("Test notification", systemImage: "bell.badge")
                        }
                        .controlSize(.small)
                    }
                }
            } header: {
                Label("About", systemImage: "info.circle")
            }
            .listRowBackground(Color.clear)
        }
        .formStyle(.grouped)
        .scrollContentBackground(.hidden)  // let the VisualEffectView blur show through
        .frame(width: 460, height: 582)

        Divider()

        // ── Reset footer ──────────────────────────────────────────────────────
        Button { showResetConfirm = true } label: {
            Text("Reset all settings to default")
                .font(.system(size: 13))
                .foregroundStyle(Color.clauxAccent)
                .underline()
                .frame(maxWidth: .infinity, alignment: .center)
        }
        .buttonStyle(.plain)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 12)

        } // end VStack
        .frame(width: 460, height: 620)
        .background(VisualEffectView(material: .sidebar, blendingMode: .behindWindow))
        .background(WindowFloater())
        .id(stateColorPreset)
        .onAppear {
            notifManager.refreshAuthStatus()
            statusLineManager.refresh()
        }

        // ── Reset confirmation ────────────────────────────────────────────────
        .confirmationDialog(
            "Erase All Data?",
            isPresented: $showResetConfirm,
            titleVisibility: .visible
        ) {
            Button("Erase Sessions & Usage Data", role: .destructive) {
                store.eraseSessionData()
                withAnimation { resetDone = true }
                DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) {
                    withAnimation { resetDone = false }
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("All in-memory session and usage data will be cleared. Your settings and Claude session files are not affected.")
        }

    }

    // MARK: – Launch at login
    private func setLaunchAtLogin(_ enable: Bool) {
        guard Bundle.main.bundleIdentifier != nil else {
            loginItemError = "Run the app as a .app bundle to enable this feature."
            return
        }
        do {
            if enable { try SMAppService.mainApp.register()   }
            else       { try SMAppService.mainApp.unregister() }
            loginItemError = nil
        } catch {
            loginItemError = error.localizedDescription
        }
    }

    // MARK: – Directory picker
    private func chooseDirectory() {
        let panel = NSOpenPanel()
        panel.canChooseFiles         = false
        panel.canChooseDirectories   = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Select"
        if panel.runModal() == .OK, let url = panel.url {
            watchDirectory = url.path
        }
    }

    private var integrationActionTitle: String {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return "Reinstall"
        case .managedNeedsRepair:
            return "Repair"
        case .notInstalled:
            return "Install"
        case .customCommand:
            return "Wrap Existing"
        case .invalidSettings:
            return "Retry Install"
        }
    }

    private var integrationSummary: String {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return "Managed by Claux"
        case .managedNeedsRepair:
            return "Managed install incomplete"
        case .notInstalled:
            return "Not installed"
        case .customCommand:
            return "Custom command detected"
        case .invalidSettings:
            return "settings.json unreadable"
        }
    }

    private var integrationSummaryColor: Color {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return .clauxGreen
        case .managedNeedsRepair, .notInstalled, .customCommand:
            return .clauxOrange
        case .invalidSettings:
            return .clauxRed
        }
    }

    private func paletteSwatch(_ name: String, color: Color) -> some View {
        VStack(spacing: 4) {
            RoundedRectangle(cornerRadius: 4)
                .fill(color)
                .frame(width: 22, height: 12)
            Text(name)
                .font(.system(size: 9))
                .foregroundStyle(.secondary)
        }
    }

    private func hourLabel(_ hour: Int) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "h:00 a"
        let date = Calendar.current.date(from: DateComponents(hour: hour)) ?? Date()
        return formatter.string(from: date).lowercased()
    }
}
