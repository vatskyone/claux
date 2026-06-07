# Claude Integration

The Claude Integration is a small Python wrapper that Claux installs into your Claude Code settings to enable plan-limit monitoring. Without it, the Plan Limits card in the Dashboard shows no data.

## How it works

Claude Code has a `statusLine` hook — a command that Claude Code calls after each API response and passes the current session state as JSON. Claux uses this hook to receive rate-limit information (5-hour and 7-day usage windows) and writes it to `~/.claude/claux/rate_limits.json`, which the desktop app reads.

The wrapper is a Python script (`statusline_wrapper.py`) that Claux installs into `~/.claude/claux/` and configures in `~/.claude/settings.json`.

## Installing the integration

### During onboarding

The onboarding flow includes a dedicated Claude Integration step. Click **Install integration** to install it automatically.

### From Settings

1. Open **Settings → Data Source → Claude Integration**
2. Click **Install**

Claux will:
1. Copy `statusline_wrapper.py` to `~/.claude/claux/statusline_wrapper.py`
2. Write the `statusLine` command into `~/.claude/settings.json`

If you already have a custom `statusLine` command, Claux detects it and preserves it by calling your original command from within the wrapper.

### After installation

Restart Claude Code (or start a new session) and send at least one message. The Plan Limits card should populate within a few seconds of the first response.

## Repair

If the integration was installed but stopped working (e.g. after a Claude Code update that reset `settings.json`), go to **Settings → Data Source → Claude Integration** and click **Repair**. This reinstalls the wrapper and re-registers the hook.

## Removing the integration

Click **Remove** in **Settings → Data Source → Claude Integration**. Claux removes the `statusLine` entry from `~/.claude/settings.json` and optionally deletes the wrapper script. If you had a custom `statusLine` command before installation, it is restored.

## What the wrapper reads

The wrapper receives Claude Code's status payload as JSON on each call. It extracts:

- 5-hour rolling window usage (tokens and spend)
- 7-day rolling window usage (tokens and spend)
- Reset timestamps for each window

It writes this to `~/.claude/claux/rate_limits.json`. The desktop app monitors this file directly.

## Unmanaged statusLine commands

If Claux detects an existing `statusLine` command in `~/.claude/settings.json` that it doesn't recognize as its own wrapper, it shows an "Unmanaged statusLine" warning in the Plan Limits card and in Settings.

You can still install the Claux integration in this case. The wrapper will call your existing command first, then append the rate-limit data collection.

## Privacy

The wrapper runs entirely locally. It reads data that Claude Code has already written to your machine and writes it to a local file. No data is sent anywhere.
