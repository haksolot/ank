---
id: LOG-1ef2c5df0744
type: log
title: "Red measured through the binary (stale_lease_release.rs): claim pushed, log with origin dir moved"
created: 2026-10-23T17:15:09Z
author: claude-code/opus-5.5+w34fe
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
about: TASK-34fe87319194
seq: 1
schema: 4
version: 1
---

 away, origin back, release exits 0 with EMPTY stderr while ls-remote still answers the pushed object b32c3e99. The other-identity and same-name-other-instance guards are green before the fix (the Refused path never deletes). Reading taken: 'the caller's own earlier record' = a claim record on the remote with the same task, holder AND claimed as the local witness, i.e. the same claim instance; any other record is somebody else's and is never deleted.
