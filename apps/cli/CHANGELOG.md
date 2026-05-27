# CLAUX CLI Changelog

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
