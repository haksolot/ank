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
schema: 1
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

## Log
- 2026-07-31T04:22:44Z claude-code@ank — four verbs landed; find scans the index rather than querying FTS5, recorded as TASK-a99baa078994 rather than substituted silently
- 2026-07-31T04:24:39Z claude-code@ank — done, proof test:local/78ab4ecd426b@26465a9
- 2026-07-31T04:25Z claude-code@ank — the two entries above were written by `ank log` and `ank done`; the whole close ran through the tool. Worth recording: the first `ank done` attempt refused with code 5, and it was right to. Its verifier is `cargo test --workspace`, which has to relink `target/debug/ank.exe` — the very process running the verifier — and Windows locks a running executable. Running the same binary from a copy outside `target/` passes. Not an ank defect and not fixable in ank: cargo reports the locked link as exit 101, indistinguishable from a failing test. It bites only a project dogfooding ank on itself under Windows, and the cure is to run an installed `ank` rather than the one just built.
- 2026-07-31T04:30Z claude-code@ank — a second proof appended, which is the one write §3 allows after done. The test proof above was produced before 44060cb, and the suite was intermittently red at that point: the `a_task` fixture chose its result by `created`, a field with second resolution, so a test asserting that `log` refuses a non-HEAD id sometimes logged on HEAD and passed backwards. The commit proof anchors the fix. This was the second reason the first `ank done` refused, and finding it took running the suite twice rather than once.
