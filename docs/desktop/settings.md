# Settings

Open Settings by clicking the gear icon in the popover header, or by right-clicking the menu bar icon → **Settings…**.

---

## General

### Show in Menu Bar

Controls when the Claux icon is visible in the menu bar.

| Option | Behavior |
|---|---|
| **Always** | Icon stays visible at all times. Default. |
| **When session is active** | Icon is hidden when no session is running; appears automatically when one starts. |

Also accessible from the right-click context menu on the menu bar icon.

### Show cost in menu bar

When enabled, the active session cost (or today's total when idle) is displayed as a `$X.XX` label beside the icon. Default: off.

### Show model badge in menu bar

When enabled, the current model name appears as a small capsule beside the icon during an active session. Default: off.

### Launch at login

Registers Claux as a login item using `SMAppService` so it starts automatically when you log in to macOS. Default: off.

> Only works inside a proper `.app` bundle. Has no effect when running the raw `swift build` binary.

### App theme

| Option | Behavior |
|---|---|
| **Auto** | Follows macOS system appearance (light/dark). Default. |
| **Light** | Always light mode. |
| **Dark** | Always dark mode. |

### State colors

Color palette used for progress bars, badges, context health, session stats, and status indicators throughout the app.

| Palette | Description |
|---|---|
| **System** | Standard macOS semantic colors. |
| **Vivid** | Brighter, higher-saturation blue/green/orange/red. |
| **High Contrast** | Maximum contrast for accessibility. |
| **Colorblind Safe** | Blue/green/orange/red palette optimized for common color vision deficiencies. |
| **Soft Contrast** | Muted, lower-saturation version for low-light environments. |

### Cost projection

Controls the period used for the "projected cost" figure in the active session card.

| Option | Projection formula |
|---|---|
| **Daily** | `burn_rate_per_hour × 24` |
| **Weekly** | `burn_rate_per_hour × 168` |
| **Monthly** | `burn_rate_per_hour × 720` |

### Monthly budget

Sets a monthly spend budget for the progress bar in the Analytics tab.

| Option | Behavior |
|---|---|
| **Off** | No budget bar shown. Default. |
| $25 / $50 / $100 / $200 / $500 / $1,000 | Budget bar appears in Analytics with color-coded fill. |

### Auto-refresh interval

How often Claux polls the session directory for changes.

| Option |
|---|
| 5 seconds |
| 10 seconds (default) |
| 30 seconds |
| 60 seconds |

### Session retention

How many days of session history Claux keeps in memory.

| Option |
|---|
| 7 days |
| 14 days |
| 30 days (default) |
| 60 days |
| 90 days |
| 1 year |

---

## Notifications

See [Notifications](notifications.md) for full documentation. Settings in this section:

- **Enable notifications** — master toggle
- **Verbosity** — Minimal / Standard / Detailed
- **Cost alert threshold** — $0.50 / $1 / $2 / $5 / $10 / $20 / $50
- **Context window alert** — 50% / 70% / 80% / 90%
- **Session ended notification** — toggle
- **Daily summary** — toggle + delivery hour (12pm / 3pm / 6pm / 9pm)
- **Weekly recap** — toggle
- **Quiet hours** — toggle + start/end time
- **Weekday only** — toggle
- **Permission status** — live status with Allow / Open System Settings button
- **CLAUDE.md quality alert** — toggle + quality threshold (30 / 50 / 70 / 85)

---

## Data Source

### Session directory

The root directory Claux watches for Claude Code session files. Default: `~/.claude`.

Claux expects to find a `projects/` subdirectory inside this path. If the directory is misconfigured, Claux auto-corrects to `~/.claude` on the next refresh.

Click the folder button to open a directory picker.

### Include cache cost in totals

When enabled (default), cache read and cache write token costs are included in session cost totals and spend summaries. When disabled, only input and output token costs are counted.

This affects the cost display throughout the app, the daily recap, and all notifications. It does not affect the token count display.

### Claude Integration

Shows the status of the Claux `statusLine` integration that feeds plan-limit data. See [Claude Integration](claude-integration.md) for full documentation.

- **Status indicator** — shows whether the integration is installed, healthy, or needs repair
- **Install / Repair button** — installs or reinstalls the managed statusLine wrapper
- **Remove button** — removes the Claux-managed wrapper and restores any previous `statusLine` command

### Erase All Data

Clears all `UserDefaults` settings and forces a full session re-scan on the next refresh. This does not delete Claude Code's session files — only Claux's stored preferences and cache.

---

## About

| Item | Description |
|---|---|
| **Version** | Current Claux version number |
| **Feedback** | Link to open an issue on GitHub |
| **Refresh sessions and plan limits** | Triggers an immediate full refresh of session data and plan-limit data |
| **Test notification** | Fires a test notification to verify the notification pipeline |
| **Reset all settings to default** | Confirmation dialog, then resets every setting to its default value |
