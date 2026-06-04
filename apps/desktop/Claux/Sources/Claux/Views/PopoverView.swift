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
    @State private var selectedDailyRecap: DailyRecap? = nil
    @State private var activeTab: PopoverTab = .dashboard

    @AppStorage("onboardingCompleted") private var onboardingCompleted: Bool = false
    @AppStorage("stateColorPreset") private var stateColorPreset: String = StateColorPreset.system.rawValue

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
                if let recap = selectedDailyRecap {
                    Color.black.opacity(0.25)
                        .ignoresSafeArea()
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.18)) { selectedDailyRecap = nil }
                        }
                        .transition(.opacity)

                    DailyRecapSheet(recap: recap) {
                        withAnimation(.easeInOut(duration: 0.18)) { selectedDailyRecap = nil }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 10))
                    .shadow(color: .black.opacity(0.18), radius: 12, y: -4)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                } else if let session = selectedSession {
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
        .id(stateColorPreset)
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didBecomeKeyNotification)) { _ in
            store.refreshNow()
        }
        .onReceive(NotificationCenter.default.publisher(for: .clauxOpenDailyRecap)) { notification in
            let dayKey = notification.userInfo?["dayKey"] as? String ?? Format.dayKey(Date())
            guard let recap = store.dailyRecap(forDayKey: dayKey) else { return }
            withAnimation(.easeInOut(duration: 0.18)) {
                activeTab = .dashboard
                selectedSession = nil
                selectedDailyRecap = recap
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .clauxOpenDashboard)) { _ in
            withAnimation(.easeInOut(duration: 0.18)) {
                activeTab = .dashboard
                selectedSession = nil
                selectedDailyRecap = nil
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .clauxOpenSession)) { notification in
            guard let sessionID = notification.userInfo?["sessionID"] as? String,
                  let session = store.session(idString: sessionID)
            else { return }

            withAnimation(.easeInOut(duration: 0.18)) {
                activeTab = session.isActive ? .dashboard : .history
                selectedDailyRecap = nil
                selectedSession = session
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

    // MARK: – Dashboard tab — session card + CLAUX logo watermark at bottom

    private var dashboardContent: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(spacing: 8) {
                if let session = store.activeSession {
                    ActiveSessionCard(session: session)
                } else {
                    NoActiveSessionView()
                }

                PlanLimitsCard(
                    snapshot: store.planLimits,
                    diagnostics: store.planLimitsDiagnostics
                )
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
        }
        .frame(height: tabHeight)
    }

    // MARK: – Analytics tab — spend totals (no sparkline) + compact analytics

    private var analyticsContent: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(spacing: 0) {
                // Sparkline hidden here — the chart below already shows spend trends.
                SpendSummaryView(summary: store.spendSummary, sparkData: [])
                    .padding(.horizontal, 12)
                    .padding(.top, 10)
                    .padding(.bottom, 6)

                Divider()
                    .padding(.horizontal, 12)

                CompactAnalyticsView()
                    .padding(.horizontal, 12)
                    .padding(.top, 6)
                    .padding(.bottom, 10)

                Button {
                    NSApp.activate(ignoringOtherApps: true)
                    if let open = clauxOpenWindow {
                        open("analytics")
                    } else {
                        openWindow(id: "analytics")
                    }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "arrow.up.left.and.arrow.down.right")
                            .font(.system(size: 10))
                        Text("Open full Analytics window")
                            .font(.system(size: 11))
                    }
                    .foregroundStyle(Color.clauxBlue)
                    .padding(.vertical, 8)
                }
                .buttonStyle(.plain)
                .padding(.bottom, 4)
            }
        }
        .frame(height: tabHeight)
    }

    // MARK: – History tab — sticky header + scrollable list

    private var historyContent: some View {
        HistoryTabView(sessions: store.recentSessions,
                       tabHeight: tabHeight) { session in
            withAnimation(.easeInOut(duration: 0.18)) {
                selectedSession = session
            }
        }
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
                            ? Color.clauxBlue
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
                    guard let session = store.activeSession else { return }
                    withAnimation(.easeInOut(duration: 0.18)) {
                        selectedSession = session
                    }
                } label: {
                    Image(systemName: "chart.bar.doc.horizontal")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(
                            store.activeSession != nil
                                ? Color(nsColor: .secondaryLabelColor)
                                : Color(nsColor: .tertiaryLabelColor)
                        )
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .disabled(store.activeSession == nil)
                .help(store.activeSession != nil ? "Open current session details" : "No active session")

                Button {
                    store.refreshNow()
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .help("Refresh sessions and plan limits")

                Button {
                    NSApp.activate(ignoringOtherApps: true)
                    if let open = clauxOpenWindow {
                        open("settings")
                    } else {
                        openWindow(id: "settings")
                    }
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

// MARK: – History tab: sticky header + scrollable session list

private struct HistoryTabView: View {
    let sessions: [ClaudeSession]
    let tabHeight: CGFloat
    let onSelect: (ClaudeSession) -> Void

    @State private var searchText: String = ""
    @FocusState private var searchFocused: Bool

    private var filtered: [ClaudeSession] {
        guard !searchText.isEmpty else { return sessions }
        let q = searchText.lowercased()
        return sessions.filter {
            ($0.title?.lowercased().contains(q) ?? false) ||
            $0.projectPath.lowercased().contains(q)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // ── Sticky header (does not scroll) ──────────────────────────────
            VStack(alignment: .leading, spacing: 5) {
                HStack {
                    Text("Recent")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                        .tracking(0.5)
                        .textCase(.uppercase)
                    Spacer()
                    let badge = searchText.isEmpty
                        ? "\(sessions.count)"
                        : "\(filtered.count)/\(sessions.count)"
                    Text(badge)
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        .padding(.horizontal, 6)
                        .padding(.vertical, 1)
                        .background(Color(nsColor: .separatorColor).opacity(0.5))
                        .clipShape(Capsule())
                        .animation(.easeInOut(duration: 0.15), value: searchText)
                }
                .padding(.horizontal, 2)

                if sessions.count >= 2 {
                    searchBar
                }
            }
            .padding(.horizontal, 12)
            .padding(.top, 10)
            .padding(.bottom, 6)

            // ── Scrollable session list ───────────────────────────────────────
            ScrollView(.vertical, showsIndicators: false) {
                if filtered.isEmpty {
                    emptyState
                        .padding(.horizontal, 12)
                        .padding(.bottom, 10)
                } else {
                    sessionList
                        .padding(.horizontal, 12)
                        .padding(.bottom, 10)
                }
            }
        }
        .frame(height: tabHeight)
    }

    // MARK: – Search bar (same style as RecentSessionsView)

    private var searchBar: some View {
        ZStack {
            HStack(spacing: 5) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 12))
                Text("Search sessions…")
                    .font(.system(size: 12))
            }
            .foregroundStyle(Color.secondary)
            .opacity(searchFocused || !searchText.isEmpty ? 0 : 1)

            HStack(spacing: 5) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 12))
                    .foregroundStyle(Color.secondary)
                TextField("", text: $searchText)
                    .textFieldStyle(.plain)
                    .font(.system(size: 12))
                    .focused($searchFocused)
                Spacer(minLength: 0)
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                        searchFocused = false
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 11))
                            .foregroundStyle(Color.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 10)
            .opacity(searchFocused || !searchText.isEmpty ? 1 : 0)
        }
        .frame(height: 26)
        .background(.regularMaterial)
        .clipShape(Capsule())
        .overlay(Capsule().stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5))
        .contentShape(Capsule())
        .onTapGesture { searchFocused = true }
        .animation(.easeInOut(duration: 0.18), value: searchFocused)
        .animation(.easeInOut(duration: 0.18), value: searchText.isEmpty)
    }

    // MARK: – Session rows

    private var sessionList: some View {
        VStack(spacing: 0) {
            ForEach(filtered.indices, id: \.self) { i in
                SessionRowView(session: filtered[i], onSelect: onSelect)
                if i < filtered.count - 1 {
                    Divider()
                }
            }
        }
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5)
        )
        .animation(.easeInOut(duration: 0.15), value: searchText)
    }

    // MARK: – Empty states

    @ViewBuilder
    private var emptyState: some View {
        if sessions.isEmpty {
            Text("No recent sessions")
                .font(.system(size: 12))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.vertical, 20)
        } else {
            VStack(spacing: 6) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 18, weight: .light))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                Text("No results for \"\(searchText)\"")
                    .font(.system(size: 12))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 20)
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5)
            )
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
                    .fill(Color.clauxBlue.opacity(0.35))
                    .frame(width: 12, height: 12)
                    .scaleEffect(pulse ? 2.2 : 1.0)
                    .opacity(pulse ? 0 : 1)
                    .animation(
                        .easeOut(duration: 1.6).repeatForever(autoreverses: false),
                        value: pulse
                    )
            }
            Circle()
                .fill(isActive ? Color.clauxBlue : Color(nsColor: .tertiaryLabelColor))
                .frame(width: 7, height: 7)
        }
        .onAppear { pulse = true }
    }
}
