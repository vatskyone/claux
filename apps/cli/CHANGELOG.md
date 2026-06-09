# CLAUX CLI Changelog

---

## [0.7.6] — 2026-06-09

### TUI

- Usage panel now reads the authoritative `~/.claude/claux/rate_limits.json` snapshot written by the Claude Code statusLine, showing the real plan-limit percentages and reset times that match the menu bar app. Falls back to log-computed values if the snapshot is absent.

## [0.7.5] — 2026-06-09

### TUI

- Dashboard usage panel now shows 5-hour and 7-day limit progress as a percentage (matching the menu bar app style), with reset countdown in `Xh Ym · HH:MM` / `Xd Yh · DD Mon HH:MM` format instead of a bare clock time.

## [0.7.4] — 2026-06-08

### TUI

- Fixed scrolling in the Sessions, Agents, Skills, and History tabs: the selected row now always stays visible when navigating past the bottom of the visible area. Agents, Skills, and History previously had a cursor but no scroll offset, so items beyond the first screenful were unreachable.

## [0.7.3] — 2026-06-08

### Reliability

- Fixed a spurious parse warning for sessions that have valid JSON but no completed assistant turns (e.g., sessions cancelled before a response). These now parse as zero-cost entries instead of emitting `warn: session log has no assistant usage entries`.

## [0.7.2] — 2026-06-01

### CLAUDE.md workflow

- Added `claux claudemd generate`:
  - creates a high-signal `CLAUDE.md` starter from local repo structure
  - supports `--project`, `--write`, `--force`, and `--json`
- Added `claux claudemd improve`:
  - reads existing `CLAUDE.md`, fills missing core sections, and preserves existing content
  - supports preview mode (default) and write mode with optional `--backup`
  - supports `--project`, `--write`, `--backup`, and `--json`

### Reliability

- Added tests for CLAUDE.md generation and improvement flows.
- Improved error messaging for missing/invalid project paths and missing `CLAUDE.md` in improve mode.

### Docs

- Updated CLI README command surface and architecture map for the new `claudemd` command set.
- Bumped crate version to `0.7.2`.

## [0.7.1] — 2026-06-01

### Reliability hardening

- Replaced panic-prone sort comparisons (`partial_cmp(...).unwrap()`) with total ordering in spend and TUI analytics paths.
- Removed fragile `unwrap()` usage in clipboard piping and JSON export/session serialization paths.
- Added parser guardrails:
  - reject session logs with no valid JSON lines
  - reject session logs with no assistant usage entries
- Optimized CLAUDE.md discovery BFS queue internals to avoid `Vec::remove(0)` churn.

### New diagnostics and setup flows

- **New `claux doctor` command** (with `--json`) for read-only diagnostics:
  - validates session source directories
  - reports active ID count
  - reports parse health (`ok/failed/total`)
  - emits actionable remediation hints
- **New `claux config init`** guided initializer.

### Config and usage-limit tracking

- Added config keys:
  - `plan-5h-limit` (USD)
  - `projects-root`
  - `sessions-root`
- TUI Usage panel now includes:
  - **Last 5h usage bar** with reset timestamp
  - weekly budget bar with reset timestamp
  - explicit blank-state UX reasons (`limit unset`, `no data yet`, `source unavailable`, `no active session`)

### Local-only growth instrumentation

- New on-device metrics store at `~/.claude/claux/local_metrics.json`.
- Tracks:
  - command usage counts
  - failure classes
  - empty-state frequency
  - TUI refresh latency buckets
- New `claux analytics local` view (`--json`, `--reset`) for inspection/reset.

### Tests

- Added unit tests for:
  - safe sort behavior with NaN totals
  - 5-hour usage window and reset math
  - parser robustness against malformed/partial logs

### Docs

- README updated with new command/config surface and version badge.

## [0.7.0] — 2026-05-28

### History tab — session checkpoints

