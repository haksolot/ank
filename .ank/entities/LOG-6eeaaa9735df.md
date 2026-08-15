---
id: LOG-6eeaaa9735df
type: log
title: "discrepancy: the criterion says \"a carriage return, or any other C0 control character\" and U+000A"
created: 2026-08-15T22:57:23Z
author: claude-code/f391
scope:
  - crates/ank-core/src/log.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-f3910718320a
seq: 1
schema: 3
version: 1
---

 is C0. Section 3 builds the log split on it -- the title is cut at the first newline, the remainder is the body between exactly one newline at each end, and the round-trip test in ank-core asserts a two-line message survives byte for byte. Refusing U+000A would refuse a message the format is written to store. Measured: the refusal covers U+0000 through U+001F except U+000A, which is what "no byte outside the grammar of section 3" actually names.
