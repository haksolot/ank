---
id: LOG-e8ade0ce2356
type: log
title: "Measured GIT_TRACE, ank context src/ in web, peer api holding 40 accepted ADRs binding web:src/**:"
created: 2026-11-02T13:37:43Z
author: claude-code/opus-5.5+task-a836-peer-default-branch
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/tests/peer_default_branch.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/peer_blocked_by.rs
  - crates/ank-cli/tests/peer_claimable.rs
  - crates/ank-cli/tests/peer_scope_reverse.rs
  - crates/ank-cli/tests/main.rs
about: TASK-a8362b510188
seq: 5
schema: 4
version: 1
---

 main's binary 4 git processes (for-each-ref, rev-parse, symbolic-ref, version); this branch 6 cold (+1 for-each-ref resolving refs/heads/main and refs/remotes/origin/main in one call, +1 cat-file --batch walking the whole .ank tree interactively), 5 warm (reading cached by commit under the reader's git dir/ank/peers/<name>/<commit>, no cat-file). Constant in entity count: one process per peer, never per file (ADR-cc65). Script: measure.sh in the worker scratchpad.
