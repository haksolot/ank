---
id: TASK-1613794deccf
type: task
slug: the-distribution-adr-covers-the-manifests-it-con
title: The distribution ADR covers the manifests it constrains
created: 2026-08-08T16:18:52Z
author: seanl@sean-laptop
status: open
scope:
  - .claude-plugin/**
  - package.json
blocked_by: [TASK-e1671f747e47, TASK-58e55ec52d7d]
done_criteria: |
  ADR-e3cb36646d77 carries .claude-plugin/** and package.json in its scope, so
  ank context .claude-plugin/plugin.json and ank context package.json each serve
  its constraint. ank check reports no dead scope and stays green.
criteria_by: creator
schema: 2
version: 1
---

Repair of a sequencing defect in this batch, not new design.

ADR-e3cb36646d77 was created naming .claude-plugin/** and package.json in its
scope, because those are the files its constraint is about. Neither existed yet,
and ank check reports a dead scope on an ADR as a fault, not a signal -- an ADR
that binds nothing is a decision nobody is reading. The two paths were dropped to
get the corpus green.

They have to come back, or the constraint stops covering the manifests it exists
for. TASK-e1671f747e47 creates .claude-plugin/, TASK-58e55ec52d7d creates
package.json; once both are in the tree, ank amend restores the scopes and the
coverage is real again.

The criteria of those two tasks could not carry this instruction: amend refuses
to touch done_criteria, frozen by construction, and the tasks were already
created. A separate task with blocked_by is the route the project prescribes for
exactly this.

Worth remembering when writing an ADR ahead of the files it governs: name only
paths that exist, and grow the scope as the files arrive.
