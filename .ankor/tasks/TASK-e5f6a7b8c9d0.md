---
id: TASK-e5f6a7b8c9d0
type: task
slug: done-verificateurs
title: done, exécution des vérificateurs et preuves
created: 2026-07-27T09:40:00Z
status: open
scope:
  - crates/ankor-cli/src/done.rs
  - crates/ankor-cli/src/verify.rs
blocked_by: [TASK-c3d4e5f6a7b8]
done_criteria: |
  done exécute tous les vérificateurs de verify via sh -c et refuse --proof
  dans ce cas, produit une entrée de preuve par vérificateur avec le hash de
  sa définition, vérifie le hash du done_criteria gelé avant exécution, et
  distingue le code 9 (environnement indisponible) du code 5 (vérificateur en
  échec).
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

Le point où « fausser coûte plus cher que faire » devient du code.
