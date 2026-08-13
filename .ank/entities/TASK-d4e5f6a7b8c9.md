---
id: TASK-d4e5f6a7b8c9
type: task
slug: two-phase-context
title: Two-phase context, orientation and execution
created: 2026-07-27T09:35:00Z
status: done
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-b2c3d4e5f6a7, TASK-c3d4e5f6a7b8]
done_criteria: |
  Without a claim, the output lists active constraints, proposals and open
  tasks in the deterministic order of the specification; with a claim, it
  switches to the task alone without ever truncating a constraint; having no
  ready task exits 0 with an explicit message. A task carrying a completion
  ref is never presented as ready: it is marked as finished on another
  branch, and is not counted among the ready tasks in the end-of-loop
  message. An indeterminable default branch does not interrupt context: the
  output stays complete, completion refs are kept and displayed, and a
  one-line warning appears exactly once.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: commit
    ref: 6ec4f70
schema: 3
version: 6
---

The command agents read most: the budget and the cutting order are the parts that
matter.

Amended by ADR-bcf222a31525. The ADR did not say so explicitly, but the mechanism
is pointless without this half: if `context` keeps announcing as ready a task
that `claim` will refuse with code 4, the agent learns the information at the
cost of a round trip, and the "context then claim" loop starts failing half the
time on a repository with several branches. `claim`'s refusal is the safety net;
the display is what avoids needing it.

The degradation on an indeterminable default branch follows §2: `context` is a
reader, and a reader does not stop because maintenance is impossible.

The scope gains `cli.rs` and `tests/cli.rs`, criterion untouched, for the two
reasons TASK-45d18f45de2c set the pattern for. A verb adds its own arm to
`dispatch` when it lands, which is what the note in `main.rs` says. And "exits 0
with an explicit message" is a statement about the process: an exit code only
exists once there is one, so it is tested by spawning the binary.

**A gap in §5's example, recorded rather than invented around.** The orientation
output shows a proposal as `ADR-19d0  [pi@host-2] Prefer idempotent migrations`,
but the ADR format carries no author: the fields are `id`, `slug`, `title`,
`created`, `status`, `scope`, `constraint`, `see`, `supersedes`, `ratified`.
There is nothing to print in those brackets. `git log` would answer it and is
porcelain, forbidden by ADR-b8884edcebe3. Proposals are therefore listed without
an identity, which the criterion does not ask for; adding an author to the
format is a specification change and belongs in its own task if it is wanted.

Short identifiers are displayed, as §3 requires and the §5 example shows
(`TASK-8f3a`). The length is the shortest that stays unambiguous across the
corpus, minimum four — a fixed four would print a prefix that `claim` then
refuses as ambiguous, which is the one thing worse than a long id.
