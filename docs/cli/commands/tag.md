# claux tag

Attaches a short label to any session. Tags appear in `claux sessions`, the TUI Sessions tab, and `claux export` output. They are stored in `~/.claude/claux/tags.json` and survive across CLI updates.

## Usage

```bash
# Set a tag
claux tag <session-id-prefix> "label"

# View the current tag for a session
claux tag <session-id-prefix>

# Remove a tag
claux tag <session-id-prefix> -r
```

## Examples

```bash
# Tag a session with a short label
claux tag abc123 "auth refactor"

# Use just enough of the ID to be unique
claux tag abc "v2 migration"

# Check what tag is set
claux tag abc123
# → auth refactor

# Remove the tag
claux tag abc123 -r
```

## Finding session IDs

Session IDs are shown in `claux sessions` output. You only need enough characters to be unique:

```bash
claux sessions
#  ○  2h ago  1h 14m  sonnet  /Users/you/myproject  $0.84
# (session ID: abc123def456...)

claux tag abc123 "my label"   # 6+ characters usually suffices
```

## Flags

| Flag | Description |
|---|---|
| `-r` / `--remove` | Remove the tag from the session |

## Tag limits

Tags are capped at 30 characters. They can contain spaces and most printable characters. Longer values are truncated when displayed in narrow terminal columns.

## Tags in the TUI

In the TUI Sessions tab, tags appear as a `[label]` column in the session list. You can also edit tags inline from the session detail overlay:

1. Open a session detail with `Enter`
2. Press `t` to enter tag-edit mode
3. Type the new tag (max 30 chars), `Enter` to save, `Esc` to cancel

## Tags in exports

The `tag` field is included in both JSON and CSV exports from `claux export`.

## Storage

Tags are stored in `~/.claude/claux/tags.json` as a map of session ID to tag string. They are never written into Claude Code's own session JSONL files.
