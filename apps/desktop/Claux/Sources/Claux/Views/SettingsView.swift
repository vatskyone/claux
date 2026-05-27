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

    // ── Appearance ───────────────────────────────────────────────────────────
    @AppStorage("appTheme")               private var appTheme:            String = "auto"

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
    @AppStorage("dailySummaryEnabled")    private var dailySummaryEnabled: Bool   = false
    @AppStorage("dailySummaryHour")       private var dailySummaryHour:    Int    = 18
    @AppStorage("claudemdAlertEnabled")   private var claudemdAlertEnabled: Bool  = true
    @AppStorage("claudemdThreshold")      private var claudemdThreshold:   Int    = 50

    // ── Data ─────────────────────────────────────────────────────────────────
    @AppStorage("monitoredDirectory")    private var watchDirectory:      String = "~/.claude"
    @AppStorage("includeCacheCost")      private var includeCacheCost:    Bool   = true

    // ── Account ──────────────────────────────────────────────────────────────
    @AppStorage("clauxAccountEmail")     private var accountEmail:        String = ""
    @AppStorage("clauxSyncEnabled")      private var syncEnabled:         Bool   = true

    // ── Local UI state ───────────────────────────────────────────────────────
    @State private var showResetConfirm   = false
    @State private var showSignInSheet    = false
    @State private var showSignOutConfirm = false
    @State private var signInEmailDraft   = ""
    @State private var signInError: String?
    @State private var resetDone          = false
    @State private var loginItemError: String?

    private var isSignedIn: Bool { !accountEmail.isEmpty }

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
                                .foregroundStyle(Color(nsColor: .systemGreen))
                                .font(.system(size: 12))
                        case .denied:
                            Label("Denied", systemImage: "xmark.circle.fill")
                                .foregroundStyle(Color(nsColor: .systemRed))
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

                Toggle("Daily summary", isOn: $dailySummaryEnabled)
                    .disabled(!notificationsOn)
                    .help("Send a daily summary notification with today's total spend")

                if dailySummaryEnabled {
                    Picker("Send at", selection: $dailySummaryHour) {
                        Text("12:00 pm").tag(12)
                        Text("3:00 pm").tag(15)
                        Text("6:00 pm").tag(18)
                        Text("9:00 pm").tag(21)
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
                                .foregroundStyle(Color(nsColor: .systemOrange))
                        }
                    }
                }

                Toggle("Include prompt-cache cost", isOn: $includeCacheCost)
                    .help("Count cache-read and cache-write tokens in session cost totals")

                LabeledContent("Usage data") {
                    HStack(spacing: 8) {
                        if resetDone {
                            Label("Reset!", systemImage: "checkmark.circle.fill")
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
                .help("Reset all Claux settings to defaults. Your Claude session files are not deleted.")

            } header: {
                Label("Data Source", systemImage: "folder")
            }
            .listRowBackground(Color.clear)

            // ── Account ───────────────────────────────────────────────────────
            Section {
                if isSignedIn {
                    LabeledContent("Account") {
                        Text(accountEmail)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }

                    LabeledContent("Plan") {
                        HStack(spacing: 6) {
                            Text("Free")
                                .foregroundStyle(.secondary)
                            Text("Upgrade")
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundStyle(Color(nsColor: .systemBlue))
                                .onTapGesture {
                                    NSWorkspace.shared.open(URL(string: "https://claux.app/upgrade")!)
                                }
                        }
                    }

                    Toggle("Sync usage data to cloud", isOn: $syncEnabled)
                        .help("Upload aggregated session stats so you can view them on any device")

                    LabeledContent("") {
                        Button("Sign out…") {
                            showSignOutConfirm = true
                        }
                        .controlSize(.small)
                        .foregroundStyle(.red)
                    }

                } else {
                    // Not signed in — marketing blurb + sign-in CTA
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Sign in to sync your usage data across devices and access the Claux web dashboard.")
                            .font(.system(size: 12))
                            .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            .fixedSize(horizontal: false, vertical: true)

                        HStack(spacing: 8) {
                            Button {
                                signInEmailDraft = ""
                                signInError = nil
                                showSignInSheet = true
                            } label: {
                                Label("Sign in with email", systemImage: "envelope")
                            }
                            .controlSize(.small)

                            Button {
                                // Apple Sign-in placeholder
                                signInEmailDraft = ""
                                signInError = nil
                                showSignInSheet = true
                            } label: {
                                Label("Sign in with Apple", systemImage: "applelogo")
                            }
                            .controlSize(.small)
                        }
                    }
                    .padding(.vertical, 4)
                }
            } header: {
                Label("Account", systemImage: "person.circle")
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
                    Link("Open an issue", destination: URL(string: "https://github.com/snowbayles/claux/issues/new")!)
                }

                LabeledContent("") {
                    HStack(spacing: 12) {
                        Button {
                            store.refreshNow()
                        } label: {
                            Label("Refresh sessions now", systemImage: "arrow.clockwise")
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
        .onAppear { notifManager.refreshAuthStatus() }

        // ── Reset confirmation ────────────────────────────────────────────────
        .confirmationDialog(
            "Erase All Data?",
            isPresented: $showResetConfirm,
            titleVisibility: .visible
        ) {
            Button("Erase All Settings", role: .destructive) {
                store.resetAllData()
                withAnimation { resetDone = true }
                DispatchQueue.main.asyncAfter(deadline: .now() + 2.5) {
                    withAnimation { resetDone = false }
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("All Claux settings will be reset to defaults. Your Claude session files are not affected.")
        }

        // ── Sign-in sheet ─────────────────────────────────────────────────────
        .sheet(isPresented: $showSignInSheet) {
            SignInSheet(
                emailDraft: $signInEmailDraft,
                errorMessage: $signInError
            ) { email in
                accountEmail = email
                showSignInSheet = false
            }
        }

        // ── Sign-out confirmation ─────────────────────────────────────────────
        .confirmationDialog(
            "Sign out of Claux?",
            isPresented: $showSignOutConfirm,
            titleVisibility: .visible
        ) {
            Button("Sign out", role: .destructive) {
                accountEmail = ""
                syncEnabled  = true
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Your local usage data will not be deleted.")
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
}

// MARK: – Sign-in sheet
private struct SignInSheet: View {
    @Binding var emailDraft:    String
    @Binding var errorMessage:  String?
    let onSignIn: (String) -> Void

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "person.circle.fill")
                .font(.system(size: 44))
                .foregroundStyle(Color(nsColor: .systemBlue))

            Text("Sign in to Claux")
                .font(.system(size: 18, weight: .semibold))

            Text("Enter your email address to create a free account or sign into an existing one.")
                .font(.system(size: 13))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .frame(maxWidth: 280)

            VStack(alignment: .leading, spacing: 6) {
                TextField("you@example.com", text: $emailDraft)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 280)

                if let err = errorMessage {
                    Text(err)
                        .font(.system(size: 11))
                        .foregroundStyle(Color(nsColor: .systemRed))
                        .frame(width: 280, alignment: .leading)
                }
            }

            HStack(spacing: 12) {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)

                Button("Continue") {
                    let trimmed = emailDraft.trimmingCharacters(in: .whitespacesAndNewlines)
                    if trimmed.contains("@") && trimmed.contains(".") {
                        onSignIn(trimmed)
                    } else {
                        errorMessage = "Please enter a valid email address."
                    }
                }
                .buttonStyle(.borderedProminent)
                .keyboardShortcut(.defaultAction)
                .disabled(emailDraft.trimmingCharacters(in: .whitespaces).isEmpty)
            }

            Text("By continuing you agree to the Claux Terms of Service and Privacy Policy.")
                .font(.system(size: 10))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .multilineTextAlignment(.center)
                .frame(maxWidth: 280)
        }
        .padding(28)
        .frame(width: 360)
    }
}
