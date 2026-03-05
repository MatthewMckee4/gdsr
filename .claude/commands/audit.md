---
allowed-tools: Read, Glob, Grep, Bash(cargo clippy:*), Bash(cargo nextest:*)
description: Audit the codebase for bugs, code smells, and improvement opportunities
---

## Context

- Current branch: !`git branch --show-current`

## Your task

Audit the codebase for bugs, unsafe patterns, and code that should be rewritten.

1. Run `cargo clippy --all-targets --all-features -- -D warnings`
2. Search for problematic patterns: `.unwrap()`, `panic!`, `unreachable!`, `unsafe`, `#[allow(`, `todo!`
3. Look for duplicated logic that should be shared
4. Check for error handling that silently swallows failures
5. Identify dead code, unused imports, or overly complex functions
6. Check test coverage gaps — modules with logic but no tests

### Output format

Group findings by severity:

**Bugs** — incorrect behavior or likely runtime failures

**Safety** — unwrap/panic in non-test code, unsafe blocks

**Code quality** — duplication, complexity, dead code

**Missing tests** — untested modules or edge cases

For each finding: `file:line` — what's wrong and what to do about it.
