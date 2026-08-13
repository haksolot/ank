---
id: TASK-1ead0e19fb73
type: task
slug: orientation-spends-its-budget-on-constraints-and
title: Orientation spends its budget on constraints and leaves nothing for the choice
created: 2026-08-11T03:48:12Z
author: seanl@sean-laptop
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Section 5 of docs/ank-spec-v1.1.md says how orientation mode allocates its budget between constraints and tasks, distinctly from execution mode, and the reasoning names what orientation is for. The measurement that settles it is recorded in the task log: what a cold ank context costs on this corpus today against what it costs with a perimeter given. The code follows, and a test through the binary asserts the allocation the specification now promises on a corpus large enough to exceed the budget.
criteria_by: creator
proof:
  - type: test
    ref: "31675652087"
    criteria: fa692895d3a2
schema: 3
version: 6
---

Observed on a cold pull of this repository, and observed again today: `ank
context` at the root answers with twelve constraints rendered in full, several
of them multi-paragraph, followed by one task line and `+13 more tasks`. The
proportion is inverted for the call the loop opens with. An agent that has not
yet chosen a perimeter receives every rule in the corpus at full length and
almost none of the work.

The two modes are not the same question and the budget does not currently
distinguish them. In execution mode a constraint is rightly never truncated:
the perimeter is settled, the rules bind the work in hand, and cutting one
would hide a rule the agent is about to break -- §4 says so and that is not in
question here. Orientation is the opposite situation: nothing binds yet,
because nothing has been chosen.

What orientation is for is choosing. That argues for the inverse allocation --
constraints named and summarised, tasks listed in enough number to pick from --
with the full text one `ank show` away, which is the same split §9 settled for
`help` and its per-verb page.

This is a specification question about §5 rather than a rendering preference,
and it should be measured before it is answered: what a cold `context` costs
today, on this corpus, against what the same call costs once a perimeter is
given.
