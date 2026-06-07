# Desktop App Overview

The Claux desktop app is a native macOS menu bar application built with Swift and SwiftUI. It lives in your menu bar, uses ~18 MB of RAM at idle, and gives you a real-time window into every Claude Code session running on your machine.

## Architecture

The app is built entirely with native Apple frameworks:

- **UI**: SwiftUI + AppKit (`NSStatusItem`, `NSPopover`, `NSMenu`)
- **File watching**: `DispatchSource` kernel-level events + 10-second poll fallback
- **Notifications**: `UNUserNotificationCenter`
- **Login items**: `SMAppService`
- **No third-party dependencies**

## What it monitors

- All active and recent Claude Code sessions under `~/.claude/projects/`
- Live cost, token breakdown, and context window fill for the active session
- Aggregate spend for today, this week, and this month
- Claude subscription plan-limit usage (5-hour and 7-day windows)
- CLAUDE.md quality score for the active session's project
- Session quality metrics: accepted edits, rejected actions, agent outcomes

## Current version

**v1.15.1** — see the [changelog](https://github.com/vatskyone/claux/blob/main/apps/desktop/Claux/CHANGELOG.md) for the full history.

## Pages in this section

| Page | What it covers |
|---|---|
| [Installation](installation.md) | Building from source, DMG install |
| [Getting Started](getting-started.md) | First launch, onboarding flow |
| [Menu Bar](menu-bar.md) | Icon states, left/right click behavior |
| [Dashboard](dashboard.md) | Active session card, plan limits |
| [Analytics](analytics.md) | Spend charts, breakdowns, budget tracker |
| [Session History](history.md) | Session list, detail sheet, quality panel |
| [Plan Limits](plan-limits.md) | 5-hour and 7-day subscription usage |
| [Notifications](notifications.md) | All alert types, verbosity, scheduling |
| [Settings](settings.md) | Complete settings reference |
| [Claude Integration](claude-integration.md) | Installing the statusLine hook |
| [Troubleshooting](troubleshooting.md) | Gatekeeper, common issues |
