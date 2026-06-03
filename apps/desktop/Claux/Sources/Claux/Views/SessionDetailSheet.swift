import SwiftUI

struct SessionDetailSheet: View {
    let session: ClaudeSession
    let onDismiss: () -> Void

    @State private var pathCopied = false

    private var quality: SessionQualityMetrics {
        session.qualityMetrics
    }

    var body: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .top, spacing: 10) {
                    VStack(alignment: .leading, spacing: 4) {
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
                            Text(ModelInfo.shortName(session.model))
                                .font(.system(size: 10, weight: .medium))
                                .foregroundStyle(ModelInfo.color(session.model))
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(ModelInfo.color(session.model).opacity(0.12))
                                .clipShape(Capsule())

                            if let ep = session.entrypointLabel {
                                Text(ep)
                                    .font(.system(size: 10, weight: .medium))
                                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 2)
                                    .background(Color(nsColor: .separatorColor).opacity(0.4))
                                    .clipShape(Capsule())
                            }

                            if let mode = quality.permissionModeLabel {
                                Text(mode)
                                    .font(.system(size: 10, weight: .medium))
                                    .foregroundStyle(Color.clauxBlue)
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 2)
                                    .background(Color.clauxBlue.opacity(0.12))
                                    .clipShape(Capsule())
                            }

                            if session.isActive {
                                HStack(spacing: 4) {
                                    Circle()
                                        .fill(Color.clauxGreen)
                                        .frame(width: 5, height: 5)
                                    Text("Active")
                                        .font(.system(size: 10, weight: .medium))
                                        .foregroundStyle(Color.clauxGreen)
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

                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 0) {
                    statCell(label: "Cost", value: Format.cost(session.totalCost), icon: "dollarsign.circle")
                    statCell(label: "Duration", value: Format.duration(session.duration), icon: "clock")
                    statCell(label: "Burn rate", value: Format.cost(session.burnRatePerHour) + "/hr", icon: "flame")
                    statCell(label: "Projected", value: Format.cost(session.projectedCost), icon: "arrow.up.right")
                    statCell(label: "Context", value: String(format: "%.0f%%", session.contextHealthFraction * 100), icon: "square.stack")
                    statCell(label: "Cache hit", value: String(format: "%.0f%%", session.cacheHitRate * 100), icon: "bolt.fill")
                }

                Divider()

                VStack(alignment: .leading, spacing: 8) {
                    sectionTitle("Tokens")

                    tokenRow(label: "Input", count: session.tokenUsage.inputTokens, color: Color(nsColor: .labelColor))
                    tokenRow(label: "Output", count: session.tokenUsage.outputTokens, color: Color(nsColor: .secondaryLabelColor))
                    tokenRow(label: "Cache read", count: session.tokenUsage.cacheReadTokens, color: Color.clauxBlue)
                    tokenRow(label: "Cache write", count: session.tokenUsage.cacheWriteTokens, color: Color.clauxGreen)

                    if session.tokenUsage.thinkingTokens > 0 {
                        tokenRow(label: "Thinking", count: session.tokenUsage.thinkingTokens, color: Color(nsColor: .systemPurple))
                    }
                }
                .padding(16)

                Divider()

                VStack(alignment: .leading, spacing: 10) {
                    HStack(alignment: .firstTextBaseline) {
                        sectionTitle("Session quality")
                        Spacer()
                        Text("\(quality.score)")
                            .font(.system(size: 18, weight: .bold, design: .rounded))
                            .monospacedDigit()
                            .foregroundStyle(qualityColor)
                        Text(quality.scoreLabel)
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundStyle(qualityColor)
                    }

                    qualityBar

                    LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 8) {
                        qualityMetricCell(
                            label: "Successful turns",
                            value: ratioText(quality.successfulAssistantTurns, quality.assistantTurns)
                        )
                        qualityMetricCell(
                            label: "Tool results",
                            value: ratioText(quality.successfulToolResults, quality.toolResults)
                        )
                        qualityMetricCell(
                            label: "Accepted edits",
                            value: ratioText(quality.acceptedEdits, quality.editProposals)
                        )
                        qualityMetricCell(
                            label: "Rejected actions",
                            value: "\(quality.rejectedToolResults)"
                        )
                        qualityMetricCell(
                            label: "Accepted as-is",
                            value: "\(quality.acceptedUnmodifiedEdits)"
                        )
                        qualityMetricCell(
                            label: "Files touched",
                            value: "\(quality.touchedFileCount)"
                        )
                    }

                    if quality.completedAgents > 0 || quality.failedAgents > 0 {
                        HStack {
                            Text("Agents")
                                .font(.system(size: 11))
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            Spacer()
                            Text("\(quality.completedAgents) completed")
                                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                                .foregroundStyle(Color(nsColor: .labelColor))
                            if quality.failedAgents > 0 {
                                Text("· \(quality.failedAgents) failed")
                                    .font(.system(size: 11, weight: .semibold, design: .monospaced))
                                    .foregroundStyle(Color.clauxRed)
                            }
                        }
                    }

                    if !quality.touchedFiles.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Touched files")
                                .font(.system(size: 11))
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            ForEach(Array(quality.touchedFiles.prefix(3)), id: \.self) { file in
                                Text(file)
                                    .font(.system(size: 10, design: .monospaced))
                                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                            }
                            if quality.touchedFiles.count > 3 {
                                Text("+\(quality.touchedFiles.count - 3) more")
                                    .font(.system(size: 10))
                                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                            }
                        }
                    }
                }
                .padding(16)

                Divider()

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
                            .foregroundStyle(pathCopied ? Color.clauxGreen : Color.clauxBlue)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 10)
            }
        }
        .frame(maxHeight: 320)
        .background(VisualEffectView(material: .sidebar, blendingMode: .withinWindow))
    }

    private var qualityColor: Color {
        switch quality.score {
        case 85...:
            return Color.clauxGreen
        case 70...:
            return Color.clauxBlue
        case 50...:
            return Color.clauxOrange
        default:
            return Color.clauxRed
        }
    }

    private var qualityBar: some View {
        GeometryReader { geo in
            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color(nsColor: .separatorColor).opacity(0.45))
                Capsule()
                    .fill(qualityColor)
                    .frame(width: max(6, geo.size.width * CGFloat(quality.score) / 100.0))
            }
        }
        .frame(height: 8)
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(.system(size: 11, weight: .semibold))
            .foregroundStyle(Color(nsColor: .secondaryLabelColor))
            .tracking(0.5)
            .textCase(.uppercase)
    }

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
        .background(.regularMaterial)
        .overlay(
            Rectangle()
                .fill(Color(nsColor: .separatorColor).opacity(0.3))
                .frame(height: 0.5),
            alignment: .bottom
        )
    }

    private func qualityMetricCell(label: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(.system(size: 10))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
            Text(value)
                .font(.system(size: 12, weight: .semibold, design: .rounded))
                .monospacedDigit()
                .foregroundStyle(Color(nsColor: .labelColor))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(Color(nsColor: .separatorColor).opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 8))
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

    private func ratioText(_ numerator: Int, _ denominator: Int) -> String {
        guard denominator > 0 else { return "n/a" }
        return "\(numerator) / \(denominator)"
    }
}

private extension SessionQualityMetrics {
    var permissionModeLabel: String? {
        guard let permissionMode else { return nil }
        switch permissionMode {
        case "acceptEdits":
            return "Auto-accept edits"
        case "default":
            return "Manual approvals"
        default:
            return permissionMode
        }
    }
}
