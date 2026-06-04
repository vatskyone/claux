import SwiftUI

struct DailyRecapSheet: View {
    let recap: DailyRecap
    let onDismiss: () -> Void

    private var dateLabel: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .full
        formatter.timeStyle = .none
        return formatter.string(from: recap.day)
    }

    var body: some View {
        ScrollView(.vertical, showsIndicators: false) {
            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .top, spacing: 10) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Daily recap")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundStyle(Color.clauxPrimary)

                        Text(dateLabel)
                            .font(.system(size: 11))
                            .foregroundStyle(Color.clauxSecondary)

                        HStack(spacing: 6) {
                            recapBadge("\(recap.sessionCount) session\(recap.sessionCount == 1 ? "" : "s")", color: .clauxBlue)

                            if let topModel = recap.topModelDisplayName {
                                recapBadge(topModel, color: ModelInfo.color(topModel))
                            }

                            if recap.totalAcceptedEdits > 0 {
                                recapBadge("\(recap.totalAcceptedEdits) accepted", color: .clauxGreen)
                            }
                        }
                    }

                    Spacer()

                    Button { onDismiss() } label: {
                        Image(systemName: "xmark.circle.fill")
                            .font(.system(size: 18))
                            .foregroundStyle(Color.clauxTertiary)
                    }
                    .buttonStyle(.plain)
                }
                .padding(16)

                Divider()

                LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 0) {
                    statCell(label: "Spend", value: Format.cost(recap.totalCost), color: .clauxBlue, icon: "dollarsign.circle")
                    statCell(label: "Sessions", value: "\(recap.sessionCount)", color: .clauxPrimary, icon: "rectangle.stack")
                    statCell(label: "Accepted edits", value: "\(recap.totalAcceptedEdits)", color: .clauxGreen, icon: "checkmark.circle")
                    statCell(label: "Rejected actions", value: "\(recap.totalRejectedActions)", color: .clauxRed, icon: "xmark.circle")
                    statCell(label: "Files touched", value: "\(recap.totalTouchedFileCount)", color: .clauxOrange, icon: "doc.text")
                    statCell(label: "Top model", value: recap.topModelDisplayName ?? "n/a", color: .clauxSecondary, icon: "sparkles")
                }

                if let topProject = recap.topProjectDisplayPath {
                    Divider()

                    VStack(alignment: .leading, spacing: 8) {
                        sectionTitle("Top project")

                        HStack(alignment: .center, spacing: 10) {
                            VStack(alignment: .leading, spacing: 3) {
                                Text(topProject)
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundStyle(Color.clauxPrimary)
                                    .lineLimit(1)
                                    .truncationMode(.middle)

                                Text("Generated \(Format.cost(recap.topProjectCost)) today")
                                    .font(.system(size: 11))
                                    .foregroundStyle(Color.clauxSecondary)
                            }

                            Spacer()
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 9)
                        .background(Color(nsColor: .separatorColor).opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                    }
                    .padding(16)
                }

                if recap.bestSession != nil || recap.mostExpensiveSession != nil {
                    Divider()

                    VStack(alignment: .leading, spacing: 8) {
                        sectionTitle("Highlights")

                        if let bestSession = recap.bestSession {
                            highlightCard(
                                title: "Best session",
                                session: bestSession,
                                accent: qualityColor(for: bestSession.qualityScore)
                            )
                        }

                        if let mostExpensiveSession = recap.mostExpensiveSession, mostExpensiveSession.id != recap.bestSession?.id {
                            highlightCard(
                                title: "Highest spend",
                                session: mostExpensiveSession,
                                accent: .clauxBlue
                            )
                        }
                    }
                    .padding(16)
                }

                Divider()

                VStack(alignment: .leading, spacing: 8) {
                    sectionTitle("Sessions")

                    ForEach(Array(recap.sessions.prefix(3))) { session in
                        sessionRow(session)
                    }

                    if recap.sessions.count > 3 {
                        Text("+\(recap.sessions.count - 3) more sessions")
                            .font(.system(size: 10))
                            .foregroundStyle(Color.clauxTertiary)
                    }
                }
                .padding(16)
            }
        }
        .frame(maxHeight: 320)
        .background(VisualEffectView(material: .sidebar, blendingMode: .withinWindow))
    }

    private func recapBadge(_ text: String, color: Color) -> some View {
        Text(text)
            .font(.system(size: 10, weight: .medium))
            .foregroundStyle(color)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.12))
            .clipShape(Capsule())
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .font(.system(size: 11, weight: .semibold))
            .foregroundStyle(Color.clauxSecondary)
            .tracking(0.5)
            .textCase(.uppercase)
    }

    private func statCell(label: String, value: String, color: Color, icon: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(color)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text(label)
                    .font(.system(size: 10))
                    .foregroundStyle(Color.clauxSecondary)
                Text(value)
                    .font(.system(size: 12, weight: .semibold, design: .monospaced))
                    .foregroundStyle(color)
                    .lineLimit(1)
            }

            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }

    private func highlightCard(title: String, session: DailyRecapSession, accent: Color) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(title)
                    .font(.system(size: 11))
                    .foregroundStyle(Color.clauxSecondary)
                Spacer()
                Text(Format.cost(session.dayCost))
                    .font(.system(size: 11, weight: .semibold, design: .monospaced))
                    .foregroundStyle(Color.clauxBlue)
            }

            Text(session.title)
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(Color.clauxPrimary)
                .lineLimit(2)

            HStack(spacing: 8) {
                recapBadge(session.modelDisplayName, color: ModelInfo.color(session.modelDisplayName))
                recapBadge("\(session.qualityScore) \(session.qualityLabel)", color: accent)
                Text(Format.duration(session.duration))
                    .font(.system(size: 10))
                    .foregroundStyle(Color.clauxSecondary)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 9)
        .background(Color(nsColor: .separatorColor).opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private func sessionRow(_ session: DailyRecapSession) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Circle()
                .fill(qualityColor(for: session.qualityScore))
                .frame(width: 7, height: 7)
                .padding(.top, 4)

            VStack(alignment: .leading, spacing: 3) {
                Text(session.title)
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundStyle(Color.clauxPrimary)
                    .lineLimit(1)

                Text(session.subtitle)
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundStyle(Color.clauxTertiary)
                    .lineLimit(1)
                    .truncationMode(.middle)

                HStack(spacing: 8) {
                    Text("\(session.qualityScore) \(session.qualityLabel)")
                        .font(.system(size: 10))
                        .foregroundStyle(qualityColor(for: session.qualityScore))
                    Text("\(session.acceptedEdits) accepted")
                        .font(.system(size: 10))
                        .foregroundStyle(Color.clauxGreen)
                    if session.rejectedActions > 0 {
                        Text("\(session.rejectedActions) rejected")
                            .font(.system(size: 10))
                            .foregroundStyle(Color.clauxRed)
                    }
                }
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 3) {
                Text(Format.cost(session.dayCost))
                    .font(.system(size: 11, weight: .semibold, design: .monospaced))
                    .foregroundStyle(Color.clauxBlue)
                Text(Format.duration(session.duration))
                    .font(.system(size: 10))
                    .foregroundStyle(Color.clauxSecondary)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(Color(nsColor: .separatorColor).opacity(0.10))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private func qualityColor(for score: Int) -> Color {
        switch score {
        case 85...:
            return .clauxGreen
        case 70...:
            return .clauxBlue
        case 50...:
            return .clauxOrange
        default:
            return .clauxRed
        }
    }
}
