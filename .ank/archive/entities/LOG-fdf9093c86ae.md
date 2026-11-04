---
id: LOG-fdf9093c86ae
type: log
title: "Measured: a scratch repo with a criterion amended+claimed on branch a, pushed to a bare origin, and"
created: 2026-11-04T14:58:39Z
author: claude-code/opus-5.5+task-6290-claim-remote-branch
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/claim_branch.rs
about: TASK-62903921874e
seq: 1
schema: 4
version: 1
---

 a clone holding a only as refs/remotes/origin/a. ank check in the clone under GIT_TRACE: binary before the fix 17 git processes (5 cat-file --batch), exit 8 with the divergence fault; after the fix 17 git processes (5 cat-file --batch), exit 0. The remote-tracking name rides in the same cat-file --batch as refs/heads/<b>, so the lookup adds no process. Remote fixed to origin, as git::remote and ADR-09b9 read it.
