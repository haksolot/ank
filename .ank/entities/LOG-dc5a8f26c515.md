---
id: LOG-dc5a8f26c515
type: log
title: "Green, measured: accept_tag 14/14 (8 new, through the binary: no key -> 7, undeclared key -> 7"
created: 2026-11-18T16:24:12Z
author: claude-code/opus-5.5+task-4b86-signature
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/accept_tag.rs
  - crates/ank-cli/tests/cli.rs
  - docs/signing.md
  - docs/config-keys.md
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
about: TASK-4b86ac17ca91
seq: 5
schema: 4
version: 1
---

 naming it, retired -> 7 naming valid_before, each leaving refs, file and tree as found; declared key -> signed tag and zero signature findings; false/absent -> unsigned tag; check -> fault on unsigned and on undeclared, plus config.yml fault when signers is empty; ank config refuses true at 7 with no key). Also green: ank-cli bins 377, schema/help/doc_replay/config 132, prose+doc_replay 29 with Vale, ank-core+ank-contract. Task body's 'ADR proposed' line is stale: ADR-2e777475c4de is accepted.
