# What is Claux?

Claux is a local-only observability tool for [Claude Code](https://claude.ai/code). It reads the session logs that Claude Code writes to your machine and surfaces what the Claude Code interface itself doesn't show: aggregate cost, context health, session history, plan-limit usage, and spend trends — all without sending a single byte to an external server.

## Why it exists

Claude Code bills per token. When you run long agentic sessions across multiple projects over days and weeks, the cost compounds in ways the inline per-message display can't communicate. You have no aggregate view, no historical chart, no budget tracking, and no alert system. Third-party dashboards solve this by uploading your session data to someone else's server — a non-starter for professional use.

Claux solves it locally. It reads `~/.claude/` directly and gives you everything you need without compromising your data.

## Two products, one source of truth

Claux ships as two independent tools that read the same local session logs:

| | [Desktop App](desktop/README.md) | [CLI](cli/README.md) |
|---|---|---|
| **Form factor** | Native macOS menu bar app | Terminal commands + live TUI dashboard |
| **Best for** | Passive ambient monitoring | Deep analysis, export, scripting |
| **Language** | Swift / SwiftUI | Rust |
| **Version** | 1.15.1 | 0.7.2 |
| **Requirements** | macOS 13+ | macOS / Linux, Rust 1.78+ |

Both tools are free. Neither requires an account or an internet connection to function.

## What Claux reads

Claux reads files Claude Code writes locally — it never modifies them:

| Path | Contents |
|---|---|
| `~/.claude/projects/<encoded-path>/*.jsonl` | Session logs — token usage, costs, model, timestamps |
| `~/.claude/sessions/<pid>.json` | Active session PID files |
| `~/.claude.json` | Account info, skill usage stats |
| `~/.claude/claux/rate_limits.json` | Plan-limit data (written by the Claux statusLine integration) |

## What Claux never does

- Sends any data off your machine
- Reads your prompts, code, or responses
- Modifies Claude Code's session files
- Require an account for the free tier

## Navigation

Use the sidebar to navigate to either product. If you're new, start with the installation guide for the product you want:

- **Desktop app**: [Installation](desktop/installation.md)
- **CLI**: [Installation](cli/installation.md)
