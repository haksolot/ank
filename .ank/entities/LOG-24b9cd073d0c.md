---
id: LOG-24b9cd073d0c
type: log
title: Two premises in the brief and the criterion do not hold against the code, measured before any edit.
created: 2026-08-24T02:05:36Z
author: claude-code/opus-5-json-budget
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-652de6ead019
seq: 0
schema: 4
version: 1
---

 First: the guard no_superseded_document_is_cited_in_the_workspace passes today, because the corpus holds exactly 40 superseded documents and the cap is 40, so shown==total by one document of margin. The live defect is real elsewhere: find --type adr --json answers total 63, shown 40, hidden 0. Second: context --json is not budgeted at all. context.rs touches cfg.context_budget at exactly one line, the human render; render_json takes no budget argument. Measured on a copy of this corpus with context_budget 300: human context prints 1 constraint and +40 broad constraints, context --json emits 41. So the clause 'context --json stays budgeted' describes code that does not exist.
