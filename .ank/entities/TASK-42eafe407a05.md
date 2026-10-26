---
id: TASK-42eafe407a05
type: task
slug: a-linked-worktree-resolves-its-peers-from-the-ma
title: A linked worktree resolves its peers from the main checkout's root
created: 2026-10-26T13:28:49Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/peer_worktree.rs
blocked_by: [TASK-d1996aa9851d]
done_criteria: |
  Through the binary, in the pair of TASK-d1996aa9851d: ank context src/ in a linked worktree of web nested at web/.worktrees/t2 lists ADR-<id>@api and prints no 'is not a corpus' warning, with no override written; a relative override in corpora.yml resolves from the main checkout's root the same way; an absolute declaration is used as written; ank claim in that worktree is refused while a blocked_by naming a task of api that is not done holds, exactly as in the main checkout. The cases are asserted in crates/ank-cli/tests/peer_worktree.rs through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 1
---

Implements ADR-a14dabe4b259. Measured on 2026-10-26 at e04c0dad: in
`web/.worktrees/t2`, `ank context src/` warned "peer 'api' at ../api is not a
corpus, answered without it" and dropped the binding, because `declared_root`
joins the path to `from.corpus`, the worktree's own root.

The anchor is the main worktree's root: the parent of `git rev-parse
--git-common-dir`, resolved to an absolute path. Look first whether `Repo`
already carries it (the claim machinery reads the common dir for refs). Count the
git processes with GIT_TRACE before and after on the main checkout, where
nothing should change, and record both with `ank log`.

Blocked by TASK-d1996aa9851d only because both rewrite the same resolution in
repo.rs and share the test file.
