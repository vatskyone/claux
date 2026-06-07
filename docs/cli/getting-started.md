# Getting Started

## Check that it's working

After installation, run:

```bash
claux
```

If Claude Code is running a session, you'll see a live session card. If not, you'll see an idle state with your most recent session.

## Five commands to know first

```bash
# What is happening right now?
claux status

# How much have I spent?
claux spend

# Show my last 20 sessions
claux sessions

# Open the live TUI dashboard
claux tui

# Run diagnostics if something seems wrong
claux doctor
```

## First-run setup

Initialize your configuration with sensible defaults:

```bash
claux config init
```

This walks you through setting your session directory, budget limits, and source paths, then writes them to `~/.claude/claux/config.json`.

You can skip this and run `claux` directly — it will auto-discover your session directory from the standard `~/.claude` location.

## Understanding the data source

Claux reads JSONL files from `~/.claude/projects/`. Each subdirectory corresponds to a Claude Code project, and each `.jsonl` file inside is a session log. Claux watches these files for changes and parses only the ones that have been modified since the last check.

If your Claude Code data is not in `~/.claude`, set the correct path:

```bash
claux config set projects-root /path/to/your/.claude/projects
claux config set sessions-root /path/to/your/.claude/sessions
```

## JSON output

Every command that produces tabular output supports a `--json` flag for machine-readable output:

```bash
claux status --json
claux spend --json
claux sessions --json
claux analytics --json
claux doctor --json
```

This makes the CLI composable with `jq`, shell scripts, and other tools.

## Next steps

- Set up budget limits: [Configuration](configuration.md)
- Explore the live dashboard: [TUI Overview](tui/README.md)
- Export your session history: [claux export](commands/export.md)
- Tag sessions for tracking: [claux tag](commands/tag.md)
- Generate a CLAUDE.md for your project: [claux claudemd](commands/claudemd.md)
