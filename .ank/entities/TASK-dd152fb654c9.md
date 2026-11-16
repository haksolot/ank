---
id: TASK-dd152fb654c9
type: task
slug: a-forbidden-act-is-warned-at-the-verb-and-signal
title: A forbidden act is warned at the verb and signalled by check as role, and self-ratified goes quiet when declared
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/check.md
  - docs/config-keys.md
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/git.rs
  - docs/quickstart.md
blocked_by: [TASK-aeab03ea086c]
done_criteria: |
  With permissions.enforce warn or deny, an identity whose role forbids the act runs accept, close, a constraint edit, ank new of each kind, or an ank config write to roles, identities, permissions, check or ratify: the act is performed, the verb exits as it would have, and stderr names the role and the entry that forbid it. ank check then reports each such recorded act as a signal of kind role naming the entity, the actor and the entry; with enforce off it reports none and no verb warns. self-ratified is not reported for an entity whose ratifier's role allows adr:accept. Each case tested through the binary (GitHub #785).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/06879ab18f86@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/95a8fa458f3c@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/bbd937544878@ec09464
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-94bc-perm
  - type: test
    ref: local/5aebbb68b300@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/ef0311bea905@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/d6fcb5d3e82a@52b3f2c
    tree: scope/80f9c88803f6
    criteria: 349384a79357
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 5
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted). GitHub #785.
