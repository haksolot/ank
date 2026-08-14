---
id: TASK-d9d364bad929
type: task
slug: re-point-the-citations-of-the-superseded-complet
title: Re-point the citations of the superseded completion-ref ADR
created: 2026-08-11T18:51:09Z
author: claude-code@nested-pebble
status: done
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Once ADR-6d8736c04cfa is accepted and ADR-bcf222a31525 is marked superseded, no source file of the crate cites the superseded id as the reason for a design: the nine citations in claim.rs, context.rs, done.rs, git.rs and human.rs name the ADR that binds, or drop the citation where history is what they were recording. The id joins the DEAD list of no_superseded_adr_is_cited_in_the_crate, which is what makes the property hold afterwards rather than being restored by hand. cargo test and ank check stay green.
criteria_by: creator
proof:
  - type: commit
    ref: 09ac9779fad6fa5f29310f4025d0c655d68d8e43
    criteria: 761bb2d72dd4
schema: 2
version: 3
---

Deliberately not folded into TASK-78326e2e3e89, which created the succession.
Three of the nine files are outside that task's scope -- context.rs, done.rs and
git.rs -- and, more to the point, the citations are still correct while the
successor is `proposed`: the succession happens at `accept` (human.rs, check_adr),
so ADR-bcf222a31525 stays accepted and keeps binding until a human ratifies the
replacement on the default branch. Re-pointing them earlier would have made live
code cite an ADR that binds nobody, which is the same defect one step to the left.

The test that will hold it already exists and already carries the reasoning:
no_superseded_adr_is_cited_in_the_crate, whose DEAD list is hand-maintained and
whose doc comment says why a comment citing a superseded ADR is worse than no
comment at all -- it hands the next reader a constraint that binds nobody, with
the authority of a decision record. Two ADRs are in that list for exactly this,
and they went on asserting a frozen agent surface long after the split they
protected had been dissolved.

Blocked by nothing in the corpus, and blocked in fact by an act only a human
performs. If the succession is never accepted, this task is closed rather than
done: there is then no superseded ADR and nothing to re-point.
