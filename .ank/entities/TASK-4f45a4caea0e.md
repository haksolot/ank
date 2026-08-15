---
id: TASK-4f45a4caea0e
type: task
slug: find-lists-what-the-coordination-plane-speaks-fo
title: find lists what the coordination plane speaks for and never names --free
created: 2026-08-15T06:08:26Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  ank find writes one final line, after the truncation line, whenever the rows it listed include tasks whose stored status is open and whose coordination record is a live claim or a completion. The line counts them and names --free. It is absent under --free, under --json and under --quiet, and absent when no listed row matches. Section 4 of docs/ank-spec-v1.1.md carries the sentence that fixes the line, beside the one that fixes the hidden count. A test in crates/ank-cli/tests/cli.rs drives the built binary: it seeds a completion ref and asserts the marker and the new line under find --status open, their absence under find --free, and no new line under find --status done.
criteria_by: creator
schema: 3
version: 3
---

Measured on a real session, then reproduced in a sandbox. A checkout behind the
default branch ran `ank find -s open` and got thirteen rows, ten of them
displaying `[finished:<sha> on <branch>]`. Every row was correct: the filter
compares the stored status, the marker comes from the ref plane, and
`marker_for` discards the stored status in its `Finished` arm. The reader's
conclusion was that the filter was broken.

It was not, and the mechanism is the one section 7 designed: the file says
`open` because the `done` lives on a branch this checkout does not have, and the
ref says otherwise so nobody redoes the work. Excluding those rows from
`--status open` would put that window back, so this task does not do it.

What is missing is the way out. `--free` already answers exactly the question
the reader was asking — it drops the rows the coordination plane speaks for
(`blocks_readiness`) and the ones whose scope meets a live claim — and nothing
in the listing says so. The listing already teaches its own two ways out on the
lines below it, `+N more, narrow with --scope <path>` and `N hidden, scope
overlaps a live claim`, and this is the third of the same kind.

Three decisions the implementation should not re-open.

The count is over all the hits, before the cap, like `hidden`. `find --status
open` is the listing of what remains (`cli.rs` summary), so the count answers
about what remains, not about what fits on the screen.

The filter is narrow on purpose: tasks only, stored status `open` only. A
`find -s done` listing also shows `[finished:...]` rows, because the completion
ref outlives the merge until `check` prunes it, and pointing that reader at
`--free` would name a command that keeps only `open` tasks and would therefore
answer a different question. Section 7 already states the rule for hints: never
name a command that would refuse on the spot.

The line is one line and it does not repeat what the markers already said. The
grammar of the two neighbours is a count, a state, a way out.

Reproduced end to end in a sandbox of two clones and two worktrees: the row
appears as `[finished:<sha> on <branch>]` under `-s open` while the file reads
`open`, `--free` answers `no match` in silence, `status` says `1 finished
elsewhere`, `claim` refuses with code 4, and `check` prunes the ref once the
merge lands, after which the same row reads `[done]`.
