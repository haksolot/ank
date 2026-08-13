---
id: TASK-7c2fa14284ff
type: task
slug: a-criterion-that-turns-out-unmeasurable-has-no-r
title: A criterion that turns out unmeasurable has no route back
created: 2026-08-07T16:36:57Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/editor.rs
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  A creator can correct a done_criteria that turned out unmeasurable, holding no claim, without the correction being recorded as the claimer's. The route is stated in docs/ank-spec-v1.1.md section 4 before the code moves, and it is exercised through the binary in crates/ank-cli/tests/cli.rs: the criterion changes, criteria_by stays creator, and ank check reports no fault. If the answer is that no such route belongs in the CLI, the specification says so and names what a creator does instead, and claim --criteria stops silently overwriting a criterion the creator already set.
criteria_by: creator
proof:
  - type: test
    ref: "31450126438"
    criteria: 99b7d783bbcc
schema: 3
version: 6
---

Found by dogfooding TASK-90442c8f0ca2. Its criterion ended on a clause that
cannot ever be true: gh api community/profile reports issue_template null for
any repository using an ISSUE_TEMPLATE/ directory rather than the legacy
single file, which is exactly the layout that task was built to produce. The
work was finished and correct; the measurement was not.

The release was right and it happened. What has no continuation is what comes
after. amend refuses --criteria unconditionally, by name and by design
(human.rs, near the flag table): a verb that edited the criterion would offer
to do the one thing the freeze exists to make visible. That reasoning holds
while a claim is held. It also fires on an open task nobody holds, and on a
task that has never been claimed at all -- measured on a fresh repository, the
refusal is unconditional, and its hint points at ank release, which a task
that was never claimed cannot run.

So the corrected criterion has two ways in, and both are wrong. A human edits
the file, which ADR-01b6dd05f0db permits and which check notices, but that is
the tool declining to do a thing it exists to do. Or claim --criteria, which
does overwrite an existing criterion, silently, and flips criteria_by to
claimer -- laundering a creator's correction into the exact shape the freeze
is meant to expose.

The contradiction is worth naming. The door that amend bolts shut is wide open
on claim, and open to the claimer, the one party the freeze constrains. One of
the two is wrong. If the position is the one the ADRs state -- immutability is
verifiable, not defended, the CLI is not a gatekeeper, check is what notices
-- then it is amend's refusal that is out of line, not claim's permissiveness.
Either way the specification decides first, then the goldens, then the code.
