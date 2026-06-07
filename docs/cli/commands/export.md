# claux export

Exports your full session history as JSON or CSV. Useful for spreadsheets, billing audits, custom dashboards, or piping into other tools.

## Usage

```bash
# JSON to stdout (default)
claux export

# CSV to stdout
claux export --format csv

# Write to a file
claux export --format csv -o sessions.csv
claux export -o sessions.json

# Limit to last N sessions
claux export -n 100

# Combine flags
claux export --format csv -n 50 -o last50.csv
```

## Flags

| Flag | Description |
|---|---|
| `--format json\|csv` | Output format. Default: `json`. |
| `-o FILE` / `--output FILE` | Write output to a file instead of stdout. |
| `-n N` | Limit to the last N sessions. Default: all sessions. |

## JSON format

The JSON export is an array of session objects:

```json
[
  {
    "id": "abc123def456",
    "project_path": "/Users/you/myproject",
    "start_time": "2026-06-07T10:00:00Z",
    "end_time": "2026-06-07T11:14:00Z",
    "duration_secs": 4440,
    "cost_usd": 0.84,
    "model": "claude-sonnet-4-6",
    "input_tokens": 42100,
    "output_tokens": 18400,
    "cache_read_tokens": 12000,
    "cache_write_tokens": 6200,
    "thinking_tokens": 2000,
    "is_active": false,
    "title": "Refactor auth middleware",
    "tag": "refactor"
  }
]
```

## CSV format

The CSV export uses the following columns:

```
id, project_path, start_time, end_time, duration_secs, cost_usd, model,
input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
thinking_tokens, is_active, title, tag
```

Example rows:

```csv
id,project_path,start_time,end_time,duration_secs,cost_usd,model,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,thinking_tokens,is_active,title,tag
abc123,/Users/you/myproject,2026-06-07T10:00:00Z,2026-06-07T11:14:00Z,4440,0.84,claude-sonnet-4-6,42100,18400,12000,6200,2000,false,Refactor auth middleware,refactor
```

## Piping examples

```bash
# Get total spend across all sessions
claux export --json | jq '[.[].cost_usd] | add'

# Find the most expensive session
claux export --json | jq 'max_by(.cost_usd) | {title, cost_usd, project_path}'

# All sessions over $1.00
claux export --json | jq '[.[] | select(.cost_usd > 1.0)]'

# Import into SQLite
claux export --format csv -o sessions.csv
sqlite3 sessions.db ".import sessions.csv sessions"
```
