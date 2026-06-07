# claux checkpoint

Saves and restores named project checkpoints. Each checkpoint records the current git state, session cost, session count, and CLAUDE.md quality score at a point in time — giving you a structured way to mark milestones, capture context, and resume work cleanly.

## Usage

```bash
claux checkpoint save
claux checkpoint save "before auth refactor"

claux checkpoint list

claux checkpoint load <id>
claux checkpoint load <id> --write

claux checkpoint delete <id>
```

## How checkpoints work

When you save a checkpoint, Claux captures:

| Field | Source |
|---|---|
| Git branch | `git rev-parse --abbrev-ref HEAD` |
| Git commit hash | `git rev-parse HEAD` |
| Lifetime project cost | Sum of all session costs for the project |
| Active session cost | Cost of the currently running session, if any |
| Session count | Total number of sessions for the project |
| CLAUDE.md score | Quality score at time of save |
| Files changed | `git diff --name-only` against the prior checkpoint's commit |

Checkpoints are stored in two places:
- **Local index**: `~/.claude/claux/checkpoints/<project-hash>.json` — fast reads by the TUI
- **Per-project copy**: `.claux/checkpoints.json` — committable, travels with the repo

## Commands

### `checkpoint save [name]`

Saves a new checkpoint for the current project (determined by the current working directory).

```bash
# Prompts for a name interactively
claux checkpoint save

# Saves immediately with the given name
claux checkpoint save "before auth refactor"
claux checkpoint save "v2 migration complete"
```

### `checkpoint list`

Shows a table of all checkpoints for the current project:

```
ID    Name                     Saved             Branch    Cost       Files
1     before auth refactor     2026-06-07 10:00  main      $14.20     8
2     v2 migration complete    2026-06-07 15:42  main      $22.50     24
```

### `checkpoint load <id>`

Prints the checkpoint's full context to stdout in Markdown format. Useful for reviewing what was captured or piping into another tool.

```bash
claux checkpoint load 1
```

Output includes: name, date, branch, commit, costs, CLAUDE.md score, and list of files changed since the prior checkpoint.

### `checkpoint load <id> --write`

Writes the context to `.claux/CONTEXT.md` in the current project directory, in addition to printing it to stdout.

```bash
claux checkpoint load 1 --write
```

The `CONTEXT.md` file is structured for agent consumption — add it to your `.claude/` project config or reference it at the start of a session so Claude Code can resume with full context.

### `checkpoint delete <id>`

Removes a checkpoint from both the local index and the per-project copy.

```bash
claux checkpoint delete 1
```

## CONTEXT.md format

The generated `.claux/CONTEXT.md` looks like:

```markdown
# Claux Checkpoint: before auth refactor

**Saved:** 2026-06-07 10:00  
**Branch:** main  
**Commit:** a1b2c3d  

## Cost summary

- Lifetime project cost: $14.20
- Session count: 8

## CLAUDE.md quality

Score: 82/100 (Good)

## Files changed since last checkpoint

- src/auth/middleware.rs
- src/auth/tokens.rs
- tests/auth_test.rs
- ...

## Summary

Checkpoint saved before refactoring the auth middleware to use JWT.
```

## TUI integration

Checkpoints are also accessible from the **History tab** in `claux tui`. See [History Tab](../tui/history-tab.md) for the interactive interface.
