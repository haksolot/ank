---
id: TASK-a7b8c9d0e1f2
type: task
slug: surface-humaine
title: check, review, accept, close et show
created: 2026-07-27T09:50:00Z
status: open
scope:
  - crates/ankor-cli/src/human.rs
blocked_by: [TASK-e5f6a7b8c9d0, TASK-f6a7b8c9d0e1]
done_criteria: |
  check couvre tous les invariants et signaux listés dans la spec et sort en 8
  sur findings, accept produit le commit signé de ratification, close exige
  --reason et révoque le claim actif, review filtre par scopes vivants.
criteria_by: creator
verify: [cargo-test, check-repo]
schema: 1
version: 1
---

check_repo (examples/) est le brouillon de check et doit disparaître au profit
de la vraie commande.
