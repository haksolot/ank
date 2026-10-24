---
id: LOG-e8d871ec40c8
type: log
title: "First ank done red on claims_remote::with_origin_only_the_verbs_run_what_they_ran_before: release"
created: 2026-10-23T17:26:34Z
author: claude-code/opus-5.5+w34fe
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
about: TASK-34fe87319194
seq: 3
schema: 4
version: 1
---

 ran 10 git processes against the pinned 8, because the record was read (rev-parse + cat-file) before every deletion. Moved the read to the lost-lease path, by blob object (record_of): the pinned count is 8 again and the full ank-cli suite runs 1132/1132 green under nextest.
