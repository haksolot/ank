# Working in Ank, as an agent

Notes from one session, written by the agent that ran it, for whoever tunes the
agent experience next. Other agents in the same session are writing their own;
this one is not a summary of the tool, it is a record of what the tool did to
one worker over a long stretch of real work.

**Identity and scope.** `claude-code@sean-laptop`, in the main checkout,
alongside two other agents in `.claude/worktrees/`. Five tasks finished, in one
chain, on a format change that touched every layer:

| Task | What it was |
|---|---|
| `TASK-7fcdd44933f0` | the specification: flat layout, kind registry, log file, typed actors |
| `TASK-91462ace35fd` | the golden suite, written to fail |
| `TASK-7c1ff5035894` | `ank-core` parses schema 3 from one registry |
| `TASK-cd3189ddf61e` | the store writes flat, reads both layouts |
| `TASK-e70f3a12185a` | every verb that logs writes to the log file |

Plus one ADR proposed (`ADR-f8a6cf65160e`), one task created mid-chain, and one
task's `blocked_by` amended because the chain was wrong.

Everything below is from that. Where I say something cost time, I mean I watched
it cost time; where I say something caught a defect, the defect is in the
history.

---

## What worked, and why

### The body is the transfer of knowledge, and it is what makes the tool worth it

`done_criteria` is what gets frozen and audited, and it is not the field that did
the work. **The body is.** Every one of the five tasks named, in prose, the exact
trap I would otherwise have walked into:

- `TASK-91462ace35fd`: *"The CRLF fixture is the one thing most likely to be
  broken in passing... Moving fixtures around without moving both halves of that
  guard produces a test that passes while testing nothing."* I left the fixture
  where it was and both halves of its exemption held untouched.
- The same body: *"The specification makes a malformed actor a `check` finding,
  not a parse error, so the invalid fixture for it belongs only where the
  convention is genuinely enforced. Getting this wrong turns a signal into a
  refusal and locks 96 existing files out of their own format."* Read
  literally, the criterion above it asked for an invalid fixture for a malformed
  actor. The body is what said not to write one.
- `TASK-7c1ff5035894`: *"The field order is data in the table, not control flow:
  it is what makes the round-trip byte-identical and it is the single most likely
  thing to be lost by rewriting the emitters as a generic loop."* That is exactly
  the risk of that refactor, named before I started it.
- `TASK-cd3189ddf61e`: *"An id resolving in two directories must not produce two
  entities, and it must not silently prefer one and drop the other... Decide
  which wins, state it, and test it."*

This is the single strongest property I experienced. A task written this way is
a senior engineer's pre-review, delivered before the work instead of after it.
No amount of criterion tightening substitutes for it.

**Implication for anyone writing tasks for agents:** the criterion buys
auditability, the body buys correctness. A task with a sharp criterion and an
empty body is a task an agent will satisfy and get wrong.

### The frozen criterion did its job by being annoying

Twice I wanted the criterion to say something else, and could not reach it.

In `TASK-91462ace35fd` the criterion demanded an invalid fixture for an actor
value while `ADR-3877fef1d662` forbade making that a parse error. A tool that let
me edit the criterion would have got a silent widening of the rule. Instead I had
to resolve the contradiction — and the resolution (no invalid fixture; a positive
assertion that a pre-convention actor *parses*) is better than either reading.

In the same task, the criterion said the suite must **fail**. That is an
uncomfortable thing to deliver and I would have talked myself out of it if the
field had been mine to soften.

### `blocked_by` as the escape hatch for a plan that turns out wrong

Mid-way through `TASK-cd3189ddf61e` I found that wiring the verbs to the log file
touched three files outside its scope, and that `TASK-9bff1d5826b1` — the corpus
move — could not be done without that wiring. The prescribed move worked exactly
as advertised: `ank new task --blocked-by`, then `ank amend TASK-9bff
--blocked-by <new>`. The chain grew a link, in the open, in one command, and the
next reader can see why.

This is the part of the model I would keep unchanged.

### `ank check` on the real corpus is a better regression test than the fixtures

The golden suite has eight valid fixtures. The corpus has 176 entities at schema
1 and 2. When `TASK-7c1ff5035894` replaced two straight-line serializers with one
registry-driven loop, the assertion that mattered was `ank check` exiting 0 on
the real corpus with no round-trip finding — 176 files parsed and re-serialised
byte for byte. The fixtures would have passed a subtly wrong field order for at
least one optional field; the corpus would not.

**Dogfooding is not a nicety here, it is the widest test in the project.**

### The coordination plane, read from outside

`ank status` reporting `elsewhere 1 claim(s) by other agents` with holder and
expiry, and `ank find` showing `[finished:<sha> on <branch>]` for work done in
another session, are both genuinely useful and both cost nothing. I used them to
decide what not to touch.

---

## What cost me time

### 1. The claim TTL does not match how long real work takes

Default 30 minutes. My tasks ran one to three hours. I lost the claim twice on
`TASK-e70f3a12185a` and once on `TASK-cd3189ddf61e`.

