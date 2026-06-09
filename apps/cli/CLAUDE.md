# CLAUDE.md

This file defines how Claude should operate in this repository.

## Project Context
- Repository root: `apps/cli` (relative to monorepo root)
- Primary language: Rust
- Framework/runtime: Cargo workspace
- Keep changes focused, minimal, and easy to review.

## Build, Test, and Run
Use these commands exactly before finalizing changes:

```bash
# Install dependencies
cargo fetch

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy --all-targets --all-features

# Run
cargo run
```

## Repository Structure
Key directories discovered during generation:
- `src`
- `src/commands`
- `test`

## Coding Conventions
- Prefer small, composable functions over large monolithic blocks.
- Keep naming explicit and consistent with existing code style.
- Avoid introducing new dependencies unless clearly justified.
- Write tests for behavior changes and edge cases.
- Preserve existing formatting/linting standards.

## Workflow for Claude
1. Read surrounding files before editing; do not guess interfaces.
2. Propose or infer a minimal plan, then implement incrementally.
3. Run build/test/lint commands relevant to the change.
4. Summarize what changed, why, and any remaining risks.

## Important Rules
- Do not rewrite unrelated files.
- Never remove user data or destructive commands without explicit approval.
- Always call out assumptions when project intent is unclear.
- Always include concrete file references in explanations.

## Definition of Done
- Build succeeds.
- Tests relevant to the change pass.
- New/changed behavior is documented in commit/summary notes.
- No obvious regressions in existing flows.
