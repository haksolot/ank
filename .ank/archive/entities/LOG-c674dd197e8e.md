---
id: LOG-c674dd197e8e
type: log
title: "Review before done. Against the criterion, clause by clause: two proposed successors from new spec"
created: 2026-09-13T17:41:06Z
author: claude-code/fdf8
scope:
  - .ank/entities/SPEC-e89b6a498634.md
  - .ank/entities/SPEC-b156a5571668.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
about: TASK-fdf872f98bf4
seq: 2
schema: 4
version: 1
---

 --supersedes (SPEC-77689b90b211, SPEC-3bccb8aee5b7), predecessor bodies whole, stored bodies equal to the edited files and differing from the predecessors by the three additions logged above and nothing else; the CLI surface lists ank update [--check] [--version <v>] with every other block line byte-identical, and one paragraph states the delegated route (npm install -g for the npm package, install.sh or install.ps1 with the executable's directory and no questions otherwise), --check installing nothing and exiting 8 when a newer release exists, only update reaching the network for that question, and the refusal of a binary under a cargo target directory; the distribution successor says an installed binary reaches a newer release through ank update by the route that placed it; both cite ADR-64f32c74a0f9 in prose and in references. skill.rs declares NOT_YET_DISPATCHED = ["update"]; the three tracked citations outside .ank/ (skill.rs 1 line, verbs.rs 1, view.rs 7 fixture sites, their 8-char prefixes included) name SPEC-77689b90b211, and git grep finds SPEC-e89b6a498634 and SPEC-b156a5571668 nowhere outside .ank/. Nothing the criterion did not ask for. Against the constraints (ank scope and ank context over skill.rs, verbs.rs, view.rs): ADR-3b6b is what the re-pointing answers; ADR-e45e held, entities read with show --json and written with new and log only; ADR-6fd6 untouched, verbs.rs changes a doc comment only; ADR-d3a8 English; no em dash in the added prose. Local: cargo fmt --check exit 0; ank check exit 0, 0 faults, 588 signals, the new ones the expected not-accepted and not-yet-superseded signals on the two successors; cargo test -p ank-cli --test skill 36 passed; cargo test -p ank-tui lib 207 passed. view.rs holds no cfg(unix) gate and no unix-only test cites these ids or reads the Commands block.
