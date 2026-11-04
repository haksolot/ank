---
id: LOG-bc7ba6b027e7
type: log
title: Dead proof 40185d5 is missing from this clone and from origin/main. The work landed as 265fdf0 (on
created: 2026-11-04T10:02:08Z
author: claude-code/opus-5.5+task-a646-proofs
scope:
  - crates/ank-cli/tests/skill.rs
  - .ank/entities/SPEC-fe8bdb84faca.md
about: TASK-36666e36744e
seq: 0
schema: 4
version: 1
---

 main, 'declare mcp and watch in the Commands block...'), already attested. check cannot clear this signal by attest: detached_commit_proofs (crates/ank-cli/src/human.rs:5362) asks git rev-list --all for every commit proof the task lists, and the dead entry stays in the append-only list, so the signal lasts as long as the sha is unreachable, whatever is appended after it. Measured on 2026-11-04 from main c9f113da.
