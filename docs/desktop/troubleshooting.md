# Troubleshooting

## macOS blocks the app on first open

**Symptom:** macOS shows "Apple cannot check it for malicious software" or "Claux cannot be opened because the developer cannot be verified."

**Cause:** Claux is not yet notarized through Apple's developer program. This is expected behavior for ad-hoc signed apps downloaded from the internet.

**Fix:**
1. Right-click (Control-click) `Claux.app` → **Open**
2. Click **Open** in the dialog

After the first approval, macOS remembers the choice and Claux opens normally.

Alternatively, strip the quarantine attribute before mounting the DMG:

```bash
xattr -cr ~/Downloads/Claux-1.15.1-release.dmg
```

---

## No sessions appear after starting Claude Code

**Symptom:** The Dashboard shows an idle state even though a Claude Code session is running.

**Checks:**
1. Go to **Settings → Data Source → Session directory** and confirm it is set to the directory that contains your `projects/` folder. This is usually `~/.claude`.
2. Confirm Claude Code has written at least one JSONL file: `ls ~/.claude/projects/`
3. Click the refresh button (↻) in the popover header to trigger an immediate scan.
4. Check that the session was started after Claux was launched — sessions that started before Claux was open are picked up on the next refresh cycle.

---

## Plan Limits card shows no data

**Symptom:** The Plan Limits card says "statusLine source not running" or "No plan-limit data yet."

**Fix:**
1. Go to **Settings → Data Source → Claude Integration** and check the integration status.
2. If it says "Not installed," click **Install**.
3. Restart Claude Code and start a new session.
4. Send at least one message. The data should populate within a few seconds of the first response.

If the status says "Unmanaged statusLine," you have an existing custom `statusLine` command. Click **Install** anyway — Claux will preserve your existing command inside its wrapper.

---

## Cost figures seem inaccurate

**Symptom:** Claux shows a different cost than Anthropic's billing dashboard.

**Explanation:** Claux calculates costs from local token counts using a built-in pricing table. Discrepancies can occur because:

- Anthropic may apply credits, discounts, or adjustments not reflected in the raw token counts
- The pricing table in Claux may not yet reflect a recent pricing change
- Cache token attribution may differ depending on whether **Include cache cost in totals** is enabled in Settings

**Recommendation:** Always treat Claux cost figures as estimates. Use the [Anthropic console](https://console.anthropic.com) for authoritative billing information.

---

## The context bar is always 0%

**Symptom:** The context health bar stays empty even during an active session.

**Cause:** Claux reads context fill from the most recent assistant message's `usage` object in the JSONL. If the session is very new or hasn't received an assistant response yet, the bar will show 0%.

**Fix:** Wait for at least one complete Claude Code response. The bar updates on the next refresh cycle.

---

## Notifications are not appearing

**Symptom:** Claux is configured to send notifications but none appear.

**Checks:**
1. Go to **Settings → Notifications → Permission status** and confirm the status is "Enabled."
2. Check **System Settings → Notifications → Claux** and confirm alerts are allowed and not silenced by Focus mode.
3. Click **Settings → About → Test notification** to verify the pipeline is working.
4. Check that quiet hours are not currently active (**Settings → Notifications → Quiet hours**).
5. Confirm the relevant notification type is enabled in the Notifications section of Settings.

---

## The menu bar icon has disappeared

**Symptom:** Claux is running but the icon is no longer visible in the menu bar.

**Cause:** This happens when the "Show in Menu Bar" setting is set to **When session is active** and no Claude Code session is currently running.

**Fix:** Start a Claude Code session and the icon will reappear. Alternatively, launch Claux again from Applications to reset the visibility setting to **Always**.

---

## Launch at login is not working

**Symptom:** Claux does not start automatically after logging in.

**Cause:** The `SMAppService` login-item API only works inside a properly bundled `.app`. It has no effect when running the raw `swift build` binary.

**Fix:** Install Claux using the DMG or build it with `bash build_app.sh`. Confirm the app is in the `/Applications` folder before enabling **Launch at login** in Settings.

---

## Getting more help

If none of the above resolves your issue, open a report on GitHub:

[github.com/vatskyone/claux/issues](https://github.com/vatskyone/claux/issues)

Include:
- macOS version (`sw_vers`)
- Claux version (Settings → About → Version)
- Steps to reproduce
- Any relevant output from Console.app filtered to "Claux"
