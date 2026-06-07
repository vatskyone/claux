# Skills Tab

The Skills tab shows all Claude Code skills tracked by Claux — both built-in skills from usage statistics and custom skills installed in `~/.claude/skills/`.

## Layout

The tab is split into two panels:

- **Skill list** — top 40% of the screen
- **Skill detail** — bottom 60% of the screen

## Skill list

```
  Skills (8)
  ──────────────────────────────────────────────────────────────
  ● run               built-in   42 uses   2h ago    ★★★★★
  ● code-review       built-in   18 uses   1d ago    ★★★★☆
  ○ my-workflow       custom      7 uses   3d ago    ★★★☆☆
  ○ ultrareview       built-in    3 uses   5d ago    ★★★☆☆
  ○ deploy-helper     custom      0 uses   never     ★☆☆☆☆
```

### Columns

| Column | Description |
|---|---|
| **Type** | `●` = built-in (from `~/.claude.json`), `○` = custom (from `~/.claude/skills/`) |
| **Name** | Skill name |
| **Source** | `built-in` or `custom` |
| **Uses** | Total invocation count |
| **Last used** | Relative time of last invocation |
| **Rating** | 1–5 stars based on usage count |

### Navigation

| Key | Action |
|---|---|
| `↑` / `↓` | Move skill cursor |
| `r` | Refresh skills list |

## Skill detail panel

Moving the cursor updates the detail panel for the selected skill:

```
  Skill detail — run
  ──────────────────────────────────────────────────────────────
  Type        built-in
  Uses        42
  Last used   2h ago
  Rating      ★★★★★  (30+ uses)

  Description
  Runs the project's application in development mode. Supports
  various project types: Node.js, Python, Rust, Go, etc.
```

For custom skills that have a `SKILL.md` file, the full content of the file is shown in the detail panel (scrollable).

## Rating thresholds

| Uses | Stars |
|---|---|
| 0 | ★☆☆☆☆ |
| 1–2 | ★★☆☆☆ |
| 3–9 | ★★★☆☆ |
| 10–29 | ★★★★☆ |
| 30+ | ★★★★★ |

## Managing skills from the CLI

Skills are managed via `claux skills` commands — creating, exporting, and importing is not done from within the TUI. See [claux skills](../commands/skills.md) for full documentation.
