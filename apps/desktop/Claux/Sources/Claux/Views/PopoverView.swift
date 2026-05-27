import SwiftUI
import AppKit

struct PopoverView: View {
    @EnvironmentObject var store: AppStore
    @Environment(\.openWindow) private var openWindow

    // The session currently shown in the detail overlay (nil = hidden)
    @State private var selectedSession: ClaudeSession? = nil

    // Onboarding: shown on first launch, dismissed after the user completes the flow.
    // Migration guard: if the user already has app settings stored (i.e. they were
    // using the app before onboarding was added), mark it complete automatically.
    @AppStorage("onboardingCompleted") private var onboardingCompleted: Bool = false

    var body: some View {
        ZStack {
            // ── Main content + session detail overlay ────────────────────────
            ZStack(alignment: .bottom) {
                VStack(spacing: 0) {
                    header
                    Divider()

                    VStack(spacing: 8) {
                        if let session = store.activeSession {
                            ActiveSessionCard(session: session)
                        } else {
                            NoActiveSessionView()
                        }

                        SpendSummaryView(summary: store.spendSummary,
                                        sparkData: Array(store.dailySpend.suffix(7)))

                        RecentSessionsView(sessions: store.recentSessions) { session in
                            withAnimation(.easeInOut(duration: 0.18)) {
                                selectedSession = session
                            }
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)

                    Divider()
                    bottomBar
                }

                // Session detail overlay — slides up from bottom when a row is tapped.
                if let session = selectedSession {
                    Color.black.opacity(0.25)
                        .ignoresSafeArea()
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.18)) { selectedSession = nil }
                        }
                        .transition(.opacity)

                    SessionDetailSheet(session: session) {
                        withAnimation(.easeInOut(duration: 0.18)) { selectedSession = nil }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 10))
                    .shadow(color: .black.opacity(0.18), radius: 12, y: -4)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                }
            }

            // ── Onboarding overlay (first launch only) ───────────────────────
            // Covers the entire popover until the user completes the 3-step flow.
            if !onboardingCompleted {
                OnboardingView()
                    .transition(.opacity)
                    .zIndex(10)
            }
        }
        .frame(width: 340)
        .background(Color(nsColor: .windowBackgroundColor))
        // Refresh session data whenever the popover window gains focus
        // (i.e. every time the user clicks the menu bar icon to open it).
        // This ensures the spend totals and session list are never stale.
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didBecomeKeyNotification)) { _ in
            store.refreshNow()
            // Migration: existing users (identifiable by already having stored settings)
            // skip the onboarding they've never seen before.
            if !onboardingCompleted,
               UserDefaults.standard.object(forKey: "autoRefreshInterval") != nil {
                onboardingCompleted = true
            }
        }
    }

    // MARK: – Header
    // ZStack lets "Live / Idle" sit at true horizontal center while
    // the title stays left-aligned and the refresh button stays right-aligned.
    private var header: some View {
        ZStack {
            // Center: Live / Idle status
            HStack(spacing: 5) {
                ActiveDot(isActive: store.activeSession != nil)
                Text(store.activeSession != nil ? "Live" : "Idle")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(
                        store.activeSession != nil
                            ? Color(nsColor: .systemBlue)
                            : Color(nsColor: .secondaryLabelColor)
                    )
            }

            // Left: title
            HStack {
                HStack(alignment: .firstTextBaseline, spacing: 6) {
                    Text("CLAUX")
                        .font(.system(size: 15, weight: .bold))
                        .foregroundStyle(Color(nsColor: .labelColor))
                    Text("v\(AppVersion.current)")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                }
                Spacer()
            }

            // Right: refresh
            HStack {
                Spacer()
                Button {
                    store.refreshNow()
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        .frame(width: 22, height: 22)
                }
                .buttonStyle(.plain)
                .help("Refresh sessions now")
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: – Bottom bar
    private var bottomBar: some View {
        HStack(spacing: 14) {
            Button {
                NSApp.activate(ignoringOtherApps: true)
                openWindow(id: "settings")
            } label: {
                HStack(spacing: 5) {
                    Image(systemName: "gearshape")
                        .font(.system(size: 11))
                    Text("Settings…")
                        .font(.system(size: 13))
                }
                .foregroundStyle(Color(nsColor: .labelColor))
            }
            .buttonStyle(.plain)

            Button {
                NSApp.activate(ignoringOtherApps: true)
                openWindow(id: "analytics")
            } label: {
                HStack(spacing: 5) {
                    Image(systemName: "chart.bar")
                        .font(.system(size: 11))
                    Text("Analytics")
                        .font(.system(size: 13))
                }
                .foregroundStyle(Color(nsColor: .labelColor))
            }
            .buttonStyle(.plain)

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(Color(nsColor: .windowBackgroundColor))
    }
}

// MARK: – Status dot (blue = live, gray = idle)
struct ActiveDot: View {
    let isActive: Bool
    @State private var pulse = false

    var body: some View {
        ZStack {
            if isActive {
                Circle()
                    .fill(Color(nsColor: .systemBlue).opacity(0.35))
                    .frame(width: 12, height: 12)
                    .scaleEffect(pulse ? 2.2 : 1.0)
                    .opacity(pulse ? 0 : 1)
                    .animation(
                        .easeOut(duration: 1.6).repeatForever(autoreverses: false),
                        value: pulse
                    )
            }
            Circle()
                .fill(isActive ? Color(nsColor: .systemBlue) : Color(nsColor: .tertiaryLabelColor))
                .frame(width: 7, height: 7)
        }
        .onAppear { pulse = true }
    }
}
