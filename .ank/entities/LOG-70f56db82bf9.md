---
id: LOG-70f56db82bf9
type: log
title: "red: 6 of 6 help_new.rs tests failed before the table existed (no kinds in help --json, help new"
created: 2026-09-27T17:09:20Z
author: claude-code/opus-5.5+af5a
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/help_new.rs
about: TASK-af5af0fe9a7f
seq: 4
schema: 4
version: 1
---

 adr exit 1). Green after one change, since all six read the same new field. Then four mutations of NEW_KINDS each turned exactly one test red: an adr requirement reason changed (requires replay, line 304), a false adr refusal of --criteria (refuses replay), spec claimed to take --constraint (flag replay: new spec --constraint exits 1), task claimed to require --criteria (requires replay: exit 0 not 7). So the replay confronts every row of the help with ank new through the binary.
