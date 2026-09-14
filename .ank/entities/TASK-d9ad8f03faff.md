---
id: TASK-d9ad8f03faff
type: task
slug: the-cold-rebuild-linearity-tests-decide-on-a-wal
title: The cold-rebuild linearity tests decide on a wall clock, and a loaded runner loses
created: 2026-09-14T08:34:00Z
author: claude-code/opus-5+cold-rebuild
status: in_progress
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  The two tests that assert the cold rebuild of the index is linear, the unit one on Index::in_memory and the binary one in tests/cli.rs, decide on a count and not on a clock: they give the same verdict on an idle laptop and on a loaded runner, and no tolerance is widened to buy that. The property under test is unchanged and a falsification says so: restoring a delete from entities_fts filtered on the id column makes the test fail. cargo test --workspace, cargo fmt --check and ank check stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Measured 2026-09-14: PR #423 (task/bearing-on) went red on macOS on
`index::tests::a_cold_rebuild_costs_twice_as_much_for_twice_the_corpus`,
index.rs:1860, a ratio over 2.5 on a loaded runner, in a change that never
touched the index. The runs of main happened to pass. The test landed with
TASK-b646631fa10a at a measured 2.03 against a limit of 2.5, and its twin
through the binary at 1.54: both take the minimum of three wall-clock runs,
which measures the runner (ADR-cc65f1388a71), exactly as the TTL renewal test
did before TASK-53ed7978621b.

**What must not be built here is a wider ratio.** A limit raised until the
runner stops losing asserts that the machine was fast enough. The property is
that removing the searchable twin of an entity costs the same whatever the
corpus holds, and SQLite counts that deterministically: the virtual-machine
steps a statement executed (`sqlite3_stmt_status` with `SQLITE_STMTSTATUS_VM_STEP`,
`Statement::get_status(StatementStatus::VmStep)` in rusqlite). A delete by rowid
seeks, and its step count does not grow with n; a delete filtered on a column
scans, and its count grows with n.
