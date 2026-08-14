---
id: TASK-f6b8eb330be5
type: task
slug: a-criterion-proved-partly-wrong-is-recorded-on-t
title: A criterion proved partly wrong is recorded on the task without releasing it
created: 2026-08-13T16:25:08Z
author: claude-code/2.1.229
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-core/src/log.rs
  - crates/ank-core/tests/golden.rs
  - crates/ank-cli/tests/cli.rs
  - docs/format.md
  - CLAUDE.md
  - crates/ank-core/tests/golden/**
blocked_by: []
done_criteria: |
  Section 3 of docs/ank-spec-v1.1.md states how a holder records that a frozen criterion rests on a false premise, what that record does and does not change, and why it is neither an edit to the criterion nor a release. The frozen hash is untouched by it, and the specification says so.
  
  The record is readable by check, which reports it as a signal on the task, and it is visible to a reader of the entity. It is never a condition of done and never weakens what done verifies.
  
  The format moves first: the specification, then the goldens, then the code, and the round-trip stays byte-identical on canonical form.
  
  Asserted through the binary: a task carrying the record still freezes and verifies its criterion by the same hash, and check names it.
criteria_by: creator
proof:
  - type: commit
    ref: 4c31f803c759fcd861f6c49c02d561c892e68731
    criteria: ce3aa0e23860
    via: submitted
schema: 3
version: 5
---

Two of the three parallel sessions met this and neither had a tool for it.

One found a criterion requiring that CLAUDE.md stop instructing an agent to carry
a CI run id by hand. CLAUDE.md never carried that instruction: the clause was
unsatisfiable as written, one of four, and the other three were met. `release
--reason` is the documented exit and would have thrown away work that was
otherwise correct. The agent recorded the discrepancy with `ank log` and said so
in the pull request, and wrote that this was a convention it invented rather than
something the tool offers. Another found a criterion demanding an invalid fixture
that a binding ADR forbade, and resolved the contradiction in prose.

**The frozen criterion is the thing to keep, and this must not weaken it.** Both
sessions said independently that the freeze was the system working: once it
forced a documented judgement call instead of a quiet edit, once it forced a
measurement that overturned the assumption the task was written on. A field that
let a holder mark a clause "wrong" and proceed would be the criterion made
editable through a side door, which is the whole failure the hash exists to
prevent. So the record changes nothing mechanically: the hash still anchors,
`done` still verifies against it, and what the record buys is that the
disagreement is in the corpus rather than in a pull request comment nobody
reopens.

**Settle the shape in the specification before adding any field.** `ank log`
already carries this today, and the honest question is whether a first-class
field earns its cost over the convention -- what `check` could say about a
logged discrepancy that it cannot say about a field, and the reverse. Section 3
is where that is decided, and the criterion above requires the answer to be
written before code moves. It is legitimate for the answer to be that the log
entry is the record and what is missing is only that `check` reads it.

This is a format change either way, so the order is not negotiable: the
specification, then the goldens written to fail, then the code.
