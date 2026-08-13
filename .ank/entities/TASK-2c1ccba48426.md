---
id: TASK-2c1ccba48426
type: task
slug: second-pass-coordination-findings
title: Second-pass coordination findings
created: 2026-08-06T23:24:57Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/**
blocked_by: []
done_criteria: |
  The four findings appear in the body, each precise enough to act on, and ank check stays green on this repo.
criteria_by: creator
proof:
  - type: test
    ref: "31456251424"
    criteria: 4d0c2b12b6b4
schema: 3
version: 5
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
