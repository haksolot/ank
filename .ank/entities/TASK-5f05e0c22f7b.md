---
id: TASK-5f05e0c22f7b
type: task
slug: check-reads-one-file-and-one-ref-per-entity-and
title: check reads one file and one ref per entity, and each is a process
created: 2026-08-20T18:22:03Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  The number of git subprocesses check spawns no longer grows with the number of entities or with the number of refs under refs/ank, measured through the binary with GIT_TRACE2_EVENT on two corpora differing only in entity count. The findings check, review and status report are identical to what they report before the change, subject by subject, level by level and message by message. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 2
---

Measured while closing TASK-1b3d7b61dc8f, which bounded what grows with dead
scopes and with ratified entities and left this untouched because the criterion
did not name it.

After that task, `ank check` on this repository starts 308 git processes, and
what remains is two questions asked once per item:

- **One file per entity off the default branch.** `git::file_at` runs
  `cat-file -p <branch>:<path>` for each, which is how §7 asks what the default
  branch carries rather than what the tree holds. 194 `cat-file` starts.
- **One ref per claim and per proof.** `rev-parse` to resolve it and `cat-file`
  to read the record. 100 `rev-parse` starts after the branch resolution was
  memoised.

At about 61 ms a start on the machine measured, that is most of what `check`
still spends. `git cat-file --batch` reads many objects down one pipe and is
already in the allowed plumbing (ADR-b8884edcebe3); `for-each-ref` answers a
whole ref namespace in one call and is allowed too, and `check` already uses it
once.

**The identical-findings half of the criterion is not a formality here.** The
ref plane decides what is claimed, by whom, and what has finished elsewhere, so
a batching that mislabels one record does not slow the tool down, it answers
the wrong thing about somebody's work. The same is true of the file read: it is
what tells a task finished on the default branch from one finished only in this
tree.

**And a lesson from the task that produced this one, recorded so it is not paid
twice.** Framing several records in one stream is where both defects of that
work lived: a separator that a record can contain (an empty `%b` producing a
third NUL) and a memo that outlived the history it described. `cat-file --batch`
has its own framing -- `<sha> <type> <size>` then the bytes -- and the size is
what makes it unambiguous. Read the size; never look for a separator.
