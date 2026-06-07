# Getting Started

## First launch

When you open Claux for the first time, the app appears as a `c` monogram in your menu bar. Click the icon to open the popover.

## Onboarding flow

On the first open, Claux shows a three-step onboarding overlay inside the popover:

### Step 1 — Welcome

Introduces Claux and explains what it does. Click **Continue** to proceed.

### Step 2 — Session directory

Claux confirms where it will read Claude Code session logs from. The default is `~/.claude`, which is where Claude Code writes its data on every standard installation.

- If your Claude Code data is in a different location, click **Change…** and select the correct directory. Claux expects the directory to contain a `projects/` subdirectory.
- If the default is correct, click **Continue**.

### Step 3 — Claude Integration

This step installs the Claux `statusLine` hook into your Claude Code settings. The integration enables plan-limit monitoring — without it, the Plan Limits card in the Dashboard will remain empty.

- Click **Install integration** to let Claux install the hook automatically.
- If you prefer to skip this now, click **Skip** — you can install it later from Settings → Data Source → Claude Integration.

See [Claude Integration](claude-integration.md) for full details.

### Step 4 — Notifications

Claux asks for macOS notification permission. Click **Allow** to enable cost alerts, context warnings, daily recaps, and session-end summaries.

If you deny permission here, you can grant it later from **System Settings → Notifications → Claux**, or from within Claux's own Settings panel.

Click **Done** to complete onboarding.

## Starting your first monitored session

1. Open a terminal and start a Claude Code session in any project.
2. Within 10 seconds, the Claux menu bar icon turns green and begins pulsing.
3. Click the icon to see the live session card.

Claux monitors all Claude Code sessions running on your machine simultaneously and automatically switches the active session card to whichever session is currently running.

## What to check first

| What you want to see | Where to find it |
|---|---|
| Current session cost and context | Dashboard tab → Active Session Card |
| Today's total spend | Dashboard tab → Plan Limits or Analytics tab |
| Which project spent the most | Analytics tab → By Project |
| Subscription plan limit usage | Dashboard tab → Plan Limits card |
| Past sessions | History tab |
| Notification setup | Settings → Notifications |

## Adjusting the session directory

If Claux shows no sessions after starting Claude Code, the monitored directory is likely set incorrectly. Go to **Settings → Data Source → Session directory** and confirm it points to the folder that contains the `projects/` subdirectory — this is usually `~/.claude`.
