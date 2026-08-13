---
id: TASK-f113addd8f40
type: task
slug: a-rebase-detaches-a-commit-proof-and-nothing-say
title: A rebase detaches a commit proof, and nothing says so
created: 2026-08-13T17:46:15Z
author: claude-code/2.1.229+main-checkout
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  check reports a commit proof whose reference resolves to no commit reachable from the corpus, as a signal naming the task and the reference, and never as a fault: an unreachable commit on one clone is a shallow clone or a branch not fetched on another, and the corpus is not what is broken.
  
  The question is asked only of commit proofs, and its cost is one git process per invocation rather than one per proof.
  
  Section 4 of docs/ank-spec-v1.1.md states it in the trust hierarchy, beside what a commit proof guarantees, before the code moves.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs: a task anchored on a commit that is then rebased away reports the signal, one anchored on a live commit reports nothing, and a shallow clone reports nothing rather than accusing every proof it cannot see.
criteria_by: creator
schema: 3
version: 1
---

Found while landing TASK-9bff1d5826b1. It was anchored with
`--proof commit:77aa58c`, which `done` validated against git at the time. The
branch was then rebased onto a newer main -- the routine this project's own guide
prescribes, since CI tests the merge and a stale base goes red where local is
green -- and the rebase replaced that commit with `e5e0d81`. The recorded
reference then resolved only on the stale remote branch, and would resolve
nowhere once that branch was force-pushed.

**Ank validates a `commit:` reference at the moment `done` writes it, and never
again.** That is the strongest proof type the tool checks itself, and a rebase
detaches it silently. The repair used here was `attest` with the surviving sha,
which is the one write allowed after `done`; the dead reference stays, because a
proof list is append-only and removing one would be the rewrite that append-only
exists to prevent. So the record is repairable and nothing points at the need to
repair it.

**A signal and never a fault, and the reason is the same one that has decided
this three times in this corpus.** A commit unreachable here is a shallow clone,
a branch never fetched, or a rebase, on somebody else's machine -- and a check
that reddens over the shape of a clone is a check people learn to ignore
(TASK-03eaa26bddd1, TASK-2ce5554d6ed0). Read the shallow case explicitly rather
than letting it fall out: a depth-1 clone can reach almost nothing, so a naive
implementation would report every commit proof in the corpus at once, which is
the volume failure §4 keeps legislating against.

Do not spend a process per proof. `rev-list --no-walk` takes many references and
answers about all of them, or the set of proofs can be tested against one listing
of what is reachable; either way the cost clause of the walk above applies here
too.

What this deliberately does not do is re-anchor anything automatically. Choosing
which commit now carries the work is a judgement -- the rebase may have split it,
or dropped it -- and §3 is explicit that appending a proof is the only legal
post-`done` write. This reports; the human or the agent decides.
