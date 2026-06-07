# claux skills

Manages Claude Code skills — both built-in skills tracked via usage statistics and custom skills stored in `~/.claude/skills/`.

## Usage

```bash
claux skills list
claux skills new <name>
claux skills export <name>
claux skills export <name> -o /destination/
claux skills import ./path/to/skill
```

## `skills list`

Shows a table of all skills, merging built-in skills (from `skillUsage` in `~/.claude.json`) with custom skills (from `~/.claude/skills/`):

```
  ┌─────────────────┬────────┬──────┬───────────┬──────────┐
  │ Skill           │ Type   │ Uses │ Last used │ Rating   │
  ├─────────────────┼────────┼──────┼───────────┼──────────┤
  │ run             │ built-in│  42  │ 2h ago   │ ★★★★★   │
  │ code-review     │ built-in│  18  │ 1d ago   │ ★★★★☆   │
  │ my-workflow     │ custom  │   7  │ 3d ago   │ ★★★☆☆   │
  └─────────────────┴────────┴──────┴───────────┴──────────┘
```

**Type** column: `built-in` for skills sourced from `~/.claude.json` usage stats, `custom` for skills installed in `~/.claude/skills/`.

## `skills new <name>`

Scaffolds a new custom skill directory at `~/.claude/skills/<name>/` with a starter `SKILL.md` template.

```bash
claux skills new my-deploy-workflow
# → Created ~/.claude/skills/my-deploy-workflow/SKILL.md
```

Edit the generated `SKILL.md` to define the skill's purpose, trigger conditions, and instructions.

## `skills export <name>`

Copies a custom skill directory to the current directory (or a specified destination), making it easy to share with a team or commit to a project repo.

```bash
# Copy to current directory
claux skills export my-workflow

# Copy to a specific destination
claux skills export my-workflow -o ~/projects/shared-skills/
```

## `skills import <path>`

Installs a skill from a local directory into `~/.claude/skills/`.

```bash
claux skills import ./my-workflow
claux skills import ~/downloads/team-skills/deploy-workflow
```

The source directory must contain a `SKILL.md` file. The skill name is taken from the directory name.

## Flags

| Command | Flag | Description |
|---|---|---|
| `export` | `-o PATH` | Destination directory. Default: current directory. |

## Skill ratings

| Uses | Stars |
|---|---|
| 0 | ★☆☆☆☆ |
| 1–2 | ★★☆☆☆ |
| 3–9 | ★★★☆☆ |
| 10–29 | ★★★★☆ |
| 30+ | ★★★★★ |

## TUI integration

Skills are also visible in the **Skills tab** of `claux tui`, which shows the same list with a detail panel for the selected skill's content. See [Skills Tab](../tui/skills-tab.md).
