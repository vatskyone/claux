# Configuration

Claux stores its configuration in `~/.claude/claux/config.json`. All values are optional — Claux works without any configuration using auto-detected defaults.

## Guided setup

Run the interactive initializer to configure everything at once:

```bash
claux config init
```

## Configuration keys

### `projects-root`

Override the directory Claux scans for Claude Code project session files.

```bash
claux config set projects-root ~/.claude/projects
```

Default: auto-detected as `~/.claude/projects`.

### `sessions-root`

Override the directory Claux scans for active session PID files.

```bash
claux config set sessions-root ~/.claude/sessions
```

Default: auto-detected as `~/.claude/sessions`.

### `weekly-budget`

Set a weekly spend budget in USD. Enables the weekly spend bar in the TUI Usage panel.

```bash
claux config set weekly-budget 50      # $50/week
claux config set weekly-budget 0       # disable
```

When set, the Usage panel in the TUI Dashboard tab shows a progress bar for the current week's spend vs. the budget.

### `plan-5h-limit`

Set the dollar value of your 5-hour Claude plan window. Enables the 5-hour usage bar in the TUI Usage panel.

```bash
claux config set plan-5h-limit 10     # $10 / 5h window
```

This value corresponds to the spending cap on your Claude Pro or Max plan for the 5-hour rolling window.

### `monthly-credit`

Set the dollar value of your monthly Claude credit or spending cap. Enables the credit usage bar in the TUI Usage panel.

```bash
claux config set monthly-credit 200   # $200/month
```

Requires `has_extra_usage` to be enabled on your Claude account (visible in `claux account`).

## Reading and removing values

```bash
# Read a single value
claux config get weekly-budget
# → 50.0

# Remove a value (disables the associated feature)
claux config unset weekly-budget
```

## Viewing the full config

```bash
cat ~/.claude/claux/config.json
```

## Config file format

```json
{
  "weekly_budget": 50.0,
  "plan_5h_limit": 10.0,
  "monthly_credit": 200.0,
  "projects_root": "/Users/you/.claude/projects",
  "sessions_root": "/Users/you/.claude/sessions"
}
```

All fields are optional. Unset fields fall back to auto-detection or disabled state.

## Data stored by Claux

Claux stores all its own data under `~/.claude/claux/`:

| File | Contents |
|---|---|
| `config.json` | Budget limits and path overrides |
| `tags.json` | Session tags (set with `claux tag`) |
| `checkpoints/<project-hash>.json` | Checkpoint index per project |
| `rate_limits.json` | Plan-limit data (written by the statusLine integration) |
| `local_metrics.json` | On-device usage metrics (command counts, latency buckets) |

None of these files contain your prompts, code, or session contents.
