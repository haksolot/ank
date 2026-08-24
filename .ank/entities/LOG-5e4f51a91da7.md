---
id: LOG-5e4f51a91da7
type: log
title: "Golden blessed: crates/ank-cli/tests/golden-json/find.json only, one line, two changes. shown 5 ->"
created: 2026-08-24T02:29:51Z
author: claude-code/opus-5-json-budget
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/find.json
about: TASK-652de6ead019
seq: 4
schema: 4
version: 1
---

 6, and one row appended, ADR-0000000000ba proposed, the row the page had been dropping. contract stayed 1, so the version did not move, and help.json did not change because this touches no CommandSpec. Wrote SPEC-f3ad3f23c09d superseding SPEC-2d0c1309e049, carrying the body forward with the find cap passage revised to say the budget stops at the terminal. It lands proposed, so SPEC-2d0c1309e049 stays accepted until a human accepts: the citation at ank-contract/src/verbs.rs:331 is therefore not orphaned yet, and I re-pointed it to the successor now so acceptance stays green. Verified first that the successor still states the rule the comment cites, at body line 291, word for word.
