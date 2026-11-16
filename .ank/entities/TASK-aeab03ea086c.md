---
id: TASK-aeab03ea086c
type: task
slug: ank-config-writes-roles-and-identities
title: ank config writes roles and identities
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/config-keys.md
  - crates/ank-core/src/reference.rs
blocked_by: [TASK-94bc2d41eb15]
done_criteria: |
  ank config identities.<identity> <role> writes the mapping and --unset removes it; ank config roles.<name>.can and roles.<name>.cannot read and write a flow sequence the way verifiers.<name>.inputs does. Each refuses a word outside the role vocabulary, and identities refuses a role that roles does not declare; the writer touches nothing else in the file. docs/config-keys.md stops saying these are edited by hand. Tested through the binary (GitHub #784 item 2).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/07eff8d5d1c6@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/4f91bd369573@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/bbd937544878@ec09464
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-94bc-perm
  - type: test
    ref: local/a2e60ab3f059@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/5c0c45d61852@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/c0fe7c3cbb7c@e7e4ede
    tree: scope/f3648d8b4d02
    criteria: c0981d4b03b8
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted). GitHub #784 item 2.
