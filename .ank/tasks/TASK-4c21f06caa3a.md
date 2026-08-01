---
id: TASK-4c21f06caa3a
type: task
slug: one-grammar-for-proof-parsed-in-one-place
title: One grammar for --proof, parsed in one place
created: 2026-07-31T22:48:47Z
status: done
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  The <type>:<ref> grammar of --proof is parsed by a single function, called by both done and attest. The error codes, the messages and the commit validation are identical by construction rather than by inspection, and each caller still names itself in its own hints. A test asserts that a malformed proof is refused the same way through both verbs.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/08ec4e6f2e87@7429cdd
    tree: scope/dc5581e02db2
    criteria: 636ea2f47b99
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@7429cdd
    tree: scope/dc5581e02db2
    criteria: 636ea2f47b99
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/5487c566af49@7429cdd
    tree: scope/dc5581e02db2
    criteria: 636ea2f47b99
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 4
---

attest was written with its own copy of the parsing, because done's version is private and widening it was outside TASK-1f4f7b57039b's scope. The duplication is real: two functions accept the same grammar, return the same codes, and validate a commit the same way.

Recorded rather than hidden. The failure mode is not the duplication itself but the drift: the day one of them learns a new proof type, or stops checking a commit against git, nothing makes the other follow. Two verbs that disagree about what a proof is would be a worse defect than the copy.

Each caller must keep naming itself in its hints. 'ank done --proof commit:<sha>' and 'ank attest <id> --proof commit:<sha>' are different next commands, and a shared parser that emitted a generic one would trade a real duplication for a broken error surface -- section 4 is explicit that the hint is the exact command to run.

## Log
- 2026-08-01T02:34:17Z seanl@sean-laptop — One parser, ProofUsage carrying what the caller must still say for itself: the command up to --proof, and the purpose completing 'proof required to ...'. Section 4 makes the hint the exact command to run, so a shared parser emitting a generic one would have traded a real duplication for a broken error surface -- 'ank done --proof commit:<sha>' is not the command an attest caller needs. criteria became Option<&str> because the two callers genuinely differ: done always holds the frozen criterion it just verified, attest records against whatever the finished task carries, which may be nothing.

The comparison test runs both real verbs over the same inputs and compares code and message, then strips each caller's own prefix from the hint and requires the remainder to match. Asserting the agreement by inspection is exactly how drift goes unnoticed, which is what this task was filed about.

The commit case broke my first version of that test and the test was wrong, not the code. When a commit is not in the repository the hint is 'git log --oneline -1 <sha>' -- the same next command whoever asked, so it carries no verb prefix at all. I had assumed every hint starts with the verb. Fixed by making the prefix strip unconditional and the prefix assertion conditional on the hint being an ank command, which is the honest shape: the two hints must agree once you remove who is speaking, and sometimes nobody is speaking.
- 2026-08-01T02:34:59Z seanl@sean-laptop — done, proof test:local/08ec4e6f2e87@7429cdd test:local/e3b0c44298fc@7429cdd test:local/5487c566af49@7429cdd