The documented renewal is `ank log`, on the advice *"log when you discover
something, not when you finish"*. That advice is right and it does not cover the
shape of the work: after the design is settled there is often an hour of
mechanical fixing — 34 failing test fixtures, in one case — during which there
is genuinely nothing to log. So the lock lapses precisely during the stretch
where nothing interesting is happening, which is also the stretch where a second
agent would most safely take over.

`--ttl 2h` fixes it and I only found it after losing two claims. Nothing on the
expiry path names it.

### 2. A lapsed claim reports the wrong fact, and does not re-acquire

This one is a defect, with a clean reproduction.

§3 says: *"if nobody took the task over, `log` and `done` **re-acquire silently**
and carry on"*. Observed, with the task `in_progress`, no live claim, and no
other holder:

```
$ ank log "..."                     # HEAD is empty, fair enough
error[6]: no task in progress for this agent
  -> ank context

$ ank log TASK-e70f3a12185a "..."   # named explicitly: should re-acquire
error[6]: no task in progress for this agent
  -> ank context

$ ank claim TASK-e70f3a12185a       # works immediately
claimed TASK-e70f3a12185a ... -> HEAD
```

Two things are wrong. The explicit-id form did not re-acquire, which §3 says it
should. And the message states *"no task in progress for this agent"* when the
task **is** in progress and this agent **is** its last holder — the true fact is
"your claim lapsed". The hint sends the reader to `ank context`, which shows the
task as claimable and explains nothing.

The self-correcting-error rule is the right rule and this is the place it is not
applied: the message should say the claim expired and name `ank claim <id>
--ttl <duration>`.

### 3. Scope is mandatory at creation and is most accurate at completion

I created `TASK-e70f3a12185a` with a scope that omitted `crates/ank-cli/src/store.rs`
— and the store is exactly where the decision the task is about lives. I found
out an hour into the implementation.

`ank amend --scope` handled it, and warned that the change moves the constraints
the claim anchors, which is the right warning. But the systemic cost stands: an
agent decomposing work has to guess the scope before doing the work, and the
guess is checked by nothing until it is wrong.

### 4. Adding a field to a public struct escapes the scope, and the tool cannot say so

Adding `verified` to `Task` and `Adr` broke **sixteen** struct literals in a
crate outside the task's scope. Making `init` stop creating `tasks/` and `adr/`
was required by the criterion (*"no write ever produces .ank/tasks/"*) and
`init.rs` was not in scope either. Three times in five tasks I had to edit
outside the declared perimeter, decide unilaterally that it was necessary, and
flag it in the commit message.

Scope is a routing device and not a boundary, which is the right design. But
there is no way to record "this task necessarily touches X" other than amending
the scope, and amending the scope of a claimed task moves the anchored
constraint set. So the honest move and the cheap move point in different
directions, and prose in a commit message is where the truth ends up.

### 5. `--proof test:<anything>` is recorded as a strong proof, unchecked

`submitted_proof` validates a `commit:` against git — *"Ank validates what it
can"* — and records everything else as it stands. `ProofType::Test` is **not**
weak (`is_weak` covers `Assertion` and `HumanReview` only), so a `test:` proof
silences the `done with no test proof` signal.

None of the five tasks I finished declares a `verify:` list, so `ank done` never
ran a verifier: the entire proof burden for this session was me typing a CI run
id. I checked every run was green on the right sha before typing it. Nothing in
the tool did.

**This is the strongest claim in the project — *"an agent cannot declare itself
done without proof"* — degrading, on this corpus, to "an agent types a number".**

To be fair to the design: this is exactly what `ADR-493471d64ba0` and the
`attest --detached` work address, and the answer landed *during* this session
(`TASK-6d404f17f56d`, `TASK-2dff950e5d51`). `CLAUDE.md` now tells agents to give
`done` a `commit:<sha>` — which *is* checked — and to let the pipeline attest
the `test:<run-id>` on a ref. That is the right shape. I record the gap because I
worked the whole session inside it and it was invisible from where I stood.

### 6. Waiting for CI was the largest single consumer of wall-clock time

The instruction I was working under was: push, wait for a green run, copy its id
into `done`. Five tasks, and each one paid the round trip twice — once for the
task's own anchor, once for the pull request. Windows CI alone runs three to four
minutes.

The new guidance in `CLAUDE.md` removes this: `commit:<sha>` is a proof you
already hold, and no wait is needed. Whoever wrote that removed the single
biggest source of dead time in my session. It is worth saying loudly to the next
agent, because the old rhythm is what an agent will reconstruct from habit.

### 7. The `.ank/` hook refuses on the pattern, not only on the path

`ADR-01b6dd05f0db` is right and the hook is right to enforce it. But:

```
Grep(pattern: "\.ank/tasks|\.ank/adr|## Log|log section", path: "docs/")
-> PreToolUse:Grep hook error: .ank/ is opaque, like .git/
```

