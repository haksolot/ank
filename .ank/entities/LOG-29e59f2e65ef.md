---
id: LOG-29e59f2e65ef
type: log
title: "Fix in repo::reach: kindless() refuses <hex>@<declared peer> at exit 2 (NotFound, the code 'show"
created: 2026-10-02T18:31:57Z
author: claude-code/opus-5.5+peer-id-refusal
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/peer_ids.rs
about: TASK-70c5bfe56e15
seq: 2
schema: 4
version: 1
---

 TASK-ffffffffffff' gets). Through the binary: show 3511@api -> exit 2 'peer api holds ADR-3511@api', hint 'ank show ADR-3511@api'; new task --blocked-by 3511@api -> exit 2, hint ADR-3511@api; show 9999@api -> exit 2 'as <KIND>-<hex>@api', no entity named; show ADR-3511@api exit 0. log uses reach_or_message, which skips the kindless check, so log never guesses. Tests peer_ids: 2 new tests red before the fix (12 run, 2 failed), 12/12 green after; the log-message test was green before too, it guards the non-regression.
