import SwiftUI
import Charts

// Compact spend card: 7-day sparkline → 3 spend cells → optional budget bar.
// sparkData should be the last 7 DailySpend entries from AppStore.dailySpend.
struct SpendSummaryView: View {
    let summary:   SpendSummary
    let sparkData: [DailySpend]   // last 7 days; empty = skip sparkline

    @AppStorage("monthlyBudget") private var monthlyBudget: Double = 0

    var body: some View {
        VStack(spacing: 0) {

            // ── 7-day sparkline ─────────────────────────────────────────────
            if hasSparkData {
                sparklineRow
                Divider()
            }

            // ── Today / This week / This month ─────────────────────────────
            HStack(spacing: 0) {
                SpendCell(label: "Today",      amount: summary.today,     previous: summary.yesterday)
                cellDivider
                SpendCell(label: "This week",  amount: summary.thisWeek,  previous: summary.prevWeek)
                cellDivider
                SpendCell(label: "This month", amount: summary.thisMonth, previous: summary.prevMonth)
            }

            // ── Monthly budget progress bar ─────────────────────────────────
            if monthlyBudget > 0 {
                Divider()
                budgetRow
            }
        }
        .background(Color(nsColor: .controlBackgroundColor))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5)
        )
    }

    // MARK: – Helpers

    private var hasSparkData: Bool {
        sparkData.contains { $0.cost > 0.001 }
    }

    private var cellDivider: some View {
        Rectangle()
            .fill(Color(nsColor: .separatorColor).opacity(0.5))
            .frame(width: 0.5)
            .padding(.vertical, 8)
    }

    // MARK: – Sparkline row

    private var sparklineRow: some View {
        let today   = Calendar.current.startOfDay(for: Date())
        let maxCost = max(0.001, sparkData.map(\.cost).max() ?? 0.001)
        return Chart {
            ForEach(sparkData) { day in
                BarMark(
                    x: .value("Day", day.date, unit: .day),
                    y: .value("Cost", day.cost)
                )
                .foregroundStyle(
                    Calendar.current.isDate(day.date, inSameDayAs: today)
                        ? Color(nsColor: .systemBlue)
                        : Color(nsColor: .systemBlue).opacity(0.28)
                )
                .cornerRadius(2)
            }
        }
        .chartYScale(domain: 0...(maxCost * 1.15))
        .chartXAxis(.hidden)
        .chartYAxis(.hidden)
        .chartLegend(.hidden)
        .frame(height: 30)
        .padding(.horizontal, 12)
        .padding(.top, 8)
        .padding(.bottom, 5)
    }

    // MARK: – Budget bar row

    private var budgetRow: some View {
        let fraction  = min(1.0, summary.thisMonth / monthlyBudget)
        let barColor: Color = fraction < 0.70 ? .clauxGreen
                            : fraction < 0.90 ? .clauxYellow
                            :                   .clauxRed
        let remaining = max(0.0, monthlyBudget - summary.thisMonth)

        return VStack(alignment: .leading, spacing: 5) {
            HStack {
                Text("Monthly budget")
                    .font(.system(size: 10, weight: .medium))
                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                Spacer()
                if remaining > 0.001 {
                    Text(Format.cost(remaining) + " remaining")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                } else {
                    Text("Over budget")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(Color(nsColor: .systemRed))
                }
            }

            GeometryReader { geo in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color(nsColor: .separatorColor).opacity(0.5))
                    RoundedRectangle(cornerRadius: 2)
                        .fill(barColor)
                        .frame(width: max(4, geo.size.width * fraction))
                        .animation(.spring(response: 0.5, dampingFraction: 0.8), value: fraction)
                }
            }
            .frame(height: 4)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

// MARK: – Individual spend cell

private struct SpendCell: View {
    let label:    String
    let amount:   Double
    let previous: Double   // prior-period baseline (0 = no history)

    // ↑ orange = more spending   ↓ green = less spending
    // Hidden when no prior history or change < 5% (noise).
    private var trend: (symbol: String, color: Color, text: String)? {
        guard previous > 0.001 else { return nil }
        let f = (amount - previous) / previous
        guard abs(f) >= 0.05 else { return nil }
        let pct = Int((abs(f) * 100).rounded())
        return f > 0
            ? ("↑", Color(nsColor: .systemOrange), "\(pct)%")
            : ("↓", Color(nsColor: .systemGreen),  "\(pct)%")
    }

    var body: some View {
        VStack(spacing: 2) {
            Text(label)
                .font(.system(size: 10, weight: .medium))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))

            Text(Format.cost(amount))
                .font(.system(size: 15, weight: .bold, design: .rounded))
                .monospacedDigit()
                .foregroundStyle(Color(nsColor: .labelColor))

            // Fixed 12 pt slot — keeps all three cells at the same height.
            ZStack {
                if let t = trend {
                    Text("\(t.symbol) \(t.text)")
                        .font(.system(size: 9, weight: .semibold, design: .monospaced))
                        .foregroundStyle(t.color)
                }
            }
            .frame(height: 12)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 9)
    }
}
