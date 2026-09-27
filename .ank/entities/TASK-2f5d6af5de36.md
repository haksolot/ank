---
id: TASK-2f5d6af5de36
type: task
slug: scope-inside-a-peer-names-the-constraints-bound
title: scope inside a peer names the constraints bound to it from elsewhere, and help says which end declares
created: 2026-09-27T16:44:23Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/peer_scope_reverse.rs
blocked_by: [TASK-c666eb306102]
done_criteria: |
  Through the binary, with corpus A declaring peer bb and an ADR in A scoped bb:**, and corpus B declaring A as peer aa: ank scope README.md inside B lists that ADR as ank context README.md inside B already serves it, with the same <id>@aa form, and an ADR in A scoped to another peer only is listed by neither. ank help config states that a binding across corpora needs both declarations, the governing corpus naming the governed one in its scope and the governed corpus declaring the governing one to read it, and ank help scope says scope reads peers as context does. A test in crates/ank-cli/tests/peer_scope_reverse.rs asserts each case through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Issue #500, the part that is a defect. Measured on 96c7fd7, and it
contradicts the issue in part: once B declares `peers.aa ../a`, `ank context
README.md` inside B does serve A's ADR scoped `bb:**`, as `ADR-…@aa`, and does
not serve one scoped `cc:**`. That is TASK-13e802e46050's criterion, and it
holds. What does not hold: `ank scope README.md` inside B, same setup, lists
nothing from A. Two verbs answer differently about what binds one path.

The rest of the issue comes from a direction nobody documented. The issue
declared the peer only in the governed repository, while a binding is resolved
at both ends: B opens A through B's declaration, and A's `bb:**` is resolved
through A's declaration back to B (`Peer::binds`). `ank help config` lists
`peers.<name>` and says nothing more. The criterion requires the help to say it.

Waits on TASK-c666eb306102 because both edit the matcher in context.rs and the
scope verb in commands.rs. Path portability is ADR-da2819aef598, a separate
task.
