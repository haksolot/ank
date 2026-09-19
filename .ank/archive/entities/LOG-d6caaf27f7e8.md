---
id: LOG-d6caaf27f7e8
type: log
title: "discrepancy: init does not honour --json. Capturing the goldens found it: ank init --json prints"
created: 2026-08-17T06:10:50Z
author: claude-code/2.1.233+integration-contract
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
  - CLAUDE.md
about: TASK-2c12b027f805
seq: 0
schema: 3
version: 1
---

 its six human lines and no document, against SPEC-cd0d3377b37f's invariant that --json is available on every command without exception. The existing sweep no_verb_puts_anything_but_json_on_stdout_under_json does iterate init, but its fixture already carries a .ank/, so init refuses, stdout is empty, and assert_json_only returns early on the empty case -- init succeeding under --json is never swept. Not fixed here: this task changes no document's shape, and init has no shape to preserve. Recorded as a task of its own.
