import SwiftUI

struct ActiveSessionCard: View {
    let session: ClaudeSession

    // ── Settings that affect display ────────────────────────────────────────
    @AppStorage("costProjectionPeriod")  private var projPeriod:           String = "monthly"
    @AppStorage("claudemdThreshold")     private var claudemdThreshold:    Int    = 50
    @AppStorage("claudemdAlertEnabled")  private var claudemdAlertEnabled: Bool   = true

    // MARK: – Projection helpers

    /// Burn-rate projected over the selected period (daily / weekly / monthly).
    private var projectedPeriodCost: Double {
        switch projPeriod {
        case "daily":   return session.burnRatePerHour * 24
        case "weekly":  return session.burnRatePerHour * 24 * 7
        default:        return session.burnRatePerHour * 24 * 30  // monthly
        }
    }

    /// Short suffix shown after the projected cost value.
    private var projPeriodSuffix: String {
        switch projPeriod {
        case "daily":  return "/d"
        case "weekly": return "/w"
        default:       return "/m"
        }
    }

    // MARK: – Body

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {

            // ── Section header ──────────────────────────────────────────────
            HStack(spacing: 6) {
                Text("Active Session")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                    .tracking(0.5)
                    .textCase(.uppercase)

                Spacer()

                Text(Format.duration(session.duration))
                    .font(.system(size: 10, weight: .regular, design: .monospaced))
                    .foregroundStyle(Color(nsColor: .systemGray))
            }
            .padding(.horizontal, 2)

            // ── Card ────────────────────────────────────────────────────────
            VStack(alignment: .leading, spacing: 0) {

                // Row 1: path + model badge
                HStack {
                    Text(session.displayPath)
                        .font(.system(size: 12, weight: .medium, design: .monospaced))
                        .foregroundStyle(Color(nsColor: .labelColor))
                        .lineLimit(1)
                        .truncationMode(.middle)

                    Spacer(minLength: 6)

                    Text(ModelInfo.shortName(session.model))
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(ModelInfo.color(session.model))
                        .padding(.horizontal, 7)
                        .padding(.vertical, 2)
                        .background(ModelInfo.color(session.model).opacity(0.12))
                        .clipShape(Capsule())
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 8)

                Divider()

                // Row 2: cost (left) + burn rate & projection (right)
                // Projection uses the user-selected period suffix: /d, /w, or /m
                HStack(alignment: .bottom) {
                    VStack(alignment: .leading, spacing: 1) {
                        Text(Format.cost(session.totalCost))
                            .font(.system(size: 22, weight: .bold, design: .rounded))
                            .foregroundStyle(Color(nsColor: .labelColor))
                            .monospacedDigit()

                        Text("current session")
                            .font(.system(size: 10))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    }

                    Spacer()

                    VStack(alignment: .trailing, spacing: 1) {
                        Text(Format.cost(session.burnRatePerHour) + "/hr")
                            .font(.system(size: 13, weight: .semibold, design: .rounded))
                            .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            .monospacedDigit()

                        // Projected cost with period suffix (e.g. "$4.20/m")
                        Text(Format.cost(projectedPeriodCost) + projPeriodSuffix)
                            .font(.system(size: 10))
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    }
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 8)

                Divider()

                // Row 3: total tokens + thinking breakdown
                // Shows: "42.3K tokens · 35% (12.5K) thinking"
                HStack(spacing: 5) {
                    let totalTok = session.tokenUsage.inputTokens
                                 + session.tokenUsage.outputTokens
                                 + session.tokenUsage.cacheReadTokens
                                 + session.tokenUsage.cacheWriteTokens

                    Text(Format.tokens(totalTok) + " tokens")
                        .font(.system(size: 12))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))

                    if session.tokenUsage.thinkingTokens > 0 {
                        let pct = Int(
                            Double(session.tokenUsage.thinkingTokens)
                            / Double(max(1, totalTok + session.tokenUsage.thinkingTokens))
                            * 100
                        )
                        Text("·")
                            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                            .font(.system(size: 10))

                        HStack(spacing: 3) {
                            Image(systemName: "brain")
                                .font(.system(size: 9))
                                .foregroundStyle(Color(nsColor: .systemBlue))
                            // Shows percentage AND actual count: "35% (12.5K) thinking"
                            Text("\(pct)% (\(Format.tokens(session.tokenUsage.thinkingTokens))) thinking")
                                .font(.system(size: 12))
                                .foregroundStyle(Color(nsColor: .systemBlue))
                        }
                    }
                    Spacer()
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 7)

