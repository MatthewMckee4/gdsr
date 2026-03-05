---
allowed-tools: Bash(gh issue *), Bash(git log *), Read, Glob, Grep
description: Find the best issues to work on next
---

## Context

- Recent commits: !`git log --oneline -20`
- Current open PRs: !`gh pr list --limit 10`

## Your task

Find the best GitHub issues to work on next. Analyze and rank them.

### Steps

1. Fetch all open issues: `gh issue list --limit 50 --json number,title,labels,body`
2. Read the codebase structure to understand what areas are well-developed vs sparse
3. Filter out:
   - Issues labeled `needs-decision`
   - Issues that overlap with any open PRs (already in progress)
4. Rank remaining issues by:
   - **Feasibility**: Can this be done in a single session? Does the codebase already have the infrastructure?
   - **Impact**: Does this unblock other issues? Is it user-facing?
   - **Clarity**: Is the issue well-specified enough to implement without ambiguity?
5. Present the top 5-10 issues in a table:
   - Issue number and title
   - Why it's a good next pick (one sentence)
   - Estimated complexity (small / medium / large)
   - Any prerequisite issues

Flag any issues that should be tackled together as a group.
