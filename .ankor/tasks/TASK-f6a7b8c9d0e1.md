---
id: TASK-f6a7b8c9d0e1
type: task
slug: verbes-restants
title: new, find, log et release
created: 2026-07-27T09:45:00Z
status: open
scope:
  - crates/ankor-cli/src/commands.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  new refuse un scope vide, find respecte le même plafond que context et
  annonce ce qu'il a coupé, log exige le claim et renouvelle le TTL, release
  exige --reason et écrit la raison dans le log.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---
