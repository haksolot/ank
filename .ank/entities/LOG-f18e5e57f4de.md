---
id: LOG-f18e5e57f4de
type: log
title: "Written and green. Three doors, one grammar: edit on both its paths, amend on all three kinds, and"
created: 2026-08-22T16:09:55Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-core/src/model.rs
  - crates/ank-core/src/lib.rs
about: TASK-3c12e0ced2c0
seq: 1
schema: 4
version: 1
---

 claim only when --criteria actually wrote a criterion. The message is <fields> (version N to M, replaced <hash12>), built in one place in entries.rs so TASK-dfe5a1bb0857 has one grammar to count against. The hash is over the canonical serialisation of the entity replaced, not over the file bytes: every other freeze here hashes parsed values, and a file is a rendering. Two existing assertions moved, because the amended: opening is gone from the work trace and the record is machinery now. Two holes closed on the way: changed_fields fell through for spec and log, so a spec edit named no field at all, and verified was missing from task and adr.
