# Sessions Tab

The Sessions tab shows a scrollable, cursor-navigable list of all Claude Code sessions. Press `Enter` on any row to open the full detail overlay.

## Session list

```
  Sessions (142 total)
  ───────────────────────────────────────────────────────────────────
  ●  just now   1h 14m  sonnet  /Users/you/myproject    $0.84  [refactor]
  ○  2h ago     42m     sonnet  /Users/you/api           $0.31
  ○  yesterday  3h 02m  opus    /Users/you/bigproject    $4.17  [v2 arch]
  ○  2d ago     18m     haiku   /Users/you/scripts       $0.04
```

### Columns

| Column | Description |
|---|---|
| Status | `●` green = active, `○` grey = ended |
| Time | Relative time since the session ended |
| Duration | Session length |
| Model | Abbreviated model (`sonnet`, `opus`, `haiku`) |
| Project | Last two path components of the project directory |
| Cost | Total session cost |
| Tag | `[label]` if a tag is set |

### Navigation

| Key | Action |
|---|---|
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `Enter` | Open session detail overlay |

The list auto-scrolls to keep the cursor row in view. A scroll indicator on the right edge shows relative position in the list.

## Session detail overlay

Pressing `Enter` opens a centered overlay (approximately 80% of the terminal width) showing the full session breakdown.

### Header

- Session title (AI-generated) or project path
- Model badge, color-coded by family
- Active / Ended status with timestamp

### Cost and timing

```
  Cost           $0.84
  Duration       1h 14m
  Burn rate      $0.68/hr
  Projection     $1.52/day
```

### Context and cache

```
  Context fill   67%   134k / 200k   [███████░░░░]
  Cache hit      68%   Grade A
```

### Token breakdown

```
  Input      42,100   ████████████░░░░░░  45%
  Output     18,400   ██████░░░░░░░░░░░░  20%
  Cache R    12,000   ████░░░░░░░░░░░░░░  13%
  Cache W     6,200   ██░░░░░░░░░░░░░░░░   7%
  Thinking    2,000   █░░░░░░░░░░░░░░░░░   2%
```

### Source

The entrypoint where the session was started:

```
  Source    VS Code Extension
```

Possible values: `VS Code Extension`, `Terminal CLI`, `Desktop App`, `JetBrains Plugin`.

### CLAUDE.md quality breakdown

```
  CLAUDE.md quality  82/100  [██████░░]  Good
  ─────────────────────────────────────────────
  ✓ Build          commands present
  ✓ Tests          test runner documented
  ✓ Run            start command present
  ✓ Structure      directory map present
  ✗ Architecture   not found — describe key components
  ✓ Conventions    style guide present
  ✗ Dependencies   not documented — list key libraries
  ✓ Commands       useful commands documented

  Suggestions:
  1. Add an ## Architecture section describing key components
  2. Document major dependencies and why they were chosen
```

### Context quality gauge

```
  Context quality  A  ████████░░░░  67% fill · 68% cache
```

### Tag editing

While the detail overlay is open:

- Press `t` to enter tag-edit mode. A cursor `▌` appears in an input field at the bottom of the overlay.
- Type a tag (max 30 characters).
- Press `Enter` to save. The session list updates immediately to show the new tag.
- Press `Esc` to cancel without saving.

### Copying the project path

Press `c` while the detail overlay is open to copy the session's project path to the macOS clipboard.

### Closing the overlay

Press `Esc` to dismiss the detail overlay and return to the session list.
