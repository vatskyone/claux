import Foundation

// MARK: – Per-model pricing (per million tokens, as of 2025)
private struct Rates {
    let inputPerM:      Double
    let outputPerM:     Double
    let cacheReadPerM:  Double
    let cacheWritePerM: Double

    static func forModel(_ model: String) -> Rates {
        let m = model.lowercased()
        if m.contains("opus") {
            return Rates(inputPerM: 15.0,  outputPerM: 75.0, cacheReadPerM: 1.50, cacheWritePerM: 18.75)
        } else if m.contains("haiku") {
            return Rates(inputPerM: 0.80,  outputPerM: 4.0,  cacheReadPerM: 0.08, cacheWritePerM: 1.00)
        } else {
            // Sonnet (default)
            return Rates(inputPerM: 3.0,   outputPerM: 15.0, cacheReadPerM: 0.30, cacheWritePerM: 3.75)
        }
    }

    func cost(input: Int, output: Int, cacheRead: Int, cacheWrite: Int) -> Double {
        let M = 1_000_000.0
        let a = Double(input)      * inputPerM
        let b = Double(output)     * outputPerM
        let c = Double(cacheRead)  * cacheReadPerM
        let d = Double(cacheWrite) * cacheWritePerM
        return (a + b + c + d) / M
    }
}

// MARK: – ISO 8601 parsing
private let isoFull: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return f
}()
private let isoBasic = ISO8601DateFormatter()

private func parseISO(_ s: String) -> Date? {
    isoFull.date(from: s) ?? isoBasic.date(from: s)
}

// MARK: – Session parser
enum SessionParser {

