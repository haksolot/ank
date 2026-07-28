---
id: TASK-aca0cb103980
type: task
slug: tolerance-crlf
title: Tolérance CRLF en lecture, LF en écriture, diagnostic dédié
created: 2026-07-28T00:22:06Z
status: open
scope:
  - crates/ankor-core/src/parse.rs
  - crates/ankor-core/src/error.rs
  - crates/ankor-core/tests/golden/**
  - crates/ankor-core/examples/check_repo.rs
blocked_by: []
done_criteria: |
  parse_entity accepte un fichier en CRLF et sa sérialisation rend du LF ;
  un golden valide en CRLF le couvre, et la suite golden n'exige plus
  l'identité octet pour octet que sur les fichiers déjà canoniques.
  Error::CrlfLineEndings existe et son message nomme les fins de ligne et
  la commande git config core.autocrlf input ; un test asserte que ce
  diagnostic, et non « frontmatter absent », est celui qui remonte pour un
  fichier CRLF. check_repo remonte la forme CRLF comme finding non fatal,
  distinct de l'erreur de forme non canonique, et sort en 0 quand c'est le
  seul écart. cargo test est vert.
criteria_by: claimer
verify: [cargo-test]
schema: 1
version: 1
---

Découvert en vérifiant un clone frais : `core.autocrlf=true` sans
`.gitattributes` rendait les 15 entités et toute la suite golden
illisibles, avec le diagnostic « frontmatter absent » qui envoie chercher
au mauvais endroit. Le `.gitattributes` posé à la racine est la correction
de fond ; ceci est le filet, pour les clones antérieurs, les archives et
les outils tiers.

Ordre imposé par ADR-63b59c5c26f7 (spec, puis goldens, puis code) : la
spec §3 « Forme canonique et round-trip » est déjà écrite, les goldens
viennent avant le parseur.

Non bloquant pour le socle CLI : `.gitattributes` suffit à rendre l'arbre
courant sain, ce qui est pourquoi cette tâche ne bloque pas
TASK-244a842bc0cc ni TASK-c8637488773c.
