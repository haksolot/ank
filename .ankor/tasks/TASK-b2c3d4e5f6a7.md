---
id: TASK-b2c3d4e5f6a7
type: task
slug: index-sqlite
title: Index SQLite dérivé et réindexation incrémentale
created: 2026-07-27T09:25:00Z
status: open
scope:
  - crates/ankor-cli/src/index.rs
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  L'index se reconstruit intégralement depuis les fichiers, la suppression de
  index.db est sans effet observable sur les sorties, et une entité modifiée
  hors CLI est reflétée à la lecture suivante sans commande explicite.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

Hash de contenu par fichier, comparaison au périmètre touché, réindexation de
ce qui a divergé. Jamais source de vérité.
