---
id: TASK-a1b2c3d4e5f6
type: task
slug: parseur-format
title: Parseur et modèle de données du format
created: 2026-07-27T09:20:00Z
status: done
scope:
  - crates/ankor-core/**
blocked_by: []
done_criteria: |
  parse/serialize round-trippent à l'octet près sur tous les fichiers de
  tests/golden/valid, chaque fichier de tests/golden/invalid est refusé avec
  l'erreur structurée attendue, et cargo test est vert.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: assertion
    ref: "suite golden verte avant mise sous git : 11 tests, round-trip identique"
schema: 1
version: 3
---

Socle de tout le reste : le CLI ne fera que composer ce crate avec git et
l'index.

## Log
- 2026-07-27T09:20Z claude-code@init — types, parseur, sérialisation canonique, gel par hash, log append-only
- 2026-07-27T09:45Z claude-code@init — suite golden : 5 fichiers valides, 9 invalides, 11 tests verts
