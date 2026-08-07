---
id: TASK-fe130d2b732c
type: task
slug: the-flat-listing-names-every-verb-and-says-what
title: The flat listing names every verb and says what none of them does
created: 2026-08-07T17:02:44Z
author: seanl@sean-laptop
status: open
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-84cfad83c308]
done_criteria: |
  docs/ank-spec-v1.1.md sections 4 and 9 are rewritten first: ank help carries one short description per verb, and each description states what the verb refuses wherever a refusal is what distinguishes it. The listing stays flat, in the order of section 4, with no headings and no grouping: ADR-c656cbcc33a9 and ADR-e17e1bbd93ff are not superseded and the descriptions are not a layering. ank help amend does not describe amend as changing a criterion. Each description is the one-line compression of the text ank help <verb> already carries, and a test in crates/ank-cli/tests/cli.rs walks both through the binary and fails when a verb is described in the listing as doing something its own help says it refuses.
criteria_by: creator
schema: 2
version: 1
---

Raised from git's own help, which carries a one-line description per verb and
gives a caller enough to choose one. Ank prints the verb and its flags and
says what none of them does.

Two things are stacked in git's output and only one of them is available.

The grouping is settled against. The headings -- start a working area,
collaborate -- are what ADR-c656cbcc33a9 removed, and the revision k note says
why: revision i left a layered help whose headings were named after callers,
layering is grouping, and a grouping printed by the binary is a claim about
who a verb is for. The order of section 4 carries the same information without
asserting a category. Nothing here reopens that.

The description is a different axis: neither a heading nor a grouping, so it
costs no superseding ADR. The objection left is token economy on the overview,
and it is weak. One session spent more than twenty lines' worth of reading
human.rs, done.rs and editor.rs, plus a scratch repository, to recover what a
description would have carried. An economy that sends the caller to the source
is not one.

The trap is what the description says. Git's one-liners work because git's
verbs do what they announce. Ank's design is that verbs refuse on state, so a
description naming only a purpose becomes a fourth surface able to
misinform -- the exact defect TASK-84cfad83c308 records on amend, where the
flag table advertises a criterion edit the binary refuses always. 'Change a
task's scope, blockers or criteria' would have confirmed that error rather
than caught it. 'Change a task's scope or blockers; never its criteria' is the
same line doing the work.

So for this tool the description and the refusal are not two features. Writing
an honest one-liner forces the refusal to be stated, which is why this is
blocked rather than parallel: TASK-84cfad83c308 settles the per-verb text
first, and each listing line is its compression. Written in the other order
they are two texts that drift, and the drift is silent.
