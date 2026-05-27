import SwiftUI

// Empty state — shown when no Claude Code session is running.
// Minimal macOS-native feel: subdued icon, clean text hierarchy.
struct NoActiveSessionView: View {
    @State private var appeared = false

    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "bolt.slash.circle")
                .font(.system(size: 28, weight: .light))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .opacity(appeared ? 1 : 0)
                .scaleEffect(appeared ? 1 : 0.85)

            Text("No active session")
                .font(.system(size: 13, weight: .semibold))
                .foregroundStyle(Color(nsColor: .secondaryLabelColor))

            Text("Start a Claude Code session to\nbegin monitoring.")
                .font(.system(size: 11))
                .foregroundStyle(Color(nsColor: .tertiaryLabelColor))
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 28)
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color(nsColor: .separatorColor).opacity(0.4), lineWidth: 0.5)
        )
        .onAppear {
            withAnimation(.spring(response: 0.4, dampingFraction: 0.7)) {
                appeared = true
            }
        }
    }
}
