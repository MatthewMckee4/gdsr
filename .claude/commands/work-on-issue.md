---
allowed-tools: Read, Write, Edit, Bash, Glob, Grep, Agent
description: Implement a GitHub issue on a feature branch
---

## Context

- Current branch: !`git branch --show-current`
- Current git status: !`git status`

## Your task

Implement GitHub issue **$ARGUMENTS**.

1. Fetch the issue details: `gh issue view $ARGUMENTS`
2. Explore the relevant area of the codebase to understand existing patterns
3. Create a feature branch if still on main: `git checkout -b claude/<short-description>`
4. Implement the feature/fix, write tests, run `cargo nextest run`
5. Run `uvx prek run -a` to pass pre-commit checks
6. Do NOT commit — leave changes staged for review
