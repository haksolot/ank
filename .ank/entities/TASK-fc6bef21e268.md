---
id: TASK-fc6bef21e268
type: task
slug: a-test-fixture-repository-is-not-maintained-unde
title: A test fixture repository is not maintained under the test
created: 2026-08-30T01:01:30Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/status.rs
  - crates/ank-tui/tests/terminal/mod.rs
blocked_by: []
done_criteria: |
  No test fixture creates a git repository with automatic maintenance left on. Every `git init` a fixture runs is followed by `gc.auto=0` and `maintenance.auto=false` on that repository, at the four sites that have one today: crates/ank-cli/src/claim.rs, crates/ank-cli/src/human.rs, crates/ank-cli/tests/status.rs and crates/ank-tui/tests/terminal/mod.rs. A test asserts it by reading the config back out of a freshly built fixture rather than by grepping the source, so a fifth fixture added later is caught by the same assertion.
  
  Measured on 2026-08-30, run 33284185681: `naming_a_claim_elsewhere_writes_nothing_in_the_corpus_it_names` failed on ubuntu-latest while passing on the other two platforms and on nine local runs. The corpus it fingerprints changed under it because git repacked the repository between the two snapshots, not because ank wrote anything: the first snapshot carries `objects/maintenance.lock` and `objects/pack/tmp_pack_mNMHSi` and six loose objects, and the second carries a multi-pack-index, two packs, `info/refs` and no loose objects at all. The assertion is right and its subject was moving.
  
  cargo test --workspace green, cargo fmt --check clean, ank check reports no new fault.
criteria_by: creator
schema: 4
version: 1
---
