---
id: LOG-06aeaa22b143
type: log
title: "scope widened to crates/ank-cli/tests/cli.rs: every_golden_conforms_to_the_shape_its_verb_declares"
created: 2026-09-27T17:10:50Z
author: claude-code/opus-5.5+af5a
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/help_new.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-af5af0fe9a7f
seq: 6
schema: 4
version: 1
---

 keys shapes by fixture, and help-verb pinned help claim --json whose kinds is empty, so help-verb.verbs[].kinds was a declaration no fixture reached. The help-verb fixture now pins help new adr --json, the per-kind document; claim stays pinned inside help.json.
