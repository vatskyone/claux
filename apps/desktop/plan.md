Design a complete, detailed implementation plan for a macOS menu bar app called "Claux" — a Claude Code session monitoring tool.

## Product Context
Claux is a monitoring tool for Claude Code (Anthropic's CLI coding agent). The menu bar app sits in the macOS menu bar and shows real-time session data: live costs, context health, token breakdown, burn-rate projections, and recent session history.

## Decisions Already Made
- **Tech stack**: Native Swift / SwiftUI (Xcode project, no Electron/Tauri)
- **MVP scope**: "Observe only" — real-time monitoring popover (no CLAUDE.md editor, no backend)
- **Data source**: Local only — reads `~/.claude/` JSONL files directly (no backend, fully offline)

## What Claux monitors
Claude Code writes session logs to `~/.claude/projects/<encoded-path>/*.jsonl`. Each JSONL file is one session. Each line is a JSON event. Key fields (from research):
- `type`: "user" | "assistant" | "result" | "system"
- `message.usage.input_tokens`, `output_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`
- `message.model`: e.g., "claude-sonnet-4-6", "claude-opus-4-5"
- `costUSD`: sometimes present directly; otherwise calculate from token counts
- `timestamp`: ISO 8601 string
- Tool call information in `message.content[]` items with `type: "tool_use"`

Claude Code pricing (as of mid-2026, estimate — these should be configurable):
- Sonnet: $3/$15 per M input/output tokens
- Opus: $15/$75 per M input/output tokens  
- Haiku: $0.80/$4 per M input/output tokens
- Cache read: 10% of input price
- Cache write: 25% of input price
- Thinking tokens: counted as output tokens (billed same way)

Context window: 200,000 tokens effective. Context health = (total_input_tokens_in_session / 200000) — warn at 70%, alert at 90%.

## Required Features (MVP)

### Menu Bar Icon
- Shows a small Claux logo (or "C" monogram) in the menu bar
- When a session is active: icon animates (subtle pulse or dot indicator) 
- When cost threshold is exceeded: icon changes to warning state
- Click opens the popover

### Main Popover (NSPopover, SwiftUI content)
**Active Session card (shown when a session is being written to in real-time):**
- Session cost so far (in $)
- Context health bar (% of 200K tokens used, color: green/yellow/red)
- Token breakdown: input / output / cache read / cache write / thinking
- Burn rate ($ per hour, calculated from session duration)
- Projected session cost (if burn rate continues for 1 hour)
- Model being used
- Time elapsed

**Recent Sessions list (last 5 sessions, sorted by recency):**
- Session folder name (project path, truncated)
- Date/time
- Total cost
- Duration
- Model used
- Token count

**Bottom bar:**
- Total spend today / this week / this month (calculated from all session files)
- "Settings" button → opens Settings window
- "Open Dashboard" button → opens https://lucky-phoenix-6b02c3.netlify.app/ in browser

### Settings Window (separate NSWindow, SwiftUI)
- **Cost alert threshold**: text field + stepper, default $5.00 — triggers macOS notification
- **Context health alert**: slider, default 80% — triggers macOS notification  
- **Launch at Login**: toggle (using SMAppService in macOS 13+)
- **Menu bar style**: icon only / icon + cost / icon + status indicator
- **Monitored directory**: defaults to ~/.claude, allows custom path
- **Reset all data**: clear local cache

### Notifications (UNUserNotificationCenter)
- "Session cost reached $X" when threshold crossed
- "Context health at X% — consider compacting" when threshold crossed
- Only one notification per session per threshold crossing (don't spam)

## Architecture

Design the complete file/folder structure for an Xcode project. Include:

1. **AppDelegate / App entry point** (SwiftUI App lifecycle with NSApplicationDelegate)
2. **StatusBarController** — manages NSStatusItem, handles click → show popover
3. **SessionMonitor** (ObservableObject) — the core data engine:
   - Uses FileSystemWatcher (FSEvents via DispatchSource or NSFilePresenter) to watch `~/.claude/projects/`
   - Parses JSONL files
   - Publishes: `activeSessions`, `recentSessions`, `totalSpend`
   - Handles debounced file change events
4. **Models**: `ClaudeSession`, `SessionMessage`, `TokenUsage`, `CostBreakdown`
5. **Views**: 
   - `PopoverView` (root SwiftUI view inside popover)
   - `ActiveSessionCard`
   - `RecentSessionsView`
   - `SessionRowView`
   - `SpendSummaryView`
   - `SettingsView`
6. **CostCalculator** — pure functions to calculate costs from token counts and model names
7. **NotificationManager** — wraps UNUserNotificationCenter
8. **Preferences** (AppStorage / UserDefaults wrapper)

## Design Requirements
- Dark theme matching Claux brand (black background, white text, gold accent #D4A853 for active indicators)
- Popover width: 320pt
- Compact, information-dense layout — no wasted space
- SF Symbols for icons
- Smooth animations (cost number rolling, health bar transitions)

## Output Required
Produce a complete, detailed, executable implementation plan including:
1. Exact Xcode project structure (file tree)
2. Complete Swift code for each key file (all models, all views, all managers)
3. Info.plist / entitlements needed
4. Step-by-step build instructions
5. How to handle edge cases (no sessions found, permission denied for ~/. claude, active session detection)
6. Testing approach

This plan will be the single source of truth for building the app — make it complete enough that a developer can build from it without asking further questions.