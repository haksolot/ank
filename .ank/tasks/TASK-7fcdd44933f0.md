---
id: TASK-7fcdd44933f0
type: task
slug: the-specification-carries-the-flat-layout-the-ki
title: The specification carries the flat layout, the kind registry, the log file and typed actors
created: 2026-08-11T22:26:44Z
author: claude-code@sean-laptop
status: in_progress
scope:
  - docs/ank-spec-v1.1.md
  - docs/format.md
blocked_by: []
done_criteria: |
  docs/ank-spec-v1.1.md states, normatively: the layout .ank/entities/<ID>.md
  with no per-kind directory (section 6); entity kinds as a registry declaring name,
  id prefix, required and optional fields and canonical field order, with unknown
  fields still rejected inside a known kind (section 3); the log as .ank/log/<ID>.md,
  append-only, line grammar unchanged, no longer part of the body (section 3); the
  actor convention and the optional verified list (section 3); schema 3 as the
  version carrying the last two, with the reader range unchanged in its lower bound
  (section 3).
  
  Section 10 keeps its 'do not anticipate' row on additional entity types and keeps
  the deferred scope-drift row, and says in both cases why the changes above do not
  lift them.
  
  docs/format.md is updated to match, field order tables included, and states the
  dual-read window for the previous layout.
  
  No source file changes in this task. ank check exits 0 on the corpus, which is
  still in the old layout and must stay readable.
criteria_by: creator
schema: 2
version: 3
---

First step of the format change, and the order is not negotiable:
specification, then goldens, then code (ADR-63b59c5c26f7). Nothing in
`crates/` moves here.

Implements the documentation half of ADR-c9f9d0d6f05d, ADR-ff294eff4d1a and
ADR-3877fef1d662. All three land in one schema bump, 2 to 3, rather than three
passes over the goldens. The layout change carries no bump on its own — it moves
files, not fields — but it ships with the others because splitting the migration
in two would move the corpus twice.

Section 3 is where most of the work is, and the trap is the reader range. The
lower bound does not move: every field introduced after version 1 stays optional
at parse time and its absence keeps meaning "written before this existed". What
must be stated plainly is why the log leaving the body needs the bump at all —
a reader that does not know would show an empty history for a task that has one,
silently, and refusing on the version is what the format does with exactly that
case.

Section 10 is the part most likely to be got wrong by being helpful. The registry
makes a new entity kind cheap; it does not make one wanted. TASK-00660963bcce
records the rejection of a `spec` kind and that reasoning is untouched. Say so in
the row rather than deleting it, or the next reader will take the registry as an
invitation.

Same for the deferred scope-drift row: rename detection on an already-dead scope
is strictly narrower than inferring `touched` from commits, and the row stays.

`docs/format.md` is not normative and is still the document a third-party writer
actually reproduces from, so a field order table that disagrees with the
serializer is a defect even though the specification is what settles it.

## Log
- 2026-08-13T04:04:01Z claude-code@sean-laptop — Spec revision p written: flat entities/ layout and kind registry (sections 3 and 6), the log as its own file replacing the body section, typed actors and the optional verified list, schema 3 with the reader range 1 to 3 and the lower bound held. Section 7 loses the log-union merge rule, which existed only because the log shared a file with unmergeable fields. Section 10 keeps both deferred rows and now says why the registry and rename detection do not lift them. format.md updated: field order tables carry verified, the log section is rewritten as a file, and the dual-read window is stated. The conformance list names the new invalid cases, which TASK-9146 makes real.
