---
id: TASK-244a842bc0cc
type: task
slug: entity-store
title: Entity store — reading, atomic writing, compare-and-swap on version
created: 2026-07-28T00:09:51Z
status: done
scope:
  - crates/ank-cli/src/store.rs
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  The store reads and writes the entities of a .ank/ directory received as a
  parameter, without depending on the config or on dispatch. One test per
  case: loading by full id and by prefix; ambiguous prefix and missing entity
  in code 2, the ambiguous one listing its candidates; a write whose base
  version diverges from the one on disk refused in code 3, with the file
  unchanged byte for byte; an accepted write incrementing version by exactly
  1; reading back after a write byte-identical to serialize_entity; a
  leftover temporary file in tasks/ neither read as an entity nor masking the
  original; a file name not carrying the entity's id refused on read; N
  threads writing the same entity from the same base version yielding exactly
  one winner, all the others in code 3, and a final file that parses, never
  truncated and never mixed.
criteria_by: claimer
verify: [cargo-test]
proof:
  - type: commit
    ref: 8b5f26ebaa7cee2884cb3590a71552bdc4d58c70
schema: 3
version: 5
---

The file layer underneath the index of TASK-b2c3d4e5f6a7: that one is a
disposable SQLite cache, it is never the source of truth and therefore
presupposes this store. No existing task carried it, even though
`claim --criteria`, `log`, `done`, `release` and `close` all write a task file.

The file lock of §6 covers the read-compare-write cycle; it is what makes the
compare-and-swap on `version` effective, since write-then-rename alone compares
nothing. Concurrency is tested with N threads inside one process rather than two
processes: deterministic and fast, where two processes would give a flaky test in
CI for the same guarantee. Exactly one winner and a never-mixed file is precisely
what the lock and the rename must produce together.
