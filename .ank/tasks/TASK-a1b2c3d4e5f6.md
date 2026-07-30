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
schema: 1
version: 4
---

The foundation for everything else: the CLI will do nothing but compose this
crate with git and the index.

## Log
- 2026-07-27T09:20Z claude-code@init — types, parser, canonical serialisation, hash freeze, append-only log
- 2026-07-27T09:45Z claude-code@init — golden suite: 5 valid files, 9 invalid, 11 tests green
