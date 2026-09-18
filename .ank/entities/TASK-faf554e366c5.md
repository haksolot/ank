---
id: TASK-faf554e366c5
type: task
slug: the-no-op-renewal-test-wins-a-clock-race-instead
title: The no-op renewal test wins a clock race instead of arranging one
created: 2026-09-18T15:56:59Z
author: claude-code/opus-5+noop-clock
status: done
scope:
  - crates/ank-cli/tests/renewal_noop.rs
blocked_by: []
done_criteria: |
  Through the binary: a_renewal_that_changes_nothing_writes_nothing_and_pushes_nothing reaches the identical-record case by arranging it rather than by retrying until the wall clock cooperates, so no run of it can fail for want of luck. The test formats its own UTC stamp, and it proves that formatting against a stamp the binary itself wrote before it relies on it, so a drift in either format is a named failure and not a silent one. Both halves of the criterion of TASK-43c2e64d1d30 keep their assertions and their recorded process counts. Green on ubuntu, macos and windows in CI, and the windows job is the one that has to be watched.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/f48c736ad3a4@16b8697
    tree: scope/56a6ed70a6ba
    criteria: b770954637de
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@16b8697
    tree: scope/56a6ed70a6ba
    criteria: b770954637de
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 5
---

Main went red at 298c01a on the `windows-latest` row alone, in the run that
followed the merge of TASK-cf81d7d57d1e, with `ten claims never shared a second
with their renewal`.

It is a flake and not a regression: `f89a9bf`, the head of that same branch and
the same tree, had passed the same job minutes earlier, and the eleven CI runs
before it were green.

The mechanism is in the test. A renewal writes a byte-identical record only when
it lands in the same wall-clock second as the write it is compared against, and
the test reaches that state by claiming and hoping, ten times over. What decides
each attempt is the phase of the second at which `claim` happens to write its
record, which nothing aligns: `cross_a_second` runs before the claim, and the
claim then takes a whole verb -- at level 1, a push included -- to reach its
write. On a runner where the gap between that write and the renewal approaches a
second, every attempt is close to a coin toss and ten of them are not enough.

The state is arrangeable. `forge_expiry_ahead` already rewrites the record on the
ref and on the origin; the same route can write the expiry a renewal in a chosen
second will compute, and the test can then enter that second at its start rather
than at an arbitrary point in it. What is left to chance is only that one
`context` reaches its renewal inside the second it began in.

A test that measures process counts must not be able to fail for a reason that
has nothing to do with process counts. CLAUDE.md says a fact is measured and
never timed; this one is measured correctly and gated on a timing accident.
