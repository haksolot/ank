---
id: LOG-a165a7c0e8ca
type: log
title: "Scope widened, approved by the orchestrator: tests/cli.rs, tests/mcp.rs, tests/writes.rs,"
created: 2026-11-19T01:29:54Z
author: claude-code/opus-5.5+w7-reject
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-core/src/model.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-contract/src/meaning.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/reject.rs
  - crates/ank-cli/tests/skill.rs
  - docs/ratifying.md
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/writes.rs
  - crates/ank-core/tests/reference_pages.rs
  - docs/entity-fields.md
about: TASK-351d5cf24884
seq: 8
schema: 4
version: 1
---

 ank-core/tests/reference_pages.rs, docs/entity-fields.md (regenerated). Each is a test or generated page that enumerates verbs or statuses. Section 4 gap: skill.rs gets DISPATCHED_AHEAD_OF_SECTION_4 = [reject] with no reverse guard, removed once the successor of SPEC-79012ba18769 listing reject is accepted.
