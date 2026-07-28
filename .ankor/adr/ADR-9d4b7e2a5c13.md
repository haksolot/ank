---
id: ADR-9d4b7e2a5c13
type: adr
slug: format-est-la-spec
title: Le format est la spec, ankor-core en est l'implémentation de référence
created: 2026-07-27T09:00:00Z
status: superseded
scope:
  - crates/ankor-core/**
  - docs/**
constraint: |
  Tout changement de format se fait dans cet ordre : la spec d'abord, puis
  les fichiers golden, puis le code. Le round-trip doit rester identique à
  l'octet près. Aucun champ n'existe dans le code sans exister dans la spec.
see: crates/ankor-core/tests/golden/
schema: 1
version: 2
---

Ankor promet que tout outil tiers peut lire et écrire les fichiers `.ankor/`
sans passer par le CLI. Cette promesse ne tient que si le format est décrit
avant d'être implémenté, et si l'implémentation de référence ne le fait jamais
dériver en silence.

L'ordre spec → goldens → code n'est pas cérémoniel : écrire le golden avant le
code force à décider de la forme canonique explicitement, plutôt que de la
laisser émerger de l'implémentation du sérialiseur.

Ces ADR d'amorçage sont ratifiés par l'historique du repo plutôt que par un
commit signé : `allowed_signers` est vide tant que le projet est solo, et
`check` doit signaler cette absence comme une limite assumée, pas la masquer.
