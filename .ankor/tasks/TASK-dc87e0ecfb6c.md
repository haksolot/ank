---
id: TASK-dc87e0ecfb6c
type: task
slug: retry-verrou-par-os
title: Retry du verrou conditionnée par l'OS, PermissionDenied fatal hors Windows
created: 2026-07-28T00:38:32Z
status: open
scope:
  - crates/ankor-cli/src/store.rs
blocked_by: []
done_criteria: |
  La décision « ce refus d'ouverture est-il une contention ? » est une
  fonction pure de l'ErrorKind et de l'OS cible, testée pour les deux
  cibles depuis n'importe quel OS. Sur Windows, PermissionDenied reste
  retentée jusqu'au délai : c'est l'état delete-pending d'un verrou en
  cours de libération. Ailleurs, elle échoue immédiatement, sans consommer
  le délai, avec un message nommant le répertoire du verrou et invitant à
  vérifier ses droits. Le message d'échec après délai distingue une
  contention d'un refus de droits. cargo test est vert.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

Bug trouvé dans TASK-244a842bc0cc, qui est `done` : une tâche neuve, jamais
une réédition. La correction posée là-bas était juste sur Windows et fausse
ailleurs — sur Unix, `PermissionDenied` est un vrai refus de droits, pas de
la contention. Le retenter dix secondes fait attendre pour rien avant un
échec certain, et noie la cause réelle sous un message de verrou.

La fonction pure est ce qui rend le critère vérifiable des deux côtés : une
branche `cfg!(windows)` en dur ne serait testable que sur la moitié des
machines, et c'est exactement le trou de couverture qui a produit le bug
d'origine.

Scope corrigé par rapport à la demande : `store.rs` vit dans `ankor-cli`,
pas dans `ankor-core` — ce dernier ne fait délibérément aucune E/S disque.
