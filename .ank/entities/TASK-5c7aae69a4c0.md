---
id: TASK-5c7aae69a4c0
type: task
slug: a-check-that-cannot-read-an-entity-prescribes-de
title: A check that cannot read an entity prescribes deleting the citations that name it
created: 2026-08-22T00:06:32Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Where a corpus holds entities this build cannot read, no finding concludes that an entity does not exist and no repair proposes removing a citation, a reference or a succession that names one. What is said instead names the cause once: resolution is incomplete because N entities declare a schema this build does not read, with the command that resolves it. A test builds a corpus where a readable entity references, is blocked by, and is superseded by entities one schema ahead of SCHEMA_VERSION, drives the binary, and asserts that no finding contains drop-reference and none says does not exist or names a succession as missing. The findings that rest on a target this build did read are unchanged, and a case asserts each still fires. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

Measured on this corpus with the released binary, against the same corpus this
morning. Nine entities declare schema 4 and `ank 0.5.0` reads three, which the
schema gate reports correctly and refuses to read. What follows the refusal is
the defect: **every check that resolves an identifier then treats unreadable as
absent**, and prescribes the repair for absent.

Nineteen faults, of which nine are the honest ones. The other ten are inventions:

    error: SPEC-183d297253ac: references SPEC-a89ba4f755e9, which does not exist
           (ank amend SPEC-183d297253ac --drop-reference SPEC-a89ba4f755e9)
    error: SPEC-acee5d9cb21b: marked superseded but no spec supersedes it

The first is prescribed eight times over, against citations that are correct.
**A reader following that hint deletes a valid reference**, and the corpus is
worse afterwards than before, which no finding in this tool is allowed to do.
The second is false for the same reason: the successor exists and is simply
unreadable here.

**The gate is right and the conclusion drawn after it is wrong.** Refusing to
read a file written under a schema this build does not know is exactly what
§3 asks for, and `schema_ahead` already counts them and says so in a warning.
The information needed to avoid every one of these ten is therefore already in
hand at the moment they are printed: this build knows it could not read nine
files, so it knows that "not found" means "not found among what I could read".

**The severity question is settled by the direction of the damage.** A finding
that under-reports leaves a corpus as it was; a finding that prescribes a
deletion of something correct makes it worse, and does so through a reader who
was doing what the tool told them. So the rule is not to soften these findings
but to stop drawing the conclusion at all while the premise is incomplete, and
to name the incompleteness once rather than once per citation.

**Why this matters beyond one stale laptop.** Every installed copy of ank is in
this state until a release carrying schema 4 ships, and an agent running `ank
check` in that state would read ten actionable repairs and could perform them.
This is the first finding in this corpus that can damage a corpus by being
followed.
