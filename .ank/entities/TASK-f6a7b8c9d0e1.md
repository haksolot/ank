---
id: TASK-f6a7b8c9d0e1
type: task
slug: remaining-verbs
title: new, find, log and release
created: 2026-07-27T09:45:00Z
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  new refuses an empty scope, find respects the same cap as context and
  announces what it cut, log requires the claim and renews the TTL, release
  requires --reason and writes the reason into the log.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: test
    ref: local/78ab4ecd426b@26465a9
    tree: scope/870f4425a3a8
    criteria: 7d3a1c068d64
    verifier: cargo-test@f14aeab36e1b
  - type: commit
    ref: 44060cb
schema: 3
version: 8
---

The scope gains `cli.rs` and `tests/cli.rs`, criterion untouched: four verbs
need four dispatch arms, and `release` deleting the claim ref is a statement
about the repository, so it is read back with git after invoking the binary.

`find` searches by scanning the index rather than through FTS5. §6 names FTS5,
and the schema version added by TASK-b2c3d4e5f6a7 makes adding the virtual table
a rebuild rather than a migration — but the criterion here is about the cap and
the announcement of what was cut, not about the mechanism, and a scan over a
corpus this size is instant. The gap is real and belongs in its own task rather
than in a silent substitution.
