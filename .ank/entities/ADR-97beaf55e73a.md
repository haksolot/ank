---
id: ADR-97beaf55e73a
type: adr
slug: a-dead-scope-is-reported-with-the-rename-that-ki
title: A dead scope is reported with the rename that killed it
created: 2026-08-11T22:19:29Z
author: claude-code@sean-laptop
status: superseded
scope:
  - crates/ank-cli/src/human.rs
  - docs/**
constraint: |
  A dead scope is reported with the rename that killed it, whenever git can name
  one. check walks the history of the dead path, and when the last commit touching
  it recorded a rename it names the new path, the commit, and the exact command
  that repairs the entity.
  
  It proposes and never repairs. No scope is edited automatically, under any
  circumstance.
  
  The proposal names a command that will not refuse on the spot. amend cannot
  change the scope of an accepted ADR, whose scope is hashed into its ratification
  commit, so for an accepted ADR the proposal is a supersession and says so.
  
  It costs git, so it runs on a dead scope and on nothing else, and is skipped in
  silence where there is no repository to ask.
  
  What is covered is the scope field. A path, a symbol or a function named in a
  body is prose, and no finding pretends otherwise.
ratified: bfbdce721d85
schema: 2
version: 3
---

## Context

Scope is the only mechanism attaching an entity to code, and in practice it is
attaching by literal path: 343 of the 462 scope entries in this corpus contain no
wildcard at all, naming a single file such as `crates/ank-cli/src/human.rs`. One
`git mv` voids every one of them.

The detection already exists and already goes red. `check` reports a scope
matching no file, as a fault for an ADR or a finished task and as a signal for
work not yet started. So a rename does not pass silently. What the finding cannot
do is the only thing the reader wants at that moment, which is to say where the
file went — and the reader, increasingly, is an agent that will not go looking
through the history on a hunch.

The result is a finding that is correct, actionable in principle, and expensive
enough in practice that it accumulates.

## Why this is not the deferred scope-drift feature

Section 10 defers `touched` inferred from commits, under the heading of
scope-drift detection. That stays deferred and this is not it.

The deferred feature infers, from history, which entities a commit *should* have
touched — a claim about work, computed over the whole corpus, on every run. What
is added here is strictly narrower: a scope is already known dead, by the check
that already runs, and one question is asked about one path. Nothing is inferred
about work, no entity is implicated that was not already reported, and the cost
is bounded by the number of dead scopes, which is normally zero.

The distinction matters because the deferred row is deferred for a good reason —
inference about work is where a tool starts being wrong confidently — and this
must not be read as that door opening.

## Plumbing

`git log` is porcelain and stays forbidden. The walk is `rev-list` for the last
commit touching the dead path, then `diff-tree -M --name-status` on that commit
for a rename entry. Both have output stable by contract across versions, which is
the criterion, and both are listed.

Rename detection is a similarity heuristic, so it answers sometimes and not
always. A path that was deleted rather than moved, or moved together with enough
edits to fall under the similarity threshold, produces the finding as it is today
and no proposal. Absence of a proposal is never evidence the file was deleted.

## Rejected

**Repairing the scope automatically.** Section 11 is explicit that the tool
detects and proposes while a human ratifies, and this is the case that shows why:
a file moving out of a constraint's perimeter and a file being renamed inside it
look identical to `diff-tree`. Only a reader knows whether the constraint was
meant to follow.

**Anchoring the scope on content instead of on paths.** It was considered as the
structural fix and it is worse than the problem. Hashing the files a scope selects
would invalidate every scope on every ordinary commit, turning a rare true finding
into a permanent false one. Scope names a place, and places move; the answer is
cheap detection and a cheap repair, not immutability.

**Proposing `ank amend` for an accepted ADR.** It would refuse, because the scope
is hashed into the ratification commit. A hint that names a command which fails on
the spot is the one thing the error style forbids, so the ADR case says
supersession instead.
