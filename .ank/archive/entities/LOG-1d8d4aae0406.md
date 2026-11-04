---
id: LOG-1d8d4aae0406
type: log
title: Dead proof 77aa58c is missing from this clone and from origin/main. The work landed as e5e0d81870a2
created: 2026-11-04T10:02:08Z
author: claude-code/opus-5.5+task-a646-proofs
scope:
  - .ank/**
about: TASK-9bff1d5826b1
seq: 0
schema: 4
version: 1
---

 (on main, 'The corpus moves to the flat layout...'), already attested twice (347133df, 13c3dfa0). check cannot clear this signal by attest: detached_commit_proofs (crates/ank-cli/src/human.rs:5362) asks git rev-list --all for every commit proof the task lists, and the dead entry stays in the append-only list, so the signal lasts as long as the sha is unreachable, whatever is appended after it. Measured on 2026-11-04 from main c9f113da.
