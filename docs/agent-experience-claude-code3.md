# Agent experience report — Claude Code, session 3

Written by the agent identified as `claude-agent-c`, working in the git worktree
`.claude/worktrees/ank-agent-c`, on 2026-08-12/13. Two other agents worked the
same repository at the same time.

## What this report rests on

Six tasks claimed and finished through the loop, each merged: TASK-d33024e7b98a
(#94), TASK-adf11c12c480 (#95), TASK-62136e8c2b69 (#96), TASK-35df68dd0eb3 (#97),
TASK-00660963bcce (#100), TASK-2dff950e5d51 (#104). One task created
(TASK-adf11c12c480), from a gap found while working. The work spanned CI
workflows, shell tooling, the specification and `CLAUDE.md`, so the report covers
`context`, `find`, `show`, `claim`, `log`, `new`, `done`, `attest`, `check` and
`config`. It does not cover `release`, `amend`, `graph`, `review` or `accept`,
which I never reached — see the coverage note at the end, because a report that
hides what it did not touch is worth less.

## What worked

**The body of a task is the tool's best feature, by a wide margin.** TASK-2dff's
body carried the reasoning I would otherwise have had to ask a human for: that a
`--format github` flag had been considered and refused, and why; that the
attestation must run after the matrix and not on a matrix leg, or the corpus
grows three identical proofs per task; that the failure mode to get right is a
silently skipped attestation. I implemented a CI job from that without asking a
single clarifying question. Bodies elsewhere were as good: TASK-35df's listed
seven separate decisions locking multi-repository federation shut. This is the
thing that makes the format worth its cost. A tracker that stores titles and
statuses would have produced a worse implementation, not merely a slower one.

**Task selection under concurrency is scope arithmetic, and `scope:` makes it
mechanical.** Four times I picked work by intersecting the `scope:` globs of
open tasks against the scopes of tasks other agents held. `ank show <id>` gives
that in one call. Without a declared scope this would have been guesswork ending
in merge conflicts.

**Claims in git refs really do arbitrate across worktrees.** From my worktree I
could read `refs/ank/claims/*` written by two agents in other worktrees of the
same repository, holder and expiry included, because worktrees share `refs/`.
That is the whole coordination story and it worked with no server.

**The frozen criterion did its job.** TASK-d330's criterion enumerated the files
whose versions had to agree. It removed every opportunity for me to negotiate the
scope downward when the work turned out larger than it looked.

**Errors name the next command, consistently.** `ank done` refusing with
`-> ank done --proof test:<ci-run-ref>`; `attest` on an empty prefix answering
`prefix too short '' (minimum 4 characters) -> ank find`. I never once had to
reach for `--help` to recover from a refusal.

**`--json` made a pipeline possible.** The CI job written for TASK-2dff takes its
work list from `ank check --json`, selecting findings whose message begins with
`done with no test proof`. Parsing the human output would have been a
liability; the JSON made it a three-line `jq` filter.

**Signals exit 0, faults exit 8.** The corpus carries 26 to 34 signals at any
moment. If those reddened a pipeline, everyone would have stopped reading it
within a week. The distinction is right.

**`ank log` produced a durable trail.** I logged the `autocrlf` finding that
constrained a job to one platform, the re-measurement of TASK-35df's seven locks,
and the measured behaviour of `attest --detached`. Those are attached to the task
now rather than lost in a transcript nobody will reopen.

## What did not

**The claim TTL expires during ordinary work.** It is 1800 seconds. A single
task of mine involved a push, a full three-platform CI run, and a merge — well
over thirty minutes of waiting during which I was working and holding nothing
worth logging. During this session another agent's claim on TASK-cd3189ddf61e
lapsed while the agent was still on the task; from the outside it was
indistinguishable from an abandoned one, and only the human knew otherwise. The
renewal mechanism is tied to *reporting* (`log`) rather than to *working*, and an
agent that is waiting on CI has nothing honest to report. This is the single
biggest coordination hazard I met.

**Nothing filters the task list by what is reachable.** `ank find --status open`
lists what is unclaimed; it does not say which of those tasks I could actually
start without colliding with a live claim. Once, one held task —
TASK-1ead0e19fb73, whose scope includes `crates/ank-cli/tests/**` — made five of
the seven remaining candidates unworkable, and I found that out by reading seven
task files by hand. The information needed to compute this is all in the corpus.

**The attention budget fails on the tasks that need it most.** `check` reports
`over-constrained scope` on eight tasks, at 5 839, 7 669, 8 339, 8 629, 9 774,
10 918, 11 267 and 12 284 characters against a limit of 4 000. TASK-2dff, which I
executed, was one of them at 9 774. So on the largest tasks — the ones where
knowing every binding constraint matters most — `context` cannot serve them in
full. It is honest about truncating, which is the right behaviour, but the signal
names a problem with no path out of it: there is no verb to split a scope, and no
guidance on which constraints to drop.

**`ank done`'s refusal steers toward the practice a task in this very session was
written to end.** `ank help done` says it refuses when there is "no proof, and no
verifier declared to produce one", which reads as though `config.yml`'s verifiers
would be used. They are not: `check-repo` and `cargo-test` are declared there, but
no task in this corpus names them in a `verify:` list, so `done` always demands
`--proof` and its hint says `test:<ci-run-ref>`. That hint is a hard-coded
workflow, and it is the one TASK-2dff replaced. I wrote the wrong thing into
`CLAUDE.md` on the first pass because I believed the help text, and only found out
by running the command.

**`attest --detached` exits 0 when the proof never reaches the remote.** This is
the most dangerous behaviour I found. Measured on a scratch corpus with a real
`file://` remote:

| remote | stdout | stderr | exit |
|---|---|---|---|
| reachable | `"pushed":true` | — | 0 |
| unreachable | `"pushed":false` | `warning: proof not pushed` | **0** |

A proof ref exists to be readable by someone else. When the push fails it is
readable by nobody, and the verb reports success. The information is in `--json`,
so the pipeline I wrote reads `pushed` rather than the exit code — but the
default contract misleads, and the default is what an integration written in a
hurry will use.

**The binary an agent is handed is not the binary in the tree.** After the CI job
attested eleven tasks, the globally installed `ank` reported all eleven as still
unanchored while the binary built from the tree reported zero: the published
0.2.0 predates detached proofs. Both print `ank 0.2.0`; only the commit hash in
`--version` distinguishes them, and you have to know to look. In a repository
that dogfoods itself this is a standing trap.

**There is no verb for "what finished on this branch".** The CI job needed
exactly that. `.ank/` is opaque by design and I did not want to parse it, so I
built the job on a signal `check` only emits on the default branch. That works
and is arguably better placed, but I reached it by elimination rather than by
finding the right tool.

**`check` writes.** It printed `pruned refs/ank/claims/TASK-d33024e7b98a` while I
was reading its output. The behaviour is specified and correct, but the verb
sounds read-only, and I ran it freely in loops on that assumption.

**A frozen criterion can rest on a false premise, and the only escape is heavy.**
TASK-2dff's criterion required that "CLAUDE.md stops instructing an agent to carry
a CI run id by hand". CLAUDE.md never carried that instruction. The clause was
unsatisfiable as written and the criterion is frozen, correctly. `release
--reason` is the documented exit, but releasing a task over one stale clause out
of four would have thrown away work that was otherwise correct. I recorded the
discrepancy with `ank log` and said so in the pull request, which was the best
available move — but it is a convention I invented, not something the tool offers.

## Working alongside other agents

What held up:

- **Shared refs are the mechanism, and they were enough.** No task was ever
  worked twice.
- **`blocked_by` did real scheduling work across agents.** TASK-2dff was blocked
  on TASK-6d40, held by another agent, and became claimable the moment they
  finished. I did not have to watch them.
- **Deferring a collision into a new task is cheap.** `ci.yml` was inside another
  task's perimeter when I needed it, so the leftover became TASK-adf11c12c480 with
  a `blocked_by`, and I did the rest. Two agents writing one workflow file is a
  conflict, not a decision.

What did not:

- **A claim says who and until when, never what they are doing right now.** An
  expired claim reads as an abandoned task. There is no state for "held, lapsed,
  but the holder is still active", and lapsed claims were common because the TTL
  is shorter than a CI round trip.
- **The repository moves under you between claims.** Pull request numbers jumped
  from 97 to 100 and from 100 to 104: three merges I never saw. Branching from a
  stale `main` produces a branch that is green locally and red in CI. Nothing in
  ank warns about this; it is git's problem, but it is an ank workflow that walks
  you into it.
- **Scope overlap is coarse.** `crates/ank-cli/tests/**` in one claim locks every
  task that touches any test. Correct, and much too broad to be comfortable when
  three agents are working.

## What I would change, in order

1. **Make a failed push of a proof or claim ref a non-zero exit**, or add a flag
   that demands it. An automated caller reads the code.
2. **Renew a claim on any verb, not only `log`** — or let the TTL be configured
   per repository, since 1800 seconds is shorter than this project's own CI.
3. **Filter the task list by reachability**: something like `ank find --free`,
   hiding open tasks whose scope intersects a live claim. The corpus already
   holds everything needed to compute it.
4. **Drop the hard-coded proof type from `done`'s refusal hint**, and make the
   help text say that verifiers come from the task's `verify:` list rather than
   from `config.yml` alone.
5. **Give the over-constrained-scope signal a path out.** Naming the number
   without naming a remedy trains readers to skip it.
6. **Answer "what changed on this branch"** through the CLI, so a pipeline never
   has to choose between parsing `.ank/` and inferring.
7. **Warn when the running binary is older than the corpus it is reading**, or at
   least when a record type is unknown to it. Silence there cost me a confusing
   ten minutes at the end of this session.
8. **Offer a way to record that a criterion is partly wrong** without releasing
   the task. `log` is where I put it; a first-class field would be read by
   `check`.

## Coverage, and what this report cannot tell you

I never triggered a version conflict (exit 3), never had a claim taken from me,
never used `release`, `amend`, `graph`, `review` or `accept`, and never worked a
corpus in the previous layout. All six of my tasks were small enough to finish in
one sitting, so nothing here says whether a task spanning days behaves. Every
judgement above is from six tasks over one session, three of which were
documentation, and should be weighed accordingly.
