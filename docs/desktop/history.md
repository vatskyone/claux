# Session History

The History tab shows all recent Claude Code sessions Claux has parsed from your monitored directory.

## Session List

The session list is a scrollable, searchable table of recent sessions. The header row is pinned and does not scroll — only the session rows scroll beneath it.

### Session row

Each row shows:

| Column | Description |
|---|---|
| **Status dot** | Green = active, grey = ended |
| **Title** | AI-generated session title (from the `aiTitle` JSONL field), or the project path if no title is available |
| **Project** | Abbreviated project path |
| **Model** | Model badge (e.g. `Sonnet 4.6`) |
| **Duration** | Total session duration |
| **Cost** | Total session cost |
| **Relative time** | How long ago the session ended (e.g. "2h ago", "yesterday") |

### Search

A native search bar appears above the session list when two or more sessions are present. Typing filters sessions in real time by title or project path. The session count badge in the header updates to show "matching/total" while a query is active.

### Right-click menu

Right-clicking (or Control-clicking) any session row shows a context menu:

| Item | Action |
|---|---|
| **Copy Path** | Copies the project directory path to the clipboard |
| **Show in Finder** | Opens the project directory in Finder |
| **Copy Session ID** | Copies the JSONL session UUID — useful for direct file inspection |

## Session Detail Sheet

Tapping any session row opens a full detail sheet as an overlay inside the popover. The sheet slides up from the bottom with a dimmed backdrop. Dismiss it by clicking the × button or by tapping anywhere outside the card.

### Detail sheet contents

#### Header

- Session title (AI-generated or project path)
- Project path with a copy button
- Entrypoint badge: **VS Code**, **Terminal**, **Desktop App**, or **JetBrains Plugin** — inferred from the session's `entrypoint` JSONL field

#### Cost and timing

| Field | Description |
|---|---|
| **Total cost** | Full session cost |
| **Duration** | Start to end time |
| **Burn rate** | Average $/hr for the session |
| **Cost projection** | Estimated total if the session ran to the configured projection period (daily/weekly/monthly) |

#### Context and cache

| Field | Description |
|---|---|
| **Context fill** | Percentage of the context window used at the session's peak |
| **Cache hit rate** | Percentage of input tokens served from cache |
| **Cache efficiency** | Color-coded grade: A (≥60%), B (30–59%), C (<30%) |

#### Token breakdown

Full breakdown of every token category:

- Input tokens
- Output tokens
- Cache read tokens
- Cache write tokens
- Thinking tokens (estimated)

#### Session quality panel

The quality panel shows the session's quality score (0–100) with a filled bar and color-coded grade, plus a breakdown of the contributing metrics:

| Metric | Description |
|---|---|
| **Quality score** | Derived score 0–100 |
| **Accepted edits** | Number of file changes accepted by the user |
| **Rejected actions** | Tool calls or file changes that were rejected |
| **Successful tool results** | Tools that completed without error |
| **Completed agents** | Sub-agent runs that finished successfully |
| **Touched files** | Distinct files modified during the session |

#### CLAUDE.md quality

If a `CLAUDE.md` was found for the session's project, the sheet shows the same 0–100 score as the Dashboard with the dimensional breakdown: length, structure, and content coverage.
