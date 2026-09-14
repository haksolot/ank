---
id: ADR-467ce7e9cda1
type: adr
slug: the-corpus-has-a-cold-half-an-entry-is-cold-with
title: The corpus has a cold half, an entry is cold with its subject, and its weight is a finding
created: 2026-09-14T13:18:05Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-cli/**
  - docs/**
constraint: |
  .ank/archive/entities/<ID>.md is a second fixed, flat directory in the same format as .ank/entities/, one file per entity, readable with no parser. The index scan does not walk it unless asked; show, log and find --all resolve into it; check verifies an archived file by digest and never re-parses it, reports an archived file that changed as a fault, and resolves a reference, a supersession, a blocker and a log subject against both roots, so that naming an archived entity is never reported as naming nothing. What is cold is a superseded document, and a log entry whose subject is cold: an entry about a superseded document, or about a task done on the default branch. A task is never cold, and an entry never outlives its subject in the hot corpus. ank archive moves the cold set, commits nothing and lands by pull request; check reports a hot entity that is cold by this rule as a signal naming that verb. config.yml declares weight: {hot_files, plane_bytes}, and check signals a corpus over either, naming what grew.
supersedes: ADR-306fdb75e265
ratified: b80ffb34e497
verified:
  - by: haksolot@vmi3223161
    at: 2026-09-14T13:32:30Z
schema: 4
version: 2
---

Supersedes ADR-306fdb75e265 on one point, measured before `ank archive`
existed to move anything (LOG-91bf1a8abb90, on a scratch clone of this
repository): the 32 superseded specs moved by hand into the archive took
`check` from 0 faults to 31. Thirteen references and seven supersessions
named a document "which does not exist", seven hot log entries were about
an entity the corpus "does not hold", and ten hot entries carried as their
scope the file of the spec they annotate, now a dead path.

## What the predecessor got wrong, and why the correction is small

The predecessor listed what is cold: a superseded document, and an entry
whose subject is a task done on the default branch. It left the entries of
a superseded document hot, and an entry is the trace of its subject (§3:
an entity's entries are a query on `about`), copied its subject's scope
when it was written, and answers `ank log <id>` beside it. Splitting the
two puts the trace on the path every verb walks while the thing it traces
is off it, and turns the copied scope into a dead one. So an entry is cold
with its subject, whichever kind the subject is.

The second half is a reading rule the predecessor implied and did not
state: a reader that resolves into the archive by id must do so wherever
an id is resolved, `check` included. A resolution that reads only the hot
corpus reports the archive as nonexistence, which is the one answer worse
than a slow one.

Everything else stands as ADR-306fdb75e265 wrote it: the directory, the
digest, the verb, the budget, and why an archive directory rather than a
packfile or git history.