- **New `History` tab** (6th tab in the TUI) — browse, save, and restore named project checkpoints
- **Checkpoint list** (top 40%) with columns: ID · Name · Saved date · Git branch · Total cost · Files changed
- **Checkpoint detail** (bottom 60%) — name, date, branch + commit, cost breakdown, CLAUDE.md score, list of files changed since the prior checkpoint, summary, and action hints
- **Inline save** — press `s` on the History tab; type a name; `Enter` to save, `Esc` to cancel
- **Write context** — press `w` to write `.claux/CONTEXT.md` into the project directory (agent-loadable context file)
- **Delete** — press `d` to remove the selected checkpoint

### `claux checkpoint` CLI command

- **`claux checkpoint save [name]`** — save a named checkpoint; prompts for name if omitted
  - Captures: git branch + commit, lifetime project cost, active session cost, session count, CLAUDE.md score
  - Computes files changed since the prior checkpoint's commit via `git diff --name-only`
- **`claux checkpoint list`** — table of all checkpoints for the current project
- **`claux checkpoint load <id>`** — print checkpoint context to stdout (Markdown format)
- **`claux checkpoint load <id> --write`** — also write `.claux/CONTEXT.md` into the project; agents can read this at session start
- **`claux checkpoint delete <id>`** — remove a checkpoint

### Checkpoint storage

- **Local index**: `~/.claude/claux/checkpoints/<project-hash>.json` — fast TUI reads
- **Per-project copy**: `.claux/checkpoints.json` — committable, travels with the code
- **CONTEXT.md**: structured Markdown capturing git state, costs, and changed files for agent consumption

### Internal

- New `src/checkpoints.rs` — `load_checkpoints()`, `save_checkpoint()`, `delete_checkpoint()`, `generate_context_md()`, `write_context_md()`, `git_diff_files()`, `infer_project_path()`
- New `src/commands/checkpoint.rs` — CLI subcommand handler
- New `Checkpoint` struct in `models.rs`
- `App` extended with `checkpoints`, `checkpoint_cursor`, `checkpoints_dirty`, `cp_name_editing`, `cp_name_buf`
- `Tab` enum extended with `History = 5`; updated all nav paths and draw routing

---

## [0.6.0] — 2026-05-28

### Dashboard — CLAUDE.md & context quality in Insights

- **CLAUDE.md quality row** added to Insights panel when a session is active: score `82/100` with a `█░` bar, color-coded Good/Fair/Weak label (Green ≥70, Yellow ≥40, Red <40)
- **Context quality grade** (A–D) added below: A = cache ≥ 60% AND fill < 75%; D = fill ≥ 90%; shows raw percentages alongside

### Dashboard — Usage panel

- **New Usage panel** below the Tokens panel:
  - Context window bar + `XX%  XXk/200k`
  - This week spend (optional budget bar via `claux config set weekly-budget N`) + reset date
  - Credit status (disabled / enabled with monthly spend vs cap)

### Sessions tab — "Dur" → "Time"

- Column header renamed from `Dur` to `Time`.

### Session detail — richer quality insights

- **Source row** added: `VSCode Extension` / `Terminal CLI` / `Desktop App` / `JetBrains Plugin` from `entrypoint` field
- **Detailed CLAUDE.md block**: score + bar + label, `✓`/`✗` per category (Build, Tests, Run, Structure, Conventions, Commands), up to 4 actionable suggestions

### Skills tab

- **5th TUI tab** with skill list (top 40%) and detail panel (bottom 60%)
- Seeded from `skillUsage` in `~/.claude.json` + `~/.claude/skills/` for custom skills
- Rating: 0=★, 1-2=★★, 3-9=★★★, 10-29=★★★★, 30+=★★★★★

### New CLI commands

- **`claux account`** — account card (name, email, plan, org, billing, sub dates, credit) + skill usage table
- **`claux skills list|new|export|import`** — skill management
- **`claux config get|set|unset <key>`** — budget limits (`weekly-budget`, `monthly-credit` in USD); stored in `~/.claude/claux/config.json`

### Internal

