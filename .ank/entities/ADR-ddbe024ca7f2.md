---
id: ADR-ddbe024ca7f2
type: adr
slug: a-role-is-read-an-act-it-forbids-is-warned-at-th
title: "A role is read: an act it forbids is warned at the verb, signalled by check, and refused only by the hook"
created: 2026-11-16T08:07:26Z
author: claude-code/plan
status: accepted
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-contract/src/env.rs
  - docs/config-keys.md
  - SECURITY.md
constraint: |
  permissions.enforce in .ank/config.yml, off, warn or deny, absent meaning off, sets how roles apply, and ank init writes warn with the identity that runs it mapped to human. The words of roles.<name>.can and .cannot are a fixed vocabulary docs/config-keys.md publishes, config:permissions among them, and a word outside it is a fault. An act is forbidden to an identity when its role lists it under cannot, or has a can list that is not ["*"] and omits it; an identity absent from identities has the role agent. Under warn or deny the verb performing a forbidden act still performs it, names on stderr the role and the entry that forbid it, and exits as it would have: the CLI never refuses on a role. Under warn or deny check signals, as the kind role, each recorded act (an accept, a close, a constraint edit, a new entity, a write to roles, identities, permissions, check or ratify) whose actor's role forbade it, and does not signal self-ratified when the ratifier's role allows adr:accept. Under deny, ank hooks run <agent> pre-tool-use refuses a shell tool call that runs an ank verb the identity's role forbids, naming the entry and the command a human runs instead. ANK_PERMISSIONS_ENFORCE, set by whoever launches the agent, may tighten the declared level and never loosen it.
amends: [ADR-43cbb20dcbf1]
ratified: 2057ee33e73e
anchor: 241ce7936d83480ececc38ec2746c2e32e86c250
verified:
  - by: claude-code/opus-5.5
    at: 2026-11-16T11:23:19Z
schema: 4
version: 2
---

## Context

GitHub #785 and #791. Roles are parsed and never read: an identity mapped to agent, whose role lists adr:accept and task:close under cannot, does both at exit 0, and check says nothing. SPEC-ff885356f1f9 section 8 promises that roles are advisory and that check can say when the record and the behaviour disagree; the second half was never built.

## Decision

Keep the rule of section 8 that the CLI never refuses on a role, since a refusal derived from ANK_AGENT is lifted by exporting another string. Build the noticing it promises: a warning at the verb, a role signal in check. Put the refusal where it holds, in the harness hook, under a level the corpus declares and the launcher may only tighten, which is the shape ADR-43cbb20dcbf1 gives guard.corpus and guard.edit. This amends that decision with a third guard.

Changing permissions is itself an act of the vocabulary, config:permissions, absent from the default agent role, and written through ank config (no new verb, ADR-376b). A repository that wants agents to ratify on their own declares adr:accept in their can, in a reviewed diff, and self-ratified stops firing because nothing is owed.

Absent means off, so no existing corpus changes behaviour when this lands; init opts a new corpus into warn and maps whoever initialises it to human, so a maintainer is not warned on their own accept.

## Consequences

Still not a security boundary: an agent can edit config.yml directly or export another identity. The claim is narrower: what the repository forbids is refused through the guarded harness and noticed everywhere else.
