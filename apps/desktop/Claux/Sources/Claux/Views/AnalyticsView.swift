import SwiftUI
import Charts

struct AnalyticsView: View {
    @EnvironmentObject var store: AppStore
    @State private var chartRange: ChartRange = .thirtyDays
    @State private var selectedDay: DailySpend?
    /// Raw cursor X position within the full chart frame (used to position the tooltip).
    @State private var hoverX: CGFloat = 0

    enum ChartRange: String, CaseIterable {
        case sevenDays  = "7D"
        case thirtyDays = "30D"

        var days: Int { self == .sevenDays ? 7 : 30 }
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                spendChartSection
                Divider()
                projectSection
                Divider()
                modelSection
            }
            .padding(20)
        }
        .frame(width: 540, height: 640)
        .nativeBlurBackground(material: .sidebar)
        .navigationTitle("Analytics")
    }

    // MARK: – Daily spend chart

    private var chartDays: [DailySpend] {
        let cutoff = Calendar.current.date(
            byAdding: .day, value: -(chartRange.days - 1), to: Calendar.current.startOfDay(for: Date())
        ) ?? .distantPast
        return store.dailySpend.filter { $0.date >= cutoff }
    }

    /// Maximum cost across visible days — used to lock the Y scale and colour bars.
    private var maxCost: Double {
        chartDays.map(\.cost).max() ?? 1
    }

    private var spendChartSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header row
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Spend Over Time")
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundStyle(Color(nsColor: .labelColor))
                    Text("Total: \(Format.cost(chartDays.reduce(0) { $0 + $1.cost }))")
                        .font(.system(size: 11))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                }
                Spacer()
                Picker("Range", selection: $chartRange) {
                    ForEach(ChartRange.allCases, id: \.self) { r in
                        Text(r.rawValue).tag(r)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 80)
                .onChange(of: chartRange) { _ in selectedDay = nil }
            }

            // ── Chart ──────────────────────────────────────────────────────
            // The RuleMark has NO annotation — that was the cause of layout shifts.
            // The tooltip is rendered inside the chartOverlay so it sits on top of
            // the chart without affecting its layout or size.
            Chart {
                ForEach(chartDays) { day in
                    BarMark(
                        x: .value("Date", day.date, unit: .day),
                        y: .value("Cost", day.cost)
                    )
                    .foregroundStyle(barColor(for: day.cost))
                    .cornerRadius(3)
                }

                if let sel = selectedDay {
                    RuleMark(x: .value("Selected", sel.date, unit: .day))
                        .foregroundStyle(Color(nsColor: .separatorColor))
                        .lineStyle(StrokeStyle(lineWidth: 1, dash: [4]))
                    // ↑ No .annotation here — tooltip is in the overlay below
                }
            }
            // Lock Y domain so bars never shift when tooltip appears/disappears
            .chartYScale(domain: 0...(max(maxCost, 0.01) * 1.15))
            .chartXAxis {
                AxisMarks(values: .stride(by: .day, count: chartRange == .sevenDays ? 1 : 7)) { _ in
                    AxisValueLabel(format: .dateTime.month(.abbreviated).day())
                        .font(.system(size: 9))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5))
                        .foregroundStyle(Color(nsColor: .separatorColor).opacity(0.3))
                }
            }
            .chartYAxis {
                AxisMarks(position: .leading) { val in
                    if let d = val.as(Double.self) {
                        AxisValueLabel {
                            Text(Format.cost(d))
                                .font(.system(size: 9))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        }
                        AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5))
                            .foregroundStyle(Color(nsColor: .separatorColor).opacity(0.3))
                    }
                }
            }
            // ── Overlay: cursor tracking + floating tooltip ────────────────
            // `geo[proxy.plotAreaFrame]` gives the plot area rect inside the
            // full chart frame (accounts for Y-axis label width offset).
            .chartOverlay { proxy in
                GeometryReader { geo in
                    let plotArea = geo[proxy.plotAreaFrame]

                    Rectangle()
                        .fill(.clear)
                        .contentShape(Rectangle())
                        .onContinuousHover { phase in
                            switch phase {
                            case .active(let location):
                                hoverX = location.x
                                // Translate full-chart X → plot-relative X before querying
                                let plotRelativeX = location.x - plotArea.minX
                                if let date: Date = proxy.value(atX: plotRelativeX) {
                                    let day = Calendar.current.startOfDay(for: date)
                                    selectedDay = chartDays.first {
                                        Calendar.current.startOfDay(for: $0.date) == day
                                    }
                                }
                            case .ended:
                                selectedDay = nil
                            }
                        }

                    // Floating tooltip — positioned at cursor X, near the top.
                    // Rendered inside the overlay so it never affects chart layout.
                    if let sel = selectedDay {
                        let tooltipW: CGFloat = 78
                        let tooltipH: CGFloat = 38
                        // Clamp so the bubble doesn't bleed outside the chart edges
                        let clampedX = min(
                            max(hoverX, plotArea.minX + tooltipW / 2),
                            plotArea.maxX - tooltipW / 2
                        )

                        VStack(spacing: 2) {
                            Text(sel.date.formatted(.dateTime.month(.abbreviated).day()))
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            Text(Format.cost(sel.cost))
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                                .foregroundStyle(Color(nsColor: .labelColor))
                        }
                        .frame(width: tooltipW, height: tooltipH)
                        .background(Color(nsColor: .controlBackgroundColor))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .stroke(Color(nsColor: .separatorColor).opacity(0.45), lineWidth: 0.5)
                        )
                        .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 2)
                        // Position: follow cursor X, pin near top of chart
                        .position(x: clampedX, y: tooltipH / 2 + 4)
                        // Animate in/out smoothly
                        .transition(.opacity.combined(with: .scale(scale: 0.9)))
                        .animation(.easeOut(duration: 0.12), value: sel.date)
                    }
                }
            }
            .frame(height: 160)
        }
    }

    private func barColor(for cost: Double) -> Color {
        guard maxCost > 0 else { return Color(nsColor: .systemBlue) }
        let fraction = cost / maxCost
        if fraction < 0.5 { return Color(nsColor: .systemBlue) }
        if fraction < 0.8 { return Color(nsColor: .systemOrange) }
        return Color(nsColor: .systemRed)
    }

    // MARK: – Project breakdown

    private var projectSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("By Project")
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Color(nsColor: .labelColor))

            if store.projectBreakdown.isEmpty {
                emptyLabel("No sessions recorded")
            } else {
                let maxCostP = store.projectBreakdown.first?.totalCost ?? 1
                VStack(spacing: 6) {
                    ForEach(store.projectBreakdown.prefix(8)) { proj in
                        HStack(spacing: 10) {
                            Text(proj.displayPath)
                                .font(.system(size: 11, design: .monospaced))
                                .foregroundStyle(Color(nsColor: .labelColor))
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .frame(width: 180, alignment: .leading)

                            GeometryReader { geo in
                                ZStack(alignment: .leading) {
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(Color(nsColor: .separatorColor).opacity(0.3))
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(Color(nsColor: .systemBlue).opacity(0.7))
                                        .frame(width: max(4, geo.size.width * (proj.totalCost / maxCostP)))
                                }
                            }
                            .frame(height: 10)

                            Text(Format.cost(proj.totalCost))
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                                .monospacedDigit()
                                .foregroundStyle(Color(nsColor: .labelColor))
                                .frame(width: 52, alignment: .trailing)

                            Text("\(proj.sessionCount) session\(proj.sessionCount == 1 ? "" : "s")")
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                                .frame(width: 60, alignment: .trailing)
                        }
                    }
                }
            }
        }
    }

    // MARK: – Model breakdown

    private var modelSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("By Model")
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Color(nsColor: .labelColor))

            if store.modelBreakdown.isEmpty {
                emptyLabel("No sessions recorded")
            } else {
                let maxCostM = store.modelBreakdown.first?.totalCost ?? 1
                VStack(spacing: 6) {
                    ForEach(store.modelBreakdown) { item in
                        HStack(spacing: 10) {
                            HStack(spacing: 5) {
                                Circle()
                                    .fill(ModelInfo.color(item.model))
                                    .frame(width: 7, height: 7)
                                Text(item.displayName)
                                    .font(.system(size: 11, weight: .medium))
                                    .foregroundStyle(Color(nsColor: .labelColor))
                            }
                            .frame(width: 100, alignment: .leading)

                            GeometryReader { geo in
                                ZStack(alignment: .leading) {
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(Color(nsColor: .separatorColor).opacity(0.3))
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(ModelInfo.color(item.model).opacity(0.7))
                                        .frame(width: max(4, geo.size.width * (item.totalCost / maxCostM)))
                                }
                            }
                            .frame(height: 10)

                            Text(Format.cost(item.totalCost))
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                                .monospacedDigit()
                                .foregroundStyle(Color(nsColor: .labelColor))
                                .frame(width: 52, alignment: .trailing)

                            Text("\(item.sessionCount) session\(item.sessionCount == 1 ? "" : "s")")
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                                .frame(width: 60, alignment: .trailing)
                        }
                    }
                }
            }
        }
    }

    // MARK: – Helpers

    private func emptyLabel(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 12))
            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
            .frame(maxWidth: .infinity, alignment: .center)
            .padding(.vertical, 12)
    }
}

