---
id: LOG-ff86bcfdb86f
type: log
title: "Fix in claim::push: a Refused{holds: None} on a lease naming a witness is retried leasing on"
created: 2026-09-22T17:35:20Z
author: claude-code/opus-5+2d77
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
about: TASK-2d779142ca70
seq: 4
schema: 4
version: 1
---

 absence (a deletion is already done there). Measured with the fixed binary on the same scratch reproduction: ank log exit 0, no warning, holder tool/1.0, expires 18:04:47Z -> 18:04:48Z, origin now carries refs/ank/claims/<id>; GIT_TRACE counts 2 push processes on that first renewal (refused lease + retry), the path only taken when the remote lacks the ref. Genuine holder on origin (forged record holder other/2) still read: log refuses at exit 6 and the local ref shows other/2. Regression test tests/remote_without_claim.rs (log, log --json, done --proof) seen red before the fix with the exact 'was taken over while logging' and error[4] 'moved while it was being completed', green after.
