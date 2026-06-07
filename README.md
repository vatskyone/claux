<p align="center">
  <img src="apps/desktop/apple-native-ui.skill" alt="" width="0" height="0" />
</p>

<h1 align="center">Claux</h1>
<p align="center"><strong>Real-time cost & session intelligence for Claude Code</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%2013%2B-black?style=flat-square&logo=apple" />
  <img src="https://img.shields.io/badge/swift-5.9-orange?style=flat-square&logo=swift" />
  <img src="https://img.shields.io/badge/rust-1.78%2B-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/no%20backend-local%20only-green?style=flat-square" />
  <img src="https://img.shields.io/badge/desktop-v1.15.1-informational?style=flat-square" />
  <img src="https://img.shields.io/badge/cli-v0.7.2-informational?style=flat-square" />
</p>

---

## What is Claux?

Claude Code is Anthropic's agentic CLI — it writes code, runs commands, and iterates autonomously. It also bills per token. When you let an agent run for hours across multiple projects, the cost can quietly compound.

**Claux watches every session so you don't have to.**

No account. No backend. No data ever leaves your machine. Claux reads Claude Code's local JSONL session logs directly from `~/.claude/` and surfaces what matters: how much this session has cost so far, how full the context window is, plan-limit usage, and what you've spent today, this week, and this month — all without interrupting your workflow.

Claux ships as two independent tools:

| | Desktop | CLI |
|---|---|---|
| **Runtime** | Native macOS menu bar app (Swift/SwiftUI) | Terminal TUI + subcommands (Rust) |
| **Primary use** | Passive monitoring, notifications, quick glance | Deep analysis, export, automation |
| **Version** | 1.15.1 | 0.7.2 |

---

## The problem

Developers using Claude Code in production face a monitoring gap:

| Problem | Impact |
|---|---|
| No native cost visibility | Surprise bills at month-end |
| No context-window awareness | Model degradation before you notice |
| No session history | Can't attribute spend to projects |
| No spend pacing | No way to stay under a budget |
| No plan-limit visibility | Unexpected rate-limit interruptions |

Claude Code's own interface shows per-message cost inline but gives you no aggregate view, no historical chart, and no alert system. Third-party dashboards require uploading your session data to an external server — a non-starter in any professional environment.

---

## Desktop app (macOS)

A native `NSStatusItem` + `NSPopover` menu bar app. Left-click toggles the popover; right-click shows a context menu. Runs invisibly at idle (~18 MB RSS).

### Popover tabs

**Dashboard**
- Live session card — cost, context health bar (green → yellow → red), token breakdown (input / output / cache read / cache write / thinking), burn rate ($/hr), cost projection, model badge, elapsed time
- Session quality score — accepted edits, rejected actions, agent outcomes, touched files
- CLAUDE.md quality score with color-coded grade
- Plan limits card — 5-hour and 7-day Claude subscription usage bars with reset countdowns

**Analytics**
- 7-day spend sparkline and 30-day daily cost bar chart (SwiftUI Charts)
- Per-project and per-model spend tables
- Monthly budget progress bar (configurable, color-coded green/yellow/red)

**History**
- Filterable session list with native search
- Right-click rows: Copy Path, Show in Finder, Copy Session ID
- Tap any row for full session detail: quality score, token breakdown, entrypoint badge

### Notifications (`UNUserNotificationCenter`)
- **Session-ended alerts** with richer diagnostics and actions (Open Session / Open Dashboard / Snooze for today)
- **Cost threshold alert** (configurable, fires once per session)
- **Context window warning** (configurable %, fires once per session)
- **Daily recap** — fires at a configured hour with today's spend and session count; opens an in-app daily recap sheet with top project, top model, accepted/rejected actions, and strongest sessions
- **Weekly recap** — optional Friday summary of the past 7 days with top project/model and editing outcomes
- **Verbosity modes** — Minimal / Standard / Detailed, with quiet-hours and weekday-only scheduling

### Settings
- Notification verbosity, quiet hours, daily/weekly recap scheduling
- Cost alert threshold · Context health alert percentage
- Show cost in menu bar · Show model badge
- App theme (Light / Dark / Auto)
- State color palette (System / Vivid / High Contrast / Colorblind Safe / Soft Contrast)
- Session retention (7 / 14 / 30 / 60 / 90 days / Forever)
- Monthly budget ($0 = off)
- Auto-refresh interval (5 / 10 / 30 / 60 s)
- Launch at login (`SMAppService`)
- Session directory (custom `~/.claude` path)
- Include cache cost in totals
- Claude integration installer (installs/repairs the `statusLine` hook that feeds plan-limit data)
- Reset all settings to default

