---
id: TASK-27cf26cbc414
type: task
slug: a-dead-scope-git-can-explain-is-a-signal-and-the
title: A dead scope git can explain is a signal, and the explanation reaches a glob
created: 2026-08-13T16:18:58Z
author: claude-code/2.1.229
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  A dead scope whose death git can explain is reported as a signal; one it cannot explain keeps the severity it has today. The rule only ever lowers a severity and never raises one, so a dead scope that is a signal today is a signal after this.
  
  scope_moved answers for a glob as well as for a literal path. The literal prefix before the first wildcard is what is asked about, and a prefix whose files git records as renamed into one directory is explained by that directory; sources landing in more than one destination produce no explanation, on the rule that silence is never evidence.
  
  Section 4 of docs/ank-spec-v1.1.md carries both before the code moves: the dead-scope entry states the two severities and what separates them, and states that the walk serves a glob through its literal prefix.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs, four fixtures: a file named literally in a finished task's scope is renamed and committed, and check exits 0 with a signal naming the new path; the same file is deleted instead, and check exits 8; a directory named by a glob is moved wholesale, and check exits 0 naming the new directory; the files under such a prefix are scattered across two directories, and check exits 8 with no explanation.
criteria_by: creator
proof:
  - type: commit
    ref: 8f7f24e
    criteria: 93424b06248c
schema: 3
version: 3
---

Unblocks TASK-9bff1d5826b1. The flat-layout move kills six scopes and `check`
goes from 0 faults to 6, which the move's own criterion forbids. Measured on the
move as it stands on branch chore/flat-layout-9bff: four of the six are
`.ank/adr/**`, one is `.ank/tasks/TASK-aca0cb103980.md`, one is
`.ank/tasks/TASK-a1b2c3d4e5f6.md`.

**The wildcard half is what makes this real, and it is the half easy to leave
out.** `scope_moved` (human.rs) returns nothing the moment a glob carries a
wildcard, so the severity change alone resolves two of the six and leaves four
red. A task that lowers the severity and stops there does not unblock the move,
and would look finished while doing it.

The severity rule, stated as a floor rather than as a table, is the part to get
right. `check_scope_alive` decides severity today by kind and status: signal for
an open or in_progress task, fault for an ADR or a finished task, because a
task's scope says where work will happen and an ADR's says what it binds. That
reasoning stays. What is added is a second question asked only of the entities
that would fault -- did git record where the path went -- and a yes lowers it to
a signal. Rewriting it as a fresh two-axis table is how an open task with an
unexplained dead scope quietly becomes a fault, which is a widening nobody asked
for.

Why this is the right shape and not a concession. `check` faulting on a dead
scope treats "this scope matches nothing" as a corpus defect. When git can name
the commit that moved the path, the corpus is not broken -- it is outdated in a
way the reader can see and follow, which is precisely what ADR-97beaf55e73a
built the walk to show. The fault is worth keeping for the case the walk cannot
explain, where the reader has nothing to go on. Without this split, any directory
rename reddens the corpus permanently: `amend` refuses a done task with code 7,
measured on TASK-3109a736c255, and TASK-1e79ff3738df settled deliberately that
such a task gets the rename named and no repair command. So today the fault fires
with no act available to anybody that would clear it, which is the shape of a
finding readers learn to skip.

The prefix walk, concretely. `.ank/adr/**` has the literal prefix `.ank/adr`.
`rev-list -1 HEAD -- .ank/adr` gives the last commit that touched it, and
`diff-tree -M -r -z --name-status --no-commit-id` on that commit gives the
renames; the sources under the prefix and their destinations are what answer.
Both verbs are already in the PLUMBING allow-list of ADR-b8884edcebe3 -- check
that rather than trusting this line, because the debug_assert is what catches a
miss and only in debug builds.

Do not take `--full-history`. `rename_of`'s doc comment explains why it is absent
there and the same reasoning holds here: default simplification walks to the
commit that made the change, and a merge is a commit `diff-tree` prints nothing
for, so asking for more history answers less often.

The silence rule is not decoration, and it is where this most easily goes wrong.
A deletion, a move under the similarity threshold, a scattered directory and a
typo that never named a real file all produce the same nothing. No wording added
here may let a reader infer which -- `scope_moved`'s doc comment already says so,
and the glob branch has one more way to be tempted, because "the prefix moved
mostly there" is a sentence that will suggest itself and must not be written.

Cost stays where it is: the walk runs only on a scope already dead, and a healthy
corpus has none. Do not hoist it above the dead-scope test to tidy the code.
