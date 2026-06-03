import Foundation

// MARK: – Token usage snapshot
struct TokenUsage {
    // Cumulative across the whole session
    var inputTokens:      Int = 0
    var outputTokens:     Int = 0
    var cacheReadTokens:  Int = 0
    var cacheWriteTokens: Int = 0
    var thinkingTokens:   Int = 0

    /// Tokens the model actually saw in the *most recent* API call
    /// = last message's (input + cache_read + cache_creation).
    /// This is the true current context-window fill.
    var contextWindowTokens: Int = 0

    /// Running total across all turns (used for spend / token-count display).
    var totalContextTokens: Int {
        inputTokens + cacheReadTokens + cacheWriteTokens
    }
}

// MARK: – Per-session quality metrics
struct SessionQualityMetrics {
    var permissionMode: String? = nil
    var assistantTurns: Int = 0
    var successfulAssistantTurns: Int = 0
    var toolProposals: Int = 0
    var toolResults: Int = 0
    var successfulToolResults: Int = 0
    var failedToolResults: Int = 0
    var rejectedToolResults: Int = 0
    var editProposals: Int = 0
    var acceptedEdits: Int = 0
    var rejectedEdits: Int = 0
    var acceptedUnmodifiedEdits: Int = 0
    var acceptedModifiedEdits: Int = 0
    var completedAgents: Int = 0
    var failedAgents: Int = 0
    var touchedFiles: [String] = []
    var score: Int = 0

    var assistantSuccessRatio: Double {
        guard assistantTurns > 0 else { return 0 }
        return Double(successfulAssistantTurns) / Double(assistantTurns)
    }

    var toolSuccessRatio: Double {
        guard toolResults > 0 else { return 1.0 }
        return Double(successfulToolResults) / Double(toolResults)
    }

    var editAcceptanceRatio: Double {
        guard editProposals > 0 else { return 1.0 }
        return Double(acceptedEdits) / Double(editProposals)
    }

    var acceptedAsSuggestedRatio: Double {
        guard acceptedEdits > 0 else { return 1.0 }
        return Double(acceptedUnmodifiedEdits) / Double(acceptedEdits)
    }

    var touchedFileCount: Int {
        touchedFiles.count
    }

    var scoreLabel: String {
        switch score {
        case 85...: return "Excellent"
        case 70...: return "Strong"
        case 50...: return "Mixed"
        default: return "Weak"
        }
    }
}

// MARK: – Per-model context-window limits (tokens)
enum ModelContextLimit {
    static func forModel(_ model: String) -> Int {
        // All current Claude 3/4 models share a 200 K context window.
        return 200_000
    }
}

// MARK: – Session
struct ClaudeSession: Identifiable {
    let id: UUID
    let projectPath: String
    let startTime: Date
    var endTime: Date?
    var totalCost: Double
    var tokenUsage: TokenUsage
    var model: String
    var isActive: Bool

    /// AI-generated title from `ai-title` JSONL entries (e.g. "Fix auth bug in API server")
    var title: String?

    /// IDE or interface that launched the session ("claude-vscode", "api", "terminal", …)
    var entrypoint: String?

    /// Quality score of the project's CLAUDE.md file (0–100).
    /// `nil` means no CLAUDE.md was found in `projectPath`.
    var claudemdScore: Int? = nil

    /// Cost broken down by calendar day (midnight local time → cost).
    /// Used to attribute multi-day sessions to the correct day bucket in spend summaries.
    var dailyCosts: [Date: Double] = [:]

    /// Session-level acceptance, execution, and quality metrics derived from JSONL tool flows.
    var qualityMetrics: SessionQualityMetrics = SessionQualityMetrics()

    // MARK: Derived

    var duration: TimeInterval {
        (endTime ?? Date()).timeIntervalSince(startTime)
    }

    /// Fraction of the context window currently used (0.0 – 1.0).
    var contextHealthFraction: Double {
        let limit  = ModelContextLimit.forModel(model)
        let tokens = tokenUsage.contextWindowTokens > 0
            ? tokenUsage.contextWindowTokens
            : tokenUsage.totalContextTokens
        return min(1.0, Double(tokens) / Double(limit))
    }

