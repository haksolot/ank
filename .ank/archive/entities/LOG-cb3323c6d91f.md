---
id: LOG-cb3323c6d91f
type: log
title: Blocked on one file this task's scope does not name. json_golden_reading_verbs pins the --json
created: 2026-08-24T01:07:49Z
author: claude-code/opus-5-restatements
scope:
  - crates/ank-contract/src/verbs.rs
  - docs/getting-started.md
about: TASK-f01e4b71c8c4
seq: 2
schema: 4
version: 1
---

 rendering of the verb table in crates/ank-cli/tests/golden-json/help.json, so any edit to a CommandSpec reddens it by construction: 284 passed, 1 failed, and the failure is that fixture and nothing else. cargo fmt --check is clean, ank check is exit 0 with no fault, and the three tests that read docs/getting-started.md off disk pass. The repair is one command, ANK_BLESS_GOLDEN=1 cargo test --test cli -- json_golden, and the diff it writes is confined to the log verb's summary, notes and refuses. It is not done here: crates/ank-cli is another agent's live perimeter in this wave, and two agents blessing the same generated fixture is how one blessing silently erases the other. The scope of this task should have carried that fixture, since no change to a verb's help can be green without it.
