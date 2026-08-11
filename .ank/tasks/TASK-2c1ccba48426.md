---
id: TASK-2c1ccba48426
type: task
slug: second-pass-coordination-findings
title: Second-pass coordination findings
created: 2026-08-06T23:24:57Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/**
blocked_by: []
done_criteria: |
  The four findings appear in the body, each precise enough to act on, and ank check stays green on this repo.
criteria_by: creator
schema: 2
version: 4
---

Read-only second pass over the coordination plane - claims, renewal, completion
refs, pruning, and what context shows while a claim is held. Nothing claimed or
done here; these are notes for later triage, in the shape of TASK-abbaab9007a0.
The two findings large enough to stand alone left as their own tasks: level 1
being unimplemented, and done not warning on constraint drift.

1. Renewal silently drops a custom TTL. claim resolves --ttl against
   claim_ttl_max and writes the result into the record (resolve_ttl,
   claim.rs:953-960). The renewal in log does not read it back: it recomputes
   DEFAULT_TTL.min(cfg.claim_ttl_max) (commands.rs:1174). So an agent that asked
   for a long lease gets it once, then falls back to thirty minutes at its first
   log - which is to say, almost immediately, since log is what agents are told
   to run often. Either the record's TTL is the lease and renewal reuses it, or
   --ttl means "for this acquisition only" and says so.

2. close leaves no completion ref. done transforms the ref into a completed
   record precisely so a task finished on an unmerged branch does not look free
   elsewhere (claim.rs:668-691). close deletes the claim outright
   (claim.rs:437-457, human.rs:1783-1789), so a task closed on a branch reads
   open everywhere else until the merge - the exact window done exists to close.
   ADR-bcf222a31525 acknowledges the gap without settling it. The asymmetry may
   be defensible (a close is a decision, not a result) but it is currently
   implicit.

3. claim::prune is dead. claim.rs:800-828 is documented as "called by check" and
   is called by nothing outside tests; the live path is human::maintain
   (human.rs:1136-1229), which reimplements the same predicate. Two copies of the
   rule that decides when a coordination ref disappears, one of them unexercised
   in production and therefore free to drift. TASK-52fbffbfdf65 already showed
   what a wrong pruning decision costs: a lost claim is a task two agents can
   hold at once.

4. Under a claim, context stops showing the other agents. With a claim held,
   context switches to execution mode and renders that task alone - criterion,
   constraints, log (context.rs:304-343). The coordination markers
   [claimed:holder] and [finished:sha on branch] are orientation-mode only. So
   the agent best placed to notice a collision - the one currently working - is
   the one that cannot see the plane any more. Possibly correct by design, since
   execution mode exists to remove choice; but if so it is unrecorded, and
   ank status does not fill the gap either: it reports the agent's own claim, not
   anyone else's.

## Log
- 2026-08-11T03:41:26Z seanl@sean-laptop — Triage pass, each finding re-measured against the tree rather than trusted from the note. All four are still live. (1) log renewal recomputes DEFAULT_TTL.min(claim_ttl_max) at commands.rs:1316 and never reads the lease back off the record, so --ttl holds for one acquisition and collapses to thirty minutes at the first log; the record carries claimed and expires, so the granted lease is recoverable without a format change. (2) close calls claim::delete at human.rs:1910 where done converts the ref to a completion record, so a task closed on a branch reads open in every other clone until the merge -- the exact window ADR-bcf2 created the completion ref to close. (3) claim::prune at claim.rs:821 has no caller outside its own tests; the live predicate is human::maintain at human.rs:1153-1229. Two copies of the rule that decides when a coordination ref disappears, one unexercised in production. (4) execution mode renders the claimed task alone and drops the [claimed:holder] and [finished:sha] markers, so the agent best placed to notice a collision is the one that cannot see the plane; status reports the caller's own claim and not anyone else's. Filing one task per finding rather than fixing them under this claim: this task's criterion is about recording, and a fix needs a criterion of its own.
- 2026-08-11T03:42:53Z seanl@sean-laptop — Four tasks filed, one per finding, each with a criterion that measures something the note did not: TASK-1b45f41e7b99 the TTL lease, TASK-78326e2e3e89 the close asymmetry, TASK-4981a1370c0b the duplicated pruning predicate, TASK-dacbcae6134c what a holder sees of the coordination plane. Three of the four are settled in the specification or an ADR before the code moves, because three of the four are questions about what the tool means rather than defects in what it does -- only the TTL one is unambiguously a bug. Noted while filing TASK-dacbcae6134c: the finding may be moot until level 1 ships, since within one clone the refs are shared and claim already refuses, and that is itself an answer worth writing down rather than a reason to drop it.
