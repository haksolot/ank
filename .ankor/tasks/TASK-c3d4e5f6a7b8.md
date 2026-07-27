---
id: TASK-c3d4e5f6a7b8
type: task
slug: claims-refs-git
title: Claims sur refs git, TTL et ré-acquisition
created: 2026-07-27T09:30:00Z
status: open
scope:
  - crates/ankor-cli/src/claim.rs
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  Un claim est pris et relâché via refs/ankor/claims/<id> sans jamais modifier
  un fichier de tâche, deux claims concurrents sur la même tâche échouent en
  code 4, l'expiration rend la tâche reprenable, et le titulaire d'origine
  ré-acquiert silencieusement si personne n'a repris.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

Porte aussi le hash du done_criteria gelé et celui des contraintes applicables
au moment du claim.