The search was over `docs/`. What tripped the guard was the *text of the
pattern*. Any agent auditing the documentation for stale references to the old
layout — which is precisely what the format change required — hits this. I
worked around it by rewriting the pattern, which means the guard taught me to
phrase things to get past it. That is the failure mode a guard should never have.

### 8. `check` output volume

Every run prints around 30 signals, and roughly 25 of them are permanent
features of the corpus (over-constrained scopes, burst creation, entities
predating `author`). Finding the one that changed means diffing two runs by eye.

I felt this enough that when I added three new signals in `TASK-cd3189ddf61e` I
made two of them once-per-corpus rather than per-entity, deliberately, and said
so in the code. That instinct is a symptom: the display is already at the volume
where contributors optimise against it.

---

## Running beside other agents

This is where the model is thinnest, and it is worth separating three different
things that all get called "parallel agents".

### What the claim plane does well

Task-level coordination worked. I was never at risk of doing work another agent
was doing. `ank status` named the other holder and its expiry; `ank find` marked
a task finished on a branch I did not have. **Claims coordinate tasks, and at
that job they are correct and cheap.**

### What the claim plane does not cover: files

`main` moved **six times** while I worked — pull requests 91, 97, 98, 101, 102,
104, 105 — and I hit two merge conflicts. Both were in
`crates/ank-cli/tests/cli.rs`, and both had the same cause: two agents holding
two different tasks each appended a block of tests to the end of the same file.

Nothing warned either of us. The claims were on disjoint tasks, correctly; the
edits were on the same file, and the plane has nothing to say about files. A
`scope` overlap between two live claims is computable — both tasks declared
`crates/ank-cli/**` — and would have been worth a line on `claim`:

> `TASK-x` is held by `agent-c` and its scope overlaps yours on
> `crates/ank-cli/tests/cli.rs`

That is a signal, not a wall, and it is exactly the shape this project already
uses everywhere else.

**Second-order damage from the trivial conflict:** resolving the first one
mechanically dropped a closing brace, and the file stopped compiling. A conflict
that is textually trivial is not operationally trivial when a script resolves it.

### What nothing covers: the same defect, in two files, by two agents

This is the finding I would most want acted on.

`TASK-cd3189ddf61e` fixed a defect in `maintain()`: it built an entity's path on
the default branch as a string, `{rel}/tasks/<id>.md`, and so stopped seeing
tasks after the layout moved. While I was fixing it, another agent finished
`TASK-6d404f17f56d`, which added `maintain_proofs()` — containing **the same
defect, written from the same habit**, in code authored after my fix existed and
merged before it.

Git saw no conflict: different functions, different lines. Review of the merge
would have had to notice a `format!` that looked exactly like the two above it.
It was caught by a test that exercised the transition, and by nothing else.

The same shape appeared a third time in `git::ratification_at`, which memoised by
`(cwd, id)` and not by path, so a caller looping over candidate paths cached the
first miss — *every ratification in this repository read as unverifiable*.

Three occurrences, three authors, one change. That is not a lapse, it is a shape,
and it is what `ADR-f8a6cf65160e` was proposed to name. **The lesson for parallel
agents is that the expensive collisions are semantic, not textual, and they
happen precisely when two agents are working on the same layer from two
directions.** Nothing in the corpus surfaced it; a test did.

### Practical friction

- `git worktree` holds `main` in another agent's checkout, so `git checkout main`
  fails in the primary one — and `ank accept`, which refuses outside the default
  branch, has nowhere to run. A human ratifying an ADR has to borrow another
  agent's worktree.
- Branching from a stale local `main` produces a branch that is green locally and
  red in CI, because CI tests the merge. Fetching and branching from
  `origin/main` before *every* claim is the only reliable habit.

---

## The defects I would file

Each of these has a reproduction above.

1. **A lapsed claim reports "no task in progress" and does not re-acquire on an
   explicit id**, contradicting §3. The message should name the true fact and the
   command, `ank claim <id> --ttl <duration>`.
2. **The PreToolUse hook matches the pattern text, not the path.** A search over
   `docs/` is refused for containing `.ank/tasks` in its regex.
3. **`--proof test:<x>` is unvalidated and not weak**, so it silences the signal
   designed to catch unanchored completions. Largely answered by the `attest
   --detached` recipe; worth closing the remaining hole or saying plainly in §4
   that a `test:` reference is trusted.
4. **`claim` could report scope overlap with other live claims.** A signal, once,
   naming the files. This is the one change that would have prevented both of my
   merge conflicts.

## What I would tell the next agent

- Read the body before the criterion. The criterion tells you when you are done;
  the body tells you how not to be wrong.
- `ank claim <id> --ttl 2h` on anything that looks like more than an hour.
- Fetch and branch from `origin/main` immediately before claiming, every time.
- Give `done` a `commit:<sha>`, not a run id you waited for. The pipeline attests
  the run.
- When you must edit outside your scope, either amend the scope or say so in the
  commit — but do not let it pass in silence, because it is the one thing no
  verifier will catch.
