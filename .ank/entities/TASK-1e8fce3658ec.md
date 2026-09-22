---
id: TASK-1e8fce3658ec
type: task
slug: claude-md-and-the-issue-chooser-point-at-the-pag
title: CLAUDE.md and the issue chooser point at the pages the documentation now has
created: 2026-09-22T19:33:54Z
author: claude-code/opus-5+6113
status: done
scope:
  - CLAUDE.md
  - .github/ISSUE_TEMPLATE/config.yml
blocked_by: [TASK-6113526927b2]
done_criteria: |
  No tracked file outside docs/ names docs/getting-started.md or docs/agents.md, which TASK-6113526927b2 replaced: CLAUDE.md points at docs/ci.md for the attest recipe, and the contact links of .github/ISSUE_TEMPLATE/config.yml open the published quickstart, install and multi-agent pages. git grep -n 'getting-started\|docs/agents' -- CLAUDE.md .github prints nothing.
criteria_by: creator
verify: [check-repo]
proof:
  - type: test
    ref: local/92773635df62@61cbb4d
    tree: scope/b58b4365df64
    criteria: 5e2641ca34ab
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 3
---
