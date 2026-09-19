---
id: LOG-a99a298e7bda
type: log
title: "Measured through the binary. Red first: tests/cli.rs"
created: 2026-09-14T14:13:59Z
author: claude-code/opus-5+plane-namespaces
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-tui/src/view.rs
about: TASK-660535c73244
seq: 4
schema: 4
version: 1
---

 help_watch_says_the_mirror_carries_claims_and_nothing_else failed on 'help watch does not say the mirror carries the claims namespace' against main f60c94c. After golden then verbs.rs: ank help watch and ank help watch --json both carry 'a mirror of refs/ank/claims/* under refs/ank/watch/<remote>/claims/, the claims and nothing else' and no 'mirror of refs/ank/*'; the help golden test and the ank-tui suite (207 tests, fixture ids re-pointed) pass. SPEC-f90c92993843 differs from SPEC-77689b90b211 by exactly one body line (difflib over ank show output), the watch paragraph, and adds ADR-4b45f344344f to references. check --json diffed with and without it: on its own subject the two signals of a proposed successor ('not yet a succession', 'read by no human'); on four other accepted specs (SPEC-3bccb8aee5b7, SPEC-861d09f3f85e, SPEC-a1234da5449a, SPEC-aa4b4f119929) one signal each, 'references <old CLI surface id>, whose succession ends on SPEC-f90c92993843, which is not accepted' -- all six clear at the signed accept. No fault added; exit 8 with and without, from the one pre-existing fault on TASK-97fd1992567a. Predecessor citations outside .ank/: 10 (full and SPEC-7768 short form) in skill.rs, verbs.rs and ank-tui view.rs, re-pointed; git grep SPEC-7768 now 0.