    /// Cache read tokens as a fraction of total input tokens (0.0 – 1.0).
    /// High value = efficient cache use = lower cost per turn.
    var cacheHitRate: Double {
        let total = tokenUsage.inputTokens + tokenUsage.cacheReadTokens + tokenUsage.cacheWriteTokens
        guard total > 0 else { return 0 }
        return Double(tokenUsage.cacheReadTokens) / Double(total)
    }

    /// $ per hour, based on elapsed time
    var burnRatePerHour: Double {
        guard duration > 60 else { return 0 }
        return totalCost / (duration / 3_600)
    }

    /// Rough 1-hour-forward projection
    var projectedCost: Double {
        totalCost + burnRatePerHour
    }

    /// User-friendly "~/…" path
    var displayPath: String {
        Format.projectPath(projectPath)
    }

    /// Human-readable entrypoint label
    var entrypointLabel: String? {
        guard let ep = entrypoint else { return nil }
        switch ep.lowercased() {
        case "claude-vscode", "vscode":      return "VS Code"
        case "claude-jetbrains", "jetbrains": return "JetBrains"
        case "terminal", "cli":              return "Terminal"
        case "api":                          return "API"
        default:
            // Strip "claude-" prefix and capitalise
            let cleaned = ep.replacingOccurrences(of: "claude-", with: "", options: .caseInsensitive)
            return cleaned.isEmpty ? nil : cleaned.capitalized
        }
    }
}

// MARK: – Spend totals
struct SpendSummary {
    var today:     Double
    var thisWeek:  Double
    var thisMonth: Double
    // Prior-period baselines used to compute trend arrows.
    // Defaulted to 0 so the .zero constant and existing callers compile unchanged.
    var yesterday:  Double = 0
    var prevWeek:   Double = 0
    var prevMonth:  Double = 0
}

// MARK: – Daily spend (for analytics chart)
struct DailySpend: Identifiable {
    var id: Date { date }
    let date:  Date    // start of the calendar day (midnight local time)
    let cost:  Double
}

// MARK: – Per-project breakdown (for analytics)
struct ProjectSpend: Identifiable {
    var id: String { path }
    let path:         String   // raw project path
    let displayPath:  String
    let totalCost:    Double
    let sessionCount: Int
}

// MARK: – Per-model breakdown (for analytics)
struct ModelSpend: Identifiable {
    var id: String { model }
    let model:        String
    let displayName:  String
    let totalCost:    Double
    let sessionCount: Int
}

// MARK: – Plan limit usage windows (Claude subscription limits)
struct PlanLimitWindow {
    /// Percent used in this window (0–100). Nil when unavailable.
    var usedPercentage: Double?
    /// Next reset timestamp for this window.
    var resetsAt: Date?

    var hasData: Bool {
        usedPercentage != nil || resetsAt != nil
    }

    /// Normalized progress (0–1), clamped for bar rendering.
    var progressFraction: Double {
        let pct = usedPercentage ?? 0
        return max(0, min(pct / 100.0, 1.0))
    }
}

struct PlanLimitsSnapshot {
    var fiveHour: PlanLimitWindow
    var sevenDay: PlanLimitWindow
    /// Last update timestamp written by the collector script.
    var updatedAt: Date?

    var hasAnyData: Bool {
        fiveHour.hasData || sevenDay.hasData
    }

    /// True when data is old enough that it should be treated as stale.
    var isStale: Bool {
        guard let updatedAt else { return true }
        return Date().timeIntervalSince(updatedAt) > 900
    }

    static let empty = PlanLimitsSnapshot(
        fiveHour: PlanLimitWindow(),
        sevenDay: PlanLimitWindow(),
        updatedAt: nil
    )
}

enum PlanLimitSourceState {
    case ready
    case waitingForFirstResponse
    case limitsUnavailableForSession
    case statusLinePendingActivation
    case statusLineNotConfigured
    case statusLineManagedElsewhere
    case statusLineNeedsRepair
    case statusLineNotRunning
    case statusLineInvalidData
    case staleData
}

struct PlanLimitsDiagnostics {
    var state: PlanLimitSourceState
    var message: String

    static let defaultState = PlanLimitsDiagnostics(
        state: .statusLineNotConfigured,
        message: "Install Claude integration to enable plan limits."
    )
}
