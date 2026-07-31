---
id: TASK-a1b2c3d4e5f6
type: task
slug: format-parser
title: Parser and data model of the format
created: 2026-07-27T09:20:00Z
status: done
scope:
  - crates/ank-core/**
blocked_by: []
done_criteria: |
  parse/serialize round-trip byte for byte on every file in
  tests/golden/valid, every file in tests/golden/invalid is rejected with the
  expected structured error, and cargo test is green.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: assertion
    ref: "golden suite green before going under git: 11 tests, identical round-trip"
  - type: test
    ref: ci://haksolot/ank/runs/30668388442@ad68cd5
schema: 1
version: 5
---

The foundation for everything else: the CLI will do nothing but compose this
crate with git and the index.

## Log
- 2026-07-27T09:20Z claude-code@init — types, parser, canonical serialisation, hash freeze, append-only log
- 2026-07-27T09:45Z claude-code@init — golden suite: 5 valid files, 9 invalid, 11 tests green
- 2026-07-31T22:00:22Z claude-code@ank — run 30668388442 green on the three OS at ad68cd5, appended beside the assertion rather than over it. The assertion stays: it is how this task was actually closed, before `ank done` existed to run anything, and replacing it would claim history went differently (ADR-85e6bbb195b8). No `verifier:` field on the new entry, deliberately. `definition_hash` covers the `run` string, the declared cargo-test is `cargo test --workspace -q` and ci.yml runs `cargo test --workspace`; writing the hash would assert a definition that did not run, which is the defect this entry exists to close wearing a better costume. What the ref anchors is checkable by anyone with the URL: a run, three platforms, and the commit it ran on.
