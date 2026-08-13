---
id: ADR-0bb7ea8991bc
type: adr
slug: a-claim-is-renewed-by-working-not-by-reporting
title: A claim is renewed by working, not by reporting
created: 2026-08-13T16:20:09Z
author: claude-code/2.1.229
status: accepted
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - docs/ank-spec-v1.1.md
constraint: |
  A verb the holder runs against the task it holds renews that claim's lease, and the repository states its own rhythm through claim_ttl_default in config.yml. Renewal with no change to the scope files stays a check signal, which is what keeps reading from becoming a way to hold.
ratified: ef81cf6398a3
schema: 3
version: 2
---

Section 3 renews a claim implicitly on `log`, on the reasoning that working is
enough to keep the lock and there is no `heartbeat` verb to memorise. The
reasoning is right and the mechanism does not implement it: `log` is *reporting*,
not working, and the two come apart exactly where it costs.

All three parallel sessions hit it independently, which is why this is a decision
and not a bug.

- One lost its claim three times. Its tasks ran one to three hours against a
  1800-second lease. It found `--ttl 2h` only after losing two.
- One called it the single biggest coordination hazard it met: a task of its own
  involved a push, a three-platform CI run and a merge, well over thirty minutes
  of holding nothing worth logging.
- One watched another agent's claim lapse while that agent was still working, and
  from outside it was indistinguishable from an abandoned task. Only the human
  knew otherwise.

**The shape of the failure is the argument.** After the design is settled there
is often an hour of mechanical fixing -- thirty-four failing fixtures, in one
measured case -- during which there is genuinely nothing to log. `ank log` is
documented as being for discovery, not for completion, and that advice is right.
So the lease lapses precisely during the stretch where nothing interesting is
happening, which is also the stretch where a second agent would most safely take
over. The lock is weakest where the work is least interruptible.

**Why renewal on any of the holder's verbs is safe, and where the guard is.**
The obvious objection is that an agent could then hold a task forever by reading
it. That objection is already answered and the answer is not new machinery:
section 4 has `check` report **repeated claim renewals with no modification to
the scope files** as a possible-hoarding signal. Visibility rather than
restriction is this project's standing choice -- it is how task flooding is
handled, how self-created blockers are handled, and how a criterion set by the
claimer is handled. Reading to hold is a lie somebody wrote down, in a corpus
that already reports it.

**`claim_ttl_default` beside `claim_ttl_max`, and not instead of it.** The cap
stops an agent granting itself a day; the default states what this repository's
work actually looks like. Thirty minutes is shorter than this project's own CI
run, so the shipped default is wrong for the repository that wrote it, and every
agent rediscovers `--ttl` by losing a claim first. A repository that can say so
once says it once.
