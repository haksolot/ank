---
id: TASK-cc1a2fb317f3
type: task
slug: ank-new-adr-can-declare-what-it-replaces
title: ank new adr can declare what it replaces
created: 2026-07-31T22:59:57Z
status: open
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
blocked_by: []
done_criteria: |
  ank new adr accepts the identifier of the ADR it replaces and writes it into the supersedes field, resolved at creation so an unknown reference is refused there rather than surfacing later in check. The flag applies to an ADR and is refused on a task, which has no such field. No verb is added: this is a flag on new. A test invokes the binary and reads the file, because what is asserted is what lands on disk.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 1
version: 1
---

commands.rs writes supersedes: None unconditionally, so an ADR that replaces another cannot be created through the CLI at all. The field exists in the model, check enforces the chain in both directions, and accept is meant to complete it -- everything is built around a value nothing can write.

Same family as --verify and --body, and the same resolution: resolved at creation, exactly as --blocked-by is a few lines above in the same function. A reference matching nothing would otherwise surface in check, as a corpus fault nobody can attribute to the act that caused it.

Refused on a task rather than dropped. A task has no supersedes field, and a flag silently ignored teaches the caller it worked.
