# CLI Overview

The Claux CLI is a terminal tool built in Rust that reads Claude Code's local session logs and gives you a real-time TUI dashboard, spend summaries, session history, CLAUDE.md management, project checkpoints, and full data export — all from your terminal.

## When to use the CLI vs the desktop app

| Situation | Tool |
|---|---|
| Passive monitoring while you work | Desktop app |
| Querying spend from a script or CI | CLI (`claux spend --json`) |
| Deep session analysis | CLI (`claux tui`) |
| Exporting data to a spreadsheet | CLI (`claux export --format csv`) |
| Generating or improving a CLAUDE.md | CLI (`claux claudemd`) |
| Saving a project checkpoint | CLI (`claux checkpoint save`) |
| Running on Linux | CLI only |

## Current version

**v0.7.2** — see the [changelog](https://github.com/vatskyone/claux/blob/main/apps/cli/CHANGELOG.md) for the full history.

## Pages in this section

| Page | What it covers |
|---|---|
| [Installation](installation.md) | Cargo install, shell completions |
| [Getting Started](getting-started.md) | Quick start, first commands |
| [Configuration](configuration.md) | Budget limits, source paths, config file |
| **Commands** | |
| [claux status](commands/status.md) | Active session card |
| [claux sessions](commands/sessions.md) | Recent session table |
| [claux spend](commands/spend.md) | Spend summary with trend indicators |
| [claux analytics](commands/analytics.md) | Charts, breakdowns, forecasting |
| [claux export](commands/export.md) | JSON and CSV export |
| [claux tag](commands/tag.md) | Session labels |
| [claux checkpoint](commands/checkpoint.md) | Project checkpoints |
| [claux claudemd](commands/claudemd.md) | CLAUDE.md generation and improvement |
| [claux account](commands/account.md) | Account info and skill usage |
| [claux skills](commands/skills.md) | Skill management |
| [claux config](commands/config.md) | Budget and path configuration |
| [claux doctor](commands/doctor.md) | Diagnostics |
| **TUI Dashboard** | |
| [Overview & Keyboard Reference](tui/README.md) | Tab structure, all key bindings |
| [Dashboard Tab](tui/dashboard-tab.md) | Token breakdown, insights, usage panel |
| [Sessions Tab](tui/sessions-tab.md) | Session list, detail overlay, tags |
| [Analytics Tab](tui/analytics-tab.md) | Charts, breakdowns, cost forecast |
| [Agents Tab](tui/agents-tab.md) | Sub-agent monitoring |
| [Skills Tab](tui/skills-tab.md) | Skill list and ratings |
| [History Tab](tui/history-tab.md) | Project checkpoints |
