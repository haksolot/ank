---
id: ADR-2f8a61c04b7d
type: adr
slug: surface-agent-figee
title: La surface agent est figée à sept verbes
created: 2026-07-27T09:05:00Z
status: accepted
scope:
  - crates/ankor-cli/**
  - skill/**
constraint: |
  La surface agent est exactement : context, claim, log, done, new, find,
  release. Aucun verbe ne s'y ajoute. Toute fonctionnalité nouvelle atterrit
  côté humain ou côté format.
schema: 1
version: 1
---

Le budget de mémorisation d'un agent est le vrai facteur limitant : le SKILL.md
est chargé en permanence et son coût est payé à chaque appel. Une surface qui
grossit d'un verbe par trimestre finit par coûter plus cher que le problème
qu'elle résout.

La séparation par audience est ce qui rend la contrainte tenable : la surface
humaine peut s'enrichir librement sans jamais toucher les sept verbes.
