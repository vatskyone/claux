import SwiftUI
import AppKit

/// First-launch onboarding overlay shown inside the popover.
/// Tracks completion via @AppStorage("onboardingCompleted").
/// Covers the entire popover and fades out when the user taps "All Done".
struct OnboardingView: View {

    @EnvironmentObject var store: AppStore
    @ObservedObject private var notifManager = NotificationManager.shared
    @ObservedObject private var statusLineManager = ClaudeStatusLineManager.shared
    @AppStorage("onboardingCompleted") private var onboardingCompleted: Bool = false
    @AppStorage("monitoredDirectory")  private var watchDirectory:      String = "~/.claude"

    @State private var step: Int = 0

    var body: some View {
        VStack(spacing: 0) {

            // ── Step progress indicator ───────────────────────────────────────
            // Active step → wide capsule in accent blue; others → small dots.
            HStack(spacing: 7) {
                ForEach(0..<4) { i in
                    Capsule()
                        .fill(i == step
                              ? Color.clauxAccent
                              : (i < step
                                 ? Color.clauxAccent.opacity(0.45)
                                 : Color(nsColor: .quaternaryLabelColor)))
                        .frame(width: i == step ? 22 : 7, height: 7)
                        .animation(.spring(response: 0.3, dampingFraction: 0.75), value: step)
                }
            }
            .padding(.top, 24)
            .padding(.bottom, 20)

            // ── Step content ──────────────────────────────────────────────────
            // The .id(step) forces SwiftUI to treat each step as a distinct view,
            // enabling the asymmetric slide transition.
            Group {
                switch step {
                case 0: welcomeStep
                case 1: pathStep
                case 2: integrationStep
                case 3: notifStep
                default: EmptyView()
                }
            }
            .frame(maxWidth: .infinity)
            .id(step)
            .transition(.asymmetric(
                insertion: .move(edge: .trailing).combined(with: .opacity),
                removal:   .move(edge: .leading).combined(with: .opacity)
            ))

            Spacer(minLength: 16)

            // ── Navigation buttons ────────────────────────────────────────────
            VStack(spacing: 8) {
                Button(action: advance) {
                    HStack(spacing: 6) {
                        Text(primaryLabel)
                            .font(.system(size: 14, weight: .semibold))
                        Image(systemName: step < 2 ? "arrow.right" : "checkmark")
                            .font(.system(size: 12, weight: .bold))
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 9)
                }
                .buttonStyle(.borderedProminent)
                .padding(.horizontal, 24)

                if step > 0 {
                    Button("← Back") {
                        withAnimation(.easeInOut(duration: 0.2)) { step -= 1 }
                    }
                    .buttonStyle(.plain)
                    .font(.system(size: 12))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                }
            }
            .padding(.bottom, 24)
        }
        .background(.thickMaterial)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: – Step 1: Welcome

    private var welcomeStep: some View {
        VStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(Color.clauxAccent.opacity(0.10))
                    .frame(width: 76, height: 76)
                Image(systemName: "c.circle.fill")
                    .font(.system(size: 46))
                    .foregroundStyle(Color.clauxAccent)
            }

            Text("Welcome to Claux")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Color(nsColor: .labelColor))

