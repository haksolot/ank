---
id: TASK-5e6aabcddbb0
type: task
slug: contributing-security-and-the-github-templates-m
title: CONTRIBUTING, SECURITY and the GitHub templates match the repository as configured
created: 2026-09-19T18:26:20Z
author: claude-code/opus-5+docs-audit
status: done
scope:
  - CONTRIBUTING.md
  - SECURITY.md
  - .github/pull_request_template.md
  - .github/ISSUE_TEMPLATE/**
blocked_by: []
done_criteria: |
  The ratify recipe in CONTRIBUTING.md runs end to end from the branch state it leaves, gh pr merge naming the branch; the MSRV paragraph names every crate carrying rust-version; the required checks it describes match the ruleset. SECURITY.md names the latest release as supported, cites no file the tree lacks, and its scope neither lists ank init's root writes as a vulnerability nor omits install.sh, install.ps1, the npm packages, ank update, ank mcp and ank watch. The issue templates' example outputs match the binary's current form and every label they apply exists. The PR template tells the author where to find a done task's id.
criteria_by: creator
proof:
  - type: commit
    ref: 9a9fb9d9a97f9e2d5d5faaf5fb96627691f265a7
    criteria: fc047663ae80
    via: submitted
schema: 4
version: 3
---

Findings from the 2026-09-19 audit: gh pr merge --merge run from main finds no PR (exit 1); rust-version is in 6 crates, not 2; the ruleset requires 6 checks incl. version check; SECURITY says 0.1.2 and cites the hook removed in 264636c; label spec does not exist; the PR template points at find --status open, where a done task no longer appears. No declared verifier reads prose, so verify: is left empty on purpose and the close is a commit proof; the future doc-output test (ADR-2b62b9a1fe67) is what will hold these pages afterwards.
