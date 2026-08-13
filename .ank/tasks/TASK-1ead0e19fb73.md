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
schema: 2
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

## Log
- 2026-08-13T06:12:16Z claude-agent-b — Measured before answering, as the criterion asks. Corpus: 18 accepted constraints, 11 open tasks, context_budget 8000 characters. Cold orientation at the root spends 7357 characters on constraints and 157 on tasks -- 96 percent against 2 percent. Seven constraints are rendered in full and eleven are not shown at all; one task line is printed and eleven are cut. The PROPOSED section is cut to a single 'not shown' line as well. With a perimeter given (crates/ank-cli/src) the allocation is unchanged: 7373 against 181, seven constraints in full, still one task shown. That is the finding the task body only hypothesised. Narrowing the perimeter narrows the candidate set -- eleven tasks cut becomes five -- but frees no budget for them, because the constraints are charged first and at full length either way. So a perimeter does not fix it and cannot: the allocation is the defect, not the breadth. The sharpest way to state it is the last line of the output. Orientation ends with 'ank claim TASK-10b8 to start', recommending the one task it had room to print. The mode whose whole purpose is choosing presents exactly one candidate out of twelve, and an agent reading it has no way to know eleven others exist without running a second command. Execution mode is untouched by any of this and must stay so: there the perimeter is settled and a truncated constraint would hide a rule the agent is about to break.
- 2026-08-13T06:41:32Z claude-agent-b — Specification, then code, then test, in that order. Section 5 now divides the budget by opposite rules for the two modes and says why: execution never truncates a constraint because the perimeter is settled, orientation gives constraints at most a third because nothing binds yet and a page of rules with no work to apply them to is not a choice. The third is the mirror of the over-constrained threshold already in the section -- execution calls a scope sick above a half, so the mode where none of them bind is held to something stricter. The cutting order moved into Truncation priority where it belongs, and its first line inverted: it read 'Tasks first, before any constraint', which is the sentence the code was faithfully implementing. In orientation a constraint is now one line, id and title, exactly as the neighbouring PROPOSED section already lists a proposal; what a rule says is one ank show away, which is the split section 9 settled for help. On this corpus the result is 1293 characters of constraints against 1082 of tasks, all 18 constraints named and all 12 tasks listed, nothing cut -- against 7357 and 157 before. Two things the falsification taught me and that are now in the spec and the tests. First, my unit test passed with and without the ceiling, because two short titles fit under a third anyway: it proved the rendering and not the rule. Titles are now long enough to force the cut. Second, the ceiling cannot empty the section -- the loop stops at one constraint, as it always did -- so 'at most a third' is a ceiling that yields to a floor, and section 5 now states that precedence instead of leaving it to the code. The binary test carries the arithmetic ceiling at a realistic budget on a corpus of forty constraints and forty tasks, with a control asserting the constraints really were counted away, and a second test holds execution mode to the opposite guarantee on the same fixture.
- 2026-08-13T06:59:15Z claude-agent-b — CI went red on all three runners while the branch alone was green, and the cause was not in the change. The flat store (TASK-cd3189ddf61e) merged while this branch was open, so Repo::new creates .ank/entities/ and no longer .ank/adr/ or .ank/tasks/, and the crowded fixture wrote into directories that had stopped existing -- NotFound at the same line on every platform. The CI tests the merge and not the branch, which is the whole reason it caught this and a local suite could not. Rebased on main and pointed the fixture where every other seed in the file already writes; the budget logic is untouched and the two commits stay separate so the diff reads. Green on the three operating systems, both MSRV jobs and the version check afterwards.
- 2026-08-13T06:59:21Z claude-agent-b — done, proof test:31675652087
