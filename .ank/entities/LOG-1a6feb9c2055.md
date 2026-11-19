---
id: LOG-1a6feb9c2055
type: log
title: "Decision (orchestrator): the 10 non-index citations of SPEC-79012ba18769 read the spec in force, so"
created: 2026-11-19T03:18:05Z
author: claude-code/opus-5.5+w9-spec-successors
scope:
  - .ank/entities/SPEC-*.md
about: TASK-876a641d3939
seq: 4
schema: 4
version: 1
---

 they move in the ratification commit with the index. The ratifier runs, all landing together: sed -i -e s/SPEC-79012ba18769/SPEC-6d686a1a24b0/g -e s/SPEC-ed5f09df66ef/SPEC-a13982bca401/g docs/specification.md crates/ank-cli/tests/skill.rs crates/ank-contract/src/verbs.rs crates/ank-tui/src/view.rs, then ank accept SPEC-6d686a1a24b0 SPEC-a13982bca401, then drop reject from DISPATCHED_AHEAD_OF_SECTION_4 in crates/ank-cli/tests/skill.rs. Rehearsed in the lab: that sed set followed by accept exits 0, and check shows 0 faults and no stale citation.
