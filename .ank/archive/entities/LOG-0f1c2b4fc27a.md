---
id: LOG-0f1c2b4fc27a
type: log
title: "self-review. Criterion axis: owes-nothing+names commit+no fault -> human.rs governs&&deleted"
created: 2026-10-24T18:24:46Z
author: claude-code/opus-5.5+w93ad
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/dead_scope_ratified.rs
about: TASK-93add59aa1de
seq: 4
schema: 4
version: 1
---

 branch, test a_deleted_scope_beside_a_live_one (ADR and spec, exit 0); all deleted -> every_scope_deleted_is_reported_as_it_was; renamed -> a_renamed_scope_beside_a_live_one (succession proposal unchanged); writes nothing -> every check() compares git status --porcelain and for-each-ref refs/ank; this repo -> 7 to 0, logged. One hunk outside the task scope: a 'mod dead_scope_ratified;' line in crates/ank-cli/tests/main.rs, required because autotests=false makes main.rs the one test target; no clause is served without it. Constraint axis (context over human.rs, the test, main.rs): ADR-cc65 no new git process (governs reads the verdicts already computed, deletion the history already walked); ADR-df10 kind stays dead-scope; ADR-3094 deleting commit still in the note, rename still proposes; ADR-6fd6 no --json shape change; ADR-d3a8 English. Nothing found beyond the main.rs line. fmt --check ok, ank check exit 0, mise run test 1697/1697 passed.
