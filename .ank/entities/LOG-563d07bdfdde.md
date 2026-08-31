---
id: LOG-563d07bdfdde
type: log
title: drift audit 2026-08-31, re-measured and holds, including the half that only shows up across
created: 2026-08-31T07:53:21Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/git.rs
  - docs/**
about: ADR-6d8736c04cfa
seq: 0
schema: 4
version: 1
---

 branches. done on branch feat left refs/ank/claims/<id> as a completion ref; back on main, where the task file still reads 'status: open', 'ank claim <id>' exits 4 with 'TASK-2b9056889f89 finished on another branch (commit b19f7de, branch feat), not merged here yet' -- the commit and the branch both named, as the constraint requires. The asymmetry holds the other way: 'ank close <id> --reason' printed 'the active claim was revoked' and left no ref at all, measured by for-each-ref before and after. A done task whose file on this branch already says done is refused at 6 on the transition rather than at 4, which is the earlier gate and not this one.
