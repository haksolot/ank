---
id: LOG-656fc5b23d1d
type: log
title: "discrepancy: the criterion requires ank check to stay green, and removing .claude/ makes it report"
created: 2026-08-15T21:20:21Z
author: claude-code/10b8
scope:
  - .claude/**
  - README.md
  - CLAUDE.md
about: TASK-10b8a29fd853
seq: 0
schema: 3
version: 1
---

 a fault. Measured on this corpus: check is ok before the change (0 faults, 104 signals) and reports exactly one fault after, TASK-3109a736c255 dead scope .claude/**  -- that finished task is the one whose criterion required the hook to be checked in, so its scope names the directory this task deletes. ADR-97beaf55e73a makes a dead scope a fault for a finished task and lowers it only where git records a rename; a deletion records none, so nothing lowers this one and the finding is correct. The only repair would be amending a done tasks scope to say it never touched the directory it added, which falsifies the record rather than repairing it, and lies outside this perimeter. Everything else in the criterion is met: .claude/settings.json and .claude/hooks/deny-ank-direct-access.mjs are gone, CLAUDE.md no longer names a PreToolUse hook and still carries ADR-01b6dd05f0db as what binds, README.md already named no hook (TASK-ab53a0d5654e deleted that line), the ratified ADR is untouched, and cargo test --workspace passes.
