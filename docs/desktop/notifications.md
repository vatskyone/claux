# Notifications

Claux uses `UNUserNotificationCenter` to deliver native macOS notifications. All notifications are optional and configurable.

## Requesting permission

Claux requests notification permission during the onboarding flow on first launch. If you skipped it or denied it, you can:

- Grant it from **System Settings → Notifications → Claux**
- Click **Allow** in **Settings → Notifications → Permission status** inside the app

The permission status row in Settings shows live status: green "Enabled", red "Denied — Open System Settings", or orange "Not requested — Allow".

## Notification types

### Cost threshold alert

Fires once per session when the session's total cost crosses a configured threshold.

- **Trigger**: Session cost ≥ threshold
- **Frequency**: Once per session, regardless of how far over the threshold it goes
- **Content**: Session title or project path, cost so far, burn rate
- **Actions**: Open Session · Open Dashboard · Snooze for today

Configure the threshold in **Settings → Notifications → Cost alert threshold** ($0.50 / $1 / $2 / $5 / $10 / $20 / $50).

### Context window warning

Fires once per session when the context window fill percentage crosses the alert threshold.

- **Trigger**: Context fill % ≥ threshold
- **Frequency**: Once per session
- **Content**: Project path, fill percentage, model name
- **Actions**: Open Session · Open Dashboard

Configure the threshold in **Settings → Notifications → Context window alert** (50% / 70% / 80% / 90%).

### Session ended

Fires when Claux detects a session has ended.

- **Trigger**: Session transitions from active to ended
- **Content**: Session title, total cost, duration, model
- **Actions**: Open Session detail · Open Dashboard

Enable or disable in **Settings → Notifications → Session ended notification**.

### Daily recap

A once-per-day summary of your Claude Code activity.

- **Trigger**: First session update after the configured delivery hour
- **Frequency**: At most once per calendar day
- **Content**: Today's total spend, session count, top project, top model
- **Action**: Opens the in-app Daily Recap sheet

The Daily Recap sheet inside the app shows:
- Spend breakdown for the day
- Top project and model by cost
- Accepted and rejected action counts
- The day's highest-scoring sessions

Configure the delivery hour in **Settings → Notifications → Daily summary** (12pm / 3pm / 6pm / 9pm). The notification is gated by quiet hours and weekday-only settings.

### Weekly recap

A once-per-week summary of the last 7 days.

- **Trigger**: Scheduled delivery (configurable day)
- **Content**: 7-day total spend, top project, top model, accepted/rejected editing outcomes
- **Action**: Opens the Dashboard

Enable and configure in **Settings → Notifications → Weekly recap**.

## Verbosity modes

Claux offers three notification verbosity levels that apply globally:

| Mode | What it changes |
|---|---|
| **Minimal** | Only cost threshold and context alerts. No session-end, daily, or weekly notifications. |
| **Standard** | All alerts enabled with standard content detail. Default. |
| **Detailed** | All alerts with richer session diagnostics in the notification body. |

Set in **Settings → Notifications → Verbosity**.

## Quiet hours

When quiet hours are enabled, Claux suppresses all notifications outside a configured window.

- Configure start and end time in **Settings → Notifications → Quiet hours**
- Notifications that would fire during quiet hours are silently dropped (not deferred)

## Weekday-only summaries

When enabled, daily and weekly recap notifications are only delivered on weekdays (Monday–Friday). Enable in **Settings → Notifications → Weekday only**.

## Snooze

Cost threshold and context window notifications include a **Snooze for today** action. Tapping it suppresses further notifications of that type for the rest of the calendar day. The snooze resets at midnight.

## Testing notifications

Go to **Settings → About → Test notification** to fire a visible "Claux Notifications Work ✓" banner immediately. This verifies the full notification pipeline is working end-to-end.
