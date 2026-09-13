---
id: TASK-38dabf191c1a
type: task
slug: the-specification-succeeds-with-the-skills-verb
title: The specification succeeds with the skills verb, the method field and the method record
created: 2026-09-13T09:21:01Z
author: claude-code/fable-5.1+planning
status: open
scope:
  - .ank/entities/SPEC-4b79c265ccb6.md
  - .ank/entities/SPEC-e258796162c4.md
  - docs/format.md
blocked_by: []
done_criteria: |
  Two proposed spec successors exist, each created with ank new spec --supersedes and carrying the predecessor's body whole with the changes below and nothing else. The successor of SPEC-4b79c265ccb6 lists skills in the Commands block with --install, lists --method on new task, amend and log, states that skills reports designated, fired and undesignated counts per sibling, and keeps every other line of the block. The successor of SPEC-e258796162c4 adds method to the task's optional fields in canonical order, states it earns no schema bump on via's terms, and adds method and create beside edit in the records vocabulary, create being the record new writes at birth (ADR-52bb0da2023a); docs/format.md's numbered table carries the same field in the same position. Both successors cite ADR-e1d750884b82, ADR-a8f9c603a0e7 and ADR-52bb0da2023a. ank check reports no fault, cargo test stays green because the suite still reads the accepted predecessor, and the two successors are named as waiting for accept in the closing log entry.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 4
version: 3
---

The suite reads the CLI surface out of the ratified spec document and names
any dispatched verb it does not find there (tests/skill.rs,
read_section_4_document), so the verb is declared here before the code lands,
the way mcp and watch were. Nothing here is code: two documents, proposed, and a
table in docs/format.md.

Write the successor with `ank new spec --supersedes <id> --body -` from the
predecessor printed by `ank show`, not from memory: the block is long and the
suite compares verbs one by one. The predecessor stays accepted until a human
runs accept on the default branch; that is the moment TASK-544ec9655570 can
claim, and its body says so.
