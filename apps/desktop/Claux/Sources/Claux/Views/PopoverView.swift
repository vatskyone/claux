import SwiftUI
import AppKit

// MARK: – Tab enum

enum PopoverTab: String, CaseIterable {
    case dashboard = "Dashboard"
    case analytics = "Analytics"
    case history   = "History"
}

// MARK: – Popover root

struct PopoverView: View {
    @EnvironmentObject var store: AppStore
    @Environment(\.openWindow) private var openWindow

    @State private var selectedSession: ClaudeSession? = nil
    @State private var activeTab: PopoverTab = .dashboard

    @AppStorage("onboardingCompleted") private var onboardingCompleted: Bool = false

    // Fixed content-area height — keeps the popover the same size on every tab.
    private let tabHeight: CGFloat = 340

    var body: some View {
        ZStack {
            // ── Main content + session detail overlay ────────────────────────
            ZStack(alignment: .bottom) {
                VStack(spacing: 0) {
                    header
                    Divider()
                    tabContent
                    tabBar
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
            if !onboardingCompleted {
                OnboardingView()
                    .transition(.opacity)
                    .zIndex(10)
            }
        }
        .frame(width: 340)
        .nativeBlurBackground(material: .menu)
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didBecomeKeyNotification)) { _ in
            store.refreshNow()
            if !onboardingCompleted,
               UserDefaults.standard.object(forKey: "autoRefreshInterval") != nil {
                onboardingCompleted = true
            }
        }
    }

    // MARK: – Tab content router

    @ViewBuilder
    private var tabContent: some View {
        switch activeTab {
        case .dashboard: dashboardContent
        case .analytics: analyticsContent
        case .history:   historyContent
        }
    }

    // MARK: – Dashboard tab — session card only

    private var dashboardContent: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(spacing: 8) {
                if let session = store.activeSession {
                    ActiveSessionCard(session: session)
                } else {
                    NoActiveSessionView()
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
        }
        .frame(height: tabHeight)
    }

    // MARK: – Analytics tab — spend summary + compact analytics

    private var analyticsContent: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(spacing: 0) {
                SpendSummaryView(summary: store.spendSummary,
                                sparkData: Array(store.dailySpend.suffix(7)))
                    .padding(.horizontal, 12)
                    .padding(.top, 10)
                    .padding(.bottom, 8)

                Divider()
                    .padding(.horizontal, 12)

                CompactAnalyticsView()
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)

                Button {
                    NSApp.activate(ignoringOtherApps: true)
                    openWindow(id: "analytics")
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "arrow.up.left.and.arrow.down.right")
                            .font(.system(size: 10))
                        Text("Open full Analytics window")
                            .font(.system(size: 11))
                    }
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    .padding(.vertical, 8)
                }
                .buttonStyle(.plain)
                .padding(.bottom, 4)
            }
        }
        .frame(height: tabHeight)
    }

    // MARK: – History tab — sessions list + search

    private var historyContent: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(spacing: 8) {
                RecentSessionsView(sessions: store.recentSessions) { session in
                    withAnimation(.easeInOut(duration: 0.18)) {
                        selectedSession = session
                    }
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
        }
        .frame(height: tabHeight)
    }

    // MARK: – Header
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

            // Right: refresh + settings (size 13, 28×28 tap target)
            HStack(spacing: 2) {
                Spacer()
                Button {
                    store.refreshNow()
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .help("Refresh sessions now")

                Button {
                    NSApp.activate(ignoringOtherApps: true)
                    openWindow(id: "settings")
                } label: {
                    Image(systemName: "gearshape")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .help("Settings")
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(Color.clear)
    }

    // MARK: – Tab bar

    private var tabBar: some View {
        VStack(spacing: 0) {
            Divider()
            Picker("", selection: $activeTab) {
                ForEach(PopoverTab.allCases, id: \.self) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
        }
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
