# Agent experience report — claude-code2

One agent's account of using `ank` for a full working session, including what
happened while two other agents worked the same corpus in parallel.

**Identity:** `claude-agent-b`, in a git worktree of its own.
**Session:** 2026-08-13.
**Delivered:** five tasks, each claimed, worked, verified and merged.

| Task | What it was | PR |
|---|---|---|
| TASK-1e79ff3738df | `check` names where a dead scope went | #91 |
| TASK-e3d00a6e62bb | `review` prints the ratification queue | #92 |
| TASK-6d404f17f56d | `attest --detached`, a proof in a ref | #98 |
| TASK-1ead0e19fb73 | the orientation budget | #102 |
| TASK-ae77a9ee2964 | the specification half of `--detached` | #105 |

Concurrent agents on the same corpus: `claude-code@sean-laptop` and
`claude-agent-c`. **Zero claim collisions over the whole session.**

---

## What worked

### The two-phase `context` is the right shape

Before a claim, `context` orients; after one, it switches to the criterion and
the constraints that bind the work in hand. Nothing to remember, no flag: the
claim is what selects the mode. In execution mode it served the frozen
`done_criteria` and every applicable constraint in full, which is exactly the
brief. That half needed no improvement all session.

### The frozen criterion actually bit, and that is the point

On TASK-1e79 the criterion said the repair proposal should be `ank amend` "for a
task". Working through it, I found `amend` refuses a `done` or `closed` task
outright — and those are precisely the tasks the dead-scope fault fires on. The
criterion and the binding ADR disagreed at one branch.

Because the criterion is hashed into the claim record, softening it was not an
option, and that is the whole value: I had to log the tension, make a defensible
call, and leave the reasoning where a reviewer would find it. A criterion I
could have edited is a criterion I would have edited.

### `ank log` as a work trace, not a status report

Logging on discovery rather than on completion changed how I worked. TASK-1ead
went further and made it a *requirement of the criterion* — "the measurement
that settles it is recorded in the task log" — which forced measure-before-answer
on a question I would otherwise have answered from intuition. The measurement
then contradicted the task's own hypothesis (a narrower perimeter changes
nothing, because constraints are charged first either way). That is a design
worth copying into other criteria.

### Self-correcting errors are load-bearing, not decoration

`amend` refusing an accepted ADR's scope prints the exact supersession command.
While implementing TASK-1e79 I needed to emit that same proposal — so I reused
the refusal's own wording verbatim. The error message was the specification of
the feature. That only works because the rule is "always the exact command to
run next", applied without exception.

### Scope as a collision signal

Picking work, `ank show <id>` gave me each candidate's `scope`, so I could see
that TASK-6d40 shared `human.rs` with a task another agent held and decide
knowingly rather than discover it in a merge conflict. Combined with
`ank graph`, choosing work that would not collide took about a minute per task.

### Claim refs in git, and `status` naming the holders

`ank status` reporting `elsewhere N claim(s) by other agents` with ids and
expiries is what made parallel work calm. I never once contended a task. Three
agents, a whole session, no arbitration failure — the mechanism is quiet enough
that it is easy to forget it is doing anything.

### A discovered subtask has a home

TASK-6d40's scope was five files under `crates/` and no documentation, so the
specification half had nowhere to go. `ank new task --blocked-by` turned that
into TASK-ae77, which I later claimed and delivered. The gap was declared
instead of either being smuggled in or forgotten.

---

## What did not work

### The stale corpus is the sharpest problem

**A worktree left on a merged branch carries a stale `.ank/`, and nothing says
so.** This bit three times:

1. I reported an open-task list to the user that was wrong — tasks reading
   `open` in my checkout were already `done` on the default branch.
