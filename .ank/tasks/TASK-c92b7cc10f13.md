---
id: TASK-c92b7cc10f13
type: task
slug: a-git-failure-in-the-signature-read-degrades-to
title: A git failure in the signature read degrades to no verdict at all
created: 2026-08-02T22:31:49Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  When the signature of a reachable ratification commit cannot be read because git fails, check says so rather than staying silent. The case is built and run through the binary.
criteria_by: creator
schema: 2
version: 3
---

Found by reading while diagnosing TASK-1ea38a17d854, not by an incident.

signature_state ends with git::signature_of(...).ok()?, so any error from the git invocation becomes None, and None is the one verdict check_adr says nothing about. The ADR then passes in silence with no signal, no fault and no count -- indistinguishable from a corpus with no allowlist.

This is the shape ADR-6b3f and the Unchecked case both refuse elsewhere: "a verification that degrades to success is not a verification", which is exactly why Unchecked is counted rather than dropped. Here the same degradation happens one level up, on the error path instead of the status path.

It is narrow: freeze_state has already reached the same commit through the same memo, so a git that answers there usually answers here. Narrow is not never -- a bad allowed_signers path, a gpg.format git refuses, an object read that fails mid-run -- and the cost of being wrong is silence on the one case the whole mechanism exists for.

The fix is probably a sixth state or a second corpus counter, not a fault: a broken environment is not a forged ratification, and reporting one as the other is how a finding becomes noise. Whatever it becomes, it must not be nothing.

## Log
- 2026-08-02T23:14:41Z seanl@sean-laptop — Fixed: signature_state no longer ends in .ok()? -- a git failure becomes Signature::Unreadable { reason }, a sixth state, and check_adr counts it into report.unreadable_signatures with the first reason kept. One corpus line, like unchecked_signatures and for the same reason: 'N ratification signature(s) could not be read, so they are neither verified nor refused: <git's own message>'. A signal, not a fault -- a broken environment is not a forged ratification, and exiting 8 would fail the check-repo verifier of every task on a machine whose only defect is its gpg config. Kept apart from Unchecked on purpose: no key for an answer is not the same as no answer. The test lever is a gpg.format git rejects, one of the causes named when the task was filed. Measured before using it: with gpg.format=bogus in the repo config, rev-list --full-history, cat-file, rev-parse, for-each-ref and symbolic-ref all still exit 0, and only the signature read exits 128 -- so the corpus is still read and the ratification commit still reached, and nothing but the signature path is under test. Proved by reverting human.rs alone: the test fails with 'check: ok' and no mention of the ADR, the silence the task was filed for. Note for later: the status check in git::signature_of is load-bearing beyond this. With gpg.format=bogus git prints 'commit <sha>' and no placeholder lines, so a parser trusting the output would read the missing %G? as N and accuse a perfectly signed ratification of being unsigned.
