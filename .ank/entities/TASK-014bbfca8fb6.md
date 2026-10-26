---
id: TASK-014bbfca8fb6
type: task
slug: re-point-every-citation-of-adr-24efa7543a16-ahea
title: Re-point every citation of ADR-24efa7543a16 ahead of its supersession
created: 2026-10-26T13:29:59Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/hooks_install.rs
blocked_by: []
done_criteria: |
  No file outside .ank/ cites ADR-24efa7543a16 under its full or any abbreviated form (command grep -rnE 'ADR-24ef' over the tree, .ank/ and target/ excluded, prints nothing), each citation names ADR-376bed7fa39a instead and still sources the claim it is cited for, and ank accept ADR-376bed7fa39a reports no citation of the superseded decision. ank check stays green.
criteria_by: creator
verify: [check-repo]
proof:
  - type: test
    ref: local/b3d0d766b3a0@d1093b9
    tree: scope/3beb0a2bbb03
    criteria: c8900fa77581
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 4
---

ADR-3b6b refuses a supersession while the workspace still cites what it retires,
and the accept's citation warnings announce a red main rather than a suggestion.
Counted on 2026-10-26 at e04c0dad: four code files cite it, verbs.rs (twice),
hooks.rs, tests/skill.rs, tests/hooks_install.rs. Grep the 9-character form too.

Also read the accepted entities that cite it, SPEC-79012ba18769, ADR-35d714834ed3
and ADR-bafbd28498cc: an accepted spec or ADR is not rewritten here, but if one
states something ADR-376bed7fa39a changes, say so in a log on this task before
the ratification, not after.

Must be done before ADR-376bed7fa39a is accepted; the --peers implementation
tasks wait on it for that reason.
