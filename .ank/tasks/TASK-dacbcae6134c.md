---
id: TASK-dacbcae6134c
type: task
slug: execution-mode-hides-the-coordination-plane-and
title: Execution mode hides the coordination plane, and nothing says whether that is meant
created: 2026-08-11T03:42:29Z
author: seanl@sean-laptop
status: open
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
blocked_by: []
done_criteria: |
  Section 5 of docs/ank-spec-v1.1.md says what a holder of a claim can see of the coordination plane and why, naming which verb answers it. If the answer is that execution mode shows nothing and status is where the question belongs, section 5 says so and status carries it; if execution mode is to show it, context does. Either way a test through the binary asserts what the specification now promises, with a second agent holding a claim on another task.
criteria_by: creator
schema: 2
version: 1
---

With a claim held, `context` switches to execution mode and renders that task
alone: criterion, constraints, log. The coordination markers `[claimed:holder]`
and `[finished:sha on branch]` are orientation-mode only.

So the agent best placed to notice a collision -- the one currently working --
is the one that cannot see the coordination plane any more. `ank status` does
not fill the gap: it reports the caller's own claim, not anyone else's.

This may be correct by design. Execution mode exists to remove choice, and a
list of what other agents hold is exactly the choice it removes; an agent that
starts reading the plane mid-task is an agent that has stopped working on its
own. If that is the reading, it is currently unrecorded, and the question will
be asked again by the next person who notices.

If it is not the reading, `status` is the natural place rather than `context`:
it already answers "where am I", it is off the loop, and it costs execution
mode nothing.

What would settle it is a measurement rather than an opinion: whether a
collision an agent could have acted on is actually reachable while a claim is
held. Level 1 is not implemented (TASK-82c3341502c1), so within one clone the
claim refs are shared and `claim` already refuses -- which may make this
finding moot until claims are pushed, and that is itself an answer worth
writing down.
