---
id: TASK-0e5a00f98cfe
type: task
slug: the-tree-walk-descends-into-other-checkouts-and
title: The tree walk descends into other checkouts, and 88 percent of what it reads is not this tree
created: 2026-08-24T18:50:37Z
author: claude-code/opus-5-reading
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  tracked_files does not descend into a directory that is a separate git checkout, which is any directory below the root carrying a .git entry of its own, and the exclusion of the corpus directory in accept's citation warning holds wherever a .ank sits rather than only at the root. Measured on this repository before and after, both the file count the walk yields and the wall time of ank check, release build, warm. The findings ank check reports are identical subject by subject except for the dead scopes the wide walk was hiding, which are named in the record: TASK-10b8a29fd853 and TASK-3109a736c255 scope .claude/**, and the checkouts living under .claude/worktrees/ made that glob match. check still exits 0. accept's orphaned-citation warning names only files of this tree. Verified through the binary on a repository carrying a nested checkout. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 5
---

Found by the warning `accept` printed while ratifying SPEC-78134d2b3cf8, on its
first real use. It named seven citations of the superseded document, and six of
them were in `.claude/worktrees/`: two other agents' checkouts of this same
repository, each carrying its own `.ank/` with its own copy of the predecessor.
One citation was real, `crates/ank-contract/src/verbs.rs:331`.

**Measured on this repository.** `git ls-files` counts 1360 tracked files. The
walk, skipping `.git`, `target` and `node_modules` as it does today, yields
11852, of which **10490 sit under `.claude/worktrees/`**: eight checkouts, 88
percent of everything it reads.

**It is not only the warning.** `tracked_files` is what `scope_verdicts`
confronts every scope glob in the corpus with, so the dead-scope half of `check`
has been asking its question against eight stale copies of the tree. No verdict
in this corpus is wrong today, because `globset` anchors a pattern at the start
of the path and none of the patterns here open with `**/`. That is luck about
which globs happen to be written, not a property, and a scope opening with `**/`
would be answered alive by a file in a checkout nobody is working in.

**Two rules, and the second is the one the citation warning needed.** A
directory carrying a `.git` entry of its own is a separate checkout and is not
this tree, whatever it is called: `.claude/worktrees` is where these happen to
sit, and naming that path would fix this instance and not the rule. And the
exclusion of `.ank/` is anchored at the root today, so a nested corpus is walked
as if it were source, which is exactly the prose ADR-1e6bcbf62e61 holds
legitimate being reported as stale.

The cost is worth measuring on the way past: this walk is on `check`'s path, and
TASK-0515cfe21421 has just been through its neighbour.
