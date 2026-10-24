---
id: LOG-a4e6e295fda6
type: log
title: "Fix: push_ref runs push --porcelain and reads only the stdout status line of the pushed ref, for"
created: 2026-10-23T14:38:41Z
author: claude-code/opus-5.5+b896
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/**
  - docs/claims.md
  - docs/exit-codes.md
  - docs/integrating.md
about: TASK-b896bfbc9c8b
seq: 3
schema: 4
version: 1
---

 its parenthesised reason; ls-remote still classifies (ref == lease and '[remote rejected]' -> Pushed::Rejected, ref moved -> Refused as before, no answer -> Unreachable). claim::Sync gains Refused(reason) with the three sentences carrying the reason; claim unwinds its local ref and exits 9; accept names the reason. claim's help gains the 9 refusal and PUSH_DEGRADES says a refusing remote is named with its reason (help.json golden blessed, exit-codes.md regenerated, integrating.md's help block refreshed -- scope amended for it). Side defect found and filed as its own task: a release after a renewal that did not reach the remote leases on the local object, git answers stale info, and the deletion reads as a lost race in silence; the test here therefore releases a claim no refused renewal touched.
