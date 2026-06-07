<h1 align="center">claux</h1>
<p align="center"><strong>Claude Code session tracker — terminal CLI + live TUI dashboard</strong></p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-black?style=flat-square" />
  <img src="https://img.shields.io/badge/rust-2021-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/no%20backend-local%20only-green?style=flat-square" />
  <img src="https://img.shields.io/badge/version-0.7.2-informational?style=flat-square" />
</p>

---

## What is claux?

Claude Code bills per token. When you run long agentic sessions across multiple projects, the cost compounds quickly — and Claude Code's own interface gives you no aggregate view, no historical chart, and no session history.

**claux is a terminal tool that reads Claude Code's local session logs and surfaces everything Claude Code doesn't.**

No account. No backend. No data ever leaves your machine. claux reads directly from `~/.claude/projects/` JSONL files and gives you a real-time TUI dashboard, spend summaries, session history, skill management, and full data export — all in your terminal.

---

## The problem

| Problem | Impact |
|---|---|
| No aggregate cost view | Surprise bills at month-end |
| No context-window awareness | Model degradation before you notice |
| No session history | Can't attribute spend to projects |
| No spend pacing | No way to stay under a weekly/monthly budget |
| No agent visibility | Sub-agent runs are a black box |

---

## Installation

### From source (requires Rust)

```bash
git clone https://github.com/vatskyone/claux.git
cd claux/apps/cli

# Build and install to ~/.cargo/bin/claux
cargo install --path .
```

### Shell completions

```bash
# zsh
claux completions zsh > ~/.zsh/completions/_claux
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -U compinit && compinit' >> ~/.zshrc

# bash
claux completions bash >> ~/.bashrc

# fish
claux completions fish > ~/.config/fish/completions/claux.fish
```

---

## Quick start

```bash
# See what's happening right now
claux

# Open the live TUI dashboard
claux tui

# How much have I spent?
claux spend

# Last 20 sessions
claux sessions
```

---

## Commands

### `claux` / `claux status`

Active session card — shows everything about the currently running Claude Code session.

```
  ● Active session
  Project    /Users/snow/myproject
  Model      claude-sonnet-4-6
  Duration   1h 14m
  Cost       $0.84
  Burn rate  $0.68/hr   → est. $1.52 by EOD
  Context    ████████░░░░  42%  84k / 200k
  Cache hit  68%  (Grade A)
  Tokens     Input 42k · Output 18k · Cache R 12k · Cache W 6k · Thinking 2k
  CLAUDE.md  82/100  Good
```

Flags: `--json`

---

### `claux sessions [-n N] [--json]`

Colored table of recent sessions with status dot, relative time, duration, model, project path, cost, and tag.

```
 ●  just now   1h 14m  sonnet  /Users/snow/myproject   $0.84  [refactor]
 ○  2h ago     42m     sonnet  /Users/snow/api          $0.31
 ○  yesterday  3h 02m  opus    /Users/snow/bigproject   $4.17  [v2 arch]
```

Flags: `-n N` (default 20), `--json`

---

### `claux spend [--json]`

Today / this week / this month spend with trend arrows vs the prior period.

```
  Today       $0.84   ↑ from $0.21 yesterday
  This week   $3.42   ↓ from $5.11 last week
  This month  $18.70
```

Flags: `--json`

---

### `claux analytics [--days N] [--json]`

30-day ASCII sparkline, daily spend table, breakdown by project, and breakdown by model with efficiency rating (K output tokens per dollar).

```
  Daily spend (30 days)
  ▁▁▂▃▁▂▄▇█▅▃▂▁▁▃▄▅▃▂▁▁▂▃▄▅▆▇█▅▃

  By Model
  claude-sonnet-4-6   $14.20   89K tok/$
  claude-opus-4-7     $4.50    12K tok/$
```

Flags: `--days N` (default 30), `--json`

Local-only product metrics:

```bash
claux analytics local
claux analytics local --json
claux analytics local --reset
```

---

### `claux export`

Dump all session history as JSON or CSV — useful for spreadsheets, billing audits, or piping into other tools.

```bash
claux export                          # JSON to stdout
claux export --format csv             # CSV to stdout
claux export --format csv -o data.csv # write to file
claux export -n 100                   # last 100 sessions only
```

CSV columns: `id, project_path, start_time, end_time, duration_secs, cost_usd, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens, thinking_tokens, is_active, title, tag`

---

### `claux tag <session-id-prefix> [label] [-r]`

Attach a short label to any session. Tags appear in `claux sessions`, `claux tui`, and `claux export`.

