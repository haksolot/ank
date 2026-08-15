---
id: LOG-df98627ebb6a
type: log
title: "Falsified the new test before trusting it: forcing style::COLOR unconditionally in dispatch turns"
created: 2026-08-08T23:01:40Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/tests/cli.rs
  - docs/**
about: TASK-4601ed18d84e
schema: 3
version: 1
---

 14 of 70 integration tests red, the new every_transition_line among them. A negative guarantee asserted by a test that cannot fail is not a guarantee. Design point worth keeping: landed() and status() read one free function state_sgr, so '-> done' and '[done]' cannot drift apart -- the test asserts the equality between the two accessors rather than against a literal, which a second table that happened to agree today would have passed.
