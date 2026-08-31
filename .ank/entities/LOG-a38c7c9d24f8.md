---
id: LOG-a38c7c9d24f8
type: log
title: "The supersession blocker, counted: 7 tracked citations of SPEC-fe8bdb84faca outside .ank/, in 3"
created: 2026-08-31T03:55:12Z
author: claude-code/opus-5+corpus
scope:
  - .ank/entities/**
about: TASK-c4f26ad5302d
seq: 1
schema: 4
version: 1
---

 files -- crates/ank-cli/tests/skill.rs:241, crates/ank-contract/src/verbs.rs:412, and five in crates/ank-tui/src/view.rs (4784, 7241, 7753, 7817, 7887). The five in the TUI are fixture rows, one of which asserts the string 'accepted' beside the id; ADR-3b6ba766a42e's walk counts a fixture like any other citation, and the fixture would in any case be asserting a status the ratification changes. All three files are perimeters outside this task's scope of .ank/entities/**, so this is recorded and not swept.

The successor differs from SPEC-fe8bdb84faca by exactly one line of body, measured with difflib over the two 'ank show' outputs: paragraph 273. Everything else, frontmatter scope and the four references included, is carried across unchanged.
