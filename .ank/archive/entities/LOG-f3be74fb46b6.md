---
id: LOG-f3be74fb46b6
type: log
title: "Measured on this corpus the moment check gained the walk: one fault,"
created: 2026-08-26T00:30:43Z
author: claude-code/opus-5+gate
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-c90651901f22
seq: 2
schema: 4
version: 1
---

 .github/scripts/npm-assemble.sh:99 citing ADR-e3cb36646d77, superseded by ADR-8b3045cf11db. The cargo guard in tests/cli.rs walks crates/**/src and tests only and had never looked at .github/, which is precisely the reach the ADR says check buys. Re-pointed it -- one identifier in a comment, and the successor carries the same sentence the comment leans on, plus .github/workflows/** in its scope. Outside my three declared files but neither of the other two agents' scopes, and shipping it would have turned main red on the exact condition this task exists to end. ank check is 0 faults now. Left the cargo guard in place: it is a dogfooding assertion over this repository's corpus rather than a third implementation of the verdict the tool ships, and removing a passing test is not what the criterion asked for.
