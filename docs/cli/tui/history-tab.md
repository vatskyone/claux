# History Tab

The History tab lets you save, review, and restore named project checkpoints without leaving the TUI. Checkpoints capture git state, session cost, and CLAUDE.md quality at a point in time.

## Layout

The tab is split into two panels:

- **Checkpoint list** — top 40% of the screen
- **Checkpoint detail** — bottom 60% of the screen

## Checkpoint list

```
  Checkpoints — /Users/you/myproject (3)
  ─────────────────────────────────────────────────────────────────────────
  ID  Name                      Saved              Branch    Cost      Files
  1   initial working state     2026-06-05 09:00   main      $4.20     —
  2   before auth refactor      2026-06-06 14:30   main      $8.90     6
  3   auth middleware done       2026-06-07 11:00   main      $14.20    18
```

### Columns

| Column | Description |
|---|---|
| **ID** | Sequential checkpoint number for the project |
| **Name** | Checkpoint name provided at save time |
| **Saved** | Timestamp when the checkpoint was saved |
| **Branch** | Git branch at the time of saving |
| **Cost** | Cumulative lifetime project cost at the time of saving |
| **Files** | Number of files changed since the prior checkpoint (`—` for the first checkpoint) |

### Navigation

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate the checkpoint list |
| `s` | Save a new checkpoint |
| `w` | Write `.claux/CONTEXT.md` for the selected checkpoint |
| `d` | Delete the selected checkpoint |

## Checkpoint detail panel

Selecting a checkpoint updates the detail panel below:

```
  Checkpoint 2 — before auth refactor
  ─────────────────────────────────────────────────────────────────────────
  Saved          2026-06-06 14:30
  Branch         main
  Commit         a1b2c3d

  Cost at save
  Lifetime cost  $8.90
  Session count  5

  CLAUDE.md quality  78/100  ██████░░  Good

  Files changed since checkpoint 1 (6 files)
  ─────────────────────────────────────────
  + src/auth/middleware.rs
  + src/auth/tokens.rs
  ~ src/main.rs
  ~ Cargo.toml
  + tests/auth_test.rs
  ~ README.md

  Actions
  ─────────────────────────────────────────
  [w] Write CONTEXT.md to project   [d] Delete
```

### Detail fields

| Field | Description |
|---|---|
| **Saved** | Full timestamp |
| **Branch** | Git branch at save time |
| **Commit** | Short commit hash at save time |
| **Lifetime cost** | Total project spend at save time |
| **Session count** | Number of sessions for the project at save time |
| **CLAUDE.md quality** | Quality score at save time with bar |
| **Files changed** | Files that changed between this checkpoint's commit and the prior checkpoint's commit |

File change indicators:
- `+` added or new file
- `~` modified
- `-` deleted

## Saving a checkpoint

Press `s` to enter save mode. A name input field appears at the bottom of the screen:

```
  Save checkpoint: █
```

Type a name (any length), then press `Enter` to save or `Esc` to cancel. The new checkpoint appears at the bottom of the list immediately.

## Writing CONTEXT.md

Press `w` with a checkpoint selected to write a `.claux/CONTEXT.md` file into the current project directory. This file contains:

- Checkpoint name, date, branch, and commit
- Lifetime cost and session count
- CLAUDE.md quality score
- List of files changed since the prior checkpoint

The `CONTEXT.md` is designed to be consumed by Claude Code at the start of a new session, giving the agent full context about where the project stands. Reference it in your project's `.claude/` config or mention it explicitly at session start.

## Deleting a checkpoint

Press `d` to delete the selected checkpoint. The deletion is immediate and affects both the local index (`~/.claude/claux/checkpoints/`) and the per-project copy (`.claux/checkpoints.json`).

## CLI equivalent

All History tab operations are also available via the `claux checkpoint` command. See [claux checkpoint](../commands/checkpoint.md) for full documentation.
