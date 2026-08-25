---
id: LOG-64ae7e0c8f48
type: log
title: Two findings the criterion did not anticipate, and both changed the code. A body line wider than
created: 2026-08-25T04:50:11Z
author: claude-code/opus-5+tui-verb
scope:
  - crates/ank-tui/**
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - Cargo.toml
  - crates/ank-cli/Cargo.toml
  - Cargo.lock
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-cli/tests/tui.rs
about: TASK-49746735127f
seq: 3
schema: 4
version: 1
---

 the window was being cut with a marker, exactly as a title is, and 'the body of a selected entity whole' forbids that in the horizontal direction as much as the vertical: frame::wrap now breaks a line into as many rows as it needs, the break carries its space so the rows join back byte for byte, and a test asserts that join at five widths. And 'no ref under refs/ank/ changed' has one exception which is not the reader's: ank show renews the lease when the id is the task the caller holds (ADR-0bb7ea8991bc), so opening your own task moves refs/ank/claims/<id> exactly as typing that command in a shell would. It did not show up at first because renew writes an identical record inside the same second, which makes it a timing-dependent write and not an absent one. The suite therefore states two things rather than one: a full session over entities the caller does not hold leaves every file and every ref byte for byte where they were, and opening the held task writes no file, creates and removes no ref, and takes no claim. What ADR-8bd76e8d7c4e forbids is renewing on its own, and a screen left open all night runs no command at all.
