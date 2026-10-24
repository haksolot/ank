---
id: LOG-03afa4f1aa3e
type: log
title: "self-review. Criterion axis: empty repo -> init_in_an_empty_repository_writes_the_running_version_at"
created: 2026-10-24T15:23:43Z
author: claude-code/opus-5.5+w96fa
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/init_requires_ank.rs
  - crates/ank-cli/tests/main.rs
about: TASK-96fa594bdb15
seq: 4
schema: 4
version: 1
---

_schema_2 (opening bytes + config requires_ank readback = ank --version token 2); --at -> init_at_writes_the_running_version_in_the_declared_corpus (detached file, source has no .ank, readback through the declaration); check -> check_on_a_fresh_corpus_exits_0_with_no_fault (exit 0, faults 0, no fault finding); existing corpus -> init_over_an_existing_corpus_leaves_requires_ank_as_it_was (schema 2 with 0.0.1, and schema 1 without the key, both byte-identical, exit 0); each case asserts content and exit code. No hunk beyond them except the mod line scope was amended for. Constraint axis over init.rs, tests/main.rs, the new test: ADR-756e (init writes the running version, no network, schema 2 with the key) met; ADR-1f5a untouched (pointer path unchanged); ADR-e64d no other key materialised, config.rs untouched; ADR-2b62 no doc block shows init's config.yml (grep of docs/README/skill for 'schema: 1' empty); ADR-cc65 no git process added; ADR-93d8 measured via mise run test, which builds the workspace. Nothing found to fix.
