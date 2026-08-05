---
id: ADR-c656cbcc33a9
type: adr
slug: the-help-is-a-flat-listing-and-the-loop-is-taugh
title: The help is a flat listing, and the loop is taught rather than printed
created: 2026-08-01T23:07:04Z
author: seanl@sean-laptop
status: superseded
scope:
  - crates/ank-cli/**
  - skill/**
  - docs/ank-spec-v1.1.md
constraint: |
  The CLI exposes one surface: every verb is available to every caller, and the CLI refuses on state, never on identity. The only hard authority line is the signed ratification commit produced by accept. Who uses which verb is policy, and policy lives above the binary: SKILL.md documents the loop for agents and its content is frozen, harness hooks enforce where enforcement is real, roles in config.yml remain advisory. ank help is one flat listing of every verb, in the order of section 4, with no headings and no grouping: the loop is what SKILL.md teaches, not what help prints.
supersedes: ADR-9ede1ffd04e2
ratified: 7fd54c01c3e0
schema: 2
version: 3
---

ADR-9ede1ffd04e2 dissolved the split between an agent surface and a human one, and kept one residue of it: a help that presents the loop first and the rest layered. This supersedes it to remove that residue, and changes nothing else.

Layering is grouping, and a grouping printed by the binary is a claim the binary makes about who a verb is for. That is exactly the claim ADR-9ede1ffd04e2 withdrew. The headings it left standing were named after callers -- agent loop, agent off-loop, human -- so an agent reading help still learns a boundary the dispatch table does not have.

The token budget the layering appeared to protect is already protected where it operates. SKILL.md is loaded permanently and its content is frozen at the loop; help is loaded on demand, read once, and pays for itself in a single round trip. A flat listing costs no session anything.

The loop survives in the one place it was ever enforced. SKILL.md teaches context, claim, show, log, done, new, find and release, and growing that content still costs a succession of this ADR. What changes is only that the binary stops repeating what the documentation already teaches.

Rejected: grouping by activity, the way git help does. It is defensible for git, whose hundred commands are unreadable in one list. Ank has sixteen, section 4 already orders them with the loop first, and an ordering carries the same information as a heading without asserting a category.

Consequence: ank help prints one listing, ank help <verb> prints usage, flags and globals with no audience line, and --json carries no audience key. The order of section 4 becomes the presentation order, and it is the only structure the output has.
