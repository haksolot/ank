---
id: ADR-302efb9c5e40
type: adr
slug: done-and-close-archive-the-entries-of-the-task-t
title: done and close archive the entries of the task they end, and an entry about a finished task is born in the archive
created: 2026-11-21T19:44:17Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-cli/**
  - docs/**
constraint: |
  An entry whose subject is a task that is done or closed in the tree it sits in is cold, with no reference to the default branch. done and close move every entry about the task they end into .ank/archive/entities/ in the same act, their own closing entry included, record each digest as ank archive does, and commit nothing: the move lands with the change that carries the close. A verb that writes an entry about a finished task, attest and amend among them, writes it in the archive. A task is never cold, and ank archive stays for a superseded document and for what predates this rule. graph prints what is not finished and the finished tasks that block it; --all prints the whole DAG.
amends: [ADR-467ce7e9cda1]
ratified: c17d94f9fd1a
anchor: 2cf89e8441741293339a64100b47527eb2a9b225
verified:
  - by: claude-code/fable-5.2+ratify
    at: 2026-11-21T22:13:07Z
schema: 4
version: 2
---

Issue #832. Amends ADR-467ce7e9cda1 on when the cold set moves; its directory, digest, reading rule and budget stand.

Measured on 1dbd58ff, this repository: `ank archive --dry-run` reports 219 cold entities left hot, log entries are 3 289 of the 4 191 tracked files under `.ank/`, and `ank graph` prints 75 KB with one task open.

The move was a separate sweep that lands by pull request, so it happens when somebody remembers it. The event that makes an entry cold is known to the binary at the moment it happens: `done` or `close` ends the task. Moving there costs no extra pull request and no job racing `attest`.

The amended ADR defines cold against the default branch. A move made by `done` on a task branch would then be an archived entry about a task the default branch does not know is done, until the merge. The definition becomes local to the tree: the entry and the status that makes it cold travel in one change, so every tree is consistent with itself.

Decided with the maintainer on 2026-11-21: tasks stay hot, because finished scopes feed the suggestions of `new task` and the dead-scope checks; what an agent pays for them is the size of `graph`, which is why its default narrows.
