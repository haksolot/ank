---
id: LOG-4ea4e7165f9b
type: log
title: "The missing --proof hint is shared: done.rs::submitted_proof serves both done and attest, so the"
created: 2026-08-13T23:11:31Z
author: claude-code/ca78
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-ca784c5feda4
seq: 0
schema: 3
version: 1
---

 one-line change from test:<ci-run-ref> to commit:<sha> lands on ank attest too. Consistent rather than accidental -- the other three hints in that function already say commit:<sha>, and the repair recipe in CLAUDE.md is ank attest --proof commit:<new-sha>. human.rs:2545 keeps test:<ci-run-ref>, and rightly: that is the missing-id refusal, not the missing-proof one.
