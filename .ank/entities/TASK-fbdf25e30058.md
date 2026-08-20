---
id: TASK-fbdf25e30058
type: task
slug: the-shape-walk-counts-empty-instances-so-an-exer
title: The shape walk counts empty instances, so an exercised shape reads as unexercised
created: 2026-08-20T05:00:48Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  every_golden_conforms_to_the_shape_its_verb_declares reports an array path as unexercised only when no instance of that path, in any fixture it walks, carries a row; a path some instance fills is not listed because another instance is empty. A path no instance anywhere fills is still named, and the pinned list is asserted as whatever that reading produces rather than loosened to a count or a subset. The comment beside the assertion says which of the two readings it is and why. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 2
---

Measured while closing TASK-106dccc7f71c, which declared the refusals six verbs
performed and left `status` with an honest empty array.

`conforms` pushes a path onto `unverified` every time it meets an empty array:

    if rows.is_empty() {
        unverified.push(here.clone());
    }

So `help.verbs[].refuses` is reported as unexercised because one verb of the
twenty-two declares no refusal, while the twenty-one others carry rows in that
same document. The element shape is reached, parsed and checked by the walk
itself, one line further down, and then reported as unseen.

The test's own docstring states the rule it means: "An empty array in a fixture
cannot show the shape of its elements, so the rows underneath it go unverified
here." That is a statement about a shape, not about an instance. A shape one
instance fills is shown; the reading that counts instances answers a question
nobody asked, and it answers it wrongly.

The consequence is not academic. The list is pinned rather than counted, on
purpose, so that a fixture which starts exercising a shape turns the test red
and has to be acknowledged. A list carrying a path that is in fact exercised
spends that mechanism on a false entry: the next reader has to go and measure
which of the two it is, which is exactly the work the pinning exists to save.

The fix is a reading, not a rewrite. What must not happen is emptying the list
by inventing a refusal for `status`, a verb that refuses on nothing and whose
empty array is a fact about it (TASK-106dccc7f71c). Nor by dropping the pin for
a count, which is the mechanism this test was given rather than a detail of it.