```bash
claux tag abc123 "refactor"   # set tag
claux tag abc123              # show current tag
claux tag abc123 -r           # remove tag
```

Tags persist in `~/.claude/claux/tags.json`. The session ID prefix only needs to be long enough to be unique.

---

### `claux account`

Account card reading your plan, billing, and org info from `~/.claude.json`, plus a skill usage table ranked by rating.

```
  Account

  Name:             Snow
  Email:            snow@example.com
  Plan:             Claude Pro
  Organization:     Personal
  Role:             admin
  Billing:          stripe_subscription
  Rate tier:        default_claude_ai
  Account since:    2024-03-15
  Subscribed since: 2024-03-15
  Extra usage:      enabled

  Skills

  ┌─────────────────┬──────┬───────────┬─────────┐
  │ Skill           │ Uses │ Last used │ Rating  │
  ├─────────────────┼──────┼───────────┼─────────┤
  │ run             │  42  │ 2h ago    │ ★★★★★  │
  │ code-review     │  18  │ 1d ago    │ ★★★★☆  │
  │ ultrareview     │   3  │ 5d ago    │ ★★★☆☆  │
  └─────────────────┴──────┴───────────┴─────────┘
```

---

### `claux skills`

Manage Claude Code skills — both built-in skills (tracked via usage stats) and custom skills in `~/.claude/skills/`.

```bash
claux skills list                      # table of all skills with ratings
claux skills new my-workflow           # scaffold ~/.claude/skills/my-workflow/SKILL.md
claux skills export my-workflow        # copy to current directory
claux skills export my-workflow -o ~/  # copy to ~/my-workflow/
claux skills import ./my-workflow      # install from a local directory
```

Skill ratings (1–5 stars) based on total invocations:

| Uses | Rating |
|---|---|
| 0 | ★☆☆☆☆ |
| 1–2 | ★★☆☆☆ |
| 3–9 | ★★★☆☆ |
| 10–29 | ★★★★☆ |
| 30+ | ★★★★★ |

---

### `claux config`

Set budget limits stored in `~/.claude/claux/config.json`.

```bash
claux config init                       # initialize defaults and source paths
claux config set weekly-budget 50     # set $50/week budget
claux config set plan-5h-limit 10     # set $10 / 5h usage limit
claux config set monthly-credit 200   # set $200/month credit cap
claux config set projects-root ~/.claude/projects
claux config set sessions-root ~/.claude/sessions
claux config get weekly-budget        # → 50.0
claux config unset weekly-budget      # remove the limit
```

Valid keys:
- `weekly-budget`
- `plan-5h-limit`
- `monthly-credit`
- `projects-root`
- `sessions-root`

---

### `claux doctor [--json]`

Read-only diagnostics for session discovery and parse health.

```bash
claux doctor
claux doctor --json
```

---

### `claux claudemd`

Generate or improve a project-level `CLAUDE.md`.

```bash
claux claudemd generate --project /path/to/repo
claux claudemd generate --project /path/to/repo --write
claux claudemd generate --project /path/to/repo --write --force

claux claudemd improve --project /path/to/repo
claux claudemd improve --project /path/to/repo --write
claux claudemd improve --project /path/to/repo --write --backup
```

Notes:
- `generate` creates a starter file from local repository structure and command conventions.
- `improve` requires an existing `CLAUDE.md` and fills missing high-signal sections while preserving existing content.
- Both subcommands support `--json` for machine-readable output.

---

### `claux checkpoint`

Save and restore named project checkpoints. Each checkpoint records git state, session cost, and CLAUDE.md quality at a point in time. Checkpoints are stored both locally (`~/.claude/claux/checkpoints/<project-hash>.json`) and inside the project (`.claux/checkpoints.json`) so they can be committed and shared with the repo.

```bash
claux checkpoint save                      # prompt for a name, then save
claux checkpoint save "before auth refactor"

claux checkpoint list                      # table of all checkpoints for current project

claux checkpoint load <id>                 # print checkpoint context as Markdown to stdout
claux checkpoint load <id> --write         # also write .claux/CONTEXT.md into the project

claux checkpoint delete <id>               # remove a checkpoint
```

The `--write` flag produces a structured `.claux/CONTEXT.md` that agents can load at session start to resume with full context: git branch, commit, cost, changed files, and CLAUDE.md score.

---

### `claux tui`

Full-screen ratatui TUI dashboard. Press `q` to quit, `←`/`→` (or `h`/`l`) to switch tabs, `r` to refresh.

