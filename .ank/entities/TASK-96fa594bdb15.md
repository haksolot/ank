---
id: TASK-96fa594bdb15
type: task
slug: ank-init-writes-requires-ank-as-the-running-vers
title: ank init writes requires_ank as the running version
created: 2026-10-24T13:37:23Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/init_requires_ank.rs
blocked_by: [TASK-109153abe8e0]
done_criteria: |
  Through the binary, in crates/ank-cli/tests/init_requires_ank.rs: ank init in an empty scratch repository writes .ank/config.yml at schema 2 with requires_ank equal to the version ank --version prints first; ank init --at <path> does the same in the declared corpus; ank check on the fresh corpus exits 0 with no fault; an init over an existing corpus does not change an existing requires_ank; each case asserts the file content and the exit code.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Applies ADR-756e8b0fe416 once it is ratified; do not claim before. Disjoint from the warning task (cli.rs, human.rs), so the two can run in parallel once the key exists.
