import SwiftUI

// Token breakdown — minimal macOS-native list style, no decorative cards.
struct TokenBreakdownView: View {
    let usage: TokenUsage

    private let columns = [GridItem(.flexible()), GridItem(.flexible())]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Tokens")
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .tracking(0.5)
                .textCase(.uppercase)

            LazyVGrid(columns: columns, spacing: 5) {
                TokenCell(
                    label: "Input",
                    count: usage.inputTokens,
                    color: Color(nsColor: .labelColor)
                )
                TokenCell(
                    label: "Cache Read",
                    count: usage.cacheReadTokens,
                    // Blue tint for cache = efficient, matches accent
                    color: Color(nsColor: .systemBlue)
                )
                TokenCell(
                    label: "Output",
                    count: usage.outputTokens,
                    color: Color(nsColor: .secondaryLabelColor)
                )
                TokenCell(
                    label: "Cache Write",
                    count: usage.cacheWriteTokens,
                    color: Color(nsColor: .systemGreen)
                )
            }

            if usage.thinkingTokens > 0 {
                HStack(spacing: 5) {
                    Image(systemName: "brain")
                        .font(.system(size: 10, weight: .medium))
                        .foregroundStyle(Color(nsColor: .systemBlue))

                    Text("Thinking: \(Format.tokens(usage.thinkingTokens))")
                        .font(.system(size: 11, weight: .medium))
                        .foregroundStyle(Color(nsColor: .systemBlue))
                }
                .padding(.top, 2)
            }
        }
    }
}

private struct TokenCell: View {
    let label: String
    let count: Int
    let color: Color

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(color)
                .frame(width: 5, height: 5)

            Text(label)
                .font(.system(size: 11))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))
                .lineLimit(1)

            Spacer(minLength: 0)

            Text(Format.tokens(count))
                .font(.system(size: 11, weight: .semibold, design: .monospaced))
                .foregroundStyle(color)
        }
    }
}
