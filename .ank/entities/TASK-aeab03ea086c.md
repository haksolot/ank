---
id: TASK-aeab03ea086c
type: task
slug: ank-config-writes-roles-and-identities
title: ank config writes roles and identities
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: open
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/config-keys.md
blocked_by: [TASK-94bc2d41eb15]
done_criteria: |
  ank config identities.<identity> <role> writes the mapping and --unset removes it; ank config roles.<name>.can and roles.<name>.cannot read and write a flow sequence the way verifiers.<name>.inputs does. Each refuses a word outside the role vocabulary, and identities refuses a role that roles does not declare; the writer touches nothing else in the file. docs/config-keys.md stops saying these are edited by hand. Tested through the binary (GitHub #784 item 2).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted). GitHub #784 item 2.
