---
id: TASK-df9c6d46e8ef
type: task
slug: log-entries-become-entities-and-the-corpus-migra
title: Log entries become entities, and the corpus migrates without losing one
created: 2026-08-15T06:56:47Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
blocked_by: [TASK-e8df857e87d7, TASK-3e68786fa443]
done_criteria: |
  A log entry is written as an entity naming the entity it is about, and nothing is appended to an existing file. ank log with an id and ank show read the entries back, newest first and oldest first respectively, capped and announcing what they cut. Entries are indexed and reachable through ank find. Any kind carries entries, an ADR included. The entries the corpus already holds are migrated, with their count asserted equal before and after and no message altered, and the previous log directory is no longer written. A test in crates/ank-cli/tests/cli.rs drives the built binary for write, read, show, find, the ADR case, and two concurrent entries merging with no conflict.
criteria_by: creator
schema: 3
version: 3
---

The deepest change of the group, and the last, because it moves 27 percent of the
corpus by volume.

**Why an entry and not a log.** An entity carries `version`, incremented on every
write under compare-and-swap. Making the per-entity log file an entity would turn
every append back into a transition, which is exactly what ADR-ff294eff4d1a
removed — and that ADR already refused the half-measure of keeping the section
and merely not bumping `version`, on the grounds that it would make `version`
mean changed, except sometimes. An entry, written once and never modified, has no
such problem: there is no file to append to, so append-only stops being a
convention the format requests and becomes a property of the storage.

**What it buys, and each of these is a gap measured today.** The log is not
indexed at all — the scan opens three directories and never the log directory —
so no question about it has an answer inside the tool. Entries gain an id, an
author, a timestamp and a scope, which is what makes them reachable by `find` and
what lets them cross a repository boundary like anything else. Two concurrent
entries become two new files, so the conflict genuinely disappears instead of
being asserted away. And a second party finally has somewhere to write: the ADR's
own example, a pipeline recording that it ran, is today served by a ref because
`attest --detached` writes no entry at all.

**The migration is the risk, and the criterion is written around it.** 487
entries across 168 files. Assert the count equal before and after and assert no
message altered — a migration that silently drops one entry is worse than no
migration, because the loss is invisible and permanent. The strict parser refuses
a whole file on its first malformed line, so a file that fails to parse must stop
the migration and name the file, never be skipped.

**Do not lose the ordering.** Entries carry an ISO timestamp kept as written and
never reformatted. `log` reads newest first and `show` oldest first, and both
orders come from the timestamp, not from a directory listing. Two entries in the
same second are possible and the order between them must be stable rather than
whatever the filesystem returns.

**Caps.** These two readers are the only ones in the tool with no budget, and
after this change they read through an index that will happily return everything.
TASK-6c0463fb4319 caps them first; do not undo that, and make sure the cap
survives the new read path.

**Any kind carries entries**, an ADR included. The refusal that named a task by
name was filed as TASK-3a0347e72bf3 and is closed by this one: the guard does not
get lifted, the storage it guarded stops existing.

The previous log directory stops being written. Whether it stops being *read* is
a judgement call this task should make explicitly and record: a corpus written by
an older build still has one, and the same courtesy the store already extends to
the previous entity layout is the precedent to follow.
