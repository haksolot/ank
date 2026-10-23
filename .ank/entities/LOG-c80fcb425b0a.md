---
id: LOG-c80fcb425b0a
type: log
title: Measured by hand, scratch repo + bare file:// origin, log with origin moved away then back, release
created: 2026-10-23T17:17:18Z
author: claude-code/opus-5.5+w34fe
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
about: TASK-34fe87319194
seq: 2
schema: 4
version: 1
---

 under GIT_TRACE. main binary (~/.local/bin/ank): exit 0, stderr EMPTY, origin keeps 9fd41537 AND the local ref is re-fetched to that same stale record (the clone reads itself as holder after releasing); 1 push, 1 ls-remote, 1 fetch, 15 git built-ins. This branch: exit 0, origin ref gone, local ref gone; 2 pushes (stale-info lease, then the retry leasing on the remote's own-instance record), 1 ls-remote, 1 fetch, 24 built-ins (the extra are the retry push and its file:// receive-pack children plus rev-parse/cat-file reads; failure path only, constant per deletion, ADR-cc65). Mutation: replacing the task+holder+claimed guard with 'true' turns both guard tests red (other identity, same name/other claimed): the release then deletes the planted claim.
