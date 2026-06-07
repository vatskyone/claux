# claux doctor

Runs read-only diagnostics to validate your session source directories and report parse health. Useful when sessions aren't appearing or data looks incorrect.

## Usage

```bash
claux doctor
claux doctor --json
```

## Output

```
  Claux Doctor
  ────────────────────────────────────────

  Session directory    ~/.claude/projects     ✓ exists
  Sessions directory   ~/.claude/sessions     ✓ exists
  Projects found       12
  Active session IDs   1
  JSONL files          12

  Parse health
  ────────────────────────────────────────
  Parsed OK     11 / 12
  Failed         1 / 12   ← abc123def.jsonl: unexpected EOF at line 4821

  Hints
  ────────────────────────────────────────
  ✓ Session discovery is working correctly
  ⚠ 1 file failed to parse — this session will not appear in results
    Run: cat ~/.claude/projects/.../abc123def.jsonl | tail -5
    to inspect the truncated file
```

## Checks performed

| Check | What it validates |
|---|---|
| **Session directory** | `projects-root` path exists and is a directory |
| **Sessions directory** | `sessions-root` path exists and is a directory |
| **Projects found** | Number of project subdirectories detected |
| **Active session IDs** | Number of active PID files found |
| **JSONL files** | Total number of session JSONL files found |
| **Parse health** | How many files parsed successfully vs failed |

## Parse health

For each JSONL file, `doctor` attempts a full parse and reports:

- **ok** — parsed successfully
- **failed** — parse error, with the filename and error message

A file typically fails to parse when:
- It was truncated mid-write (Claude Code was killed during an active session)
- It contains malformed JSON (rare, usually an encoding issue)
- It has no valid JSON lines at all
- It has no assistant usage entries (e.g. a session with only user messages)

Failed files are silently excluded from all Claux results. `claux doctor` is how you find out about them.

## Flags

| Flag | Description |
|---|---|
| `--json` | Output all diagnostic results as a JSON object |

## JSON output

```json
{
  "projects_root": "/Users/you/.claude/projects",
  "sessions_root": "/Users/you/.claude/sessions",
  "projects_root_exists": true,
  "sessions_root_exists": true,
  "projects_found": 12,
  "active_ids": 1,
  "jsonl_files": 12,
  "parse_ok": 11,
  "parse_failed": 1,
  "failed_files": [
    {
      "path": "/Users/you/.claude/projects/abc/abc123def.jsonl",
      "error": "unexpected EOF at line 4821"
    }
  ],
  "hints": [
    "Session discovery is working correctly",
    "1 file failed to parse — this session will not appear in results"
  ]
}
```

## When to run doctor

- Sessions aren't appearing in `claux sessions` or the TUI
- Cost figures look unexpectedly low
- After a system crash or forced Claude Code quit
- After changing your `projects-root` or `sessions-root` paths with `claux config`
