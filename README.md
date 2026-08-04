# ank

**The stupid coordination tool — tasks and architecture decisions as files in your repo, readable by any coding agent.**

An agent that spawns on your codebase can read every line of it. It cannot read
your tracker, your wiki, or the thread where you decided six months ago that
sessions must never be self-contained JWTs. So it writes plausible code that
violates a rule nobody wrote down where it could be found.

Ank puts that layer in the repository, attached to the code it constrains, in a
format an agent consumes in one call and under 2000 tokens.

## What it looks like

Two kinds of file, flat, in `.ank/`. A decision that constrains code:

```yaml
---
id: ADR-3c7e0b9142af
type: adr
title: Opaque sessions rather than stateless JWT
status: accepted
scope:
  - src/auth/**
constraint: |
  Do not introduce self-contained JWTs for user auth.
  Every session goes through the Redis store.
---
```

And a unit of work, with what would prove it finished:

```yaml
---
id: TASK-8f3a91c2d4e7
type: task
title: Migrate auth to opaque sessions
status: open
scope:
  - src/auth/**
blocked_by: [TASK-51c2a7f0]
done_criteria: |
  Auth integration tests pass, and no reference to
  jwt.verify remains in src/auth/
verify: [auth-tests, no-jwt]
---
```

An agent asks what applies where it is about to work, and gets only that:

```
$ ank context src/auth/

CONSTRAINTS (2 active)
  ADR-3c7e  Do not introduce self-contained JWTs for user auth.
            Every session goes through the Redis store.
  ADR-8b41  Rate limiting mandatory on every public endpoint.

TASKS (2)
  TASK-8f3a  [claimed:claude-code@host-3] Migrate auth to opaque sessions
  TASK-51c2  [open] Add secret rotation

> ank claim 51c2 to start
```

## The four ideas

**Scope, not hierarchy.** Constraints and work are two independent planes,
joined only by a list of globs. An agent gets what binds it without traversing
anything, and a constraint written last year applies to work created today.
Grouping by scope also happens to be verifiable — a glob is confronted with the
filesystem, a label is not.

**Nobody declares themselves done.** A task names verifiers; `ank done` runs
them itself and records what actually ran, hashed. The agent never reports its
own result, because an agent that reports its own result can simply be wrong.

**Freezing is verifiable, not defended.** The CLI cannot stop anyone from
editing a file, and it does not pretend to. Every frozen field is anchored by a
hash in something the editor does not control — the claim record, the signed
ratification commit — and `check` compares. Editing a criterion to unblock
yourself does not unblock anything; it makes the divergence visible.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Undo, history and
recovery are git's. There is no daemon, no server, no central arbiter, and
nothing to run. That is what "stupid" means here.

## What it is not

- **Not a tracker.** No cycles, estimates, velocity, roadmap or burndown. Ank
  can export to a tracker for human visibility; it does not replace one.
- **Not a wiki.** Only what is actionable or binding for an agent goes in. A
  decision that constrains code, yes. Meeting notes, no.
- **Not a security boundary.** The guardrails protect against an agent drifting,
  not against a malicious actor.

## Status: pre-v1, and the CLI runs

Thirteen verbs work end to end — the eight of the agent surface (`context`,
`claim`, `show`, `log`, `done`, `new`, `find`, `release`), the five of the human
one (`check`, `review`, `accept`, `close`, `attest`), plus `init` and `help`.
There are no published binaries yet: build from source with
`cargo build --release`.

This repository is built by dogfooding its own format. The development plan
lives in [`.ank/`](.ank/), and the tool now reads, claims and closes its own
tasks — the last several were closed by `ank done`, which ran the verifiers and
wrote the proofs itself. The corpus is validated by `ank check` on every CI run.

Still missing before v1: binaries for the three platforms and the installable
skill. `ank find` scans the index rather than querying FTS5, which is a
performance gap and not a behavioural one.

## Documentation

Four documents, and which one you want depends on what you are about to do.

- **You want to use it.** [`docs/getting-started.md`](docs/getting-started.md)
  takes you from install to a first claim and a first `done`, with real output
  at every step. It assumes nothing and does not send you to the specification.
- **You are writing a tool that reads or writes `.ank/`.**
  [`docs/format.md`](docs/format.md) has the field order, the emission rules,
  the two hashes and the conformance suite — the mechanical half, for a second
  implementation.
- **You want to know why it is shaped this way, or you need the normative
  answer.** [`docs/ank-spec-v1.1.md`](docs/ank-spec-v1.1.md) is the source of
  truth. It argues the design and settles every question the other two defer to
  it; it is not a tutorial, and the two documents above exist so that it does
  not have to be one.
- **You are an agent working on this repository.**
  [`CLAUDE.md`](CLAUDE.md) has the conventions, and the section below has the
  loop. The development plan itself lives in `.ank/` — a DAG of tasks through
  `blocked_by`, and the decisions in `.ank/adr/` — reached through the CLI.

## For agents working on this repository

Build it once, then let it drive:

    cargo build --release        # target/release/ank

1. `ank context` — what constrains the perimeter and what is claimable, in one
   call. Run it before anything else; run it again with a claim held and it
   switches to execution mode, giving the criterion and the constraints in full.
2. `ank claim <id>` — takes the task. It refuses, with the reason and the next
   command, if the task is held, finished on another branch, blocked, or has no
   criterion to be measured against.
3. `ank log "<message>"` — record what you learned. It renews the claim.
4. `ank done` — runs the declared verifiers itself and writes the proofs. Never
   set `status: done` by hand: the point of the tool is that nobody declares
   themselves finished.

`ank check` validates the corpus (parse, byte-for-byte round-trip, `blocked_by`
references) and must stay green after any edit to `.ank/`.

`.ank/` is opaque to an agent, the way `.git/` is (ADR-01b6dd05f0db): reach it
with `ank show <id>` for an entity whole, `ank find` to list, `ank context` to
learn what binds. The tool knows what the files do not — the context budget, the
frozen criterion, who holds which claim — and a `PreToolUse` hook in
[`.claude/`](.claude/) refuses the direct route rather than trusting anyone to
remember. A human with an editor keeps every power they had; `check` remains
what notices.

That was already the rule for writing, and for a sharper reason: `claim` and
`done` write to git refs and run verifiers, so editing the files by hand skips
both. Reading joined it once there was nothing left the CLI could not serve.

Detailed conventions are in [`CLAUDE.md`](CLAUDE.md).

## Layout

    crates/ank-core   parser and data model — the reference implementation of the format
    crates/ank-cli    the `ank` binary — thirteen verbs, plus init and help
    docs/             the specification (source of truth), getting started, the format
    .ank/             Ank's own development plan, in the Ank format
    skill/            the bootstrap skill for agents

## Development

    cargo test                  # full suite, format conformance included
    cargo fmt --check
    ank check                   # validates .ank/: parse, round-trip, references

`crates/ank-core/tests/golden/` is the format conformance suite, reusable by any
third-party tool: `valid/` must round-trip byte for byte once normalised,
`invalid/` must be rejected with the expected error. One valid file is in CRLF
on purpose and must come back in LF — the format is read in either and written
in one.

## Licence

GPL-3.0 — see [LICENSE](LICENSE). The copyleft covers the tool's code, not the
format: your `.ank/` files, and the third-party tools that read or write them,
are not derivative works.