                Divider()

                // Row 4: cache efficiency (hidden when no cache activity)
                if session.cacheHitRate > 0 {
                    let hitRate = session.cacheHitRate
                    let cacheColor: Color = {
                        if hitRate >= 0.60 { return Color(nsColor: .systemGreen) }
                        if hitRate >= 0.30 { return Color(nsColor: .systemYellow) }
                        return Color(nsColor: .systemRed)
                    }()

                    HStack(spacing: 5) {
                        Text("Cache efficiency")
                            .font(.system(size: 12))
                            .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                        Spacer()
                        Text(String(format: "%.0f%%", hitRate * 100))
                            .font(.system(size: 12, weight: .semibold, design: .monospaced))
                            .foregroundStyle(cacheColor)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 7)

                    Divider()
                }

                // Row 5: context window fill bar
                contextRow

                // Row 6: CLAUDE.md quality bar (only when file exists in project)
                if let score = session.claudemdScore {
                    Divider()
                    claudemdRow(score: score)
                }
            }
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5)
            )
        }
        .onAppear { }
    }

    // MARK: – Context window row

    private var contextRow: some View {
        let fraction  = session.contextHealthFraction
        let barColor: Color = {
            if fraction < 0.70 { return Color(nsColor: .systemBlue) }
            if fraction < 0.90 { return Color(nsColor: .systemYellow) }
            return Color(nsColor: .systemRed)
        }()
        let label: String = {
            if fraction < 0.70 { return "Healthy" }
            if fraction < 0.90 { return "Warning" }
            return "Critical"
        }()

        return VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Context window")
                    .font(.system(size: 11))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                Spacer()
                HStack(spacing: 4) {
                    Text(String(format: "%.0f%%", fraction * 100))
                        .font(.system(size: 11, weight: .semibold, design: .monospaced))
                        .foregroundStyle(barColor)
                    Text("·")
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        .font(.system(size: 10))
                    Text(label)
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(barColor.opacity(0.8))
                }
            }

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 3)
                        .fill(Color(nsColor: .separatorColor).opacity(0.5))
                    RoundedRectangle(cornerRadius: 3)
                        .fill(barColor)
                        .frame(width: max(6, geo.size.width * fraction))
                        .animation(.spring(response: 0.6, dampingFraction: 0.8), value: fraction)
                }
            }
            .frame(height: 4)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    // MARK: – CLAUDE.md quality row

    private func claudemdRow(score: Int) -> some View {
        let barColor: Color = score >= 70 ? Color(nsColor: .systemGreen)
                            : score >= 40 ? Color(nsColor: .systemYellow)
                            :               Color(nsColor: .systemRed)

        let belowThreshold = claudemdAlertEnabled && score < claudemdThreshold

        let label: String = score >= 70 ? "Good"
                          : score >= 40 ? "Basic"
                          :               "Poor"

        return VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("CLAUDE.md quality")
                    .font(.system(size: 11))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                Spacer()
                if belowThreshold {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 9))
                        .foregroundStyle(Color(nsColor: .systemYellow))
                }
                HStack(spacing: 4) {
                    Text("\(score)")
                        .font(.system(size: 11, weight: .semibold, design: .monospaced))
                        .foregroundStyle(barColor)
                    Text("· \(label)")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(barColor.opacity(0.8))
                }
            }

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 3)
                        .fill(Color(nsColor: .separatorColor).opacity(0.5))
                    RoundedRectangle(cornerRadius: 3)
                        .fill(barColor)
                        .frame(width: max(4, geo.size.width * Double(score) / 100.0))
                        .animation(.spring(response: 0.6, dampingFraction: 0.8), value: score)
                }
            }
            .frame(height: 4)

            if belowThreshold {
                Text("Consider updating your CLAUDE.md for better AI context")
                    .font(.system(size: 10))
                    .foregroundStyle(Color(nsColor: .systemYellow).opacity(0.9))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}
