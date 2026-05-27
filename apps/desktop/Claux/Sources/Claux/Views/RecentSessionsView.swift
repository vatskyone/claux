import SwiftUI

struct RecentSessionsView: View {
    let sessions: [ClaudeSession]
    let onSelect: (ClaudeSession) -> Void

    @State private var searchText: String = ""

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
                SessionSearchField(text: $searchText)
            }

            // ── Session list or empty state ───────────────────────────────────
            if filteredSessions.isEmpty {
                emptyState
            } else {
                sessionList(filteredSessions)
            }
        }
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

// MARK: – Native macOS search field
// Wraps NSSearchField so we get the built-in magnifying-glass icon and
// the clear (×) button that SwiftUI's plain TextField doesn't provide.
private struct SessionSearchField: NSViewRepresentable {
    @Binding var text: String

    func makeNSView(context: Context) -> NSSearchField {
        let field = NSSearchField()
        field.placeholderString = "Search sessions…"
        field.delegate = context.coordinator
        field.controlSize = .small
        field.focusRingType = .none
        return field
    }

    func updateNSView(_ field: NSSearchField, context: Context) {
        // Sync state back from SwiftUI (e.g. programmatic clear) without
        // triggering an infinite loop on every keystroke.
        if field.stringValue != text {
            field.stringValue = text
        }
    }

    func makeCoordinator() -> Coordinator { Coordinator(text: $text) }

    class Coordinator: NSObject, NSSearchFieldDelegate {
        @Binding var text: String
        init(text: Binding<String>) { _text = text }

        func controlTextDidChange(_ notification: Notification) {
            guard let field = notification.object as? NSSearchField else { return }
            text = field.stringValue
        }
    }
}