### Menu bar icon

| State | Appearance |
|---|---|
| Idle | Static `c` monogram, system text color |
| Active session | Green `c` monogram with radial pulse animation |
| Optional overlays | Live cost in $ · Model badge (configurable) |

Right-click: Settings · Show in Menu Bar (Always / When session is active) · Quit.

### Build & run (desktop)

```bash
git clone https://github.com/vatskyone/claux.git
cd claux/apps/desktop/Claux

# Build the .app bundle and launch (required for notifications + login items)
bash build_app.sh run

# Build a drag-and-drop DMG installer
bash build_dmg.sh

# Fast compile check (no bundle, notifications disabled)
swift build
```

`build_app.sh run` produces `Claux.app`, signs it ad-hoc, and opens it. The menu bar icon appears immediately.  
`build_dmg.sh` produces `apps/desktop/Claux/dist/Claux.dmg`.

**Requirements:** macOS 13 Ventura or later · Swift 5.9+ (Xcode 15+) · No third-party dependencies.

---

## CLI app (Rust)

A terminal-first companion for deeper inspection, automation, and scripting. All subcommands are composable and support `--json` output.

### TUI (`claux tui`)

A full-screen keyboard-driven dashboard with six tabs:

| Tab | What it shows |
|---|---|
| **Dashboard** | Active session card with token breakdown bars + live Insights panel (cache grade, context health, cost projection, model, thinking %) |
| **Sessions** | Scrollable session list with tagging, cursor navigation, session detail overlay |
| **Analytics** | 7-day vertical bar chart · 30-day sparkline · by-project and by-model tables with model efficiency (K tok/$) · Monthly cost forecast |
| **Agents** | Live sub-agent list with status, type, XP/level, quality stars, cost, duration; detail panel with token breakdown and output preview |
| **Skills** | Skill list seeded from `~/.claude.json` + `~/.claude/skills/` with star ratings |
| **History** | Named project checkpoints — save, restore, diff files since last checkpoint, write `.claux/CONTEXT.md` for agent consumption |

### Subcommands

```
claux status [--json]              Active session card
claux sessions [-n N] [--json]     Recent session table
claux spend [--json]               Today / this week / this month with trend arrows
claux analytics [--days N] [--json] Daily chart + project + model breakdown

claux export [--format csv|json] [--output FILE] [-n N]   Export session history
claux tag <id> [label] [-r]        Attach or remove a label on any session
claux checkpoint save|list|load|delete                     Named project checkpoints

claux claudemd generate [--project PATH] [--write] [--json]   Generate a CLAUDE.md starter
claux claudemd improve  [--project PATH] [--write] [--backup] [--json]   Fill gaps in existing CLAUDE.md

claux account                      Account info + skill usage table
claux skills list|new|export|import
claux config get|set|unset <key>   Budget limits and config (weekly-budget, monthly-credit, etc.)
claux config init                  Guided first-run initializer
claux doctor [--json]              Diagnostics — validates session dirs, parse health, remediation hints
claux analytics local [--json] [--reset]   On-device usage metrics

claux completions zsh|bash|fish    Shell completion scripts
```

### Build & run (CLI)

```bash
cd claux/apps/cli

cargo build --release
cargo run -- tui          # launch TUI
cargo run -- status       # quick status card
```

**Requirements:** Rust 1.78+ · macOS (primary target).

---

## Architecture (desktop)

