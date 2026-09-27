---
id: LOG-a56b6a47b1de
type: log
title: "Measured through target/debug/ank in a scratch pair a(peers.bb=../b)/b: new -b TASK-430fc9fe@bb"
created: 2026-09-27T21:09:01Z
author: claude-code/opus-5.5+0861
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/peer_blocked_by.rs
  - crates/ank-core/src/model.rs
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/registry.rs
  - crates/ank-core/src/lib.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
about: TASK-08615a199a6a
seq: 9
schema: 4
version: 1
---

 (prefix) stores blocked_by: [TASK-430fc9fe08d2@bb]; -b TASK-ffffffffffff@bb exits 2 'entity not found in peer bb: TASK-ffffffffffff@bb' -> ank find @bb. graph line: '(blocked by TASK-430fc9fe08d2@bb, open)'. claim while open: exit 7 -> ank show TASK-…@bb. Peer file set to done by hand, then b moved away: claim exit 7 naming peer 'bb' and 'an edge that cannot be read holds', hint the --user override; check signal 'cannot be read, so it holds', exit 0. peers.bb unset: check fault 'an undeclared peer', exit 8; claim exit 7 -> ank config peers.bb <path>. Redeclared: claim exit 0. Peer tree incl .git: 39 files, sha256-of-hashes 44af6de7827e identical from after my own edit to the end across 8 verbs, no index.db. Mutations: removing check_peer_blockers turns the claim test red ('claimed' where refusal expected); removing check's and graph's peer loops turns check/graph tests red. cargo test --workspace: 0 failures. Unreadable peer is a signal, not a fault, because whether a sibling checkout exists is the machine's fact, not the corpus's (CI has none). Left as is: context's claimable listing does not read peer edges; other_ready_task skips any task with a peer edge instead.
