# claux claudemd

Generates a new `CLAUDE.md` from scratch or improves an existing one by filling in missing sections. Both subcommands run entirely locally — no AI API calls are made.

## Usage

```bash
# Generate a new CLAUDE.md
claux claudemd generate
claux claudemd generate --project /path/to/repo
claux claudemd generate --project /path/to/repo --write
claux claudemd generate --project /path/to/repo --write --force
claux claudemd generate --json

# Improve an existing CLAUDE.md
claux claudemd improve
claux claudemd improve --project /path/to/repo
claux claudemd improve --project /path/to/repo --write
claux claudemd improve --project /path/to/repo --write --backup
claux claudemd improve --json
```

## `claudemd generate`

Creates a high-signal `CLAUDE.md` starter file from the repository's structure. Claux scans the project directory and infers:

- **Build commands** — detects `Cargo.toml`, `package.json`, `Makefile`, `pyproject.toml`, etc.
- **Test commands** — detects test frameworks from dependency files
- **Run commands** — infers from scripts, binaries, and entry points
- **Repository structure** — maps key directories and their purpose
- **Conventions** — infers language and framework from file extensions

By default, `generate` prints the result to stdout without writing anything. Use `--write` to write the file.

### Flags

| Flag | Description |
|---|---|
| `--project PATH` | Path to the project directory. Default: current working directory. |
| `--write` | Write the generated content to `CLAUDE.md` in the project directory. |
| `--force` | Overwrite an existing `CLAUDE.md` without prompting. Only valid with `--write`. |
| `--json` | Output the generated content and metadata as JSON. |

### Example output

```markdown
# Project

Brief description inferred from README or package metadata.

## Build

\`\`\`bash
cargo build
\`\`\`

## Test

\`\`\`bash
cargo test
\`\`\`

## Run

\`\`\`bash
cargo run
\`\`\`

## Repository structure

- `src/` — main application source
- `tests/` — integration tests
- `Cargo.toml` — package manifest

## Key conventions

- Language: Rust 2021
- No unsafe code outside `src/ffi/`
- All public functions require doc comments
```

---

## `claudemd improve`

Reads an existing `CLAUDE.md` and fills in any missing high-signal sections without removing or overwriting content you've already written. Claux identifies which of the eight core topic categories are absent and generates content only for those gaps.

### Core categories scored

| Category | What Claux looks for |
|---|---|
| Build | Build commands, toolchain setup |
| Tests | How to run tests, test frameworks |
| Run | How to start the application |
| Structure | Directory layout, key files |
| Conventions | Coding style, naming, patterns |
| Commands | Useful shell commands and scripts |
| Architecture | High-level system design notes |
| Dependencies | Key libraries and why they're used |

By default, `improve` prints the improved content to stdout. Use `--write` to apply it.

### Flags

| Flag | Description |
|---|---|
| `--project PATH` | Path to the project. Default: current working directory. |
| `--write` | Write the improved content back to `CLAUDE.md`. |
| `--backup` | Create `CLAUDE.md.bak` before writing. Only valid with `--write`. |
| `--json` | Output the result and a diff of added sections as JSON. |

### Example: improve with backup

```bash
claux claudemd improve --write --backup
# → Backed up to CLAUDE.md.bak
# → Added sections: Tests, Architecture
# → Wrote CLAUDE.md
```

## CLAUDE.md quality scoring

Claux scores `CLAUDE.md` files on a 0–100 scale used throughout the CLI and TUI:

| Dimension | Max points | What it measures |
|---|---|---|
| Length | 30 | Whether the file is substantive (> 10 lines) |
| Structure | 30 | Presence of headers, code blocks, bullet lists |
| Content coverage | 40 | How many of the 8 core categories are present |

| Score | Label |
|---|---|
| 70–100 | Good |
| 40–69 | Fair |
| 0–39 | Weak |

This score appears in `claux status`, the TUI Sessions tab detail overlay, `claux account`, and checkpoint data.
