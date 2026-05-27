import SwiftUI
import AppKit

struct SessionRowView: View {
    let session: ClaudeSession
    let onSelect: (ClaudeSession) -> Void

    @State private var hovered = false

    var body: some View {
        Button { onSelect(session) } label: {
            HStack(spacing: 10) {

                // Status dot
                Circle()
                    .fill(dotColor)
                    .frame(width: 6, height: 6)

                // Left: title/path + metadata
                VStack(alignment: .leading, spacing: 2) {
                    // Prefer AI-generated title; fall back to display path
                    Text(session.title ?? session.displayPath)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Color(nsColor: .labelColor))
                        .lineLimit(1)
                        .truncationMode(.middle)

                    HStack(spacing: 3) {
                        Text(Format.relativeTime(session.startTime))
                            .font(.system(size: 10))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))

                        Text("·")
                            .font(.system(size: 9))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))

                        Text(Format.duration(session.duration))
                            .font(.system(size: 10, design: .monospaced))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))

                        Text("·")
                            .font(.system(size: 9))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))

                        Text(ModelInfo.shortName(session.model))
                            .font(.system(size: 10, weight: .medium))
                            .foregroundStyle(ModelInfo.color(session.model))

                        if let ep = session.entrypointLabel {
                            Text("·")
                                .font(.system(size: 9))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                            Text(ep)
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        }
                    }
                }

                Spacer(minLength: 6)

                // Right: cost + chevron hint
                HStack(spacing: 4) {
                    Text(Format.cost(session.totalCost))
                        .font(.system(size: 12, weight: .semibold, design: .rounded))
                        .monospacedDigit()
                        .foregroundStyle(Color(nsColor: .labelColor))

                    Image(systemName: "chevron.right")
                        .font(.system(size: 8, weight: .semibold))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        .opacity(hovered ? 1 : 0)
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 7)
            .background(
                hovered
                    ? Color(nsColor: .selectedContentBackgroundColor).opacity(0.12)
                    : Color.clear
            )
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovered = $0 }
        .contextMenu {
            // ── Primary actions ────────────────────────────────────────────
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(session.projectPath, forType: .string)
            } label: {
                Label("Copy Path", systemImage: "doc.on.doc")
            }

            Button {
                NSWorkspace.shared.open(URL(fileURLWithPath: session.projectPath))
            } label: {
                Label("Show in Finder", systemImage: "folder")
            }

            Divider()

            // ── Debug / power-user ─────────────────────────────────────────
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(session.id.uuidString, forType: .string)
            } label: {
                Label("Copy Session ID", systemImage: "number")
            }
        }
    }

    private var dotColor: Color {
        if session.isActive { return Color(nsColor: .systemGreen) }
        let hoursAgo = -session.startTime.timeIntervalSinceNow / 3600
        if hoursAgo < 6  { return Color(nsColor: .systemBlue) }
        if hoursAgo < 24 { return Color(nsColor: .secondaryLabelColor) }
        return Color(nsColor: .tertiaryLabelColor)
    }
}
