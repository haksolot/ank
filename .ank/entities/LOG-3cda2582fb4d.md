---
id: LOG-3cda2582fb4d
type: log
title: Scope amended to carry crates/ank-cli/tests/golden-json/help.json, and the fixture blessed. The
created: 2026-08-24T01:13:52Z
author: claude-code/opus-5-restatements
scope:
  - crates/ank-contract/src/verbs.rs
  - docs/getting-started.md
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-f01e4b71c8c4
seq: 4
schema: 4
version: 1
---

 amend warned that the scope change moves the constraints the live claim anchors, which is the documented behaviour rather than a refusal: only --criteria is refused under a freeze. The blessed diff is three changes and all three are inside the log verb, the summary tail, notes going from empty to one line, and the refusal string; no other verb and no other golden moved, so nothing but this edit reached the contract document. Worth keeping as a rule: a CommandSpec is pinned twice, once by ank help and once by help.json, and a task that edits one without naming the other cannot go green.
