---
id: TASK-ec579d3a566e
type: task
slug: a-dead-scope-git-records-as-deleted-is-a-signal
title: A dead scope git records as deleted is a signal, not a fault
created: 2026-08-15T21:29:04Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  A dead scope whose path git records as deleted is reported as a signal naming the commit that removed it, on the terms ADR-97beaf55e73a already gives a rename, and a death git cannot name at all stays a fault. An ADR superseding ADR-97beaf55e73a carries its rename clause forward unchanged and states the deletion case, arriving proposed. A test in crates/ank-cli/tests/cli.rs drives the built binary over a repository where a finished task scopes a path a later commit deleted, and asserts the finding is a signal naming that commit and that the process exits 0; a second case, a scope naming a path that never existed, is asserted to stay a fault and to exit 8. The test fails against the code before the change, and what it printed is recorded in this task's log. cargo test --workspace and ank check are green.
criteria_by: creator
proof:
  - type: commit
    ref: e0442b6
    criteria: 1297f275e644
    via: submitted
schema: 3
version: 3
---

Found by retiring the dogfooding hook (TASK-10b8a29fd853). Deleting `.claude/`
killed the `.claude/**` scope of TASK-3109a736c255, the finished task that put
the directory there, and `check` reported a fault no verb can clear: `amend`
refuses a `done` task, so the corpus would carry one permanent fault and
`ank check` would exit 8 for ever.

**The rule already has the right shape; it is looking for one thing too few.**
ADR-97beaf55e73a lowers a dead scope from a fault to a signal wherever git can
say where the path went, and its reasoning is explicit: the fault is for "the
death git cannot explain, where the reader has nothing", because "a fault nobody
can clear is a finding readers learn to skip". A deletion is not that case. Git
records the commit that removed a path as plainly as it records a rename — the
reader can see it, date it and read its message — so the condition the ADR
states is met and only the implementation disagrees, because the walk asks
`rename_of` and a deletion is not a rename.

**What must not move is the silence.** A path that never existed, a typo, a glob
that matched nothing from the day it was written: git has nothing to say about
any of them, the reader still has nothing, and the fault is what that is for.
The third state added for a shallow clone is the precedent — an answer git
cannot give is different from an answer it gives, and only the second lowers the
severity.

The severity rule still only ever lowers, and nothing here proposes a repair: a
finished task's scope is not amendable, so the note names the commit and stops.
That is the shape ADR-97beaf55e73a already requires of a proposal that would be
refused on the spot.

The real instance is waiting: PR #149 is held until this lands, because merging
it first is what puts the unclearable fault on the default branch.
