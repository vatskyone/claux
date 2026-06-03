import SwiftUI

// Standalone context health bar (used by legacy callers if any).
// Blue fill = healthy, matching the macOS toggle/accent blue.
struct ContextHealthBar: View {
    let fraction: Double   // 0.0 – 1.0

    private var barColor: Color {
        Color.contextHealthColor(fraction)
    }

    private var statusLabel: String {
        if fraction < 0.70 { return "Healthy" }
        if fraction < 0.90 { return "Warning" }
        return "Critical"
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Context Health")
                    .font(.system(size: 12))
                    .foregroundStyle(Color(nsColor: .secondaryLabelColor))

                Spacer()

                HStack(spacing: 4) {
                    Text(String(format: "%.0f%%", fraction * 100))
                        .font(.system(size: 12, weight: .semibold, design: .monospaced))
                        .foregroundStyle(barColor)

                    Text("·")
                        .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                        .font(.system(size: 11))

                    Text(statusLabel)
                        .font(.system(size: 11, weight: .medium))
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
                        .animation(
                            .spring(response: 0.6, dampingFraction: 0.8),
                            value: fraction
                        )
                }
            }
            .frame(height: 5)
        }
    }
}
