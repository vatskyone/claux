# Menu Bar

## Icon states

The Claux menu bar icon communicates session status at a glance.

| State | Appearance | Meaning |
|---|---|---|
| **Idle** | Static `c` monogram, system text color | No active Claude Code session detected |
| **Active** | Green `c` monogram with radial pulse animation | A Claude Code session is currently running |
| **Cost overlay** | `$X.XX` beside the icon | Active session cost (when "Show cost in menu bar" is enabled) |
| **Model badge** | Model name capsule beside the icon | Model currently in use (when "Show model badge in menu bar" is enabled) |

The overlay and badge can be toggled independently in **Settings → General**.

## Left click

Left-clicking the Claux icon toggles the popover open and closed. The popover opens centered beneath the icon and stays open until you click away or click the icon again.

Every time the popover opens, Claux performs an immediate refresh of both session data and plan-limit data — so the numbers are always current when you glance at it.

## Right click

Right-clicking (or Control-clicking) the icon shows a native context menu:

| Menu item | Action |
|---|---|
| **Settings…** | Opens the Settings window |
| **Show in Menu Bar → Always** | Keep the icon visible at all times |
| **Show in Menu Bar → When session is active** | Hide the icon when no Claude Code session is running |
| **Quit Claux** | Quit the app |

## Menu bar visibility

You can control when the icon is visible:

- **Always** — the icon stays in the menu bar at all times. This is the default.
- **When session is active** — the icon is hidden when no Claude Code session is detected and appears automatically when one starts.

This setting is accessible from the right-click context menu or from **Settings → General → Show in Menu Bar**.

> **Note:** If you set visibility to "When session is active" and there is currently no active session, the icon will disappear. To bring it back, start a Claude Code session or relaunch Claux from Applications.

## The popover

The popover is a 340-point-wide panel that contains three tabs:

- **Dashboard** — active session stats and plan limits
- **Analytics** — spend charts and breakdowns
- **History** — recent session list and search

The tab bar is pinned at the bottom of the popover. The content area height is fixed at 340 pt so switching tabs never resizes the window.

Navigation between tabs is covered in the [Dashboard](dashboard.md), [Analytics](analytics.md), and [Session History](history.md) pages.