```
┌────────────────────────────────────────────────────────────────┐
│  ● Dashboard   Sessions   Analytics   Agents   Skills  History │
└────────────────────────────────────────────────────────────────┘
```

---

## TUI — tab by tab

### Dashboard tab

Split into two columns. Left column has two panels stacked vertically:

**Tokens panel** — horizontal bar chart per token type:
```
Input    ███████████░░░░░░░  42,100   45%
Output   ██████░░░░░░░░░░░░  18,400   20%
Cache R  ████░░░░░░░░░░░░░░  12,000   13%
Cache W  ██░░░░░░░░░░░░░░░░   6,200    7%
Thinking █░░░░░░░░░░░░░░░░░   2,000    2%
─────────────────────────────────────────
Total 93,200 tokens · Cache hit 68%  Grade A
```

**Usage panel** — context fill, weekly spend vs budget, credit status:
```
Context window  ████████░░░░  42%  84k / 200k
This week       ██░░░░░░░░░░  $3.42 / $50.00  resets Mon 2026-06-01
Credit          enabled  $18.70 of $200.00 used
```

Right column — **Insights panel** (updates every 5 s):
```
Cache efficiency  A  68% — optimal
Context health    ✓ Healthy  42% full
Cost projection   $0.84 so far · $0.68/hr · est. $1.52 EOD
Model             claude-sonnet-4-6
Thinking          8% of output
Efficiency        9.4K tok/$
CLAUDE.md         82/100  ██████░░  Good
Context quality   A  cache 68% · fill 42%
```

When no session is active, the panel shows lifetime stats: total sessions, total spend, avg cost/session, overall cache-hit %, and the best cache-hit session.

---

### Sessions tab

Scrollable list of all sessions. `↑`/`↓` (or `k`/`j`) to move; `Enter` to open the detail overlay; `c` to copy the project path to clipboard (macOS).

**Session detail overlay** shows:
- Project path, model badge, active/ended status
- Cost, duration, burn rate, context fill %, cache hit %
- Full token breakdown bars
- **Source** — `VSCode Extension` / `Terminal CLI` / `Desktop App` / `JetBrains Plugin`
- **CLAUDE.md breakdown** — score + bar + label + ✓/✗ per category (Build, Run, Tests, Structure, Conventions, Commands) + up to 4 actionable suggestions
- Context gauge bar
- Tag (press `t` inside the overlay to edit inline; `Enter` saves, `Esc` cancels)

---

### Analytics tab

- **7-day bar chart** — one proportional `█` column per day, today highlighted blue, per-day cost labels and day-of-week axis
- **30-day sparkline** — compact trend line
- **By Project / By Model tables** — side by side, model table includes `K tok/$` efficiency column

---

### Agents tab

Monitors every sub-agent spawned by Claude Code in the active session.

**Agent list** (top 38% of screen):
```
● Explore    Lv3 [████░░░░]  Find API endpoints in src/   ★★★★☆  $0.02  14s
✓ Plan       Lv2 [██░░░░░░]  Design auth refactor          ★★★★★  $0.08  42s
✗ general    Lv1 [░░░░░░░░]  Search for test helpers       ★★☆☆☆  $0.01   8s
```

**Agent detail panel** (bottom 62%) — full prompt preview, token bars as % of parent session, output preview, quality label. Press `r` to refresh.

**XP / Level system** — cumulative across all sessions:

| Level | Tasks completed |
|---|---|
| Lv.1 | 1–4 |
| Lv.2 | 5–14 |
| Lv.3 | 15–29 |
| Lv.4 | 30–59 |
| Lv.5 | 60+ |

**Quality stars** computed from output completeness and length:

| Stars | Meaning |
|---|---|
| ★★★★★ | Rich output (≥ 500 chars, no errors) |
| ★★★★☆ | Good output |
| ★★★☆☆ | Moderate output |
| ★★☆☆☆ | Minimal or contains error keywords |
| ★☆☆☆☆ | Did not complete |

A green `●` dot on the Agents tab label means at least one agent is still running.

---

### Skills tab

Skill list (top 40% of screen) + detail panel (bottom 60%).

**Skill list** — name, type (custom `●` / built-in `○`), uses, last used, rating stars. `↑`/`↓` to navigate; `r` to refresh.

**Skill detail** — description, usage count, last used timestamp, rating, and SKILL.md content preview for custom skills.

---

### History tab

Browse, save, and restore named project checkpoints without leaving the TUI.

**Checkpoint list** (top 40%) — columns: ID · Name · Saved date · Git branch · Total cost · Files changed since prior checkpoint.

