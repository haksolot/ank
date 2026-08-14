---
id: TASK-5c7ebad62d93
type: task
slug: a-detached-proof-that-misses-the-remote-exits-no
title: A detached proof that misses the remote exits non-zero, and ci.yml stops reading the flag
created: 2026-08-13T16:23:46Z
author: claude-code/2.1.229
status: in_progress
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - .github/workflows/ci.yml
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  ank attest --detached exits non-zero when the push of refs/ank/proof is refused, with the warning it already prints and a hint naming the next command. Its help says the verb fails on an unreachable remote. Verbs whose write also landed on disk are untouched and still exit 0 with a warning, which the test asserts for claim so the change cannot silently generalise.
  
  The attestation step of .github/workflows/ci.yml reads the exit code, and the comment explaining why it read the flag instead is removed rather than left contradicting the code.
  
  Section 7 of docs/ank-spec-v1.1.md states the rule and which side of it each verb is on, before the code moves.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs against a file:// remote made unreachable: attest --detached exits non-zero, claim on the same remote exits 0 and warns.
criteria_by: creator
schema: 3
version: 2
---

Implements ADR-af533e7a3e03. The measurement, on a scratch corpus with a real
`file://` remote: reachable gives `"pushed":true` and exit 0; unreachable gives
`"pushed":false`, a warning on standard error, and **exit 0**.

The strongest evidence that the contract is wrong is already committed. The
attestation step of `ci.yml` carries a comment saying to read the flag and not
the exit code, because trusting the code would go green having lost the
attestation. The first integration ever written against this verb was written
around it. Removing that comment is part of the work, and not a tidy-up: a
comment explaining a workaround that no longer exists is how the next reader
learns to distrust the verb again.

**Do not generalise the change.** `claim` writes a ref too and stays on the
degrade side, because the claim it recorded still governs this clone and the risk
of a concurrent claim is displayed rather than hidden -- section 2, and
`Sync::warning` says exactly that in `claim.rs`. `Sync::proof_warning` is already
a separate sentence beside it, on the reasoning that a claim not pushed can be
taken twice while a proof not pushed is invisible to everyone. That split is the
seam this change follows. The test asserting `claim` still exits 0 is what keeps
the change from spreading.

The hint matters as much as the code. A pipeline that fails here has a remote it
could not reach, and the self-correcting-error rule means naming what to run --
not generic help, and not a suggestion to retry blindly.