    /// Parse a single `.jsonl` session file and return a `ClaudeSession`.
    /// `activeSessionIds` is the set of session IDs found in `~/.claude/sessions/`.
    static func parse(url: URL, activeSessionIds: Set<String>) -> ClaudeSession? {
        guard let raw = try? String(contentsOf: url, encoding: .utf8) else { return nil }

        let sessionId          = url.deletingPathExtension().lastPathComponent
        let isActiveByDir      = activeSessionIds.contains(sessionId)

        var firstDate:    Date?
        var lastDate:     Date?
        var projectPath:  String?
        var latestModel   = "claude-sonnet-4-6"
        var sessionTitle: String?
        var entrypoint:   String?

        var totalInput   = 0
        var totalOutput  = 0
        var totalCacheR  = 0
        var totalCacheW  = 0
        var totalThink   = 0
        var totalCost    = 0.0
        var lastContextWindow = 0
        var dailyCosts: [Date: Double] = [:]
        let calendar = Calendar.current

        // Timestamp of the entry currently being processed.
        // Set before the assistant-branch guard so each assistant turn
        // can attribute its cost to the correct calendar day.
        var currentEntryDate: Date? = nil

        for line in raw.components(separatedBy: "\n") {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            guard !trimmed.isEmpty,
                  let data = trimmed.data(using: .utf8),
                  let obj  = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
            else { continue }

            // Timestamps
            if let tsStr = obj["timestamp"] as? String, let ts = parseISO(tsStr) {
                if firstDate == nil { firstDate = ts }
                lastDate = ts
                currentEntryDate = ts
            }

            // Working directory
            if projectPath == nil, let cwd = obj["cwd"] as? String, !cwd.isEmpty {
                projectPath = cwd
            }

            // Entrypoint (capture once — "claude-vscode", "terminal", "api", …)
            if entrypoint == nil, let ep = obj["entrypoint"] as? String, !ep.isEmpty {
                entrypoint = ep
            }

            let type = obj["type"] as? String ?? ""

            // AI-generated session title (from "ai-title" entries)
            // Note: the JSONL field is "aiTitle" (camelCase), not "title"
            if type == "ai-title", sessionTitle == nil,
               let t = obj["aiTitle"] as? String, !t.isEmpty {
                sessionTitle = t
            }

            // Cost & token data only from assistant entries
            guard type == "assistant",
                  let msgBody = obj["message"] as? [String: Any]
            else { continue }

            if let m = msgBody["model"] as? String, !m.isEmpty { latestModel = m }

            guard let usage = msgBody["usage"] as? [String: Any] else { continue }

            let inp    = usage["input_tokens"]                as? Int ?? 0
            let out    = usage["output_tokens"]               as? Int ?? 0
            let cacheR = usage["cache_read_input_tokens"]     as? Int ?? 0
            let cacheW = usage["cache_creation_input_tokens"] as? Int ?? 0

            // Thinking tokens: the usage object doesn't expose a dedicated count,
            // so we estimate from the text length of any "thinking" content blocks
            // (Claude Code JSONL writes thinking as { "type": "thinking", "thinking": "…", "signature": "…" })
            // Approximation: ~4 characters per token (standard rule of thumb).
            var thinkForTurn = 0
            if let contentBlocks = msgBody["content"] as? [[String: Any]] {
                for block in contentBlocks {
                    if block["type"] as? String == "thinking",
                       let text = block["thinking"] as? String {
                        thinkForTurn += max(1, text.count / 4)
                    }
                }
            }

            totalInput  += inp
            totalOutput += out
            totalCacheR += cacheR
            totalCacheW += cacheW
            totalThink  += thinkForTurn

            // Track the most-recent turn's full context window fill
            lastContextWindow = inp + cacheR + cacheW

            let rates    = Rates.forModel(latestModel)
            let turnCost = rates.cost(input: inp, output: out, cacheRead: cacheR, cacheWrite: cacheW)
            totalCost += turnCost

            // Attribute this turn's cost to its calendar day (local timezone).
            // This lets AppStore show the correct "today" spend even for sessions
            // that started before midnight.
            let turnDate = currentEntryDate ?? (firstDate ?? Date())
            let dayStart = calendar.startOfDay(for: turnDate)
            dailyCosts[dayStart, default: 0] += turnCost
        }

        guard let startTime = firstDate else { return nil }

        // Score the project's CLAUDE.md if we have a path
        let claudemdScore: Int? = projectPath.flatMap { SessionParser.scoreClaudeMd(at: $0) }

        let recentlyModified: Bool = {
            guard let attr = try? FileManager.default.attributesOfItem(atPath: url.path),
                  let mod  = attr[.modificationDate] as? Date else { return false }
            return Date().timeIntervalSince(mod) < 90
        }()

        let isActive = isActiveByDir || recentlyModified
        let endTime  = isActive ? nil : lastDate

        return ClaudeSession(
            id:          UUID(uuidString: sessionId) ?? UUID(),
            projectPath: projectPath ?? "Unknown",
            startTime:   startTime,
            endTime:     endTime,
            totalCost:   totalCost,
            tokenUsage:  TokenUsage(
                inputTokens:         totalInput,
                outputTokens:        totalOutput,
                cacheReadTokens:     totalCacheR,
                cacheWriteTokens:    totalCacheW,
                thinkingTokens:      totalThink,
                contextWindowTokens: lastContextWindow
            ),
            model:        latestModel,
            isActive:     isActive,
            title:        sessionTitle,
            entrypoint:   entrypoint,
            claudemdScore: claudemdScore,
            dailyCosts:   dailyCosts
        )
    }

    // MARK: – CLAUDE.md directory search
    //
    // Searches for a CLAUDE.md file starting from `startDir`:
    //   1. Walks UP the directory tree (matching Claude Code's own search strategy).
    //   2. If not found above, walks DOWN up to `maxDownDepth` levels (handles
    //      the case where `claude` is invoked from a parent of the project root).
    //
    // Skips hidden dirs and common build/dependency dirs on the downward pass.
    private static func findClaudeMd(startingAt startDir: String) -> String? {
        let fm   = FileManager.default
        let home = NSHomeDirectory()

        // Pass 1 – walk up
        var dir = startDir
        for _ in 0..<8 {
            let candidate = (dir as NSString).appendingPathComponent("CLAUDE.md")
            if fm.fileExists(atPath: candidate) { return candidate }
            let parent = (dir as NSString).deletingLastPathComponent
            if parent == dir || dir == home || dir == "/" { break }
            dir = parent
        }

        // Pass 2 – walk down (breadth-first, depth ≤ 4, skipping junk dirs)
        return findClaudeMdDown(at: startDir, depth: 0, maxDepth: 4)
    }

