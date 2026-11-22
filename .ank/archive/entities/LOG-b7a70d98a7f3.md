---
id: LOG-b7a70d98a7f3
type: log
title: "closed: Already done by 16e827b, decision and test both, and verified by building ank from this"
created: 2026-08-18T18:30:43Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-305cf978d37d
seq: 2
schema: 3
version: 1
---

 tree: check answers ok and exits 0 on this corpus. The task was written from a binary older than the fix, so it recorded the old binary's verdict as the corpus's state.
