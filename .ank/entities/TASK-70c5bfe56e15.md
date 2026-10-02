---
id: TASK-70c5bfe56e15
type: task
slug: a-peer-id-given-without-its-kind-is-refused-nami
title: A peer id given without its kind is refused naming the form that works
created: 2026-10-02T18:08:29Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/peer_ids.rs
blocked_by: []
done_criteria: |
  Through the binary, in a corpus declaring a peer named api that holds exactly one entity whose id starts with ADR-3511: ank show 3511@api and ank new task --blocked-by 3511@api are each refused at the exit code an unresolvable id gets today, and the refusal names ADR-3511@api as the form that works. When the peer holds no entity with that hex prefix, the refusal says a peer id carries its kind, as <KIND>-<hex>@<peer>, and names no entity. ank show ADR-3511@api still resolves as before. A text that is not in an identifier's place is never read as one: on a claimed task, ank log "ship 3511@api" writes an entry whose message is that text.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/59cd47feea81@098a552
    tree: scope/88cf95f2bea5
    criteria: f85e149b8aa8
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@098a552
    tree: scope/88cf95f2bea5
    criteria: f85e149b8aa8
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Found by the worker of TASK-fbe6db79560a while writing docs/peers.md
(LOG-3b4dcabfc60a): `ank show 3511@api` exits 2 with "entity not found:
3511@api", where a bare local prefix resolves.

The refusal is deliberate and stays: `peer_id` in repo.rs reads `<id>@<peer>`
only when the part before the `@` has the shape `KIND-hex`, so that a log
message such as `ask me@home` stays a message. What is wrong is the answer: an
identifier the user got nearly right is called "not found", where #517 set the
rule that a refused id names the form that works. Decided in planning: keep the
kind mandatory, make the refusal name the working form. Accepting a bare hex
prefix was rejected because a verb that also takes a message would have to
guess which one it was handed.
