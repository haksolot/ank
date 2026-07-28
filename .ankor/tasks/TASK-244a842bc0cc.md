---
id: TASK-244a842bc0cc
type: task
slug: store-entites
title: Store d'entités — lecture, écriture atomique, compare-and-swap sur version
created: 2026-07-28T00:09:51Z
status: open
scope:
  - crates/ankor-cli/src/store.rs
blocked_by: [TASK-a1b2c3d4e5f6]
done_criteria: |
  Le store lit et écrit les entités d'un répertoire .ankor/ reçu en
  paramètre, sans dépendre de la config ni du dispatch. Un test par cas :
  chargement par id complet et par préfixe ; préfixe ambigu et entité
  introuvable en code 2, l'ambigu listant ses candidats ; écriture dont la
  version de base diverge de celle sur disque refusée en code 3, fichier
  inchangé octet pour octet ; écriture acceptée incrémentant version
  d'exactement 1 ; relecture après écriture identique octet pour octet à
  serialize_entity ; fichier temporaire résiduel dans tasks/ ni lu comme
  entité ni masquant l'original ; nom de fichier ne portant pas l'id de
  l'entité refusé à la lecture.
criteria_by: claimer
verify: [cargo-test]
schema: 1
version: 1
---

Couche fichiers sous l'index de TASK-b2c3d4e5f6a7 : celui-ci est un cache
SQLite jetable, il n'est jamais source de vérité et présuppose donc ce
store. Aucune tâche existante ne le portait, alors que `claim --criteria`,
`log`, `done`, `release` et `close` écrivent tous un fichier de tâche.

Le verrou de fichier du §6 couvre le cycle lecture-comparaison-écriture ;
c'est lui qui rend le compare-and-swap sur `version` effectif, write-then-
rename seul ne comparant rien. Le critère porte sur le résultat observable
plutôt que sur la concurrence réelle, qui donnerait un test instable.
