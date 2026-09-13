---
id: TASK-fdf872f98bf4
type: task
slug: the-specification-succeeds-with-the-update-verb
title: The specification succeeds with the update verb
created: 2026-09-13T17:18:49Z
author: claude-code/opus-5+planning
status: done
scope:
  - .ank/entities/SPEC-e89b6a498634.md
  - .ank/entities/SPEC-b156a5571668.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
blocked_by: []
done_criteria: |
  Two proposed spec successors exist, each created with ank new spec --supersedes and carrying the predecessor body whole with the changes below and nothing else. The successor of SPEC-e89b6a498634 lists ank update [--check] [--version <v>] in the Commands block, keeps every other line of the block, and states in one paragraph what ADR-64f32c74a0f9 decides: the route delegated to, --check installing nothing and exiting 8 when a newer release exists, only update reaching the network, and the refusal of a binary under a cargo target directory. The successor of SPEC-b156a5571668 states in its distribution section that an installed binary reaches a newer release through ank update, by the route that placed it. Both cite ADR-64f32c74a0f9. crates/ank-cli/tests/skill.rs declares update in NOT_YET_DISPATCHED, and every tracked citation outside .ank/ of either predecessor names its successor instead, so that ank accept on each successor succeeds, measured on a throwaway clone and quoted in the closing log entry. cargo test stays green and ank check reports no fault.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/62a9f8577098@dab7e87
    tree: scope/12161f5fcc76
    criteria: 2d56d780c201
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@dab7e87
    tree: scope/12161f5fcc76
    criteria: 2d56d780c201
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
  - type: test
    ref: local/a66d13262b59@dab7e87
    tree: scope/12161f5fcc76
    criteria: 2d56d780c201
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 3
---

The suite reads the CLI surface out of the ratified spec document
(tests/skill.rs), so the verb is declared there before the code lands, exactly
as TASK-38dabf191c1a did for skills. Write each successor with
`ank new spec --supersedes <id> --body -` from the predecessor printed by
`ank show`, never from memory.

Two lessons from that supersession, measured then (PR #405): accept refuses a
successor while a tracked file outside .ank/ still cites the retired id (fixture
ids in crates/ank-tui/src/view.rs count), and a verb listed in the ratified
block and not yet dispatched turns every_verb_section_4_lists_ships_or_is_declared_unimplemented
red unless NOT_YET_DISPATCHED declares it. Both belong in this task so the
accept that follows is one command and not a repair.

accept is a human act: name the two successors and ADR-64f32c74a0f9 as waiting
in the closing entry.
