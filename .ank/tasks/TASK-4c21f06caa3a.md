---
id: TASK-4c21f06caa3a
type: task
slug: one-grammar-for-proof-parsed-in-one-place
title: One grammar for --proof, parsed in one place
created: 2026-07-31T22:48:47Z
status: open
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  The <type>:<ref> grammar of --proof is parsed by a single function, called by both done and attest. The error codes, the messages and the commit validation are identical by construction rather than by inspection, and each caller still names itself in its own hints. A test asserts that a malformed proof is refused the same way through both verbs.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 1
version: 1
---

attest was written with its own copy of the parsing, because done's version is private and widening it was outside TASK-1f4f7b57039b's scope. The duplication is real: two functions accept the same grammar, return the same codes, and validate a commit the same way.

Recorded rather than hidden. The failure mode is not the duplication itself but the drift: the day one of them learns a new proof type, or stops checking a commit against git, nothing makes the other follow. Two verbs that disagree about what a proof is would be a worse defect than the copy.

Each caller must keep naming itself in its hints. 'ank done --proof commit:<sha>' and 'ank attest <id> --proof commit:<sha>' are different next commands, and a shared parser that emitted a generic one would trade a real duplication for a broken error surface -- section 4 is explicit that the hint is the exact command to run.
