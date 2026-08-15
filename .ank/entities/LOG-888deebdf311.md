---
id: LOG-888deebdf311
type: log
title: "scope: the criterion asks for the refusal \"at the one place a log entry is built, so every verb"
created: 2026-08-15T22:57:37Z
author: claude-code/f391
scope:
  - crates/ank-core/src/log.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-f3910718320a
seq: 3
schema: 3
version: 1
---

 that writes one is covered". That place is entries::record in crates/ank-cli/src/entries.rs -- log, done, release --reason and human all call it directly, and none of them goes through a funnel in commands.rs. The rule itself belongs in ank-core/src/log.rs, which is where the message and its split live; entries.rs is the one line that calls it. Adding crates/ank-cli/src/entries.rs to the scope with amend rather than reaching outside it silently.
