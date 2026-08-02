---
id: TASK-c92b7cc10f13
type: task
slug: a-git-failure-in-the-signature-read-degrades-to
title: A git failure in the signature read degrades to no verdict at all
created: 2026-08-02T22:31:49Z
author: seanl@sean-laptop
status: open
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  When the signature of a reachable ratification commit cannot be read because git fails, check says so rather than staying silent. The case is built and run through the binary.
criteria_by: creator
schema: 2
version: 1
---

Found by reading while diagnosing TASK-1ea38a17d854, not by an incident.

signature_state ends with git::signature_of(...).ok()?, so any error from the git invocation becomes None, and None is the one verdict check_adr says nothing about. The ADR then passes in silence with no signal, no fault and no count -- indistinguishable from a corpus with no allowlist.

This is the shape ADR-6b3f and the Unchecked case both refuse elsewhere: "a verification that degrades to success is not a verification", which is exactly why Unchecked is counted rather than dropped. Here the same degradation happens one level up, on the error path instead of the status path.

It is narrow: freeze_state has already reached the same commit through the same memo, so a git that answers there usually answers here. Narrow is not never -- a bad allowed_signers path, a gpg.format git refuses, an object read that fails mid-run -- and the cost of being wrong is silence on the one case the whole mechanism exists for.

The fix is probably a sixth state or a second corpus counter, not a fault: a broken environment is not a forged ratification, and reporting one as the other is how a finding becomes noise. Whatever it becomes, it must not be nothing.