**Checkpoint detail** (bottom 60%) — name, date, branch + commit hash, cost breakdown, CLAUDE.md score, list of files changed since the prior checkpoint, and action hints.

**Inline actions:**

| Key | Action |
|---|---|
| `s` | Save a new checkpoint — type a name, `Enter` to confirm, `Esc` to cancel |
| `w` | Write `.claux/CONTEXT.md` into the project directory (agent-loadable context file) |
| `d` | Delete the selected checkpoint |
| `↑` / `↓` | Navigate checkpoint list |

Each checkpoint captures: git branch + commit, lifetime project cost, active session cost, session count, and CLAUDE.md score at the time of saving. The per-project copy at `.claux/checkpoints.json` is committable and travels with the repo.

---

## Configuration

### Budget limits

Set via `claux config`:
- `plan-5h-limit` — enables the 5-hour usage bar in the Usage panel
- `weekly-budget` — enables the weekly spend bar in the Usage panel
- `monthly-credit` — enables the credit usage bar (requires `has_extra_usage` enabled on your account)
- `projects-root` / `sessions-root` — override log discovery paths

Stored in `~/.claude/claux/config.json`.

### Session tags

Stored in `~/.claude/claux/tags.json`. Survive across CLI updates — never stored inside Claude Code's own session files.

---

## Architecture

```
apps/cli/src/
├── main.rs           # CLI entry point · clap subcommand routing
├── models.rs         # ClaudeSession · TokenUsage · SpendSummary · AccountInfo
│                     # ClaudemdAnalysis · SkillInfo · ClauxConfig · AgentRun
├── parser.rs         # JSONL → ClaudeSession · per-model pricing
│                     # score_claudemd() · score_claudemd_detailed() · find_claudemd_path()
│                     # parse_agents() · two-pass tool_use/tool_result matching
├── monitor.rs        # Session discovery under ~/.claude/projects/
│                     # mtime cache · active-ID detection · agent loading
├── spend.rs          # compute_spend · compute_daily_spend
│                     # compute_project_breakdown · compute_model_breakdown
│                     # compute_monthly_forecast · MonthlyForecast
├── format.rs         # cost · tokens · duration · relative_time
│                     # model_short_name · project_path helpers
├── render.rs         # comfy-table + owo-colors helpers
│                     # make_table · active_dot · cost_colored · context_bar
│                     # spend_sparkline · trend · model_colored
├── account.rs        # load_account_info() from ~/.claude.json
│                     # load_skill_usage() from skillUsage block
├── skills.rs         # load_skills() — merges disk + usage stats
│                     # skill_rating() — uses → 1–5 star rating
├── config.rs         # load_claux_config() / save_claux_config()
│                     # backed by ~/.claude/claux/config.json
├── claudemd.rs       # CLAUDE.md generation/improvement helpers
├── checkpoints.rs    # checkpoint persistence + context writer
├── metrics.rs        # local-only usage metrics counters
├── usage.rs          # shared usage-window/progress state helpers
├── tags.rs           # load_tags() / save_tag()
│                     # backed by ~/.claude/claux/tags.json
└── commands/
    ├── status.rs     # claux status
    ├── sessions.rs   # claux sessions
    ├── spend.rs      # claux spend
    ├── analytics.rs  # claux analytics (+ local metrics)
    ├── export.rs     # claux export (JSON + CSV)
    ├── tag.rs        # claux tag
    ├── account.rs    # claux account
    ├── skills.rs     # claux skills list|new|export|import
    ├── config.rs     # claux config get|set|unset|init
    ├── claudemd.rs   # claux claudemd generate|improve
    ├── doctor.rs     # claux doctor
    ├── checkpoint.rs # claux checkpoint save|list|load|delete
    └── tui.rs        # ratatui TUI — App state machine · all draw_* functions
```

### Data flow

```
~/.claude/projects/<encoded-path>/*.jsonl
        │
        ▼  mtime cache — only re-parses changed files
monitor::load_sessions()
        │  → Vec<ClaudeSession> with dailyCosts, tokenUsage, agentRuns
        ▼
commands::*::run(&sessions)          claux status / sessions / spend / analytics
        OR
commands::tui::run()                 ratatui event loop — 5 s auto-refresh
        │  → App state machine
        ▼
draw_dashboard / draw_sessions_list / draw_analytics / draw_agents_screen / draw_skills_screen / draw_history_screen
```

### Key design decisions

**Per-turn cost attribution** — Sessions that span midnight are attributed per-turn using the timestamp of each assistant response, not the session start time. `dailyCosts: HashMap<NaiveDate, f64>` in `ClaudeSession` stores cost keyed by local-timezone day. This makes today's spend figure accurate regardless of when a session started.

