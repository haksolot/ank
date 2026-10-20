---
id: LOG-64eec41bb5cd
type: log
title: "Review fix: the Python edit script that wrote prepare_reanchor used non-raw triple-quoted strings,"
created: 2026-10-20T17:52:27Z
author: claude-code/4b25
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/src/verbs.rs
about: TASK-4b25843e6151
seq: 4
schema: 4
version: 1
---

 so Python consumed each backslash-newline (leaving 18 spaces inside two refusal messages) and turned each \n escape into a literal newline in the tag message format. Measured red against HEAD f136bf03: 'a run of spaces in: "error[7]: no ratify ADR-0000000006cc commit is reachable from this clone, so there is nothing to                  re-anchor from"'. Fixed with real escapes; the refusals test now covers exit 7 in a depth-1 clone and asserts no double space in each refusal's first line. ratify_ref 16 passed, accept unit tests 14 passed.
