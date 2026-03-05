---
allowed-tools: Read, Glob, Grep, Bash(cargo clippy:*), Bash(cargo nextest:*)
description: Review current branch changes against project standards
---

## Context

- Current branch: !`git branch --show-current`
- Diff from main: !`git diff main...HEAD`
- Unstaged changes: !`git diff`

## Your task

Review ALL changes on this branch against the project's CLAUDE.md standards.

1. Read each changed file in full to understand context (don't rely solely on the diff)
2. Check that tests exist and are appropriate for the changes
3. Run `cargo clippy --all-targets --all-features -- -D warnings`
4. Verify code follows the patterns in neighboring files

### Output format

**VERDICT: APPROVED** or **VERDICT: CHANGES_NEEDED**

If changes needed, list specific actionable items:
- file:line — what to fix and why
