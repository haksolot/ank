---
id: TASK-345c35a8beba
type: task
slug: the-orientation-budget-cuts-a-page-that-already
title: The orientation budget cuts a page that already fitted
created: 2026-08-24T04:20:36Z
author: claude-code/opus-5-context-budget
status: open
scope:
  - crates/ank-cli/src/context.rs
blocked_by: []
done_criteria: |
  On a perimeter whose orientation page fits within context_budget, context cuts nothing on either surface, and a test pins it on the golden corpus at the budget golden_repo declares. A cut that does not reduce the rendered page is not taken. The floor of SPEC-a1234da5449a is unchanged: one constraint and one task always survive, and a page genuinely over budget is still cut. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

Measured on the golden corpus of `tests/cli.rs` (one accepted constraint, one
proposed decision, one spec, four open tasks over `src/**`) at the
`context_budget: 400` that `golden_repo()` pins:

    uncut page   381 characters, one of everything, nothing cut
    at budget 400  373 characters, and five rows gone

The page **already fitted**. The budget cut a spec, a proposal and three tasks
off a page eighteen characters under the limit, and the result is *shorter by
eight characters than the cut it paid for*.

**Two defects compose.** The first is in the first-half loop:

    while !specs.is_empty()
        && priced(&constraints, &specs, cut_constraints, cut_specs + 1) > share

`priced` is handed the **full** spec list *and* `cut_specs + 1`, so it renders a
state that never exists: every row still present, plus a `+1 not shown` notice
for a row that was not removed. On this corpus the real section costs 98 against
a share of 133 and the hybrid costs 149, so a section that fitted with 35
characters to spare is cut. The post-cut state is `priced(specs_after_pop,
cut_specs + 1)`; the pre-cut state is `priced(specs, cut_specs)`. The condition
asks for neither.

The second is that **a cut notice costs more than the row it replaces**, so
cutting inflates the page and the second-half loop cascades:

    spec row      24 characters  ->  `+1 not shown, ank find --type spec ...`      51
    proposal row  30 characters  ->  `+1 not shown, ank find --type adr ...`       69
    task row      45 characters  ->  `+3 more tasks, ank find --type task ...`     49

Each cut makes the page larger, the loop sees it still over budget, and it
descends to the floor of one constraint and one task. The loop has no guard for
"this cut did not reduce the page", so it cannot stop.

**Why it surfaced now.** It has been the human page's behaviour since the budget
was written, and nothing pinned the human page of `context` on a corpus close to
the limit. TASK-ecf0f37f68c9 gave `--json` the same fitting decision
(ADR-3e6ce108edcd), which put the cut rows in a golden fixture where the
conformance test walks them: `every_golden_conforms_to_the_shape_its_verb_declares`
now reports `context.proposed` and `context.specs` as declarations no fixture
reaches, because the budget emptied both arrays.

That failure is the symptom. Raising the fixture's budget hides it; fixing the
arithmetic removes it, and `tests/golden-json/context.json` then returns to the
one-of-everything document it held before, with no fixture change at all.

**What must not be lost.** §5 keeps its floor — one constraint and one task
always survive — and keeps cutting where a page genuinely does not fit. What
changes is that a page which fits is not cut, and that a cut which does not
shrink the page is not taken. `the_budget_still_governs_the_page_context_serves`
in `tests/cli.rs` pins the genuinely-over case at twelve tasks and twelve
proposals; its counters will move if the arithmetic changes, and the diff is to
be read rather than blessed.
