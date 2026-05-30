# CLAUX — AI Development Guide

This file is read automatically by Claude Code at the start of every session.
Follow these rules on every change, no matter how small.

---

## Mandatory workflow for every code change

### 1. Bump the version
Every time you create or modify a `.swift` file, update `AppVersion.current` in:
```
Sources/Claux/Design.swift
```
Version format: `MAJOR.MINOR.PATCH`
- **PATCH** — bug fixes, UI tweaks, copy changes, small improvements
- **MINOR** — new features, new views, new data model fields
- **MAJOR** — architectural rewrites, breaking API changes

The version string in `Design.swift` is the **single source of truth**. Do not hardcode version strings anywhere else — both `PopoverView.swift` and `SettingsView.swift` already reference `AppVersion.current`.

### 2. Update CHANGELOG.md
After every batch of changes in a session, append a new entry at the top of:
```
CHANGELOG.md
```
Format:
```markdown
## [X.Y.Z] — YYYY-MM-DD

### New Features
- **Feature name** (`FileName.swift`) — what it does and why.

### Improvements
- **What changed** (`FileName.swift`) — what was changed and the effect.

### Bug Fixes
- **What was fixed** (`FileName.swift`) — what was broken and how it's fixed.

### Internal
- Non-user-visible changes (refactors, deleted files, new structs, etc.)
```
Only include sections that have entries. Skip empty sections.

### 3. Build and verify
After every set of changes run:
```bash
bash build_app.sh run
```
This builds the `.app` bundle and re-launches. Always confirm `Build complete!` before declaring work done.

---

## Project structure

```
Sources/Claux/
├── ClauxApp.swift          # App entry point, MenuBarExtra, WindowGroups, MenuBarLabel
├── AppStore.swift          # Central @ObservableObject — sessions, spend, analytics data
├── SessionMonitor.swift    # File-system watcher + mtime cache, 10 s poll timer
├── SessionParser.swift     # JSONL → ClaudeSession (tokens, cost, title, entrypoint)
├── NotificationManager.swift # UNUserNotificationCenter alerts (cost, context, session-end)
├── Models.swift            # ClaudeSession, TokenUsage, SpendSummary, DailySpend, etc.
├── Design.swift            # AppVersion, colours, ModelInfo, Format helpers, CardStyle
└── Views/
    ├── PopoverView.swift       # Menu bar popover root
    ├── ActiveSessionCard.swift # Live session stats card
    ├── RecentSessionsView.swift
    ├── SessionRowView.swift    # Tappable row → SessionDetailSheet
    ├── SessionDetailSheet.swift
    ├── SpendSummaryView.swift
    ├── NoActiveSessionView.swift
    ├── ContextHealthBar.swift
    ├── TokenBreakdownView.swift
    ├── SettingsView.swift      # Settings panel (General/Notifications/Data/Account/About)
    └── AnalyticsView.swift     # Analytics window (daily chart, per-project, per-model)
```

---

## Key conventions

- **Active session detection** — primary: `~/.claude/sessions/<pid>.json` contains `sessionId` matching the JSONL filename. Fallback: file mtime < 90 s.
- **Context window fill** — always use the LAST assistant message's `input_tokens + cache_read_input_tokens + cache_creation_input_tokens`. Never sum across all turns.
- **Incremental parsing** — `SessionMonitor` caches parsed sessions by `(URL, mtime)`. Only re-parse files whose `contentModificationDate` has changed.
- **Notifications guard** — check `NotificationManager.notificationsAvailable` (requires `Bundle.main.bundleIdentifier != nil`) before any `UNUserNotificationCenter` call.
- **Launch at login** — `SMAppService.mainApp.register() / unregister()`. Only works inside a proper `.app` bundle.
- **Model naming** — use `ModelInfo.shortName(_:)` everywhere. Never hardcode "Sonnet", "Haiku", etc.
- **Formatting** — use `Format.cost()`, `Format.tokens()`, `Format.duration()`, `Format.relativeTime()` — never format these inline.
- **Colors** — use the `Color.claux*` semantic aliases from `Design.swift` — never hardcode `nsColor` calls in views directly unless adding a new semantic concept.

---

## Building

```bash
# Build + launch .app bundle (required for notifications + login items)
bash build_app.sh run

# Build only (no launch, faster for compile checks)
swift build
```

The raw `swift build` binary is useful for fast compile-error checks but lacks a `CFBundleIdentifier`, so notifications are silently disabled.

---

## Current version: 1.7.2
