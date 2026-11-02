---
id: TASK-42eafe407a05
type: task
slug: a-linked-worktree-resolves-its-peers-from-the-ma
title: A linked worktree resolves its peers from the main checkout's root
created: 2026-10-26T13:28:49Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/peer_worktree.rs
blocked_by: [TASK-d1996aa9851d]
done_criteria: |
  Through the binary, in the pair of TASK-d1996aa9851d: ank context src/ in a linked worktree of web nested at web/.worktrees/t2 lists ADR-<id>@api and prints no 'is not a corpus' warning, with no override written; a relative override in corpora.yml resolves from the main checkout's root the same way; an absolute declaration is used as written; ank claim in that worktree is refused while a blocked_by naming a task of api that is not done holds, exactly as in the main checkout. The cases are asserted in crates/ank-cli/tests/peer_worktree.rs through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@e90039f
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/8113816f1be4@0a0437a
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-d199-peer-worktree
  - type: test
    ref: local/d77826d726e4@0a0437a
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-d199-peer-worktree
  - type: test
    ref: local/fd890b9ed4f8@0a0437a
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-d199-peer-worktree
  - type: test
    ref: local/bb4fa8ae1c4c@e90039f
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/2582389207de@e90039f
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/2c9a815c8aff@e90039f
    tree: scope/bc4ae702d6b0
    criteria: e1fb4f2326af
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
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
