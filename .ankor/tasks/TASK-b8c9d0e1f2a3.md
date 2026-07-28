---
id: TASK-b8c9d0e1f2a3
type: task
slug: distribution-multiplateforme
title: Binaires Linux, macOS et Windows, et skill d'amorçage
created: 2026-07-27T09:55:00Z
status: open
scope:
  - .github/workflows/release.yml
  - skill/**
blocked_by: [TASK-a7b8c9d0e1f2]
done_criteria: |
  La CI produit des binaires pour les trois OS à chaque tag, la suite passe
  sur les trois, et le SKILL.md installé tient les sept verbes sans dépasser
  une page.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 2
---

Sur Windows, sh est résolu depuis Git for Windows ; un sh introuvable sort en
code 9, jamais en repli vers cmd.

Scope restreint à `release.yml` : la matrice de test des trois OS est
TASK-ca4714f5c719, et `.github/workflows/**` recouvrait son fichier.