- New `src/account.rs`, `src/skills.rs`, `src/config.rs` data-layer modules
- New structs in `models.rs`: `AccountInfo`, `ClaudemdAnalysis`, `SkillInfo`, `SkillSource`, `ClauxConfig`
- `parser.rs`: added `score_claudemd_detailed()` and `find_claudemd_path()`
- `App` extended with `account_info`, `claux_config`, `skills`, `skill_cursor`, `skills_dirty`, `detail_analysis`

---

## [0.5.0] — 2026-05-27

### Session Export (`claux export`)
- New `claux export` command — dumps all session history as JSON (default) or CSV
- `--format csv` — produces a flat table with columns: `id, project_path, start_time, end_time, duration_secs, cost_usd, model, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens, thinking_tokens, is_active, title, tag`
- `--output FILE` — write directly to a file instead of stdout
- `-n N` — limit to last N sessions (default: all)
- Tags are included in the export

### Monthly Cost Forecast (`Analytics` tab)
- New **Forecast** panel in the Analytics tab (between the 30-day sparkline and the project/model tables)
- Shows four figures side by side: **Daily avg (7d)** · **Month to date** · **Est. end of month** · **Annual proj.**
- All projections based on 7-day rolling average spend; projected EOM accounts for remaining calendar days in the current month

### Session Tagging
- New `claux tag <session-id-prefix> [label]` command — attach a short label to any session
  - `claux tag abc123 "refactor"` — set tag
  - `claux tag abc123` — show current tag
  - `claux tag abc123 -r` — remove tag
- Tags persist in `~/.claude/claux/tags.json`
- **Sessions list** in `claux tui` shows a `[tag]` column next to session title
- **Session detail overlay** shows the current tag and `[t] edit` hint
- Pressing `t` inside the detail overlay opens an inline tag input mode:
  - Type any text (max 30 chars), `Enter` saves, `Esc` cancels, `Backspace` deletes
  - The cursor `▌` is shown live in the input field
  - Saving immediately reloads sessions so the new tag is visible in the list
- Tags are included in `claux export` output

### Internal
- New `src/tags.rs` module — `load_tags()`, `save_tag()` backed by `~/.claude/claux/tags.json`
- Added `tag: Option<String>` field to `ClaudeSession` (loaded and merged in `monitor::load_sessions`)
- Added `MonthlyForecast` struct and `compute_monthly_forecast()` to `spend.rs`
- Added `tag_editing: bool` and `tag_input_buf: String` to TUI `App` state
- Updated `draw_sessions_list` with a `show_tags` flag and `Tag` column
- Updated footer to show tag-edit hints contextually

---

## [0.4.0] — 2026-05-27

### Agents Tab — live sub-agent monitoring

