# claux config

Manages Claux configuration stored in `~/.claude/claux/config.json`. Configuration is optional — Claux works without any config using auto-detected defaults.

## Usage

```bash
claux config init
claux config set <key> <value>
claux config get <key>
claux config unset <key>
```

## `config init`

Interactive guided setup. Prompts for each configurable value with sensible defaults and writes the result to `~/.claude/claux/config.json`.

```bash
claux config init
```

Run this once after installation to configure budget limits and source paths.

## `config set <key> <value>`

Sets a configuration value.

```bash
claux config set weekly-budget 50
claux config set plan-5h-limit 10
claux config set monthly-credit 200
claux config set projects-root ~/.claude/projects
claux config set sessions-root ~/.claude/sessions
```

## `config get <key>`

Reads a single configuration value.

```bash
claux config get weekly-budget
# → 50.0

claux config get projects-root
# → /Users/you/.claude/projects
```

Returns an empty line if the key is not set.

## `config unset <key>`

Removes a configuration value, reverting the associated feature to its disabled or auto-detected state.

```bash
claux config unset weekly-budget   # disables weekly budget bar
claux config unset projects-root   # reverts to auto-detection
```

## Configuration keys

| Key | Type | Default | Description |
|---|---|---|---|
| `weekly-budget` | float (USD) | unset | Weekly spend budget. Enables the weekly bar in the TUI Usage panel. |
| `plan-5h-limit` | float (USD) | unset | 5-hour plan window limit. Enables the 5h bar in the TUI Usage panel. |
| `monthly-credit` | float (USD) | unset | Monthly credit cap. Enables the credit bar in the TUI Usage panel. |
| `projects-root` | path | `~/.claude/projects` | Directory containing Claude Code project JSONL files. |
| `sessions-root` | path | `~/.claude/sessions` | Directory containing Claude Code active-session PID files. |

## Config file location

```
~/.claude/claux/config.json
```

The file uses snake_case keys internally:

```json
{
  "weekly_budget": 50.0,
  "plan_5h_limit": 10.0,
  "monthly_credit": 200.0,
  "projects_root": "/Users/you/.claude/projects",
  "sessions_root": "/Users/you/.claude/sessions"
}
```

You can edit this file directly if preferred — Claux reads it fresh on each command invocation.
