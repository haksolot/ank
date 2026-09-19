---
id: LOG-6bd9e7cc9393
type: log
title: "Measured, not read. Against the built binary: `ank help --json` answers 26 verbs, 24 of which carry"
created: 2026-08-31T09:13:27Z
author: claude-code/opus-5+golden
scope:
  - crates/ank-cli/tests/schema.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/fixture/**
  - crates/ank-cli/tests/cli.rs
about: TASK-49b10f02d209
seq: 2
schema: 4
version: 1
---

 a non-empty `returns`; `tests/golden-json/` held 26 fixtures covering 22 verbs; the set difference is exactly {read, tui}, which is the finding confirmed rather than restated. `mcp` and `watch` are the two with `output: &[]` and are asked for nothing.

The blind spot, measured as a blind spot. Before this change the suite was green with both fixtures absent: `every_golden_conforms_to_the_shape_its_verb_declares` starts at the directory, so a verb with no fixture is a file it never opens. `every_verb_that_declares_a_document_has_a_golden_fixture` in tests/schema.rs runs from ank_contract::COMMANDS to the directory instead. Red-first, twice, and the second is the one that matters: with both fixtures absent it failed naming ["read", "tui"]; with read.json deleted, naming ["read"]; and with `watch` given a one-field shape it does not have and no fixture, naming ["watch"] -- so the next verb added with a declared shape and no fixture reddens the suite instead of landing in the same silence. The verbs.rs mutation was reverted with `git checkout`.

tui is not a verb a fixture is meaningless for, and that was checked before assuming it. `ank tui --json` into a pipe refuses at exit 9 (measured), but on a terminal it answers one document and opens no session, which tests/tui.rs already drives through a pseudo-terminal. So the fixture is captured there, after `cargo build --workspace` (ADR-93d8ef477c00), and it carries rows in both arrays -- claims[] and entities[] -- so the element shapes are exercised rather than reached through an empty list. It is #[cfg(unix)]: ConPTY is not reached by this workspace, so on Windows the file is still checked against the declaration by the conformance walk, and only the re-capture does not run. Mutated the fixture ("shown":2 to 3) to confirm the compare path is not vacuous: red.

read is captured in tests/schema.rs, on the corpus that file already builds -- no git anywhere, because `read` coordinates nothing (ADR-9307e5d214a7) -- and its ADR is minted through the binary rather than seeded, so nothing writes into .ank/ by hand. The identifier is re-minted if it begins with 0000, which is the prefix the redaction keeps for seeded ids: once in 65536 that fixture would otherwise carry a live identifier and redden the next run.

Cost paid outside the declared scope, stated rather than buried. cli.rs asserts `checked == 26`, a hard-coded count of fixtures, so two more fixtures redden it wherever they are captured; the footprint there is that one number and the comment above it. tests/fixture/mod.rs is a second copy of cli.rs's redaction, because an integration test is a crate of its own and neither tui.rs nor schema.rs can reach `golden` in cli.rs; folding the two is TASK-7a9d945640e3, not done here because cli.rs is a perimeter three other agents are working in. Scope was amended before the claim, not after, so no frozen field moved.

Suite green on this tree: 334 unit, 328 cli, 12 schema, 29 tui, 27 watch, 24 skill, 14 status, 13 adopt, 12 corpora_schema, 12 mcp, 6 init_at, 5 guide, 20 contract, 20 core, 28 core golden, 6 tui-crate; fmt clean; ank check exit 0, no fault.