    private static let skipDirs: Set<String> = [
        ".git", "node_modules", ".build", "DerivedData", "Pods",
        "vendor", ".swiftpm", "dist", "build", ".next", "__pycache__",
    ]

    private static func findClaudeMdDown(at dir: String, depth: Int, maxDepth: Int) -> String? {
        let fm = FileManager.default
        let candidate = (dir as NSString).appendingPathComponent("CLAUDE.md")
        if fm.fileExists(atPath: candidate) { return candidate }
        guard depth < maxDepth else { return nil }

        guard let items = try? fm.contentsOfDirectory(atPath: dir) else { return nil }
        for item in items.sorted() {
            if item.hasPrefix(".") || skipDirs.contains(item) { continue }
            let sub = (dir as NSString).appendingPathComponent(item)
            var isDir: ObjCBool = false
            guard fm.fileExists(atPath: sub, isDirectory: &isDir), isDir.boolValue else { continue }
            if let found = findClaudeMdDown(at: sub, depth: depth + 1, maxDepth: maxDepth) {
                return found
            }
        }
        return nil
    }

    // MARK: – CLAUDE.md quality scorer (0–100; nil if file absent)
    //
    // Reads the nearest CLAUDE.md (up or down the tree) and scores it across
    // three dimensions:
    //   1. Length   (0–30 pts) — longer files tend to be more complete
    //   2. Structure (0–30 pts) — headers, code blocks, bullet lists
    //   3. Content   (0–40 pts) — 8 key topic categories × 5 pts each
    //
    // Score recomputes whenever the session JSONL is re-parsed (mtime change).
    // If CLAUDE.md changes independently, use "Refresh sessions now" to rescore.
    static func scoreClaudeMd(at projectPath: String) -> Int? {
        guard let mdPath = findClaudeMd(startingAt: projectPath) else { return nil }
        guard let content = try? String(contentsOfFile: mdPath, encoding: .utf8),
              content.trimmingCharacters(in: .whitespacesAndNewlines).count > 10
        else { return nil }

        let lower = content.lowercased()
        let lines = content.components(separatedBy: "\n")
        let words = content.split { $0.isWhitespace }.count
        var score = 0

        // 1. Length (0–30 pts)
        switch words {
        case 0..<30:    score += 0
        case 30..<80:   score += 8
        case 80..<150:  score += 16
        case 150..<300: score += 23
        default:        score += 30
        }

        // 2. Structure (0–30 pts)
        let headings   = lines.filter { $0.hasPrefix("#") }.count
        let codeBlocks = max(0, (content.components(separatedBy: "```").count - 1) / 2)
        let bullets    = lines.filter { $0.hasPrefix("- ") || $0.hasPrefix("* ") }.count
        score += min(15, headings   * 5)
        score += min(10, codeBlocks * 5)
        score += min(5,  bullets    / 4)

        // 3. Content coverage (0–40 pts — 8 categories × 5 pts each)
        let topics: [[String]] = [
            ["build", "compile", "swift build", "npm run", "yarn", "make ", "gradle", "cmake"],
            ["test", "pytest", "jest ", "xcode test", "unit test", "spec"],
            ["run ", "start ", "launch", "execute", "serve"],
            ["structure", "architecture", "layout", "directory", "folder", "project"],
            ["convention", "style guide", "pattern", "naming", "format", "lint"],
            ["important", "note:", "warning", "do not", "never ", "always ", "avoid"],
            ["command", "script", "bash", "shell", "cli"],
            ["workflow", "process", "step", "instruction", "guideline"],
        ]
        for group in topics {
            if group.contains(where: { lower.contains($0) }) { score += 5 }
        }

        return min(100, score)
    }
}
