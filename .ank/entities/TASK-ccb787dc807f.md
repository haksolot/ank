---
id: TASK-ccb787dc807f
type: task
slug: a-closing-delimiter-with-no-final-newline-is-not
title: A closing delimiter with no final newline is not diagnosed as missing frontmatter
created: 2026-09-19T18:25:44Z
author: claude-code/opus-5+docs-audit
status: open
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
blocked_by: []
done_criteria: |
  An entity file whose closing --- is the last byte, with no newline after it, is either read or refused with a message naming the closing delimiter; it is never reported as missing frontmatter that must start with ---. A golden fixture pins the case.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
schema: 4
version: 2
---

Found during the 2026-09-19 audit of docs/format.md, whose line 330 warns against exactly this misleading diagnosis.
