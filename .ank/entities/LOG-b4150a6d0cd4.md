---
id: LOG-b4150a6d0cd4
type: log
title: "closed: written from an incomplete search and wrong entire: the npm packages do ship the licence"
created: 2026-08-17T20:11:52Z
author: claude-code/2.1.233+exposition
scope:
  - .github/workflows/release.yml
  - npm/**
about: TASK-edddce600c4f
seq: 0
schema: 3
version: 1
---

 they declare. .github/scripts/npm-assemble.sh:62 copies the root LICENSE into each platform package and line 79 copies it into the wrapper, which is why .gitignore carries npm/*/LICENSE -- the file is a release-time artefact, not a missing one. I grepped release.yml and the package.json files and never opened .github/scripts/, and the .gitignore entry was the clue I had in hand and did not follow. Nothing to do, and the relicensing already reaches npm: the text those packages ship is the root LICENSE, which is now Apache-2.0.
