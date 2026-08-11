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
schema: 2
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

## Log
- 2026-08-11T01:25:27Z seanl@sean-laptop — Direction settled with the maintainer: amend opens on state, claim closes. amend --criteria is allowed unless a live claim anchors the criterion, leaves criteria_by untouched, and is logged; claim --criteria stops overwriting a criterion that already exists and points at amend, so it only ever sets an absent one -- which is the case section 3 designed it for, and which keeps criteria_by:claimer meaning exactly one thing. Discovery that shrinks the work: edit.rs already implements this rule. check_frozen refuses a moved done_criteria only when live_claim_anchor finds a claim in force, and returns Ok otherwise, with the comment saying refusals are on state and never on identity. So a route through the CLI already existed -- ank edit -- and it was never scriptable and never named in the specification. amend will reuse that helper rather than restate the rule: it moves to claim.rs, where both callers can reach it.
- 2026-08-11T01:25:34Z seanl@sean-laptop — amended: +scope crates/ank-cli/src/claim.rs, +scope crates/ank-cli/src/edit.rs, +scope crates/ank-cli/src/cli.rs, +scope crates/ank-cli/tests/**
- 2026-08-11T01:38:28Z seanl@sean-laptop — Specification first, then the code, as ADR-63b5 orders. Section 4 now states the route: amend --criteria refused only while a live claim freezes the criterion, criteria_by untouched because an amend is not a claim, and claim --criteria sets an absent criterion and never replaces one. Revision l records it. The state test is claim::live, one function both amend and edit call, so the two cannot drift; edit had it privately and the specification said nothing about it. Two consequences beyond the criterion. SKILL.md line 65 became false the moment amend changed -- it taught that amend will not touch done_criteria, that release is the route -- so the sentence is corrected and metadata.revision bumped to 8295e2081364; this is a correction, not growth, the verbs and modes it teaches are unchanged and it stays inside the 140/1200 ceiling. And cli.rs::refused is now called by nothing, amend having been its only user: left in place, with its doc comment on the pattern, since the crate root allows dead code and deleting it would take listed and listed_flags with it. Verified through the binary on a scratch repository: the correction goes through unclaimed, is refused under a claim naming the holder, and claim --criteria refuses with amend as the next command.
- 2026-08-11T01:46:22Z seanl@sean-laptop — done, proof test:31450126438