            Text("Claux lives in your menu bar and shows real-time stats for every Claude Code session — spend, tokens, context health, and more.")
                .font(.system(size: 13))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 28)
        }
        .padding(.top, 4)
    }

    // MARK: – Step 2: Session directory

    private var pathStep: some View {
        VStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(Color.clauxAccent.opacity(0.10))
                    .frame(width: 76, height: 76)
                Image(systemName: "folder.fill")
                    .font(.system(size: 38))
                    .foregroundStyle(Color.clauxAccent)
            }

            Text("Session Directory")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Color(nsColor: .labelColor))

            Text("Claux watches this folder for Claude Code session files. The default path is correct for most setups.")
                .font(.system(size: 13))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 28)

            // Monospaced path pill
            let expanded = (watchDirectory as NSString).expandingTildeInPath
            Text(expanded)
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(Color(nsColor: .labelColor))
                .lineLimit(1)
                .truncationMode(.middle)
                .padding(.horizontal, 12)
                .padding(.vertical, 7)
                .background(.regularMaterial)
                .clipShape(RoundedRectangle(cornerRadius: 7))
                .overlay(
                    RoundedRectangle(cornerRadius: 7)
                        .stroke(Color(nsColor: .separatorColor).opacity(0.5), lineWidth: 0.5)
                )
                .padding(.horizontal, 24)

            Button("Change…") { chooseDirectory() }
                .controlSize(.small)
                .buttonStyle(.bordered)
        }
        .padding(.top, 4)
    }

    // MARK: – Step 3: Notifications

    private var integrationStep: some View {
        VStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(Color.clauxAccent.opacity(0.10))
                    .frame(width: 76, height: 76)
                Image(systemName: "link.badge.plus")
                    .font(.system(size: 38))
                    .foregroundStyle(Color.clauxAccent)
            }

            Text("Claude Integration")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Color(nsColor: .labelColor))

            Text("Claux installs a managed statusLine wrapper so plan limits work automatically. Existing custom statusLine commands are preserved and wrapped.")
                .font(.system(size: 13))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 28)

            Label(integrationStatusTitle, systemImage: integrationStatusIcon)
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(integrationStatusColor)
                .padding(.top, 2)

            Text(statusLineManager.inspection.message)
                .font(.system(size: 11))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .padding(.horizontal, 24)

            if case let .customCommand(command) = statusLineManager.inspection.state {
                Text(command)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(Color(nsColor: .labelColor))
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)
                    .background(.regularMaterial)
                    .clipShape(RoundedRectangle(cornerRadius: 7))
                    .overlay(
                        RoundedRectangle(cornerRadius: 7)
                            .stroke(Color(nsColor: .separatorColor).opacity(0.5), lineWidth: 0.5)
                    )
                    .padding(.horizontal, 24)
            }

            if let lastOperationMessage = statusLineManager.lastOperationMessage {
                Text(lastOperationMessage)
                    .font(.system(size: 11))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 24)
            }
        }
        .padding(.top, 4)
        .onAppear { statusLineManager.refresh() }
    }

    private var notifStep: some View {
        VStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(Color.clauxAccent.opacity(0.10))
                    .frame(width: 76, height: 76)
                Image(systemName: "bell.badge.fill")
                    .font(.system(size: 38))
                    .foregroundStyle(Color.clauxAccent)
            }

            Text("Stay Informed")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Color(nsColor: .labelColor))

            Text("Get alerts when session spend crosses a limit, the context window fills up, or at end of day with a cost summary.")
                .font(.system(size: 13))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .multilineTextAlignment(.center)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 28)

            // Live permission status
            permissionView
        }
        .padding(.top, 4)
        .onAppear { notifManager.refreshAuthStatus() }
    }

    @ViewBuilder
    private var permissionView: some View {
        switch notifManager.authStatus {
        case .authorized, .provisional:
            Label("Notifications enabled", systemImage: "checkmark.circle.fill")
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(Color.clauxGreen)
                .padding(.top, 2)

        case .denied:
            VStack(spacing: 5) {
                Label("Blocked by System Settings", systemImage: "xmark.circle.fill")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Color.clauxRed)
                Text("Open System Settings → Notifications → Claux to enable.")
                    .font(.system(size: 11))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 16)
            }
            .padding(.top, 2)

        case .notDetermined:
            Button {
                NotificationManager.shared.requestPermission()
                // Refresh status after the system dialog closes.
                DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
                    notifManager.refreshAuthStatus()
                }
            } label: {
                Label("Allow Notifications", systemImage: "bell.badge")
                    .font(.system(size: 13))
            }
            .buttonStyle(.bordered)
            .controlSize(.regular)
            .padding(.top, 2)

        @unknown default:
            EmptyView()
        }
    }

    // MARK: – Helpers

    private var primaryLabel: String {
        switch step {
        case 0: return "Get Started"
        case 1: return "Looks Good"
        case 2:
            switch statusLineManager.inspection.state {
            case .managedReady:
                return "Continue"
            case .managedNeedsRepair:
                return "Repair Integration"
            case .customCommand:
                return "Wrap Existing Command"
            case .notInstalled, .invalidSettings:
                return "Install Integration"
            }
        default: return "All Done"
        }
    }

    private func advance() {
        if step == 2 {
            switch statusLineManager.inspection.state {
            case .managedReady:
                withAnimation(.easeInOut(duration: 0.2)) { step += 1 }
            default:
                if statusLineManager.installOrRepair() {
                    store.refreshNow()
                    withAnimation(.easeInOut(duration: 0.2)) { step += 1 }
                }
            }
        } else if step < 3 {
            withAnimation(.easeInOut(duration: 0.2)) { step += 1 }
        } else {
            withAnimation(.easeOut(duration: 0.3)) { onboardingCompleted = true }
        }
    }

    private func chooseDirectory() {
        let panel = NSOpenPanel()
        panel.canChooseFiles          = false
        panel.canChooseDirectories    = true
        panel.allowsMultipleSelection = false
        panel.prompt                  = "Select"
        if panel.runModal() == .OK, let url = panel.url {
            watchDirectory = url.path
        }
    }

    private var integrationStatusTitle: String {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return "Integration ready"
        case .managedNeedsRepair:
            return "Needs repair"
        case .notInstalled:
            return "Not installed"
        case .customCommand:
            return "Custom command will be wrapped"
        case .invalidSettings:
            return "Settings file needs attention"
        }
    }

    private var integrationStatusIcon: String {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return "checkmark.circle.fill"
        case .managedNeedsRepair:
            return "wrench.and.screwdriver"
        case .notInstalled:
            return "square.and.arrow.down"
        case .customCommand:
            return "arrow.triangle.branch"
        case .invalidSettings:
            return "exclamationmark.triangle"
        }
    }

    private var integrationStatusColor: Color {
        switch statusLineManager.inspection.state {
        case .managedReady:
            return .clauxGreen
        case .managedNeedsRepair, .notInstalled, .customCommand:
            return Color(nsColor: .systemOrange)
        case .invalidSettings:
            return .clauxRed
        }
    }
}