**Incremental mtime cache** — `SessionCache` maps `PathBuf → (SystemTime, ClaudeSession)`. Only files whose modification time has changed are re-parsed on each refresh. In a large workspace with one active session, exactly one file is re-parsed per tick.

**Active session detection** — Two signals combined with OR: (1) `~/.claude/sessions/<pid>.json` contains a `sessionId` matching the JSONL filename; (2) file mtime < 90 seconds. Handles both clean exits and crashed sessions.

**Sub-agent matching** — `parse_agents()` does a two-pass scan of the session JSONL: first pass collects all `tool_use` events with `name = "Agent"`, second pass matches each to its `tool_result` by `tool_use_id`. The agent's own JSONL file (in the `subagents/` companion directory) is then parsed for per-token breakdown. This keeps agent parsing fully offline and self-contained.

**CLAUDE.md lazy analysis** — Full `ClaudemdAnalysis` (per-category breakdown, suggestions) is computed on demand when the session detail overlay opens, not during initial session load. It's cached in `App.detail_analysis` and recomputed only when a different session is selected.

---

## Pricing reference

| Model | Input | Output | Cache read | Cache write |
|---|---|---|---|---|
| claude-opus-4.x | $15.00 / M | $75.00 / M | $1.50 / M | $18.75 / M |
| claude-sonnet-4.x | $3.00 / M | $15.00 / M | $0.30 / M | $3.75 / M |
| claude-haiku-4.x | $0.80 / M | $4.00 / M | $0.08 / M | $1.00 / M |

Model ID matching uses substring (`opus` / `sonnet` / `haiku`) so new model versions are picked up automatically.

---

## Keyboard reference

### Global

| Key | Action |
|---|---|
| `←` / `→` or `h` / `l` | Switch tabs |
| `q` | Quit |
| `r` | Force refresh |

### Sessions tab

| Key | Action |
|---|---|
| `↑` / `↓` or `k` / `j` | Move cursor |
| `Enter` | Open session detail |
| `c` (in detail) | Copy project path to clipboard (macOS) |
| `t` (in detail) | Edit tag inline |
| `Esc` | Close detail overlay |

### Agents tab

| Key | Action |
|---|---|
| `↑` / `↓` | Move agent cursor |
| `r` | Refresh agent list |

### Skills tab

| Key | Action |
|---|---|
| `↑` / `↓` | Move skill cursor |
| `r` | Refresh skills |

### History tab

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate checkpoints |
| `s` | Save new checkpoint |
| `w` | Write `.claux/CONTEXT.md` |
| `d` | Delete selected checkpoint |

---

## Roadmap

### v0.8.0 — Alerts & automation
- [ ] `claux watch` — stay-running process that posts macOS notifications when cost or context thresholds are crossed
- [ ] `--cost-alert N` and `--context-alert N` flags
- [ ] Webhook / Slack integration for spend alerts

### v0.9.0 — Team & sync
- [ ] `claux export --since <date>` for incremental exports
- [ ] Per-project spend budgets (not just weekly total)
- [ ] JSON feed mode for external dashboard integrations

### v1.0.0 — Stable
- [ ] Linux binary releases
- [ ] Homebrew formula
- [ ] Automated pricing table updates

---

## Contributing

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/claux.git
cd claux/apps/cli

# 2. Build
CARGO_TARGET_DIR=/tmp/claux_build cargo build

# 3. Make changes to src/
# 4. Bump version in Cargo.toml (PATCH for fixes, MINOR for features)
# 5. Add an entry to CHANGELOG.md
# 6. Verify
CARGO_TARGET_DIR=/tmp/claux_build cargo build  # must compile clean
cargo test

# 7. Open a PR against main
```

Every PR must include a version bump and a CHANGELOG.md entry. No exceptions.

---

## Project layout

```
claux/
├── apps/
│   ├── cli/                    # This crate (Rust)
│   │   ├── src/                # All Rust source files
│   │   ├── Cargo.toml
│   │   ├── CHANGELOG.md
│   │   └── README.md
│   └── desktop/
│       └── Claux/              # macOS menu bar app (Swift/SwiftUI)
├── docs/
├── packages/
└── README.md                   # Repo overview (desktop app)
```

---

<p align="center">
  Built with Rust · Zero backend · Zero telemetry · Reads <code>~/.claude/</code> locally
  <br />
  <a href="https://github.com/vatskyone/claux">github.com/vatskyone/claux</a>
</p>
