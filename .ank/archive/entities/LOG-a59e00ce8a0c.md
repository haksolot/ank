---
id: LOG-a59e00ce8a0c
type: log
title: "discrepancy: the criterion asks that a tagged release publish the archive and its .sha256 alongside"
created: 2026-08-16T04:08:03Z
author: claude-code/054f
scope:
  - .github/workflows/release.yml
about: TASK-054fd964221f
seq: 1
schema: 3
version: 1
---

 the others, and that clause is not observable from this branch. Tagging publishes to npm and creates a public release, so I did not run it and no proof of the archive existing can come from here. What I did verify: the workflow parses, the build matrix carries four rows with one key set (os, target, archive) and unique values on each, the new row is x86_64-apple-darwin with archive tar.gz, so the packaging step takes its else branch and copies target/x86_64-apple-darwin/release/ank, tars it and writes the .sha256 exactly as the two other tar.gz rows do. The upload artefact name is the target, which stays unique, and the publish job globs dist/*.tar.gz and dist/*.sha256 after a merged download, so the new files travel with the rest. The version job is untouched and npm-smoke keeps its three rows: npm-assemble.sh matches artefacts by target substring, and x86_64-apple-darwin matches none of the three package names, so nothing it downloads changes. The end-to-end proof of the archive is a release, and only a release.
