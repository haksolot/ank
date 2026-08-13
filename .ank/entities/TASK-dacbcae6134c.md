---
id: TASK-dacbcae6134c
type: task
slug: execution-mode-hides-the-coordination-plane-and
title: Execution mode hides the coordination plane, and nothing says whether that is meant
created: 2026-08-11T03:42:29Z
author: seanl@sean-laptop
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Section 5 of docs/ank-spec-v1.1.md says what a holder of a claim can see of the coordination plane and why, naming which verb answers it. If the answer is that execution mode shows nothing and status is where the question belongs, section 5 says so and status carries it; if execution mode is to show it, context does. Either way a test through the binary asserts what the specification now promises, with a second agent holding a claim on another task.
criteria_by: creator
proof:
  - type: test
    ref: "31518704754"
    criteria: be34a1c3e2e7
schema: 3
version: 5
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

## Notes carried in the log

What a holder could act on is smaller than it looks, and that is what makes status the right home rather than context. At level 0 the refs are shared inside the clone, so two agents cannot hold one task -- claim refuses with code 4 and names the holder -- and a holder reading that somebody else is on another task learns something true and acts on none of it. Worth reporting, not worth paying for on every turn. When claims are pushed (level 1, TASK-82c3341502c1) the answer becomes worth more and nothing here has to move, because it is already a question status answers.
status therefore names every live claim in the repository, read through the same context::coordination map every listing verb uses rather than a second enumeration, sorted by id so two runs that changed nothing print the same bytes. It says "elsewhere no claim by another agent" when there is none: silence and "this verb does not answer that" read identically, and relocating a question is worthless if the new home is mute. --json carries elsewhere on the same terms as also_held.
One process lesson, paid for here. My first falsification of the new test was a regex that matched nothing, so the file was unchanged and the test passed -- and a falsification that does not change the file is a green that means nothing, indistinguishable from a test that guards the behaviour. Removing the block by line range turned it red on the right assertion. Check that the file moved before trusting what the suite says about it.
