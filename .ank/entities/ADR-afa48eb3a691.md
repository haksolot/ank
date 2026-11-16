---
id: ADR-afa48eb3a691
type: adr
slug: a-config-write-is-warned-at-the-verb-and-never-s
title: A config write is warned at the verb and never signalled by check, since nothing records its actor
created: 2026-11-16T12:56:46Z
author: claude-code/plan
status: proposed
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
constraint: |
  check signals, as the kind role, only the acts the corpus records with their actor: an accept, a close, a constraint edit, a new entity. A write to roles, identities, permissions, check or ratify through ank config is warned at the verb under warn or deny, and refused by the hook under deny, and check does not signal it, because ADR-e64dfaafd578 keeps the writer from recording anything beside config.yml and no other record carries who wrote it.
amends: [ADR-ddbe024ca7f2]
schema: 4
version: 3
---

## Context

ADR-ddbe024ca7f2 lists a write to roles, identities, permissions, check or ratify among the recorded acts check signals. Implementing it (TASK-dd152fb654c9) measured that no such record exists: ank config writes config.yml and nothing else, as ADR-e64dfaafd578 requires, so check has no actor to judge. The commit author is not an actor ank records, and reading it would judge git identity rather than ANK_AGENT.

## Decision

Narrow the check half to the acts the corpus records. A config write is still warned at the verb and still refused by the hook under deny, which is where the refusal of a permission change matters. A change to the permission block stays a diff in review.

## Consequences

A forbidden permission change made under warn leaves a warning on stderr and no finding. Recording it would need a decision that lets ank config write a record, which ADR-e64dfaafd578 forbids today.
