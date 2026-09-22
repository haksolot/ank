---
id: LOG-0dddacc2ef05
type: log
title: "regression test crates/ank-cli/tests/archive_digest.rs, through the binary: green with the fix;"
created: 2026-09-22T18:03:40Z
author: claude-code/opus-5+77df
scope:
  - crates/ank-cli/tests/**
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/human.rs
about: TASK-77df1446260f
seq: 6
schema: 4
version: 1
---

 with human.rs reverted it fails at the first run after rm index.db, check exit 0 against expected 8, faults []. Fix reapplied.
