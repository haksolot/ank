---
id: TASK-1c09ca53ebce
type: task
slug: a-blocked-by-may-name-a-peer-task-not-yet-on-the
title: A blocked_by may name a peer task not yet on the peer's default branch, and it holds as a signal
created: 2026-10-26T13:29:33Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/peer_default_branch.rs
blocked_by: [TASK-a8362b510188]
done_criteria: |
  Through the binary, in a scratch pair api/web with both declarations: ank new task --blocked-by TASK-<id>@api in web, where that task exists only on a non-default local branch of api, exits 0 and warns naming the branch; the same command naming a task api holds on no branch refuses at exit 2; ank claim of the new task in web refuses at exit 7 naming the peer task; ank check in web reports the edge as a signal and exits 0, with api checked out on that branch and with api checked out on its default branch; once the branch is merged into api's default branch and the task is done there, check reports nothing about the edge and claim succeeds. A test in crates/ank-cli/tests/peer_default_branch.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 1
---

Implements the unmerged-reference half of ADR-09b985a5627e. Measured on
2026-10-26 at e04c0dad: `ank check` in `web` faulted at exit 8, "blocked_by
names TASK-99e0f7cba34e@api, a task peer 'api' does not hold" (human.rs, the
peer-edge arm of check), while api sat on `main` and the task lived on its
branch `work`. With api moved away the same edge was a signal at exit 0. The
state of another repository decided whether this one's check was red.

The orchestrator's case is the pair planned before either lands. Searching the
peer's other local branches for the id at `new` is one `git for-each-ref` plus a
lookup per branch; count it with GIT_TRACE and log the number. It is paid only
when the default branch does not hold the id, which is the rare path.

A task the peer holds on no branch keeps exit 2: that is the mistyped id, and the
eager resolution of ADR-c23bef1cc93e exists for it.