```
Sources/Claux/
├── ClauxApp.swift            # App entry point — ClauxStatusAppDelegate + ClauxStatusItemController
│                             # NSStatusItem + NSPopover + NSMenu; left-click = popover, right-click = menu
├── AppStore.swift            # @ObservableObject central state
│                             # Owns SessionMonitor + RateLimitMonitor; computes all derived spend data
├── SessionMonitor.swift      # File-system engine
│                             # DispatchSource watchers on ~/.claude/projects/**
│                             # 10 s fallback poll timer · (URL, mtime) parse cache
├── SessionParser.swift       # JSONL → ClaudeSession
│                             # Per-model pricing · per-turn daily cost attribution for cross-midnight sessions
│                             # CLAUDE.md quality scorer (0–100) with TCC-safe directory traversal
│                             # Session quality: accepted edits, rejected actions, agent outcomes
├── RateLimitMonitor.swift    # Watches ~/.claude/claux/rate_limits.json
│                             # Publishes PlanLimitsSnapshot (5-hour + 7-day windows)
├── ClaudeStatusLineManager.swift  # Installs/repairs the Claude statusLine integration
│                                  # Writes a managed wrapper preserving any existing custom command
├── NotificationManager.swift # UNUserNotificationCenter
│                             # Cost · context · session-end · daily recap · weekly recap
│                             # Verbosity modes, quiet hours, per-day delivery tracking
├── Models.swift              # Value types: ClaudeSession · TokenUsage · SpendSummary
│                             # DailySpend · ProjectSpend · ModelSpend · PlanLimitsSnapshot
├── Design.swift              # Single source of truth: AppVersion · semantic Color aliases
│                             # ModelInfo · Format helpers · CardStyle · StateColorPreset
└── Views/
    ├── PopoverView.swift          # Root popover (Dashboard / Analytics / History tabs)
    │                              # Onboarding gating · session detail overlay
    ├── ActiveSessionCard.swift    # Live session stats card
    ├── PlanLimitsCard.swift       # 5h + 7d subscription usage bars
    ├── DailyRecapSheet.swift      # In-app daily recap drill-down sheet
    ├── SpendSummaryView.swift     # Sparkline + spend cells + budget bar
    ├── RecentSessionsView.swift   # Session list with search filter
    ├── SessionRowView.swift       # Tappable row + right-click context menu
    ├── SessionDetailSheet.swift   # Full session breakdown + quality panel
    ├── ContextHealthBar.swift     # Animated fill bar
    ├── TokenBreakdownView.swift   # Token category breakdown
    ├── NoActiveSessionView.swift  # Empty state
    ├── OnboardingView.swift       # First-launch overlay (directory + integration + notifications)
    ├── SettingsView.swift         # General / Notifications / Data / About
    └── AnalyticsView.swift        # Analytics window (chart + project + model tables)
```

### Data flow

```
~/.claude/projects/<encoded-path>/*.jsonl
        │
        ▼  DispatchSource (kernel-level file events) + 10 s poll
SessionMonitor
        │  mtime cache — only re-parses changed files
        ▼
SessionParser.parse(url:activeSessionIds:)
        │  → ClaudeSession { dailyCosts: [Date: Double], qualityScore, tokenUsage, … }
        ▼
AppStore.updateUI(from:)          ◄── RateLimitMonitor (rate_limits.json watcher)
        │  computeSpend · computeDailySpend · computeProjectBreakdown · computeModelBreakdown
        ▼
SwiftUI views — published on RunLoop.main
```

### Key engineering decisions

**Per-turn cost attribution** — The naïve approach (bucket entire session by `startTime`) breaks for any session that runs past midnight. Claux instead records cost and timestamp for every assistant turn, builds a `[Date: Double]` map keyed by local-timezone day-start, and sums those buckets in `computeSpend`.

**Incremental parse cache** — `SessionMonitor` maintains a `[URL: (mtime: Date, session: ClaudeSession)]` dictionary. On each watcher tick only files whose `contentModificationDate` has changed are re-parsed. A 10-session workspace with one active session re-parses exactly one file per tick.

**Active session detection** — Two signals with OR: (1) `~/.claude/sessions/<pid>.json` contains a `sessionId` matching the JSONL filename; (2) file mtime < 90 seconds.

**TCC-safe CLAUDE.md traversal** — The CLAUDE.md scorer walks up and down the directory tree to find project docs, but explicitly skips macOS privacy-protected home directories (`Desktop`, `Documents`, `Downloads`, `Movies`, `Music`, `Pictures`) to prevent unexpected TCC permission prompts.

**AppKit menu bar** — `MenuBarExtra` intercepts all mouse events at the `NSStatusItem` level, making SwiftUI's `.contextMenu` inoperative. Claux uses a native `NSStatusItem` + `NSPopover` + `NSMenu` stack so left-click deterministically toggles the popover and right-click reliably shows a native context menu on all macOS versions.

