---
id: LOG-b6b35961b8fb
type: log
title: Measured through the binary, on a scratch corpus whose origin is a bare repository moved out of the
created: 2026-08-31T07:37:43Z
author: claude-code/opus-5+degrade
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-c5c93cc8e5f8
seq: 0
schema: 4
version: 1
---

 way between the claim and the revocation -- the only shape where the risk is real, since a claim that never travelled cannot go stale on a remote that never had it.

Before (ank built from c48e002):
  claim, origin reachable   -> ref reaches origin, exit 0
  release, origin gone      -> stdout "released <id> -> open", stderr 0 bytes, --json warnings [], exit 0
  close, origin gone        -> stdout "closed <id> -> closed", stderr 0 bytes, exit 0
  for-each-ref on origin    -> the claim ref is STILL THERE after both

After:
  release, origin gone      -> stderr "warning: claim deletion not pushed: the claim is gone in this clone only, and another clone still reads the task as held until the claim expires", exit 0
  close, origin gone        -> the same sentence on stderr, exit 0
  --json                    -> the document is byte-identical, warning on stderr only
  no remote at all          -> both verbs silent, exit 0

The divergence was the "warns" clause of ADR-af533e7a3e03 and nothing else: the exit code was already right and the help already declared PUSH_DEGRADES, so it was honest. The ADR is not what is wrong here -- the risk it asks to be displayed is the stale ref measured above -- so no supersession was written and no ratified text was touched.

Cause: claim::delete_at did `let _ = push(...)`, discarding the result, so neither verb could see it. It now returns Deleted { existed, sync }. The sentence is a third one beside Sync::warning and Sync::proof_failure, because it is a third risk, and it names no local status: release leaves the task open and close leaves it closed. Standard error in both modes, which is where done puts the same class of warning (ADR-6fd69efb629c: stdout under --json is a parser input).

Red-first, both halves independently: with only the release eprintln reverted the test fails on "release degraded in silence"; with only the close eprintln reverted it fails on "close degraded in silence". cargo test --workspace green, 334 + 327 + the rest, 0 failed. cargo fmt --check clean.