- **New `Agents` tab** (4th tab in the TUI) dedicated to monitoring every sub-agent spawned by Claude Code in the active session
- **Agent list** (top 38% of screen) — sortable by start time, showing:
  - Status dot: `●` green = running, `✓` gray = completed, `✗` red = failed
  - Agent type (`Explore`, `Plan`, `general-purpose`, `claude-code-guide`, etc.)
  - **XP / Level system** — `Lv1 [████░░░░]` based on how many tasks that agent type has completed globally across all sessions:
    - Lv.1: 1–4 tasks · Lv.2: 5–14 · Lv.3: 15–29 · Lv.4: 30–59 · Lv.5: 60+
  - Task description (one-line summary from Claude's `input.description`)
  - **Quality stars** `★★★★☆` (1–5) computed from output completeness and length:
    - 5 = rich output (≥ 500 chars, no error keywords) · 4 = good · 3 = moderate · 2 = minimal/errors · 1 = did not complete
  - Cost and duration per agent
  - **Green `●` dot** on the Agents tab label when any agent is still running
- **Agent detail panel** (bottom 62%) — updates as cursor moves:
  - Full task description and prompt preview
  - Status / duration / model / cost line
  - Per-token breakdown bars with percentage share of the parent session total (Input / Output / Cache R)
  - Output preview (first 250 chars of the agent's result)
  - Quality label explaining the score
  - Falls back gracefully when the sub-agent JSONL file is unavailable
- **`r` key** on the Agents tab triggers an immediate agent list refresh
- **Global XP counts** computed once on first tab visit then cached; re-computed on full refresh

### Bug fix — phantom sessions from `subagents/` directories

- `collect_jsonl` in `monitor.rs` was recursing into `subagents/`, `tool-results/`, and `memory/` companion directories under `~/.claude/projects/`, causing those sub-agent files to appear as zero-cost sessions in the Sessions list and analytics — this is now fixed

### Internal

- Added `AgentRun` struct to `models.rs` with full agent metadata, token usage, cost, model, quality score
- Added `compute_quality_score()` and `agent_level()` free functions to `models.rs`
- Added `jsonl_path: PathBuf` field to `ClaudeSession` for sub-agent file lookup
- Added `parse_agents()` to `parser.rs` — two-pass JSONL scan matching `tool_use`→`tool_result` pairs; enriches each run with token data from the sub-agent's own JSONL
- Added `extract_tool_result_text()` helper to handle both `String` and `[{type:"text"}]` content formats
- Added `load_agents_for_session()` and `compute_agent_type_counts()` to `monitor.rs`
- Extended `Tab` enum with `Tab::Agents = 3`; updated all nav paths
- Added `draw_agents_screen`, `draw_agent_list`, `draw_agent_detail`, `stars`, `xp_bar`, `quality_style`, `quality_label`, `agent_duration_str`, `wrap_text` helpers to `tui.rs`

---

## [0.3.0] — 2026-05-27

### Dashboard — full redesign
- **Removed** recent sessions list from Dashboard tab; the tab is now focused entirely on the active session
- **Token breakdown panel** (left column) — visual horizontal bar chart per token type (Input / Output / Cache R / Cache W / Thinking) with exact counts and proportional `█░` bars; summary line shows total tokens + cache-hit % with color-coded grade (green ≥ 60%, yellow ≥ 30%, red below)
- **Insights panel** (right column) — context-aware recommendations updating live every 5 s:
  - **Cache efficiency grade A–D** with actionable tip when below 50% ("reuse system prompts" / "add persistent system prompt")
  - **Context health** with three thresholds: ✓ Healthy (< 75%), ↑ Consider /compact (75–90%), ⚠ Run /compact now! (> 90%)
  - **Cost projection** — current session cost + burn rate + estimated spend by EOD + rough weekly estimate
  - **Model indicator** with color (Opus = magenta, Sonnet = blue, Haiku = green)
  - **Extended thinking %** — shows thinking tokens as % of output when > 0
  - **Efficiency metric** — K output tokens per dollar (higher is better value)
  - **Lifetime stats** (shown when no active session): total sessions, lifetime spend, avg cost/session, total output tokens, overall cache-hit %, best cache-hit session

### Analytics — 7-day detail chart + model efficiency
- **7-day vertical bar chart** (`draw_7day_chart`) replaces old sparkline as primary view:
  - Header line: total / avg per day / peak day with date
  - Proportional vertical `█` bars, one column per day; today's column highlighted in blue
  - Axis separator + day-of-week labels + per-day cost labels
  - Handles missing days (no data = zero bar)
- **30-day sparkline** retained as compact secondary trend line below the 7-day chart
- **By Project / By Model tables** rendered side-by-side to use horizontal space
- **Model efficiency column** added to the Model table: `K tok/$` = thousands of output tokens per dollar (shows relative value of each model for your workload)

### Internal
- Added `chrono::{Duration as ChronoDuration, Local}` import to `tui.rs`
- Added `use std::collections::HashMap` to `tui.rs` for model-output aggregation
- Extracted `draw_7day_chart`, `draw_30day_sparkline`, `draw_project_table`, `draw_model_table`, `draw_token_breakdown`, `draw_insights_panel` as standalone functions
- Removed `draw_sessions_list` call from `draw_dashboard` (still used in Sessions tab)

---

## [0.2.0] — 2026-05-27

### New Features
- **BIOS-style TUI navigation** (`src/commands/tui.rs`) — Full keyboard-navigable multi-screen dashboard:
  - **Tab bar** at top of every screen with `←` / `→` (or `h` / `l`) to cycle between Dashboard, Sessions, and Analytics tabs. Active tab rendered with reversed style; green `●` dot on Dashboard tab when a session is live.
  - **Sessions screen** — full-screen scrollable session list with cursor row highlighted; `↑` / `↓` (or `k` / `j`) moves the cursor; auto-scrolls when cursor reaches viewport edge; scroll indicator on right edge.
  - **Analytics screen** — daily spend ASCII bar chart (30 days, `▁▂▃▄▅▆▇█` block chars), By Project table, By Model table; `↑` / `↓` scrolls through project rows.
  - **Session Detail overlay** — `Enter` on any session row opens an 80×85% centered popup with: title/path, model badge, active status, cost/duration/burn/context/cache stats, full token breakdown (input/output/cache read/cache write/thinking), CLAUDE.md quality score, context gauge bar.
  - **Copy path** — `c` inside detail overlay pipes the session project path to `pbcopy` (macOS clipboard).
  - **Context-sensitive footer** — hint bar changes per screen (switch / select / scroll / back).
- **`App` state machine** (`src/commands/tui.rs`) — replaced bare variables with structured `App` struct (`tab`, `session_cursor`, `session_scroll`, `analytics_scroll`, `detail_open`) and `Tab` enum; `handle_key()` extracted as a testable pure function.

### Improvements
- `draw_sessions_list()` now accepts an optional cursor index and scroll offset, making it reusable for both Dashboard (no cursor) and Sessions screen (with cursor).
- Version string in tab bar auto-read from `CARGO_PKG_VERSION` at compile time.

---

## [0.1.1] — 2026-05-27

### New Features
- **Shell completions** (`src/main.rs`) — added `claux completions <shell>` subcommand via `clap_complete`. Supports `zsh`, `bash`, and `fish`. Install with:
  ```bash
  claux completions zsh > ~/.zsh/completions/_claux
  ```
- PATH and fpath setup added to `~/.zshrc` so `claux` is available in all new shells without sourcing cargo env.

### Internal
- Added `clap_complete = "4"` dependency to `Cargo.toml`.
- Added `"cargo"` feature to `clap` for version macro access.

---

## [0.1.0] — 2026-05-27

### New Features
- **`claux status [--json]`** — active session card: path, model, duration, cost, burn rate, context fill %, cache hit %, token count, CLAUDE.md quality score.
- **`claux sessions [-n N] [--json]`** — colored `comfy-table` of recent sessions with status dot, relative time, duration, model, title/path, cost.
- **`claux spend [--json]`** — today / this week / this month spend with trend arrows (↑/↓) vs previous period.
- **`claux analytics [--days N] [--json]`** — ASCII sparkline + daily table, By Project table, By Model table.
- **`claux tui`** — initial static live dashboard (active session + spend + session list), auto-refresh every 5 s.
- **`claux completions <shell>`** — shell completion script generator.

### Internal
- `src/models.rs` — `ClaudeSession`, `TokenUsage`, `SpendSummary`, `DailySpend`, `ProjectSpend`, `ModelSpend`.
- `src/parser.rs` — JSONL parsing, per-model pricing table, thinking-token estimation, per-turn daily cost attribution, CLAUDE.md scoring (length + structure + content coverage).
- `src/monitor.rs` — session discovery under `~/.claude/projects/`, active-ID detection from `~/.claude/sessions/`, mtime cache to skip re-parsing unchanged files, 90 s fallback mtime threshold.
- `src/format.rs` — `cost`, `tokens`, `duration`, `relative_time`, `model_short_name`, `project_path` helpers. 4 unit tests.
- `src/spend.rs` — `compute_spend`, `compute_daily_spend`, `compute_project_breakdown`, `compute_model_breakdown`.
- `src/render.rs` — `comfy-table` and `owo-colors` helpers: `make_table`, `active_dot`, `cost_colored`, `context_bar`, `spend_sparkline`, `trend`, `model_colored`.