// MARK: – Compact analytics — optimised for the 340 pt popover

struct CompactAnalyticsView: View {
    @EnvironmentObject var store: AppStore
    @State private var chartRange: AnalyticsView.ChartRange = .thirtyDays
    @State private var selectedDay: DailySpend?
    @State private var hoverX: CGFloat = 0

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            spendChartSection
            Divider()
            projectSection
            Divider()
            modelSection
        }
    }

    // MARK: – Chart

    private var chartDays: [DailySpend] {
        let cutoff = Calendar.current.date(
            byAdding: .day, value: -(chartRange.days - 1),
            to: Calendar.current.startOfDay(for: Date())
        ) ?? .distantPast
        return store.dailySpend.filter { $0.date >= cutoff }
    }

    private var maxCost: Double { chartDays.map(\.cost).max() ?? 1 }

    private var spendChartSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Spend Over Time")
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(Color(nsColor: .labelColor))
                    Text("Total: \(Format.cost(chartDays.reduce(0) { $0 + $1.cost }))")
                        .font(.system(size: 11))
                        .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                }
                Spacer()
                Picker("Range", selection: $chartRange) {
                    ForEach(AnalyticsView.ChartRange.allCases, id: \.self) { r in
                        Text(r.rawValue).tag(r)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 72)
                .onChange(of: chartRange) { _ in selectedDay = nil }
            }

            Chart {
                ForEach(chartDays) { day in
                    BarMark(
                        x: .value("Date", day.date, unit: .day),
                        y: .value("Cost", day.cost)
                    )
                    .foregroundStyle(barColor(for: day.cost))
                    .cornerRadius(3)
                }
                if let sel = selectedDay {
                    RuleMark(x: .value("Selected", sel.date, unit: .day))
                        .foregroundStyle(Color(nsColor: .separatorColor))
                        .lineStyle(StrokeStyle(lineWidth: 1, dash: [4]))
                }
            }
            .chartYScale(domain: 0...(max(maxCost, 0.01) * 1.15))
            .chartXAxis {
                AxisMarks(values: .stride(by: .day, count: chartRange == .sevenDays ? 1 : 7)) { _ in
                    AxisValueLabel(format: .dateTime.month(.abbreviated).day())
                        .font(.system(size: 9))
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                    AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5))
                        .foregroundStyle(Color(nsColor: .separatorColor).opacity(0.3))
                }
            }
            .chartYAxis {
                AxisMarks(position: .leading) { val in
                    if let d = val.as(Double.self) {
                        AxisValueLabel {
                            Text(Format.cost(d))
                                .font(.system(size: 9))
                                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        }
                        AxisGridLine(stroke: StrokeStyle(lineWidth: 0.5))
                            .foregroundStyle(Color(nsColor: .separatorColor).opacity(0.3))
                    }
                }
            }
            .chartOverlay { proxy in
                GeometryReader { geo in
                    let plotArea = geo[proxy.plotAreaFrame]
                    Rectangle()
                        .fill(.clear)
                        .contentShape(Rectangle())
                        .onContinuousHover { phase in
                            switch phase {
                            case .active(let location):
                                hoverX = location.x
                                let plotRelativeX = location.x - plotArea.minX
                                if let date: Date = proxy.value(atX: plotRelativeX) {
                                    let day = Calendar.current.startOfDay(for: date)
                                    selectedDay = chartDays.first {
                                        Calendar.current.startOfDay(for: $0.date) == day
                                    }
                                }
                            case .ended:
                                selectedDay = nil
                            }
                        }
                    if let sel = selectedDay {
                        let tooltipW: CGFloat = 78
                        let tooltipH: CGFloat = 38
                        let clampedX = min(
                            max(hoverX, plotArea.minX + tooltipW / 2),
                            plotArea.maxX - tooltipW / 2
                        )
                        VStack(spacing: 2) {
                            Text(sel.date.formatted(.dateTime.month(.abbreviated).day()))
                                .font(.system(size: 10))
                                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                            Text(Format.cost(sel.cost))
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                                .foregroundStyle(Color(nsColor: .labelColor))
                        }
                        .frame(width: tooltipW, height: tooltipH)
                        .background(Color(nsColor: .controlBackgroundColor))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                        .overlay(RoundedRectangle(cornerRadius: 6)
                            .stroke(Color(nsColor: .separatorColor).opacity(0.45), lineWidth: 0.5))
                        .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 2)
                        .position(x: clampedX, y: tooltipH / 2 + 4)
                        .transition(.opacity.combined(with: .scale(scale: 0.9)))
                        .animation(.easeOut(duration: 0.12), value: sel.date)
                    }
                }
            }
            .frame(height: 140)
        }
    }

    private func barColor(for cost: Double) -> Color {
        guard maxCost > 0 else { return Color(nsColor: .systemBlue) }
        let fraction = cost / maxCost
        if fraction < 0.5 { return Color(nsColor: .systemBlue) }
        if fraction < 0.8 { return Color(nsColor: .systemOrange) }
        return Color(nsColor: .systemRed)
    }

    // MARK: – Project breakdown (2-row compact layout)

    private var projectSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("By Project")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(Color(nsColor: .labelColor))

            if store.projectBreakdown.isEmpty {
                emptyLabel("No sessions recorded")
            } else {
                let maxCostP = store.projectBreakdown.first?.totalCost ?? 1
                VStack(spacing: 8) {
                    ForEach(store.projectBreakdown.prefix(5)) { proj in
                        VStack(alignment: .leading, spacing: 3) {
                            HStack {
                                Text(proj.displayPath)
                                    .font(.system(size: 11, design: .monospaced))
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                Text(Format.cost(proj.totalCost))
                                    .font(.system(size: 11, weight: .semibold, design: .rounded))
                                    .monospacedDigit()
                                    .foregroundStyle(Color(nsColor: .labelColor))
                            }
                            HStack(spacing: 6) {
                                GeometryReader { geo in
                                    ZStack(alignment: .leading) {
                                        RoundedRectangle(cornerRadius: 2)
                                            .fill(Color(nsColor: .separatorColor).opacity(0.3))
                                        RoundedRectangle(cornerRadius: 2)
                                            .fill(Color(nsColor: .systemBlue).opacity(0.7))
                                            .frame(width: max(4, geo.size.width * (proj.totalCost / maxCostP)))
                                    }
                                }
                                .frame(height: 6)
                                Text("\(proj.sessionCount) session\(proj.sessionCount == 1 ? "" : "s")")
                                    .font(.system(size: 10))
                                    .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                            }
                        }
                    }
                }
            }
        }
    }

    // MARK: – Model breakdown

    private var modelSection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("By Model")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(Color(nsColor: .labelColor))

            if store.modelBreakdown.isEmpty {
                emptyLabel("No sessions recorded")
            } else {
                let maxCostM = store.modelBreakdown.first?.totalCost ?? 1
                VStack(spacing: 6) {
                    ForEach(store.modelBreakdown) { item in
                        HStack(spacing: 8) {
                            HStack(spacing: 5) {
                                Circle()
                                    .fill(ModelInfo.color(item.model))
                                    .frame(width: 7, height: 7)
                                Text(item.displayName)
                                    .font(.system(size: 11, weight: .medium))
                                    .foregroundStyle(Color(nsColor: .labelColor))
                            }
                            .frame(width: 90, alignment: .leading)

                            GeometryReader { geo in
                                ZStack(alignment: .leading) {
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(Color(nsColor: .separatorColor).opacity(0.3))
                                    RoundedRectangle(cornerRadius: 3)
                                        .fill(ModelInfo.color(item.model).opacity(0.7))
                                        .frame(width: max(4, geo.size.width * (item.totalCost / maxCostM)))
                                }
                            }
                            .frame(height: 10)

                            Text(Format.cost(item.totalCost))
                                .font(.system(size: 11, weight: .semibold, design: .rounded))
                                .monospacedDigit()
                                .foregroundStyle(Color(nsColor: .labelColor))
                                .frame(width: 48, alignment: .trailing)
                        }
                    }
                }
            }
        }
    }

    // MARK: – Helpers

    private func emptyLabel(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 12))
            .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
            .frame(maxWidth: .infinity, alignment: .center)
            .padding(.vertical, 10)
    }
}