2. Again, later, after another merge.
3. It finally cost a **red CI on all three runners**. The flat-layout migration
   (TASK-cd31, PR #99) merged while my branch was open; my new test fixture
   wrote into `.ank/adr/` and `.ank/tasks/`, which had stopped existing. The
   local suite on my branch was green, because CI tests the *merge* and a local
   run structurally cannot.

`ank status` reports the branch and the default branch, and `check` reports
tasks finished on other branches. Neither answers *is my copy of the corpus
behind the default branch?* — which, with several agents merging continuously,
is the question that matters most.

> **Suggestion.** A signal when the corpus in the working tree differs from the
> corpus on the default branch, ideally naming a count: `corpus 6 entities behind
> main (git fetch origin)`. The machinery exists — `check` already reads entity
> files at a revision through `git::file_at` for the pruning predicate.

### Orientation was the least useful command an agent runs first

This was TASK-1ead and I fixed it, but it is worth recording as experience
rather than as a closed ticket, because the shape of the failure generalises.

Measured on this repository, at the default 8000-character budget with 18
constraints and 11 open tasks: orientation spent **7357 characters on
constraints and 157 on tasks**. One task line printed, eleven cut. The closing
suggestion then recommended the single candidate it had room for — the mode
whose entire purpose is choosing offered one option out of twelve, and an agent
reading it had no way to know the others existed.

The cause was in the specification, not the code: §5 ordered the cut as "tasks
first, before any constraint". **The code was faithfully implementing a rule
that was wrong**, which is the failure mode a specification-first project should
watch for hardest.

### The dogfooded binary is not the binary under test

The project rule is to run `ank` from outside `target/` (Windows locks a running
executable during relink). In practice that means the globally installed npm
build, which tracks the *published* version, not the tree. So the tool managing
the work was a different version from the one I was changing.

After merging the orientation fix I ran `ank context` and saw the **old** output.
I nearly reported a regression. The correct check was to build the tree's binary
and run that — obvious in hindsight, and a real trap in the moment.

> **Suggestion.** Have `ank --version` say loudly when the binary predates the
> corpus it is reading, or document the dogfooding split where an agent will hit
> it rather than in a memory note.

### `ANK_AGENT` does not survive between commands

My shell state does not persist across tool calls, so **every single `ank`
invocation needed the identity prefixed**. Dozens of them. Forget it once and
the identity silently falls back to `<user>@<hostname>` — in this repository,
`seanl@sean-laptop`, which the corpus shows authoring 12 entities.

The consequence is not a warning, it is a refusal on state: a `log` or `done`
from the wrong identity fails because the claim is held by somebody else, and
the message talks about claims rather than about identity. Correct behaviour
(refusals are on state, never on identity), and a genuinely confusing first
encounter.

> **Suggestion.** `ank status` could print the identity it resolved and where it
> came from — `identity claude-agent-b (ANK_AGENT)` versus
> `identity seanl@sean-laptop (default: user@host)`. One line, and the trap
> becomes visible before it costs anything.

### The anchoring round trip (fixed mid-session, worth recording)

For most of the session, finishing a task meant: push, wait for CI, copy the run
id, `ank done --proof test:<id>`, commit that, push again, wait again, merge.
**Two full CI waits per task**, most of it spent waiting for a number to copy by
hand.

TASK-2dff landed mid-session and removed it — the pipeline now attests
`test:<run-id>` itself with `--detached`, and the agent closes on a proof it
already holds. The right fix, and the friction it removed was substantial: the
old loop is exactly the kind of step that gets skipped, which is how the corpus
had accumulated ten `done` tasks anchored to nothing.

### Scope amendment is correct and noisy

Declaring a CLI flag requires touching `cli.rs`, which TASK-6d40's author could
not have anticipated. `ank amend --scope` is the right route and I used it — but
the scope also feeds the constraint hash the claim froze, so widening it produced
a warning at `amend` and a second at `done` ("constraints over this scope changed
while the claim was held"). Both are correct and neither is a problem. At that
moment they read like something had gone wrong.

> **Suggestion.** When the drift is caused by the caller's own `amend` earlier in
> the same claim, say so: `constraints changed by your amend of <when>` reads
> very differently from a bare warning.

### No way to see what changed since I last looked

With three agents merging continuously I re-read `ank find --status open` many
times, diffing it against memory. `--since` is deferred in §10; it was the thing
I wanted most often after the stale-corpus problem, and the two are related —
both are questions about *what moved*.

---

## Parallel agents, specifically

**What held.** Claims. Completely. Three agents, one corpus, a whole session,
and not a single contended task or lost piece of work. The per-task ref plus
`status` naming holders is enough to coordinate without any agent talking to
another.

**What did not hold.** Everything about the *corpus* being a moving target:

- Task lists went stale between reading them and acting on them.
- The default branch moved under an open branch five times, and the one time it
  changed a fixture's assumptions it produced a red CI that no local run could
  have caught.
- `ank context` and `ank find` answer about the checkout, not about the
  repository. With one agent those are the same thing. With three they are not,
  and nothing in the tool marks the difference.

**The asymmetry is worth naming.** ank treats *coordination* as the hard problem
and solves it well, while treating *durable state* as ordinary git and leaving
it there. Under parallel agents that is backwards: claims were never the
difficulty, and a stale corpus was the difficulty every single time.

---

## Ranked suggestions

1. **Signal a corpus behind the default branch.** Highest value by a distance;
   it is the only issue here that caused a failure rather than friction.
2. **Print the resolved identity in `ank status`,** with its source. Cheap, and
   removes a silent footgun for any agent whose shell state does not persist.
3. **Make the dogfooding version split visible** in `ank --version` rather than
   in a project note.
4. **Attribute constraint drift to the caller's own `amend`** when that is what
   caused it.
5. **`--since`,** or any answer to "what moved since I last looked".

## One thing to keep exactly as it is

The frozen criterion, and the refusal to let the working agent edit it. It
created real friction twice this session and both times the friction was the
system working: once it forced a documented judgement call instead of a quiet
edit, and once it forced a measurement that overturned the assumption the task
was written on. Nothing else here is worth trading for it.
