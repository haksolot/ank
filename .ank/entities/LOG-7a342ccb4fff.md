---
id: LOG-7a342ccb4fff
type: log
title: "Reproduced through the binary (peer api holding ADR-3511aabbccdd): ank show 3511@api -> exit 2"
created: 2026-10-02T18:30:08Z
author: claude-code/opus-5.5+peer-id-refusal
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/peer_ids.rs
about: TASK-70c5bfe56e15
seq: 1
schema: 4
version: 1
---

 'entity not found: 3511@api', hint 'ank find 3511@api'; show 9999@api same; new task --blocked-by 3511@api same exit 2; show ADR-3511@api exit 0. The message lacks 'in peer', so it is the local Store::NotFound path, not Peer::refusal: peer_id() rejects '3511' (no KIND-) and reach() returns Here(raw).
