---
id: LOG-1072d31bcadb
type: log
title: Closed green. Accept measured on a throwaway clone of this tree at dab7e87 with the branch
created: 2026-09-13T17:50:23Z
author: claude-code/fdf8
scope:
  - .ank/entities/SPEC-e89b6a498634.md
  - .ank/entities/SPEC-b156a5571668.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
about: TASK-fdf872f98bf4
seq: 4
schema: 4
version: 1
---

 committed on main (f56f959), ADR-64f32c74a0f9 still proposed there: 'ank accept SPEC-77689b90b211' printed 'accepted SPEC-77689b90b211 -> 87d0429' and 'superseded SPEC-e89b6a498634', exit 0; 'ank accept SPEC-3bccb8aee5b7' printed 'accepted SPEC-3bccb8aee5b7 -> cdd90e8' and 'superseded SPEC-b156a5571668', exit 0. After both accepts, on that clone with its own CARGO_TARGET_DIR: ank check 'ok, 395 tasks, 90 adr, 677 signal(s)' (the 89 over this tree's 588 are thin-proof signals: a local clone carries no proof refs, 205 lines of that finding there); cargo test -p ank-cli --test skill 36 passed; cargo test -p ank-tui 207, 2, 8 and 2 passed. Falsified twice on that clone: with NOT_YET_DISPATCHED emptied, every_verb_section_4_lists_ships_or_is_declared_unimplemented failed with '§4 lists ank update, the binary does not dispatch it, and it is not in NOT_YET_DISPATCHED'; reset before the accepts with the one verbs.rs citation restored, accept refused: 'error[7]: 1 citation of SPEC-e89b6a498634, which ratifying SPEC-77689b90b211 would retire, remains in 1 file / crates/ank-contract/src/verbs.rs:440', exit 7. Clone and its target deleted. ank done in this tree: cargo-test ok (400.8s), fmt-check ok, check-repo ok. Waiting on accept, a human act on the default branch: SPEC-77689b90b211 (The CLI surface) and SPEC-3bccb8aee5b7 (Bootstrapping, teaching and distribution); ADR-64f32c74a0f9 is reported accepted on main at 1aaa9bf by the orchestrator. TASK-161c402c27fb removes update from NOT_YET_DISPATCHED when it dispatches the verb.
