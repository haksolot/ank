---
id: TASK-244a842bc0cc
type: task
slug: store-entites
title: Store d'entités — lecture, écriture atomique, compare-and-swap sur version
created: 2026-07-28T00:09:51Z
status: done
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
  l'entité refusé à la lecture ; N threads écrivant la même entité depuis
  la même version de base donnant exactement un gagnant, tous les autres
  en code 3, et un fichier final parseable, jamais tronqué ni mixte.
criteria_by: claimer
verify: [cargo-test]
proof:
  - type: commit
    ref: 8b5f26ebaa7cee2884cb3590a71552bdc4d58c70
schema: 1
version: 4
---

Couche fichiers sous l'index de TASK-b2c3d4e5f6a7 : celui-ci est un cache
SQLite jetable, il n'est jamais source de vérité et présuppose donc ce
store. Aucune tâche existante ne le portait, alors que `claim --criteria`,
`log`, `done`, `release` et `close` écrivent tous un fichier de tâche.

Le verrou de fichier du §6 couvre le cycle lecture-comparaison-écriture ;
c'est lui qui rend le compare-and-swap sur `version` effectif, write-then-
rename seul ne comparant rien. La concurrence est testée par N threads
dans un même processus et non par deux processus : déterministe et rapide,
là où deux processus donneraient un test instable en CI pour la même
garantie. Un unique gagnant et un fichier jamais mixte est exactement ce
que le verrou et le renommage doivent produire ensemble.

## Log
- 2026-07-28T00:25Z claude-code@ankor — claim manuel (le CLI n'existe pas) ; critere pose par le claimer, signal assume
- 2026-07-28T00:34Z claude-code@ankor — store.rs : verrou par create_new, write-then-rename, CAS sur version ; 9 tests
- 2026-07-28T00:36Z claude-code@ankor — le test 16 threads a sorti un bug Windows : verrou en delete-pending rendant ERROR_ACCESS_DENIED et non ERROR_FILE_EXISTS, donc fatal au lieu d'etre reessaye
- 2026-07-28T00:38Z claude-code@ankor — done : cargo-test vert, preuve commit 8b5f26e
