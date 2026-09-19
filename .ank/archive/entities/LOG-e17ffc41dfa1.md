---
id: LOG-e17ffc41dfa1
type: log
title: The criterion's 250ms is met on Linux (84-156ms) and macOS, and is not met on Windows (376ms). The
created: 2026-08-28T19:16:32Z
author: claude-code/opus-5+status-counts-cheap
scope:
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/status.rs
about: TASK-be17972988d9
seq: 3
schema: 4
version: 1
---

 shortfall is not the corpus and cannot be moved from this task's perimeter.

Counted with a git shim on the PATH: ank status --json spawns thirteen git processes. Two 'git --version', two 'rev-parse --path-format=absolute --git-common-dir', two 'rev-parse --show-toplevel', three 'for-each-ref refs/ank/' (claim::on_task, context::plane, and the enumeration this change needs for its key), and one each of 'symbolic-ref HEAD', 'symbolic-ref refs/remotes/origin/HEAD', 'rev-parse main^{commit}' and 'rev-list --max-parents=0 --reverse HEAD'. Process creation is about 25ms on a Windows runner against 2-3ms on Linux, so those thirteen are roughly 325ms before the verb reads anything.

Measured on the CI runners through the binary, floor of nine runs over a thousand entities: status 376ms / index-opening verb 231ms on Windows, 156/111 on Linux, 84/73 on macOS. Remove the corpus read entirely and Windows would still be at the wall.

Of the thirteen, this change owns two, and eleven live in git.rs, repo.rs, claim.rs and context.rs. TASK-5690eae1e008 carries them. What the ADR asserts -- that the summary is not a re-derivation -- is proved on all three platforms by the ratio the same test asserts beside the wall: status against check, 25x on this fixture.
