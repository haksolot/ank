---
id: ADR-3abc4b33153f
type: adr
slug: a-scope-is-confronted-with-the-tree-git-counts-a
title: A scope is confronted with the tree git counts, and one matching only ignored files is a signal
created: 2026-10-01T09:37:48Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
constraint: |
  A scope is confronted with the files git counts as this work tree: tracked and untracked, minus what .gitignore, .git/info/exclude and the global excludes name, minus what is gone from disk. Outside a repository it is confronted with a walk of the filesystem. A dead scope whose glob matches files that exist on disk only where git ignores them is a signal, never a fault: it names one such path and the command that shows the rule ignoring it, git check-ignore -v <path>. That answer is found by walking the glob's literal prefix on disk, reads no file content, starts no git process, and runs on a dead scope and on nothing else.
amends: [ADR-3094538d831e]
ratified: 7b171ee45f57
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-02T12:04:08Z
schema: 4
version: 2
---

Raised by issue #522 and PR #523. On a repository holding large git-ignored
directories, `tracked_files()` walked the disk blind to `.gitignore`, and the
stale-citation scan then read every byte of those directories whenever a
document was superseded. Measured on a reproduction (579 MB and 20 500 files
under an ignored directory, one ratified supersession): `status`, `check` and
`review` read 528 MB on `f18b60d`, and 0.3 MB with #523, which asks
`git ls-files` instead. `accept` of a supersession went from 528 MB and 13 s to
0.1 MB.

**What the fix changes beyond speed.** Asking git changes what a scope is
confronted with. A file present on disk but ignored no longer keeps a scope
alive, and since no commit ever carried it, ADR-3094538d831e keeps that death a
fault: on the reproduction, `check` went from exit 0 to exit 8 on an ADR scoped
at a generated file, while `scope` and `context` still named that ADR as
governing it. SPEC-cf285efcdca4 also states the opposite ("confronting one is a
walk of the filesystem"), and its successor carries this decision.

**Why git's tree is the right one.** The specification already refuses a scope
that matches "a path that exists on one laptop and in no CI", and
TASK-0e5a00f98cfe excluded nested checkouts for the same reason: a scope alive
on files nobody commits is a label, alive here and dead in every clean clone.
`check` now gives one verdict whatever the checkout carries beside the code.

**Why a signal and not a fault.** ADR-3094538d831e lowers a death git can
explain and keeps the fault for the one where the reader has nothing. An
ignored match is explained: the file is right there, and a rule names why git
does not count it. Leaving it a fault would turn green corpora red on upgrade,
with nothing telling their owner why.

**Why no git process.** `git check-ignore -v` would name the rule itself, but it
is not in the plumbing ADR-9307e5d214a7 admits. The walk of the literal prefix
is the shape ADR-3094538d831e already uses to ask about a glob, it reads
directory entries only, and the command is handed to the reader, who runs it
once if they care.

## Rejected

**Keeping the filesystem walk for scopes and asking git only for citations.**
It keeps two answers to "what files does this tree hold", and the walk still
descends into every ignored directory and nested checkout.
