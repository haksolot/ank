---
id: ADR-a8f9c603a0e7
type: adr
slug: a-task-names-its-method-the-load-is-recorded-and
title: A task names its method, the load is recorded, and the rate is read, never enforced
created: 2026-09-13T09:20:42Z
author: claude-code/fable-5.1+planning
status: proposed
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - crates/ank-contract/**
  - skill/**
  - docs/**
constraint: |
  A task may carry method, one name among the sibling skills the binary carries, set by ank new task --method <name> or ank amend --method <name>; a name the binary does not carry is refused at exit 7 when the task is written, naming the ones it does. The field is optional, its absence means nobody designated one, and it earns no schema bump: a reader without the field concludes nothing false from its absence. ank context in execution mode prints the designated method on one line beneath the criterion, as the skill to load before the first edit. done never reads the field, check never faults a task for a method that did not fire, and no verifier inspects the route: the field is a recommendation, and ADR-e4a5a8873fe3 stands whole. A sibling that executes under a claim opens, after the claim, with one instruction that exists in no other file: ank log --method <name>, which writes a log entry whose records is method and whose title is the name, refused exactly where a log write is refused. ank skills, run in a corpus, reports per sibling how many tasks designate it, how many of those carry its entry, and how many entries fired on tasks that designated none; the rate is read from the corpus, printed to whoever asks, and sent nowhere. ank-plan teaches to set the method when the task's shape calls for one.
schema: 4
version: 1
---

## Context

A skill that is never loaded teaches nothing, and the evidence says that is the
common case. Vercel measured the skill never invoked in 56% of runs, for a gain
of zero against the same content injected always; Liu et al. (arXiv 2604.04323)
measured Claude Opus 4.6 at 55.4% with curated skills force-loaded, 51.2% when
the agent chooses, 40.1% with independent search, 35.4% with none. Each step of
leaving the load to the agent gives back part of the gain. Nothing in this
corpus measured its own skills until 2026-09-13, and the first measurement is
in the log of this decision: on the Claude Code transcripts retained locally for
this repository, the contract loaded in 8 sessions of 18, and ank-loop loaded
once across the sessions that claimed tasks after it existed.

The one thing ank can do that a harness cannot is bind a policy to a moment it
already knows: `context` after a claim serves the criterion and the constraints
of that task, and it can name the skill on the same page. The author of a task
knows its shape when writing it, the way they know its verifiers
(ADR-443590981e41): a criterion that names a defect calls for the diagnostic
loop, an implementation calls for red-green.

## The decision

Designation, record, rate. The author names the method on the task and the tool
names it back at the moment of execution. The sibling records its own load by
an instruction only it carries, so the entry proves the load by construction:
an agent that never read the body never saw the instruction. The rate is a
count over the corpus, printed by the same verb that prints the catalogue.

None of it is dispatch. `done` measures the tree; `check` validates the name and
nothing about whether it fired; a task with a method and no entry closes green.
A rate that was enforced would be a route graded, and an agent graded on its
route learns to fake the route (ADR-e4a5a8873fe3, Rejected).

## Rejected

- Guessing the method from the criterion's vocabulary. A deterministic tool
  that guesses, on words that drift; the author knows and the field is cheap.
- A free-text field. A name misremembered fails in silence at the one moment
  the recommendation was for; `--verify` was made to refuse at write time for
  the same reason.
- A `check` signal on a done task whose method never fired. It is the verifier
  that inspects the route, and it pressures the agent to write the entry
  without the load.
- A schema bump for `method`. On `via`'s terms (SPEC-e258796162c4): a bump is
  for a reader that would be wrong, and a reader that meets no method
  concludes only that none was designated.
- Recording the load anywhere but the log. The log is the entity that already
  records machinery apart from the trace (`records: edit`), lives in git, and
  needs no telemetry endpoint this project would have to run and disclose.

## What the record does not cover

ank-plan and ank-drift run under no claim, and `ank log` refuses a write on an
open task nobody holds; their load is measured from harness transcripts and not
from the corpus. The executing three, loop, tdd and diagnose, carry the
instruction after the claim, so it is never the first thing a sibling does
before there is a task to write on.

## Consequences

`method` joins the task's optional fields in the successor of SPEC-e258796162c4
and in docs/format.md; `--method` joins `new task`, `amend` and `log` in the
successor of SPEC-4b79c265ccb6; `method` joins the `records` vocabulary check
knows. The contract says in one sentence that a task may name its method and
that `context` prints it; three siblings gain one line each, and ank-plan gains
the flag.
