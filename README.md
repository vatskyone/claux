<p align="center">
  <img src="apps/desktop/apple-native-ui.skill" alt="" width="0" height="0" />
</p>

<h1 align="center">Claux</h1>
<p align="center"><strong>Real-time cost & session intelligence for Claude Code — native macOS menu bar app</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%2013%2B-black?style=flat-square&logo=apple" />
  <img src="https://img.shields.io/badge/swift-5.9-orange?style=flat-square&logo=swift" />
  <img src="https://img.shields.io/badge/SwiftUI-native-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/no%20backend-local%20only-green?style=flat-square" />
  <img src="https://img.shields.io/badge/version-1.4.1-informational?style=flat-square" />
</p>

---

## What is Claux?

Claude Code is Anthropic's agentic CLI — it writes code, runs commands, and iterates autonomously. It also bills per token. When you let an agent run for hours across multiple projects, the cost can quietly compound.

**Claux sits in your menu bar and watches every session in real time.**

No account. No backend. No data ever leaves your machine. Claux reads Claude Code's local JSONL session logs directly from `~/.claude/` and surfaces what matters: how much this session has cost so far, how full the context window is, and what you've spent today, this week, and this month — all without interrupting your workflow.

---

## The problem

Developers using Claude Code in production face a monitoring gap:

| Problem | Impact |
|---|---|
| No native cost visibility | Surprise bills at month-end |
| No context-window awareness | Model degradation before you notice |
| No session history | Can't attribute spend to projects |
| No spend pacing | No way to stay under a budget |

Claude Code's own interface shows per-message cost inline but gives you no aggregate view, no historical chart, and no alert system. Third-party dashboards require uploading your session data to an external server — a non-starter in any professional environment.

---

## The solution

Claux is a zero-permission, zero-network macOS menu bar app that:

- **Monitors** `~/.claude/projects/` with `DispatchSource` file-system watchers + a 10-second poll fallback
- **Parses** JSONL session logs incrementally — only re-parses files whose `mtime` has changed
- **Calculates** real API costs using per-model pricing (Opus / Sonnet / Haiku, including cache read/write)
- **Alerts** you when a session crosses your cost threshold or your context window is filling up
- **Shows** a 7-day spend sparkline, per-project and per-model breakdowns, and a monthly budget tracker

<table>
<tr>
<td width="50%">

**Active session card**
- Live cost (updates every 10 s)
- Context health bar (green → yellow → red)
- Token breakdown: input / output / cache read / cache write / thinking
- Burn rate ($/hr) + 1-hour projection
- Model badge + elapsed time

</td>
<td width="50%">

**Spend summary**
- Today / This week / This month with ↑↓ trend vs prior period
- 7-day sparkline chart (SwiftUI Charts)
- Monthly budget progress bar
- Per-project and per-model analytics window

</td>
</tr>
</table>

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

A menu bar utility that runs 24/7 must be invisible when idle. Electron's baseline resource consumption is incompatible with that requirement. SwiftUI gives us native performance, native appearance, automatic dark/light mode, and access to every macOS system API — without compromise.

---

## Architecture

