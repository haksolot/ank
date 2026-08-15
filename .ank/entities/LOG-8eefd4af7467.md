---
id: LOG-8eefd4af7467
type: log
title: "One parser, ProofUsage carrying what the caller must still say for itself: the command up to"
created: 2026-08-01T02:34:17Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
about: TASK-4c21f06caa3a
seq: 0
schema: 3
version: 1
---

 --proof, and the purpose completing 'proof required to ...'. Section 4 makes the hint the exact command to run, so a shared parser emitting a generic one would have traded a real duplication for a broken error surface -- 'ank done --proof commit:<sha>' is not the command an attest caller needs. criteria became Option<&str> because the two callers genuinely differ: done always holds the frozen criterion it just verified, attest records against whatever the finished task carries, which may be nothing.
