---
id: ADR-ff294eff4d1a
type: adr
slug: the-log-is-a-file-of-its-own-and-a-task-file-cha
title: The log is a file of its own, and a task file changes only on a transition
created: 2026-08-11T22:17:16Z
author: claude-code@sean-laptop
status: superseded
scope:
  - crates/ank-core/**
  - crates/ank-cli/**
  - docs/**
constraint: |
  The log of an entity is a file of its own, .ank/log/<ID>.md, append-only, one
  timestamped line per entry. The line grammar does not change: a dash, the
  timestamp, the identity, an em dash, the message.
  
  A task file changes only on a real transition. Appending a log entry writes no
  frontmatter, bumps no version, and touches no file carrying a frozen field.
  
  The log stays a work trace and never proof. Nothing authoritative is anchored in
  it and no hash chains over it, which is what makes an append by a second party
  harmless.
  
  Any entity kind may carry a log. A missing log file means no entry, never an
  error.
ratified: 79b697b2f062
schema: 2
version: 3
---

## Context

The log is a `## Log` section at the end of the task file. Section 3 settled it
there and gave the reason: appending at the end of the file is a one-line git
diff, and a separate file "would have given equivalent diffs while doubling the
number of objects to resolve".

That argument is correct and it is not the whole account. Three costs sit on the
other side of the scale and none of them was weighed, because two of them did not
exist yet.

## The three things the original decision did not weigh

**The file that must be most stable is the one that churns most.** A task file
carries `done_criteria`, whose hash is frozen in the claim record and which
`check` compares against. Every `ank log` rewrites that file and increments
`version`. The volume is not theoretical: 123 of 132 tasks in this corpus carry a
log, and the log is where an agent is told to write whenever it learns something.
So the single artefact whose stability the freeze mechanism exists to make
observable is the artefact that is rewritten most often, and every one of those
rewrites is a diff a reviewer has to look past to see whether the criterion
moved.

**The merge driver is already conceded, and this is its only hard rule.** Section
7 ships no `.ank/` merge driver and fixes the two rules a future one will apply:
`version` = max + 1, and the log section = union ordered by timestamp. The second
rule exists only because the log shares a file with fields that cannot be unioned.
A file that contains nothing but appended lines does not need a driver at all —
git's own union of two appends is already the answer, and the rule stops being
something to automate later.

**A second party has nowhere to append.** This is new. A pipeline recording that
it ran, a reviewer noting why a task was handed back, a second agent in the same
tree: each of them, today, must rewrite durable state carrying a frozen field to
add one line of trace. Once a proof can be recorded outside the tree, a trace that
cannot is the remaining half of the same problem.

## The original argument, answered on its own terms

The one-line diff was the property being protected, and the move protects it
better, not worse. An append to a file that only ever grows is a one-line diff
with no context lines drawn from anything else, on a path a reviewer can
configure away entirely.

"Doubling the number of objects to resolve" is the real cost and it is paid. The
answer is that resolution never went through the directory in the first place:
the file name is the id, the layout is flat, and `.ank/log/TASK-<id>.md` is
computed from the same id as `.ank/entities/TASK-<id>.md` with no lookup. What
doubles is the number of files on disk, not the number of decisions a reader
makes. Set against a corpus where the most-edited file is also the one carrying
the frozen field, that is the cheaper side.

## Rejected

**A log in a git ref.** It would remove the file entirely and it would remove the
log from the tree, where it is read by humans looking at a diff and by anyone who
clones without the `refs/ank/*` refspec. The log is durable state and travels with
the code; only the coordination plane belongs in refs.

**One log file for the whole corpus.** Every append would contend with every other
one, which is the same mistake as a single claim ref, and the file would be
unreadable within a month.

**Keeping the section and merely not bumping `version` on append.** It would fix
the churn signal and leave the merge behaviour and the second-party problem
untouched, while making `version` mean "changed, except sometimes".

## Consequences

`ank show` reads two files where it read one. That is the visible cost and it is
accepted: `show` already assembles more than one source, and a missing log file is
simply an empty log.

The schema moves, because a reader that does not know the log has left the body
would show an empty history for a task that has one, silently. Refusing on the
version is what the format does with exactly this case.

The `## Log` sections already written are moved by the same migration that moves
the entities, and the lines are copied verbatim: the grammar does not change, so
nothing about an existing entry is reinterpreted.