```
Sources/Claux/
├── ClauxApp.swift            # App entry point · MenuBarExtra · WindowGroups
│                             # · NSStatusBarButton action interception for right-click menu
├── AppStore.swift            # @ObservableObject central state
│                             # · Owns SessionMonitor · computes all derived spend data
├── SessionMonitor.swift      # File-system engine
│                             # · DispatchSource watchers on ~/.claude/projects/**
│                             # · 10 s fallback poll timer
│                             # · (URL, mtime) parse cache — O(1) hit rate in steady state
├── SessionParser.swift       # JSONL → ClaudeSession
│                             # · Per-model pricing (Opus/Sonnet/Haiku)
│                             # · Per-turn daily cost attribution for cross-midnight sessions
│                             # · CLAUDE.md quality scorer (0–100, used in analytics)
│                             # · ISO 8601 full + basic date parser
├── NotificationManager.swift # UNUserNotificationCenter
│                             # · Cost threshold · context health · session-end · daily summary
├── Models.swift              # Value types: ClaudeSession · TokenUsage · SpendSummary
│                             # · DailySpend · ProjectSpend · ModelSpend
├── Design.swift              # Single source of truth for: AppVersion · semantic Color aliases
│                             # · ModelInfo (name + badge color) · Format helpers · CardStyle
└── Views/
    ├── PopoverView.swift          # Root popover · onboarding gating · session detail overlay
    ├── ActiveSessionCard.swift    # Live session stats card
    ├── SpendSummaryView.swift     # Sparkline + spend cells + budget bar
    ├── RecentSessionsView.swift   # Session list with NSSearchField filter
    ├── SessionRowView.swift       # Tappable row + right-click context menu
    ├── SessionDetailSheet.swift   # Full session breakdown sheet
    ├── ContextHealthBar.swift     # Animated fill bar (green/yellow/red)
    ├── TokenBreakdownView.swift   # Token category breakdown
    ├── NoActiveSessionView.swift  # Empty state
    ├── OnboardingView.swift       # 3-step first-launch overlay
    ├── SettingsView.swift         # Tabbed settings (General/Notifications/Data/About)
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
        │  → ClaudeSession { dailyCosts: [Date: Double], tokenUsage, … }
        ▼
AppStore.updateUI(from:)
        │  computeSpend  — iterates dailyCosts per turn (cross-midnight accurate)
        │  computeDailySpend — 30-day chart data
        │  computeProjectBreakdown / computeModelBreakdown
        ▼
SwiftUI views — published on RunLoop.main
```

### Key engineering decisions

**Per-turn cost attribution** — The naïve approach (bucket entire session by `startTime`) breaks for any session that runs past midnight. Claux instead records cost and timestamp for every assistant turn, builds a `[Date: Double]` map keyed by local-timezone day-start, and sums those buckets in `computeSpend`. Today's figure is always accurate regardless of when the session started.

**Incremental parse cache** — `SessionMonitor` maintains a `[URL: (mtime: Date, session: ClaudeSession)]` dictionary. On each watcher tick, only files whose `contentModificationDate` has changed are re-parsed. A 10-session workspace with one active session re-parses exactly one file per tick in steady state.

**Active session detection** — Two signals combined with OR: (1) `~/.claude/sessions/<pid>.json` contains a `sessionId` matching the JSONL filename; (2) file mtime < 90 seconds. This handles both the primary case (Claude Code writes a PID file) and the fallback (crashed session that left no PID file still shows as active for a short window).

**Right-click menu without breaking left-click** — `MenuBarExtra` with `.window` style intercepts all mouse events at the `NSStatusItem` level, making SwiftUI's `.contextMenu` silently inoperative. Solution: a zero-size `NSViewRepresentable` overlay inside the SwiftUI label walks the superview chain at `viewDidMoveToWindow` to find `NSStatusBarButton`, saves its original `action` + `target` (SwiftUI's popover toggle), replaces them with a custom handler that routes left-clicks back to the original target and right-clicks to an `NSMenu` via `NSMenu.popUpContextMenu(_:with:for:)`.

**Theme propagation to MenuBarExtra panels** — `NSPanel` windows used by `MenuBarExtra` ignore SwiftUI's `.preferredColorScheme()`. `AppThemeModifier` additionally sets `NSApp.appearance` directly (aqua / darkAqua / nil) on `.onAppear` and `.onChange(of: appTheme)`, which propagates to every window in the process including the status bar panel.

---

## Getting started

### Requirements

- macOS 13 Ventura or later
- Swift 5.9+ (bundled with Xcode 15+)
- No external dependencies — zero third-party packages

### Build & run

```bash
git clone https://github.com/harukifujimoto/claux.git
cd claux/apps/desktop/Claux

# Build the .app bundle and launch it (required for notifications + login items)
bash build_app.sh run

# Build only (fast compile check — no bundle, notifications disabled)
swift build
```

`build_app.sh run` produces `Claux.app`, signs it ad-hoc, and opens it. The menu bar icon appears immediately.

### How it works after launch

1. Claux reads `~/.claude/projects/` — Claude Code's default session directory
2. If you use a custom directory, set it in **Settings → General → Session directory**
3. Open Claude Code and start a session — the menu bar icon pulses green within 10 seconds
4. Click the icon to see live cost, context health, and token breakdown

