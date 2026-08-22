---
id: TASK-e2da6b0cc817
type: task
slug: check-follows-a-succession-instead-of-asking-for
title: check follows a succession instead of asking for the citation to be rewritten
created: 2026-08-21T23:54:18Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  check resolves a spec's reference through the succession chain and reports nothing where that chain ends on an accepted entity, whatever its length: a test builds a corpus whose citation is two hops behind and asserts no finding names the citing document. The three findings that remain are unchanged and tested beside it, each in its own case: absent is a fault, a kind a specification may not cite is a fault, and not yet accepted is a signal naming ank accept. A chain ending on a superseded entity that nothing replaces keeps the signal it has today, in the same words. Nothing is written to make a reference resolve: a test asserts the citing document's file is byte for byte what it was before a check that resolved one, and that its version did not move. The check catalogue of a spec superseding SPEC-f353359663d5 states the rule as it now stands, and the clause it replaces is the only prose that changes. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 2
---

ADR-c88f99e1c16e is the decision; this is all of the code it costs, and it is
less than the decision suggests. `chain_head` already walks the succession,
cycles included, and `check_references` already calls it: today only to name a
successor in the signal it prints. What changes is that the walk decides the
finding instead of decorating it.

**The branch to delete is the one that already half exists.** The current rule
lets a citation off when the citing document also references the end of the
chain, which is a reader following a succession, spelled by hand and stored
twice. That special case goes away by becoming the general one.

**The negative tests are the point of this task**, more than the positive one.
Removing a finding is easy to do too widely, and three findings sit beside the
one being removed: a reference to something absent, a reference to a kind a
specification may not cite, and a reference to a document not yet accepted.
Each keeps its severity and its words, and each gets a case, because a change
that quietly took a fault down to silence would be worse than the churn it was
meant to end.

**The write-nothing assertion is not a formality.** The whole argument against
the alternative design was that repairing citations in place would write to
nine entities and, under ADR-16813b3bcf37, leave nine machinery entries from one
`accept`. A test that reads the citing file before and after and compares bytes
is what keeps this implementation on the side of that argument.

**What must not be built here.** No normalisation on write, no refusal on write,
no verb that rewrites a stored identifier. The resolution is the reader's and
lives in one function.
