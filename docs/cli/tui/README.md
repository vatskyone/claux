# TUI Dashboard

`claux tui` opens a full-screen terminal dashboard built with [Ratatui](https://ratatui.rs). It auto-refreshes every 5 seconds and gives you a live view across six tabs.

## Launching

```bash
claux tui
```

Press `q` at any time to quit.

## Tab structure

```
┌────────────────────────────────────────────────────────────────┐
│  ● Dashboard   Sessions   Analytics   Agents   Skills  History │
└────────────────────────────────────────────────────────────────┘
```

A green `●` dot appears on the Dashboard tab label when a session is active. A green `●` dot appears on the Agents tab label when a sub-agent is currently running.

| Tab | Page | What it shows |
|---|---|---|
| Dashboard | [Dashboard Tab](dashboard-tab.md) | Live session stats, token breakdown, insights, usage panel |
| Sessions | [Sessions Tab](sessions-tab.md) | Scrollable session list with tagging and detail overlay |
| Analytics | [Analytics Tab](analytics-tab.md) | Spend charts, project/model breakdowns, monthly forecast |
| Agents | [Agents Tab](agents-tab.md) | Sub-agent monitoring with XP levels and quality stars |
| Skills | [Skills Tab](skills-tab.md) | Skill list with usage stats and ratings |
| History | [History Tab](history-tab.md) | Project checkpoints — save, review, restore |

## Global keyboard reference

| Key | Action |
|---|---|
| `←` / `→` | Switch to the previous / next tab |
| `h` / `l` | Switch tabs (vim-style) |
| `r` | Force immediate refresh |
| `q` | Quit |

## Sessions tab keyboard reference

| Key | Action |
|---|---|
| `↑` / `↓` | Move session cursor |
| `k` / `j` | Move cursor (vim-style) |
| `Enter` | Open session detail overlay |
| `c` (in detail) | Copy project path to clipboard (macOS) |
| `t` (in detail) | Edit tag inline |
| `Enter` (in tag edit) | Save tag |
| `Esc` | Close detail overlay / cancel tag edit |

## Agents tab keyboard reference

| Key | Action |
|---|---|
| `↑` / `↓` | Move agent cursor |
| `r` | Refresh agent list |

## Skills tab keyboard reference

| Key | Action |
|---|---|
| `↑` / `↓` | Move skill cursor |
| `r` | Refresh skills |

## History tab keyboard reference

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate checkpoints |
| `s` | Save new checkpoint (prompts for name) |
| `w` | Write `.claux/CONTEXT.md` to the project directory |
| `d` | Delete selected checkpoint |
| `Enter` (in name input) | Confirm checkpoint name |
| `Esc` (in name input) | Cancel |

## Auto-refresh

The TUI refreshes all data every 5 seconds automatically. Press `r` to trigger an immediate refresh at any time.

The refresh interval is not currently configurable from the TUI. Use `claux config` to adjust source paths if sessions aren't appearing.
