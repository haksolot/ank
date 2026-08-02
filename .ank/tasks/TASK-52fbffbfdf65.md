---
id: TASK-52fbffbfdf65
type: task
slug: check-prunes-a-live-claim-ref-when-it-runs-from
title: check prunes a live claim ref when it runs from an older checkout
created: 2026-08-02T22:31:35Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  A claim ref held on a task that exists on the default branch survives an ank check run from a worktree checked out at a commit predating that task. Proved through the binary, with the ref read by git afterwards.
criteria_by: creator
proof:
  - type: test
    ref: ci://github/30771346358
    criteria: ce710f86f3e8
schema: 2
version: 4
---

Observed three times while reproducing TASK-1ea38a17d854, each time on a claim I was holding. A detached worktree at 7cde6ce, an older commit, then ank check in it: "pruned refs/ank/claims/TASK-1ea38a17d854", and the ref was gone from the shared ref store. The task was in_progress on main; the claim had to be retaken twice before done could run.

maintain treats a ref whose id is absent from statuses as an orphan and deletes it. Its own comment says "a ref for a task that no longer exists anywhere", and that is the gap: statuses is built from the entities of the current working tree, so the code answers "does not exist here" while the comment claims "does not exist anywhere". refs/ank/ is shared across worktrees, so an old checkout deletes claims that a current one is actively holding.

The settled branch below it already knows how to ask the right question: it reads the task at default_branch through git::file_at rather than from disk. The orphan branch could ask the same way before deleting, and report rather than prune when the answer is unreachable -- the reader's behaviour section 2 asks for.

The blast radius is small but it is the coordination plane: a lost claim is a task two agents can hold at once.

## Log
- 2026-08-02T22:58:12Z seanl@sean-laptop — Fixed in maintain: the orphan test now asks the default branch through git::file_at before deleting, the same source and the same call the settled test below it already uses. Ok(Some) means the task exists on the default branch and this checkout is merely older than it -- not an orphan, and silent, because a signal there would fire on every check run from every branch predating the task. Ok(None) prunes as before. Err keeps the ref and reports once through unreachable_branch: unable to ask is not permission to delete. Proved by reverting only human.rs with the test in place: it fails with 'pruned refs/ank/claims/TASK-000000000001', the incident's exact line. The test also carries the other half, or the fix could have been 'never prune': a task whose file is deleted and the deletion committed to the default branch is still collected in the same pass that leaves the live claim alone.
- 2026-08-02T23:03:35Z seanl@sean-laptop — done, proof test:ci://github/30771346358
