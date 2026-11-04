---
id: LOG-bd37ea32e53c
type: log
title: "body diff SPEC-9796c1a07c1b -> SPEC-ff885356f1f9, measured: diff of the body from ank show --json"
created: 2026-10-24T18:13:13Z
author: claude-code/opus-5.5+w7956
scope:
  - docs/specification.md
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-cli/tests/skill.rs
about: TASK-79566ec42789
seq: 4
schema: 4
version: 1
---

 content of each, 2 lines changed at 190c190 ; every other line identical. Word diff: {+A rescope (§4, ADR-bb9ccdba9be7) writes a new tag in place of the old one, under the same ref and with compare-and-swap, and its message carries, beside the new hash, the **previous anchor** and the **previous scope list**: what lets `check` recompute the previous anchor from the current `constraint`, or body, and the previous scope, and so prove that the rescope moved the perimeter and nothing else, without keeping the previous tag object under a second ref. A mismatch is a fault.+}
