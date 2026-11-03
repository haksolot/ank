---
id: LOG-d43eca646d33
type: log
title: "Measured with GIT_TRACE: ank claim runs 13 git processes, up from 12 (claims_remote.rs pins claim"
created: 2026-11-03T12:47:34Z
author: claude-code/opus-5.5+task-346b-claim-branch
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/claim_branch.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/claims_remote.rs
  - docs/integrating.md
about: TASK-346bdb3b4378
seq: 7
schema: 4
version: 1
---

 and claim2 at 13 now). The one added is git symbolic-ref --quiet HEAD, the branch the record names; seen in the claim trace beside the 11 it ran before (version, rev-parse dirs, config remotes, for-each-ref, rev-parse claim ref, hash-object, update-ref, ...). Not folded into the memoised rev-parse: --symbolic-full-name HEAD fails on an unborn branch. docs/integrating.md check --json plane_bytes 173 -> 186, the 13 bytes of 'branch: main' plus newline. Per the orchestrator: the 0.9.2 compat break (LOG-36d3053f540c) is accepted through requires_ank in the release, with record tolerance filed separately.
