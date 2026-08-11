---
id: ADR-6d8736c04cfa
type: adr
slug: close-leaves-no-completion-ref-and-the-asymmetry
title: close leaves no completion ref, and the asymmetry with done is the decision
created: 2026-08-11T18:45:19Z
author: claude-code@nested-pebble
status: proposed
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/git.rs
  - docs/**
constraint: |
  The claim ref is not deleted at done: it becomes a completion ref pointing
  at the HEAD commit of the done, with no TTL, and is pruned only once the
  task appears done or closed on the default branch. claim refuses a task
  carrying a completion ref (code 4), naming the commit and the branch.
  accept refuses to run outside the default branch, with no way around it.
  
  close leaves no completion ref. It deletes the claim ref, revoking any live
  claim in the same operation, and a task closed on an unmerged branch is
  claimable everywhere else until the closure lands on the default branch. The
  asymmetry with done is the decision and not an omission: done earns a
  repository-wide refusal with a frozen criterion, declared verifiers and a
  proof, and close is gated by a reason alone.
supersedes: ADR-bcf222a31525
schema: 2
version: 2
---

## Context

ADR-bcf222a31525 created the completion ref, and named this gap without settling
it. Its own words, under "What this ADR does not do":

> It does not give `close` a completion ref. A task closed on an unmerged branch
> carries the same invisibility window as a finished one, but `close` is a human
> act and a rare one, and the pruning predicate already accepts `closed` — the
> gap, if it shows up, can be fixed without touching the format.

That deferral has outlived its premise. ADR-e17e1bbd93ff dissolved the human
side: every verb is available to every caller, and section 3 says `close` is
outside the loop SKILL.md teaches "rather than closed to agents". `close` is not
a human act, because there is no such category left. The sentence has to be
replaced rather than left standing, since it is the only reason the corpus
records for an asymmetry between the two terminal transitions.

Level 1 also shipped since (TASK-82c3341502c1): claim refs are pushed, so the
deletion `close` performs now travels to every clone rather than staying in one.
What was a local asymmetry is now a repository-wide one, which is the scale the
decision has to be right at.

## Decision

The asymmetry is the decision. `close` deletes the claim ref and leaves nothing
behind; `done` leaves a completion record. Four reasons, in the order they
matter.

**`done` earns the repository-wide refusal and `close` does not.** A completion
record makes every other `claim` fail with code 4, and since level 1 that
refusal holds across the whole repository. `done` reaches it through a criterion
frozen at claim, the verifiers the task declares, and a proof recorded in the
file. `close` reaches its transition through a string. Granting the same power
to the cheaper act would make an unproven decision, taken on a branch nobody has
reviewed, binding on everyone.

**`close` revokes somebody else's live claim** (section 3): the ref is deleted
whoever holds it, and the holder learns of it at its next `log`. If `close` also
wrote a record that refused every subsequent `claim`, one agent could take a task
away from another *and* prevent anyone from picking it up, unilaterally and
without merging anything. The revocation is defensible precisely because what
follows it is a free task.

**The two errors are not the same size.** A `done` invisible elsewhere costs
work that was already performed — the expensive, unrecoverable waste the
completion ref was built for. A `close` invisible elsewhere costs work on a task
somebody proposed to abandon, and if that work completes it produces a proof,
which is evidence against the closure rather than a loss. The failure modes are
asymmetric in the same direction as the acts.

**Symmetry would cost the format, which ADR-bcf222a31525 hoped to avoid.**
`claim`'s refusal reads the record and says "finished on another branch". Saying
"closed on another branch" needs a discriminator the completion record does not
carry, and adding one is a format change: the specification first, then the
goldens, then the code (ADR-63b59c5c26f7). A record that lied about which
transition produced it would be worse than the gap.

## Consequences

None on the code, which already behaves this way: `close` calls `claim::delete`,
and the pruning predicate already accepts `closed` alongside `done`. What
changes is that the behaviour is now a rule with its reasons attached, and an
integration test asserts it through the binary from a second checkout, where
before it was an untested consequence of an unstated choice.

The window itself is not closed and is not claimed to be. A task closed on an
unmerged branch reads `open` everywhere else until the closure lands on the
default branch. That is accepted, at the price named above.

## Alternatives rejected

**A completion record for `close`.** The symmetry is superficially attractive and
is answered in full above: it hands the cheaper act the more binding power, and
it costs the format.

**A third record state, neither claim nor completion.** It would let `claim`
refuse accurately and separately, and it is a format change with a migration for
one rare case — the same price as the alternative above, paid for a narrower
benefit.

**Leaving the question open.** That is the state this supersedes. The code read
as an omission rather than as a choice, and the only reason on record had been
retired by a later decision without anyone noticing (TASK-78326e2e3e89).
