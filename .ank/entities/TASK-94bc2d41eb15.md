---
id: TASK-94bc2d41eb15
type: task
slug: permissions-enforce-is-a-config-key-the-role-voc
title: permissions.enforce is a config key, the role vocabulary is published and checked, and init opts a new corpus into warn
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/init_bootstrap.rs
  - docs/config-keys.md
  - crates/ank-core/src/reference.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/guard_config.rs
  - crates/ank-cli/tests/golden-json/config-list.json
blocked_by: [TASK-fa89fb00791f]
done_criteria: |
  ank config permissions.enforce reads, and writes, off, warn or deny, refuses any other value, and reads off when the key is absent. docs/config-keys.md lists the fixed vocabulary of roles.<name>.can and .cannot, config:permissions included, and ank check faults .ank/config.yml when a can or cannot entry is a word outside it. ank init, on the run that creates .ank/, writes permissions.enforce: warn and maps the identity running it to human under identities. Tested through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/0ec03747a5af@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/4f4e89b3f55d@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/bbd937544878@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/4e18be279013@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/7fd555475ccb@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/b9ef185d0425@ec09464
    tree: scope/ab592c97ce7e
    criteria: 30be8d0d87a9
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 7
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted).
