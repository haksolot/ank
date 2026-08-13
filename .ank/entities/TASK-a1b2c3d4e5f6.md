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
schema: 3
version: 5
---

The foundation for everything else: the CLI will do nothing but compose this
crate with git and the index.
