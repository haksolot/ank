---
id: TASK-ac2ff41162c6
type: task
slug: the-unit-tests-under-src-leave-their-scratch-dir
title: The unit tests under src leave their scratch directories behind too
created: 2026-08-27T18:20:40Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/editor.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-daemon/src/stream.rs
  - crates/ank-tui/src/stream.rs
blocked_by: [TASK-553740e7af11]
done_criteria: |
  After a green cargo test --workspace into an empty temporary directory, the only entries left are the roots of the run itself: no ank-edit-*, ank-signers-*, ank-daemon-stream-* or ank-tui-stream-* directory or file remains. The scheme is the one TASK-553740e7af11 established and not a second one -- a root named for the process, holding a lock the kernel frees when the process dies, swept by the next run.
criteria_by: creator
schema: 4
version: 3
---

TASK-553740e7af11 fixed the integration suites and measured what it did not
reach. Counted on 2026-08-27 in `/tmp`, before that task: `ank-declared` 1959,
`ank-daemon-it-home` 915, `ank-tui-stream-*` 1158 across six names,
`ank-daemon-stream-*` 303 across three. After it, a full workspace run into an
empty directory still left 16 `ank-edit-*` files, 19 `ank-signers-*` and nine
stream directories.

These come from `#[cfg(test)] mod tests` blocks inside `src/`, not from
`tests/`, which is why the earlier task's scope could not reach them: its
criterion named the four integration families and it met them.

The helper cannot simply be shared. `crates/ank-cli/tests/scratch/mod.rs` is a
test-only module of an integration binary and nothing under `src/` can name it.
Either it moves somewhere both can reach, or each crate carries the same thirty
lines a second time -- `crates/ank-tui/tests/terminal/mod.rs` already carries
one such copy, and its comment records why: two crates cannot share a test
helper without a dev-dependency between them, and this crate's dependency tree
is asserted by its own suite.

That is the decision this task has to make first, and it is small enough to make
while doing it rather than before.
