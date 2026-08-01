---
id: TASK-cc1a2fb317f3
type: task
slug: ank-new-adr-can-declare-what-it-replaces
title: ank new adr can declare what it replaces
created: 2026-07-31T22:59:57Z
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
blocked_by: []
done_criteria: |
  ank new adr accepts the identifier of the ADR it replaces and writes it into the supersedes field, resolved at creation so an unknown reference is refused there rather than surfacing later in check. The flag applies to an ADR and is refused on a task, which has no such field. No verb is added: this is a flag on new. A test invokes the binary and reads the file, because what is asserted is what lands on disk.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/d6c40dd8dfc2@47ddf47
    tree: scope/c2322106b970
    criteria: 3c401b957d27
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@47ddf47
    tree: scope/c2322106b970
    criteria: 3c401b957d27
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/85c794fa7c89@47ddf47
    tree: scope/c2322106b970
    criteria: 3c401b957d27
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 4
---

commands.rs writes supersedes: None unconditionally, so an ADR that replaces another cannot be created through the CLI at all. The field exists in the model, check enforces the chain in both directions, and accept is meant to complete it -- everything is built around a value nothing can write.

Same family as --verify and --body, and the same resolution: resolved at creation, exactly as --blocked-by is a few lines above in the same function. A reference matching nothing would otherwise surface in check, as a corpus fault nobody can attribute to the act that caused it.

Refused on a task rather than dropped. A task has no supersedes field, and a flag silently ignored teaches the caller it worked.

## Log
- 2026-08-01T01:54:11Z seanl@sean-laptop — A flag on new, not a verb, so ADR-2f8a is untouched. Resolved at creation with store.resolve, the same doctrine as --blocked-by fifteen lines above: a reference matching nothing would otherwise surface in check as a corpus fault nobody can attribute to the act that caused it. The kind is checked too -- store.resolve happily resolves a TASK prefix, and an ADR superseding a task is not a chain accept or check can make sense of, so that is a 1 naming what it found. Refused on a task rather than dropped, for the reason --verify is refused on an ADR: a task has no such field and a flag silently ignored teaches the caller it worked; the hint points at --blocked-by, which is what the caller probably wanted. The binary test reads the file rather than the parsed entity, because a resolution that never reached the serializer would pass on the model and leave the field absent on disk -- and the field's whole problem was that nothing wrote it.
- 2026-08-01T01:54:40Z seanl@sean-laptop — done, proof test:local/d6c40dd8dfc2@47ddf47 test:local/e3b0c44298fc@47ddf47 test:local/85c794fa7c89@47ddf47
