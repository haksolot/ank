---
id: TASK-072c2ccc524d
type: task
slug: check-names-a-corpus-behind-the-default-branch-a
title: check names a corpus behind the default branch, and status says how far
created: 2026-08-13T16:22:26Z
author: claude-code/2.1.229
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  check emits a signal, once for the corpus, when an entity file in the working tree differs from the same file on the default branch or is absent from one of the two, naming the count and the git command that closes the gap. status carries the same fact on one line. Neither fetches: the assertion that no network call is made is made by the test, not assumed.
  
  Where the default branch cannot be resolved, or there is no repository, the question is skipped in silence rather than warned about, on the same rule the rename walk already follows.
  
  Section 4 of docs/ank-spec-v1.1.md lists the signal among what check reports, and section 10's deferred --since row records that its trigger was observed twice and that it stays deferred, with this signal named as what answers the cheap half.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs: a clone whose corpus is behind its default branch, one that is ahead, one that is level, and one with no resolvable default branch.
criteria_by: creator
proof:
  - type: commit
    ref: 5ae180f46367b9aea065846de0e25a7fdcd65996
    criteria: 5b709261f76a
schema: 3
version: 3
---

Implements ADR-47e2ac102f58. This is the finding that caused a failure rather
than friction, and the only one of its kind in three reports.

Reuse `git::file_at(cwd, rev, path)` (git.rs). It reads a file at a revision
through `cat-file`, it is already what the pruning predicate uses, and it already
distinguishes an absent path from an unresolvable revision -- so a mistyped
`default_branch` cannot read as "nothing has moved". That distinction is the one
worth keeping: this signal is about a difference, and an error about the
revision must never render as one.

**Once for the corpus, never per entity.** Section 4 fixed this rule twice
already, for entities predating `author` and for pre-convention actor values, and
for the same reason each time: one line per file is the volume that teaches a
reader to stop reading `check`. A corpus six entities behind the default branch
would otherwise print six lines saying one thing.

The count is what makes it actionable, and the count is a comparison and not a
history walk. Do not reach for `rev-list` to say how many commits behind: the
question is about the corpus, a branch can move ten times without touching
`.ank/`, and an answer in commits would fire constantly and mean nothing.

Do not fetch, and assert that. A verb that fetched to answer would rewrite the
plane underneath every other agent in the clone -- the same argument that made
`status --remote` read with `ls-remote` and that keeps `check` from touching
files while it prunes refs. The test must fail if a fetch is introduced, which
means asserting on the absence of the network call rather than on the output
alone.

The `--since` row of section 10 is edited in the same pass, and only to record
what happened: two of three parallel sessions asked for "what moved since I last
looked", the trigger was therefore observed, and the row stays deferred because
`--since` needs per-agent seen-state where this needs none. A deferral whose
trigger has fired and is not written down is how a document starts disagreeing
with the people using it.