---

## Features

### Menu bar icon
| State | Appearance |
|---|---|
| Idle (no active session) | Static `c` monogram, system text color |
| Active session | Green `c` monogram with radial pulse animation |
| Optional overlays | Cost in $ · Model badge (configurable) |

Right-click the icon for: Open Dashboard · Settings · Check for Updates · Show in Menu Bar (Always / When active / Never) · Quit.

### Popover
- **Active session card** — cost, context bar, token table, burn rate, projection, model, elapsed time
- **Spend summary** — 7-day sparkline · today/week/month with trend indicators · monthly budget bar
- **Recent sessions** — filterable list (native `NSSearchField`) with right-click: Copy Path, Show in Finder, Copy Session ID
- **Session detail** — full token breakdown, CLAUDE.md quality score, entrypoint (VS Code / Terminal / API)

### Notifications (`UNUserNotificationCenter`)
- Session cost exceeded threshold (once per session)
- Context window at alert percentage (once per session)
- Session ended summary (opt-in)
- Daily spend summary at configured hour (opt-in)

### Analytics window
- 30-day daily cost bar chart (SwiftUI Charts)
- Per-project spend table
- Per-model spend table

### Settings
- Cost alert threshold · Context health alert · Session-end notification
- Daily summary (enabled + hour)
- Show cost in menu bar · Show model badge
- App theme (Light / Dark / Auto)
- Session retention (7 / 14 / 30 / 60 / 90 days / Forever)
- Monthly budget ($0 = off)
- Auto-refresh interval (5 / 10 / 30 / 60 s)
- Launch at login (`SMAppService`)
- Session directory (custom `~/.claude` path)
- Include cache cost in totals
- Reset all settings to default

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

### v1.5 — Visibility & control
- [ ] Dynamic menu bar icon visibility — "When Claude Code is running" mode (requires replacing `MenuBarExtra` with manual `NSStatusItem`)
- [ ] Global keyboard shortcut to toggle the popover (⌃⌥C, user-configurable)
- [ ] Session search across all projects

### v1.6 — Intelligence layer
- [ ] Anomaly detection — flag sessions with unusual cost spikes
- [ ] Cost projection for the current billing period
- [ ] CLAUDE.md improvement suggestions based on quality score

### v2.0 — Team edition
- [ ] iCloud sync for lifetime spend totals across multiple Macs
- [ ] CSV / JSON export
- [ ] Slack / webhook alert integration
- [ ] Team spend aggregation (local network, no external server)

---

## Contributing

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/claux.git
cd claux/apps/desktop/Claux

# 2. Make changes to Sources/Claux/
# 3. Bump AppVersion.current in Sources/Claux/Design.swift (PATCH for fixes, MINOR for features)
# 4. Add a CHANGELOG.md entry
# 5. Build and verify
bash build_app.sh run

# 6. Open a PR against main
```

The codebase follows the conventions in [CLAUDE.md](apps/desktop/Claux/CLAUDE.md) — read it before submitting. Every PR must include a version bump and changelog entry.

---

## Project structure

```
claux/
├── apps/
│   └── desktop/
│       └── Claux/                  # Swift package (macOS app)
│           ├── Sources/Claux/      # All Swift source files
│           ├── Package.swift
│           ├── build_app.sh        # Build + launch script
│           ├── CLAUDE.md           # AI coding guidelines (read by Claude Code)
│           └── CHANGELOG.md        # Full version history
├── docs/                           # Product and engineering docs
├── packages/                       # Shared packages (future)
└── README.md
```

---

## Why this matters

Claude Code's adoption is accelerating. As agentic AI becomes a daily development tool, the gap between "I ran a session" and "I understand what it cost and why" is a real pain point for every professional using it.

Claux is the first native macOS client for that gap. It's built by developers who use Claude Code every day, designed with the constraints that actually matter — no telemetry, no accounts, no servers, no bloat — and it runs invisibly in the background doing exactly one thing well.

---

<p align="center">
  Built with Swift · Designed for macOS · Zero dependencies · Zero backend
  <br />
  <a href="https://github.com/vatskyone/claux">github.com/vatksyone/claux</a>
</p>
