---
id: TASK-e9dfaf187a1b
type: task
slug: two-ank-processes-on-one-corpus-race-on-the-deri
title: Two ank processes on one corpus race on the derived index
created: 2026-08-15T18:17:43Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/index.rs
blocked_by: []
done_criteria: |
  Two ank processes that read the same corpus at the same time both answer. A test in crates/ank-cli/tests/cli.rs spawns several concurrent invocations of an index-opening verb against one repository, through the built binary, and asserts that every one of them exits 0 and that none reports the index in its error. The falsification is recorded in the task's log: the same test run against the code before the fix, with what it printed. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 2
---

Measured in CI on 2026-08-15, on all three platforms at once, from a change that
only added a reader: three tests in `tests/skill.rs` each ran `ank find` against
this repository's own corpus on their own thread, and two of the three came back
`error[1]: index: attempt to write a readonly database` and `error[1]: index:
disk I/O error`. The same suite passes on a developer machine, which is what a
concurrency defect looks like and not what a flake looks like.

The cause is in `Index::try_open`: the connection is `Connection::open(path)`
with no `busy_timeout` and no journal mode set, so SQLite's default is to fail a
contended write immediately rather than to wait. Every verb that opens the index
is on this path — `find`, `context`, `scope` among them — and each of them
*writes* while reading, because the index refreshes what diverged at read time
(§6).

**This is the nominal case failing, not an exotic one.** §7 states one working
tree per agent as the design, and worktrees of one repository share a `.ank/`;
two agents running `ank context` within the same second is the ordinary shape of
a parallel session. The index is derived, disposable and rebuildable, and §6
says deleting it is always safe — so a reader losing a race has every right to
wait, and none of what it would compute is authoritative enough to be worth an
error.

The error the caller gets is also wrong in a second way: it names the index and
tells the reader to delete it, which repairs nothing here and throws away a
correct cache to work around a lock.

Two candidate fixes, and the task should measure rather than assume. A
`busy_timeout` is the smaller one and probably sufficient: it makes a contended
open wait instead of failing. WAL mode allows a reader and a writer to proceed
together and would help the read half further, but it changes the on-disk shape
of a file `init` gitignores and `check` rebuilds, so it is a decision with more
surface than it looks. Whichever is chosen, the timeout has to be bounded: a
verb that hangs on a lock is worse than one that fails, and §4's exit codes have
a slot for an environment that will not answer.

`tests/skill.rs` reads the corpus once for the whole binary as a way around this
while it stands. That is a workaround in a test and must not be read as the
fix — it removes one suite from the race and leaves every caller in it.
