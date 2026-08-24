---
id: TASK-ecf0f37f68c9
type: task
slug: context-json-is-not-budgeted-and-adr-3e6ce108edc
title: context --json is not budgeted, and ADR-3e6ce108edcd says it is
created: 2026-08-24T02:21:00Z
author: claude-code/opus-5-json-budget
status: in_progress
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  context --json is served under context_budget, the same page the human render already spends, and a test drives the binary on a corpus past the budget and asserts that the --json document carries what the human page carried and no more. The four listing verbs stay whole under --json. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 4
---

ADR-3e6ce108edcd exempts `context` from the rule it lays on the four listing
verbs: "context keeps the budget under --json, because deciding what a reader is
handed first is that verb's answer rather than a limit on it".

**The exemption describes code that does not exist.** Measured while
implementing TASK-652de6ead019, on a fixture of twelve tasks and twelve
proposed decisions at `context_budget: 400`:

    ank context          4 tasks, 0 proposals, "+12 not shown", "+8 more tasks"
    ank context --json   all 24 rows

`context.rs` reads `cfg.context_budget` at exactly one line, the human
`render`. `render_json` takes no budget argument and never has: it was
introduced whole in 6ec4f70 and the call site has not changed since. So this is
not a regression to restore, it is a rule that was written down as though it
already held.

**Filed rather than folded into TASK-652de6ead019.** That task's criterion says
`context --json` "stays budgeted", which is not a thing it can stay. Closing
the gap means giving `render_json` the budget, which means lifting the fitting
decision out of `render` so both surfaces spend the same page, and that is a
change to what a machine surface returns rather than a change of default. It
wants its own claim and its own review.

Two directions exist and this task does not pick one. Either the code learns
the ADR, or the ADR learns the code and is amended to say the budget is the
human page and `context` is no exception either. The measurement above is the
same argument for both; what must not survive is the corpus asserting one and
the binary doing the other.
