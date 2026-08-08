---
id: TASK-1613794deccf
type: task
slug: the-distribution-adr-covers-the-manifests-it-con
title: The distribution ADR covers the manifests it constrains
created: 2026-08-08T16:18:52Z
author: seanl@sean-laptop
status: done
scope:
  - .claude-plugin/**
  - package.json
blocked_by: [TASK-e1671f747e47, TASK-58e55ec52d7d]
done_criteria: |
  ADR-e3cb36646d77 carries .claude-plugin/** and package.json in its scope, so
  ank context .claude-plugin/plugin.json and ank context package.json each serve
  its constraint. ank check reports no dead scope and stays green.
criteria_by: creator
proof:
  - type: test
    ref: "31271211833"
    criteria: f05f6caf5326
schema: 2
version: 6
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

## Log
- 2026-08-08T17:49:40Z seanl@sean-laptop — released: Releasing to measure, not because the approach is wrong: ank context <path> ignores the path while a claim is held, so whether a proposed ADR's constraint is served cannot be observed from inside the claim.
- 2026-08-08T18:06:52Z seanl@sean-laptop — Ratified as db7cfd0300f6, and the criterion's clause is now observable: ank context package.json and ank context .claude-plugin/plugin.json both print CONSTRAINTS (1 active) with the full constraint text, where an hour ago the same commands printed PROPOSED (1, non-binding) and the title alone. That difference is the whole reason this task waited on a signature rather than on work.
- 2026-08-08T18:12:14Z seanl@sean-laptop — done, proof test:31271211833
