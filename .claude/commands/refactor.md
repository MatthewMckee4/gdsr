---
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, Agent
description: Refactor a specific area of the codebase
---

## Context

- Current branch: !`git branch --show-current`
- Current git status: !`git status`

## Your task

Refactor: **$ARGUMENTS**

1. Explore the relevant area to understand the current structure
2. Create a feature branch if still on main: `git checkout -b claude/refactor-<short-description>`
3. Refactor with zero behavior changes — all existing tests must still pass
4. Run `cargo nextest run` to verify nothing broke
5. Run `uvx prek run -a` to pass pre-commit checks
6. Do NOT commit — leave changes staged for review
