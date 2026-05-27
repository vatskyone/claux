# CLAUX CLI Changelog

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