**Theme propagation** — `NSPanel` windows used by `MenuBarExtra` ignore SwiftUI's `.preferredColorScheme()`. `AppThemeModifier` sets `NSApp.appearance` directly on `.onAppear` and `.onChange`, propagating the theme to every window in the process.

---

## Why native Swift, not Electron

| | Claux (Swift/SwiftUI) | Electron alternative |
|---|---|---|
| Memory at idle | ~18 MB | ~200 MB+ |
| Launch time | <0.3 s | 2–4 s |
| Menu bar rendering | Native `NSStatusItem` | Custom WebView chrome |
| macOS notifications | `UNUserNotificationCenter` | Node.js shim |
| Launch at login | `SMAppService` | Workaround scripts |
| File watching | `DispatchSource` kernel events | `chokidar` (polling) |
| App size | <2 MB | 150 MB+ |

---

## Pricing model reference

| Model | Input | Output | Cache read | Cache write |
|---|---|---|---|---|
| claude-opus-4.x | $15.00 / M | $75.00 / M | $1.50 / M | $18.75 / M |
| claude-sonnet-4.x | $3.00 / M | $15.00 / M | $0.30 / M | $3.75 / M |
| claude-haiku-4.x | $0.80 / M | $4.00 / M | $0.08 / M | $1.00 / M |

Claux uses model ID substring matching (`opus` / `sonnet` / `haiku`) so it automatically covers new model versions without updates.

---

## Roadmap

### Near-term
- [ ] Global keyboard shortcut to toggle the popover (⌃⌥C, user-configurable)
- [ ] Anomaly detection — flag sessions with unusual cost spikes
- [ ] CLAUDE.md improvement suggestions surfaced inline in the desktop app
- [ ] iCloud sync for lifetime spend totals across multiple Macs

### Longer-term
- [ ] CSV / JSON export from the desktop app
- [ ] Slack / webhook alert integration
- [ ] Team spend aggregation (local network, no external server)

---

## Contributing

```bash
# 1. Fork and clone
git clone https://github.com/vatskyone/claux.git

# Desktop — make changes to apps/desktop/Claux/Sources/Claux/
# Bump AppVersion.current in Sources/Claux/Design.swift (PATCH for fixes, MINOR for features)
# Add a CHANGELOG.md entry, then:
cd apps/desktop/Claux && bash build_app.sh run

# CLI — make changes to apps/cli/src/
# Bump version in Cargo.toml, add a CHANGELOG.md entry, then:
cd apps/cli && cargo test && cargo build
```

Read [apps/desktop/Claux/CLAUDE.md](apps/desktop/Claux/CLAUDE.md) before submitting desktop changes. Every PR must include a version bump and changelog entry.

---

## Project structure

```
claux/
├── apps/
│   ├── desktop/
│   │   └── Claux/                  # Swift package (macOS app)
│   │       ├── Sources/Claux/      # All Swift source files
│   │       ├── Package.swift
│   │       ├── build_app.sh        # Build + launch script
│   │       ├── build_dmg.sh        # Build drag-and-drop DMG installer
│   │       ├── CLAUDE.md           # AI coding guidelines
│   │       └── CHANGELOG.md        # Full desktop version history
│   └── cli/
│       ├── src/                    # Rust source files
│       │   ├── commands/           # Subcommand handlers (tui, status, export, …)
│       │   ├── models.rs           # ClaudeSession, AgentRun, Checkpoint, …
│       │   ├── parser.rs           # JSONL parsing + CLAUDE.md scoring
│       │   ├── monitor.rs          # Session discovery + mtime cache
│       │   ├── spend.rs            # Spend aggregation + forecasting
│       │   └── tui.rs              # Terminal UI rendering
│       ├── Cargo.toml
│       └── CHANGELOG.md            # Full CLI version history
├── docs/                           # Product and engineering docs
├── packages/                       # Shared packages (future)
└── README.md
```

---

## Why this matters

Claude Code's adoption is accelerating. As agentic AI becomes a daily development tool, the gap between "I ran a session" and "I understand what it cost and why" is a real pain point for every professional using it.

Claux is built by developers who use Claude Code every day, designed with the constraints that actually matter — no telemetry, no accounts, no servers, no bloat — and it runs invisibly in the background doing exactly one thing well.

---

<p align="center">
  Built with Swift + Rust · Designed for macOS · Zero dependencies · Zero backend
  <br />
  <a href="https://github.com/vatskyone/claux">github.com/vatskyone/claux</a>
</p>
