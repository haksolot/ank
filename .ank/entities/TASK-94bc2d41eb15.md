---
id: TASK-94bc2d41eb15
type: task
slug: permissions-enforce-is-a-config-key-the-role-voc
title: permissions.enforce is a config key, the role vocabulary is published and checked, and init opts a new corpus into warn
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: open
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/init_bootstrap.rs
  - docs/config-keys.md
blocked_by: [TASK-fa89fb00791f]
done_criteria: |
  ank config permissions.enforce reads, and writes, off, warn or deny, refuses any other value, and reads off when the key is absent. docs/config-keys.md lists the fixed vocabulary of roles.<name>.can and .cannot, config:permissions included, and ank check faults .ank/config.yml when a can or cannot entry is a word outside it. ank init, on the run that creates .ank/, writes permissions.enforce: warn and maps the identity running it to human under identities. Tested through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted).
