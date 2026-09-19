---
id: LOG-6100ed5ff82e
type: log
title: Wrote both successors from the predecessor bodies extracted out of ank show --json content (LF,
created: 2026-09-13T17:36:08Z
author: claude-code/fdf8
scope:
  - .ank/entities/SPEC-e89b6a498634.md
  - .ank/entities/SPEC-b156a5571668.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
about: TASK-fdf872f98bf4
seq: 1
schema: 4
version: 1
---

 byte-exact), edited with Python write_bytes, then ank new spec --supersedes --body -. Measured with difflib against the extracted predecessor, the stored body equal to the edited file: SPEC-77689b90b211 (CLI surface, supersedes SPEC-e89b6a498634) adds 1 line to the Commands block (ank update [--check] [--version <v>], after ank skills and before ank help, the neighbourhood ank help groups under set up a repository) and 1 paragraph after the --method paragraph, and changes no other line; SPEC-3bccb8aee5b7 (distribution, supersedes SPEC-b156a5571668) adds 1 paragraph after the Binary distribution paragraph. Scope and references carried from each predecessor, ADR-64f32c74a0f9 appended to both reference lists and cited in both bodies. Schema moved 3 to 4 on the distribution successor, which is what new writes. Note for planning: the exit code table still says 8 is reserved for check, while the criterion has update --check exit 8 when a newer release exists; the table was left whole because the criterion allows no other change.
