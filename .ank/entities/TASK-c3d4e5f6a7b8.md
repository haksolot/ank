---
id: TASK-c3d4e5f6a7b8
type: task
slug: claims-on-git-refs
title: Claims on git refs, TTL, re-acquisition and completion refs
created: 2026-07-27T09:30:00Z
status: done
scope:
  - crates/ank-cli/src/claim.rs
blocked_by: [TASK-a1b2c3d4e5f6, TASK-244a842bc0cc, TASK-c8637488773c, TASK-038c83ba44c5]
done_criteria: |
  A claim is recorded through refs/ank/claims/<id>, without the holder or the
  expiry ever appearing in a task file (the open -> in_progress move,
  however, is expected), two concurrent claims on the same task fail with
  code 4, expiry makes the task claimable again, and the original holder
  re-acquires silently if nobody took it over. The ref carries two states,
  distinguished by the record read and never by the address: claim (holder,
  expiry, hash of the frozen done_criteria, hash of the applicable
  constraints) and completed (HEAD commit, branch, identity, timestamp).
  release and close delete the ref; the move to completed neither deletes it
  nor carries a TTL. A ref holding a completed record makes claim fail with
  code 4, with a message naming the commit and the branch and distinct from
  the one for a claim held by another agent. The module exposes pruning — the
  task appears done or closed on the default branch — without ever calling it
  itself. A record in an unknown state is a named error, never a silent
  fallback to the other state.
criteria_by: creator
verify: [cargo-test]
proof:
  - type: commit
    ref: 92b1a5a
schema: 3
version: 6
---

Also carries the hash of the frozen done_criteria and that of the constraints
applicable at claim time.

Amended by ADR-bcf222a31525, which turns the claim ref into a completion ref at
`done` instead of deleting it. The task was not claimed, so the criterion is
amended with no freeze to lift. What is added is the ref's second state and its
pruning predicate; what does not change is the address — `refs/ank/claims/<id>`,
one ref per task (ADR-4e7c25b1f639).

`blocked_by` gains TASK-038c83ba44c5: writing a `completed` record requires
reading the current branch, and pruning it requires reading a file as it appears
on the default branch. Both primitives are git plumbing and live in `git.rs`,
outside this scope.

Pruning is exposed here but called by `check` (TASK-a7b8c9d0e1f2): a reader does
not sanitise the coordination plane underneath everyone else, and concentrating
maintenance in a single command is what makes its timing predictable.

The criterion sits at the module's API level rather than the binary's, because
`done.rs` is out of scope and comes later. It is TASK-e5f6a7b8c9d0 that checks
through the binary that an `ank done` does leave a completion ref.
