---
id: LOG-646ad49dce67
type: log
title: "falsified rather than asserted: with the --json branch of init::run disabled, both the sweep"
created: 2026-08-17T06:51:59Z
author: claude-code/2.1.233+integration-contract
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-9e63827380a1
seq: 0
schema: 3
version: 1
---

 (no_verb_puts_anything_but_json_on_stdout_under_json) and the golden for init go red, and both go green again when it is restored. The sweep's hole was never assert_json_only returning early on an empty stdout -- that is correct for a refusal -- it was that the fixture could only ever make init refuse, so the empty case was the only one it saw.
