---
id: LOG-5af24c2fa351
type: log
title: Measured on ank 0.7.0 (50f4b39) in a throwaway corpus, counting changed paths with 'git status
created: 2026-08-31T04:00:26Z
author: claude-code/opus-5+corpus
scope:
  - .ank/entities/**
about: TASK-53a95782ae5c
seq: 0
schema: 4
version: 1
---

 --porcelain -- .ank' rather than hashing files, so the measurement of ADR-01b6dd05f0db does not itself do what ADR-01b6dd05f0db forbids. index.db is gitignored, so it never enters the count.

Writes, each exit 0: ank read (1 path), ank amend --scope (2), ank amend --blocked-by (2), ank edit --title (2), ank config default_branch main (1). Writes nothing, each exit 0: ank scope, ank graph, ank status, ank review, ank check, ank log <id> in its read form, ank show, ank find, ank context (0 paths each). ADR-01b6dd05f0db lists none of read, amend, edit or config; its writing list is 'ank new, claim, log, done, release, attest, close and accept'.

Two nuances worth recording. ank config writes .ank/config.yml, which is not an entity, and it is on the writing list because the constraint governs .ank/ and not only its entities. And ank read is a reading act that writes: it records the reading, which is why it belongs on the writing side of an enumeration about bytes.
