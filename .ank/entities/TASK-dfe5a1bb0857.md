---
id: TASK-dfe5a1bb0857
type: task
slug: check-counts-what-an-entity-accounts-for-and-say
title: check counts what an entity accounts for, and says so when the arithmetic does not close
created: 2026-08-21T20:45:35Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/entries.rs
blocked_by: [TASK-3c12e0ced2c0]
done_criteria: |
  An entity carrying at least one machinery entry is accounted for, and check reports a signal naming both numbers when its version exceeds what those entries account for. An entity carrying none is silent, and a test asserts that a corpus of entities written before this existed produces not one finding from this rule. The finding is a signal and never a fault, so the exit code of a corpus whose arithmetic does not close is still 0 and no done is blocked by it. Driven through the binary: an entity edited twice through the CLI and then edited a third time by writing the file directly is reported with its two counts, and the same entity before that third edit is not. The check catalogue of a spec superseding the current CLI surface document describes the finding, what it derives and what it deliberately does not conclude. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 3
version: 3
---

The teeth of ADR-16813b3bcf37, and the reason the trace is worth writing: a
trail nobody counts is a trail nobody reads. After this, hiding an edit means
forging two things a single write cannot both fix, the version the entity
carries and the entries it accounts for.

**A signal, and the exit code is the whole argument.** §11 is explicit that a
finding which reddens a pipeline over something a reader must judge teaches the
reader to stop reading `check`, and this is exactly such a finding: an entity
whose arithmetic does not close was edited outside the CLI, which is legal, is
what a human with an editor does, and is what ADR-01b6dd05f0db permits a human
while asking it of no agent. What the signal says is that it happened, not that
it was wrong.

**The bootstrap is the absence of a first entry**, which is what makes this
affordable at all: no schema moves, no corpus is migrated, no entity gains a
field it did not have, and the thousand entries already written stay silent
until the CLI next edits what they are about. The test that matters most is
therefore the negative one, on a corpus that predates the whole mechanism.

**The falsification the criterion asks for is a direct file write**, performed
by the test rather than described: edit an entity twice through the binary, then
write its file with the bytes changed and the version bumped by hand, and watch
the count disagree. Anything less tests that the arithmetic adds up, which was
never in doubt, rather than that it catches what it exists to catch.
