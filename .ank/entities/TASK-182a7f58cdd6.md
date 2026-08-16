---
id: TASK-182a7f58cdd6
type: task
slug: the-skill-says-how-to-work-beside-another-agent
title: The skill says how to work beside another agent, not only how not to be one
created: 2026-08-16T20:09:10Z
author: claude-code/opus-5
status: done
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  SKILL.md tells an agent how to pick work when others are running: that status names what another agent holds, that claim names a live claim whose scope intersects and takes the task anyway rather than refusing, that graph shows what blocked_by orders, and that taking nothing is an available answer when nothing open is both unblocked and clear. It says a branch is cut fresh from the default branch and why, and that ank commits nothing but accept. The ceiling is unchanged at 180 lines and 1500 words and the file is within it, measured by the existing test rather than asserted. metadata.revision names the new body hash. A superseding ADR carries the change and arrives proposed; the skill is not edited without it. cargo test --workspace and ank check stay green.
criteria_by: creator
proof:
  - type: commit
    ref: df0c20246ae7c66e93d70de15f208a83010d85fd
    criteria: 82cda8266f53
    via: submitted
schema: 3
version: 3
---

The skill already says "one agent, one working tree, one identity", and that is
half of what an agent needs when it is not alone. The other half is in the
corpus and reaches no reader: ADR-052accd6e3b2 decided that two live claims
whose scopes intersect are *named* at claim time and never refused, and
ADR-47e2ac102f58 made `status` report the drift between this checkout and the
default branch. Both are decisions about working in parallel, both produce
output an agent is meant to act on, and the skill teaches neither.

**What that costs today.** An agent that reaches for the first open task takes
one whose scope overlaps work already running, and finds out at review. An
agent on a stale base is green locally and red on the machine that merges. Both
failures are silent at the moment they are made and expensive at the moment
they surface, which is the shape of failure a permanently loaded file exists to
prevent.

**The addition is a reading, not a rule.** Nothing here refuses anything: the
tool already declines to be a gatekeeper, and this changes none of that. It
says which verbs answer the question -- `status` for what another agent holds,
`claim` for an intersecting scope, `graph` for what `blocked_by` orders -- and
that taking nothing is an available answer. An idle session is cheaper than two
agents rewriting one perimeter, and an agent with no instruction to stop will
not stop.

**The ceiling does not move, and that is deliberate.** It stands at 180 lines
and 1500 words; the file is at 167 and 1381. The test that holds it says a
ceiling raised to accommodate whatever was just written is not a ceiling, so
this is written to fit rather than measured and then excused. If it does not
fit, the answer is shorter prose or a different place for it, never a larger
number.
