import SwiftUI

struct RecentSessionsView: View {
    let sessions: [ClaudeSession]
    let onSelect: (ClaudeSession) -> Void

    @State private var searchText: String = ""
    @FocusState private var isSearchFocused: Bool

    // Filter by AI title OR raw project path (case-insensitive).
    private var filteredSessions: [ClaudeSession] {
        guard !searchText.isEmpty else { return sessions }
        let q = searchText.lowercased()
        return sessions.filter {
            ($0.title?.lowercased().contains(q) ?? false) ||
            $0.projectPath.lowercased().contains(q)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {

            // ── Section header ────────────────────────────────────────────────
            HStack {
                Text("Recent")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                    .tracking(0.5)
                    .textCase(.uppercase)

                Spacer()

                // Badge shows "filtered/total" when a query is active.
                let badge = searchText.isEmpty
                    ? "\(sessions.count)"
                    : "\(filteredSessions.count)/\(sessions.count)"
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

            // ── Search field ──────────────────────────────────────────────────
            // Shown once there are at least 2 sessions to search through.
            if sessions.count >= 2 {
                searchBar
            }

            // ── Session list or empty state ───────────────────────────────────
            if filteredSessions.isEmpty {
                emptyState
            } else {
                sessionList(filteredSessions)
            }
        }
    }

    // MARK: – Search bar
    // Default: icon + placeholder text centered in the capsule.
    // Active (focused or text entered): icon slides left, cursor appears.

    private var searchBar: some View {
        ZStack {
            // Centered idle state (icon + placeholder together)
            HStack(spacing: 5) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 12))
                Text("Search sessions…")
                    .font(.system(size: 12))
            }
            .foregroundStyle(Color.secondary)
            .opacity(isSearchFocused || !searchText.isEmpty ? 0 : 1)

            // Active state: icon on left, text field
            HStack(spacing: 5) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 12))
                    .foregroundStyle(Color.secondary)
                TextField("", text: $searchText)
                    .textFieldStyle(.plain)
                    .font(.system(size: 12))
                    .focused($isSearchFocused)
                Spacer(minLength: 0)
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                        isSearchFocused = false
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 11))
                            .foregroundStyle(Color.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 10)
            .opacity(isSearchFocused || !searchText.isEmpty ? 1 : 0)
        }
        .frame(height: 26)
        .background(.regularMaterial)
        .clipShape(Capsule())
        .overlay(Capsule().stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5))
        .contentShape(Capsule())
        .onTapGesture { isSearchFocused = true }
        .animation(.easeInOut(duration: 0.18), value: isSearchFocused)
        .animation(.easeInOut(duration: 0.18), value: searchText.isEmpty)
    }

    // MARK: – Session list

    @ViewBuilder
    private func sessionList(_ items: [ClaudeSession]) -> some View {
        VStack(spacing: 0) {
            ForEach(items.indices, id: \.self) { i in
                SessionRowView(session: items[i], onSelect: onSelect)
                if i < items.count - 1 {
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

    // MARK: – Empty state

    @ViewBuilder
    private var emptyState: some View {
        if sessions.isEmpty {
            // No sessions at all — shown when the session list is genuinely empty.
            Text("No recent sessions")
                .font(.system(size: 12))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.vertical, 20)
        } else {
            // Sessions exist but none match the current query.
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

