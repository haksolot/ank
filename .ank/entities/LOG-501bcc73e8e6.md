---
id: LOG-501bcc73e8e6
type: log
title: "Reproduced in scratch pair a(peers.bb=../b)/b with target/debug/ank at 6c99e7f7: task L blocked_by"
created: 2026-10-02T17:31:42Z
author: claude-code/opus-5.5+peer-claimable
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/peer_claimable.rs
about: TASK-d28565b8ed63
seq: 1
schema: 4
version: 1
---

 TASK-4e2a@bb (open in b). context --json gives L ready:true and the text lists it in the ready block; claim L exits 7 'is blocked by TASK-4e2a84b4724b@bb'. Same ready:true with b's status rewritten done, and with b moved away (no warning printed: the peer is never opened). Hint: P held by 'other', claim P as t -> exit 4, hint 'ank context', with L the only other open task in scope; with b's TASK done the hint is still 'ank context', so L is never offered. Hypothesis: build_orientation counts blockers_left over r.blocked_by only, and other_ready_task drops any row whose peer_blocked_by is non-empty; neither calls the peer resolution check_peer_blockers uses.
