---
id: TASK-cd3189ddf61e
type: task
slug: the-store-writes-the-flat-layout-reads-both-and
title: The store writes the flat layout, reads both, and check names the leftover
created: 2026-08-11T22:28:04Z
author: claude-code@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-7c1ff5035894]
done_criteria: |
  Every write lands in .ank/entities/<ID>.md and no write ever produces
  .ank/tasks/ or .ank/adr/. A corpus in either layout is read, and a corpus holding
  both is read as one corpus with no entity counted twice.
  
  ank check reports a corpus still in the previous layout, once, naming the command
  that moves it, and reports it as a signal rather than a fault -- a corpus that
  still reads is not broken.
  
  check reports the entities whose author predates the actor convention once for
  the corpus and never per file, and reports an agent-authored entity carrying no
  human reading as a signal.
  
  index.db rebuilds from either layout with no migration, and deleting it stays
  safe.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs on three fixtures: old
  layout, new layout, and both at once.
criteria_by: creator
schema: 2
version: 3
---

Fourth step, and the last one before this repository's own corpus moves.

The dual read is a window, not a feature. Nobody outside this repository has a
corpus yet, so the window exists for one release and for early adopters who have
one; say so where the code implements it, or it becomes permanent by inertia.

The both-at-once case is the one that bites. An id resolving in two directories
must not produce two entities, and it must not silently prefer one and drop the
other — that is how a corpus develops two versions of a task that disagree. Decide
which wins, state it, and test it.

`load_path` already refuses a file whose name does not match the id inside it, and
that check is unaffected by the move. Keep it exactly as strict: with the
directory no longer carrying the kind, the file name is the only thing left
stating it, and the cross-check is what makes that safe.

The leftover-layout finding is a signal and not a fault, deliberately. A corpus in
the old layout still parses, still round-trips, still answers every verb. Turning
that into exit 8 would redden a pipeline over a file location, which is the kind of
finding that teaches a reader to stop reading `check`.

The two author signals follow rules `check` already applies elsewhere. Reporting
the pre-convention set once for the corpus is the same choice already made for the
48 entities predating `author`, and for the same reason: one line per file adds a
line for every file written before the rule existed.

`index.db` is derived and carries the path already. Confirm it rebuilds rather
than assuming it; a stale index that resolves to a moved file is a bug that will
look like a parser problem.

## Log
- 2026-08-13T05:53:23Z claude-code@sean-laptop — Store, index and check read both layouts and write only the flat one. Decisions: the flat copy wins when an id resolves in both, because every write lands there so it is the newer by construction; and a write of an entity still in the previous layout removes that file in the same operation, so the both-at-once state is never something the ordinary loop produces. Interrupted between the two acts leaves the entity in both places, which read_path_of already resolves and which heals on the next write -- the other order would lose the entity. Three defects surfaced that the layout change would have hidden: git::ratification_at memoises by (cwd, id) and not by path, so looping candidate paths at the call site cached the first miss and read every ratification in this repository as unverifiable; maintain() built its own tasks/<id>.md path rather than asking; and git add refuses a pathspec matching neither tree nor index, so accept stages the previous layout's path only when a file is actually there. Two things outside this task's scope and both forced: init created tasks/ and adr/, which is a writer producing the layout no writer produces, and it now creates entities/ and log/; and the not-implemented hint named a .ank path, which ADR-01b6dd05f0db says nothing should. The log wiring is not here: it touches commands.rs, done.rs and context.rs, so it is TASK-e70f3a12185a, and TASK-9bff now waits on it because a corpus whose logs move needs a CLI that reads them.
