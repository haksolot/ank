---
id: TASK-d4e5f6a7b8c9
type: task
slug: context-biphase
title: context biphasé, orientation et exécution
created: 2026-07-27T09:35:00Z
status: open
scope:
  - crates/ankor-cli/src/context.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  Sans claim, la sortie liste contraintes actives, propositions et tâches
  ouvertes dans l'ordre déterministe de la spec ; avec un claim, elle bascule
  sur la tâche seule sans jamais tronquer une contrainte ; l'absence de tâche
  prête sort en 0 avec un message explicite.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

La commande la plus lue par les agents : le budget et l'ordre de coupe sont
la partie qui compte.
