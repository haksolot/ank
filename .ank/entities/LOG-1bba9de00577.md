---
id: LOG-1bba9de00577
type: log
title: The idle test needed a second half to mean anything. A screen left alone for three seconds leaves
created: 2026-08-25T06:18:55Z
author: claude-code/opus-5+reader-acts
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-b50b340c0bb1
seq: 4
schema: 4
version: 1
---

 refs/ank/* byte for byte where it was, but so would a broken instrument, so the test then runs one renewing verb by hand and asserts the refs moved. Without that, 'nothing changed' is a claim about the comparison and not about the session. Three seconds and not one: the record writes expires at second resolution, and a renewal inside the same second would be invisible.
