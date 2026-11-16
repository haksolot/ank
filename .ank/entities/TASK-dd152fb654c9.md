---
id: TASK-dd152fb654c9
type: task
slug: a-forbidden-act-is-warned-at-the-verb-and-signal
title: A forbidden act is warned at the verb and signalled by check as role, and self-ratified goes quiet when declared
created: 2026-11-16T08:08:02Z
author: claude-code/plan
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/check.md
  - docs/config-keys.md
blocked_by: [TASK-aeab03ea086c]
done_criteria: |
  With permissions.enforce warn or deny, an identity whose role forbids the act runs accept, close, a constraint edit, ank new of each kind, or an ank config write to roles, identities, permissions, check or ratify: the act is performed, the verb exits as it would have, and stderr names the role and the entry that forbid it. ank check then reports each such recorded act as a signal of kind role naming the entity, the actor and the entry; with enforce off it reports none and no verb warns. self-ratified is not reported for an entity whose ratifier's role allows adr:accept. Each case tested through the binary (GitHub #785).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Implements ADR-ddbe024ca7f2, which is proposed: do not claim before a human has ratified it (ank show ADR-ddbe024ca7f2 says accepted). GitHub #785.
