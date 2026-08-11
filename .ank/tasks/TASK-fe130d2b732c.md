---
id: TASK-fe130d2b732c
type: task
slug: the-flat-listing-names-every-verb-and-says-what
title: The flat listing names every verb and says what none of them does
created: 2026-08-07T17:02:44Z
author: seanl@sean-laptop
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
blocked_by: [TASK-84cfad83c308]
done_criteria: |
  docs/ank-spec-v1.1.md sections 4 and 9 are rewritten first: ank help carries one short description per verb, and each description states what the verb refuses wherever a refusal is what distinguishes it. The listing stays flat, in the order of section 4, with no headings and no grouping: ADR-c656cbcc33a9 and ADR-e17e1bbd93ff are not superseded and the descriptions are not a layering. ank help amend describes the criterion as the verb actually treats it, changed only where no live claim freezes it, and never as an unconditional edit. Each description is the one-line compression of the text ank help <verb> already carries, and a test in crates/ank-cli/tests/cli.rs walks both through the binary and fails when a verb is described in the listing as doing something its own help says it refuses.
criteria_by: creator
proof:
  - type: test
    ref: "31455480061"
    criteria: d3521eadc0f8
schema: 2
version: 9
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

## Log
- 2026-08-11T03:13:47Z seanl@sean-laptop — released: One clause of the criterion was overtaken by TASK-7c2fa14284ff, done two hours ago: it requires that ank help amend not describe amend as changing a criterion, and amend now changes a criterion no live claim freezes. Obeying the clause would mean writing a description that misdescribes the verb, which is the defect this task exists to prevent. Releasing to correct that one clause through amend --criteria, then re-claiming. Nothing else in the criterion moves.
- 2026-08-11T03:13:57Z seanl@sean-laptop — amended: done_criteria
- 2026-08-11T03:14:07Z seanl@sean-laptop — amended: +scope crates/ank-cli/src/cli.rs
- 2026-08-11T03:16:22Z seanl@sean-laptop — Started by correcting the criterion through the route TASK-7c2f built two hours ago, on a real case rather than a fixture: one clause required that ank help amend not describe amend as changing a criterion, and amend now changes one no live claim freezes, so obeying the clause meant writing a description that misdescribes the verb. Released with the reason, amended the clause, re-claimed; criteria_by stayed creator and the whole trail is in the log. Direction settled with the maintainer: in the flat listing the description takes the place of the flag names, git-style, and the flags stay one ank help <verb> away where they carry their placeholders and their refusals. The listing keeps its height and gains its meaning, which is the token-economy objection section 9 line 1083 currently states, answered rather than ignored. The mechanical rule the test needs: a listing description may not name a flag the verb does not offer. That is exactly the amend defect of TASK-84cfad83c308 -- the old --criteria was declared refused, listed:false, so a description naming it would have failed -- and it catches init naming --repo through refuses_globals. It is checkable through help --json, which already carries flags and refuses per verb.
- 2026-08-11T03:27:37Z seanl@sean-laptop — Implemented with no new field: the listing prints spec.summary, the same string ank help <verb> prints above the flags. One text rather than two is stronger than any test comparing them, and the test asserts the identity anyway so a future renderer cannot paraphrase. Descriptions too long for the column are folded on words and indented under themselves, never truncated -- the clause a verb refuses on is always the tail of the sentence, which is exactly where truncation would drop it. Six summaries gained a refusal clause where the refusal is what distinguishes the verb: claim, log, done, accept, close, init. The mechanical rule is that every --flag a description names is offered by the verb or preceded by the word refuses, checked against the flags and global lines of the verb's own page. Proven to bite before being trusted: rewriting init's description to say '--repo names the target' fails with the verb, the flag and what the verb actually offers. Three existing tests asserted flags in the flat listing and now assert them on the per-verb page, which is where they moved -- help_lists_every_verb_of_the_table, help_answers_outside_a_repository, and the surface() helper two path-classification tests walk.
- 2026-08-11T03:31:20Z seanl@sean-laptop — done, proof test:31455480061
