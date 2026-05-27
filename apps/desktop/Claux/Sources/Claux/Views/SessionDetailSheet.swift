import SwiftUI

struct SessionDetailSheet: View {
    let session: ClaudeSession
    let onDismiss: () -> Void

    @State private var pathCopied = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // ── Header ────────────────────────────────────────────────────────
            HStack(alignment: .top, spacing: 10) {
                VStack(alignment: .leading, spacing: 4) {
                    // Session title (AI-generated) or fallback to display path
                    if let title = session.title {
                        Text(title)
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundStyle(Color(nsColor: .labelColor))
                            .lineLimit(2)
                            .fixedSize(horizontal: false, vertical: true)
                    } else {
                        Text(session.displayPath)
                            .font(.system(size: 13, weight: .semibold, design: .monospaced))
                            .foregroundStyle(Color(nsColor: .labelColor))
                            .lineLimit(1)
                    }

                    HStack(spacing: 6) {
                        // Model badge
                        Text(ModelInfo.shortName(session.model))
                            .font(.system(size: 10, weight: .medium))
                            .foregroundStyle(ModelInfo.color(session.model))
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(ModelInfo.color(session.model).opacity(0.12))
                            .clipShape(Capsule())

                        // Entrypoint badge
                        if let ep = session.entrypointLabel {
                            Text(ep)
                                .font(.system(size: 10, weight: .medium))
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color(nsColor: .separatorColor).opacity(0.4))
                                .clipShape(Capsule())
                        }

                        // Active badge
                        if session.isActive {
                            HStack(spacing: 4) {
                                Circle()
                                    .fill(Color(nsColor: .systemGreen))
                                    .frame(width: 5, height: 5)
                                Text("Active")
                                    .font(.system(size: 10, weight: .medium))
                                    .foregroundStyle(Color(nsColor: .systemGreen))
                            }
                        }
                    }
                }

                Spacer()

                Button { onDismiss() } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 18))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                }
                .buttonStyle(.plain)
            }
            .padding(16)

            Divider()

            // ── Stats grid ────────────────────────────────────────────────────
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 0) {
                statCell(label: "Cost",      value: Format.cost(session.totalCost),                    icon: "dollarsign.circle")
                statCell(label: "Duration",  value: Format.duration(session.duration),                 icon: "clock")
                statCell(label: "Burn rate", value: Format.cost(session.burnRatePerHour) + "/hr",      icon: "flame")
                statCell(label: "Projected", value: Format.cost(session.projectedCost),                icon: "arrow.up.right")
                statCell(label: "Context",   value: String(format: "%.0f%%", session.contextHealthFraction * 100), icon: "square.stack")
                statCell(label: "Cache hit", value: String(format: "%.0f%%", session.cacheHitRate * 100), icon: "bolt.fill")
            }

            Divider()

            // ── Token breakdown ───────────────────────────────────────────────
            VStack(alignment: .leading, spacing: 8) {
                Text("Tokens")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                    .tracking(0.5)
                    .textCase(.uppercase)

                tokenRow(label: "Input",        count: session.tokenUsage.inputTokens,      color: Color(nsColor: .labelColor))
                tokenRow(label: "Output",       count: session.tokenUsage.outputTokens,     color: Color(nsColor: .secondaryLabelColor))
                tokenRow(label: "Cache read",   count: session.tokenUsage.cacheReadTokens,  color: Color(nsColor: .systemBlue))
                tokenRow(label: "Cache write",  count: session.tokenUsage.cacheWriteTokens, color: Color(nsColor: .systemGreen))

                if session.tokenUsage.thinkingTokens > 0 {
                    tokenRow(label: "Thinking", count: session.tokenUsage.thinkingTokens, color: Color(nsColor: .systemPurple))
                }
            }
            .padding(16)

            Divider()

            // ── Footer: path + copy ───────────────────────────────────────────
            HStack(spacing: 8) {
                Text(session.projectPath)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    .lineLimit(1)
                    .truncationMode(.middle)

                Spacer()

                Button {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(session.projectPath, forType: .string)
                    withAnimation { pathCopied = true }
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.8) {
                        withAnimation { pathCopied = false }
                    }
                } label: {
                    Label(pathCopied ? "Copied!" : "Copy path",
                          systemImage: pathCopied ? "checkmark" : "doc.on.doc")
                        .font(.system(size: 11))
                        .foregroundStyle(pathCopied ? Color(nsColor: .systemGreen) : Color(nsColor: .systemBlue))
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        // Full 340-width to match the popover — the overlay in PopoverView sets the frame
        .background(Color(nsColor: .windowBackgroundColor))
    }

    // MARK: – Sub-views

    private func statCell(label: String, value: String, icon: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 1) {
                Text(label)
                    .font(.system(size: 10))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                Text(value)
                    .font(.system(size: 13, weight: .semibold, design: .rounded))
                    .monospacedDigit()
                    .foregroundStyle(Color(nsColor: .labelColor))
            }

            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(Color(nsColor: .controlBackgroundColor))
        .overlay(
            Rectangle()
                .fill(Color(nsColor: .separatorColor).opacity(0.3))
                .frame(height: 0.5),
            alignment: .bottom
        )
    }

    private func tokenRow(label: String, count: Int, color: Color) -> some View {
        HStack {
            Circle().fill(color).frame(width: 5, height: 5)
            Text(label)
                .font(.system(size: 11))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
            Spacer()
            Text(Format.tokens(count))
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .foregroundStyle(color)
        }
    }
}
