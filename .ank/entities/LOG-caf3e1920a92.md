---
id: LOG-caf3e1920a92
type: log
title: "exit.rs: removed accept-off-the-default-branch from the enum doc (its §4 quote is gone from"
created: 2026-10-20T19:22:11Z
author: claude-code/79b9
scope:
  - crates/ank-contract/src/exit.rs
  - docs/exit-codes.md
about: TASK-79b9b5195c8e
seq: 1
schema: 4
version: 1
---

 SPEC-a817; example now a blocked claim), the Prerequisite doc and the exit 7 description. docs/exit-codes.md regenerated with cargo run -p ank-contract --bin exit-codes: one row changed. Measured: grep for accept off/ran off/was run off is 0 in both files; reference_pages 7/7, doc_replay 20/20, ank-contract lib 23/23. Falsified: the HEAD page against the new source fails exit_codes_page::the_exit_code_page_is_what_the_command_prints (1 of 2 red), regenerated page 2/2 green.
