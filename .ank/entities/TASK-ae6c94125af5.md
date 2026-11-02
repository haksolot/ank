---
id: TASK-ae6c94125af5
type: task
slug: status-find-and-context-take-peers-and-add-each
title: status, find and context take --peers and add each declared peer's answer, claims included
created: 2026-10-26T13:30:20Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/peer_aggregate.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/main.rs
blocked_by: [TASK-a8362b510188, TASK-014bbfca8fb6]
done_criteria: |
  Through the binary, in a scratch trio web (peers.api ../api, peers.db ../db), api and db, each holding one open task and one live claim by a distinct identity: ank find --type task --peers in web lists the tasks of all three, those of a peer addressed <id>@<peer>; ank status --peers in web names the live claims of all three with the repository each lives in, and its constraint count includes the peer ADRs that bind web; ank context --peers src/ in web lists the ready tasks of web and of each peer; with db moved away the three verbs answer for web and api and warn once naming db; without --peers each verb prints exactly what it prints today; no file and no ref under api or db is modified; the --json form of each carries the peer of every row under a key the contract table declares. A test in crates/ank-cli/tests/peer_aggregate.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@52aa854
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/546558753fcb@5063aa2
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-971e-half-declared
  - type: test
    ref: local/28ea993c4812@5cb8ca7
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ae6c-peers-aggregate
  - type: test
    ref: local/40992441fdb8@5cb8ca7
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-ae6c-peers-aggregate
  - type: test
    ref: local/7ad8a0a678f7@52aa854
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/1ce807c613bc@52aa854
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/f2adf9838ea0@52aa854
    tree: scope/249fd3af23d5
    criteria: 9ee319ecb9ca
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Implements the read half of ADR-376bed7fa39a, which must be accepted first: the
surface is frozen and a flag before its supersession is the thing ADR-24ef
forbids.

Measured on 2026-10-26 at e04c0dad in the pair api/web: `ank status` in web
said "1 constraint(s)" while a peer ADR also bound it, and its `elsewhere` listed
web's claims only; `ank find --type task --all` listed web's tasks only.

**status has a cost rule.** ADR-f3d1 says status does not read the corpus to
count what it reports, and a peer has no index this verb may write
(LOG-879c2ce798a1: Index::open writes index.db into the corpus it opens). Count
a peer's entities from the tree at its default branch (one ls-tree) and its
claims with one for-each-ref over its refs/ank/claims, and count the git
processes of `status --peers` with GIT_TRACE: record the number per peer with
`ank log`.

A peer's claims are shown and never arbitrated: claims stay per repository
(ADR-a1de673043b4, ADR-ed3e14d0f991). A peer is read at its default branch
(TASK-a8362b510188).
