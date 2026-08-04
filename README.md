<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/ank-dark.svg">
    <img src="assets/ank.svg" alt="" width="88" height="88">
  </picture>
</p>

<h1 align="center">ank</h1>

<p align="center">
  <strong>The stupid coordination tool</strong><br>
  Tasks and architecture decisions as files in your repo, readable by any coding agent.
</p>

<p align="center">
  <a href="https://github.com/haksolot/ank/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/haksolot/ank/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/haksolot/ank/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/haksolot/ank"></a>
  <a href="LICENSE"><img alt="Licence" src="https://img.shields.io/badge/licence-GPL--3.0-blue"></a>
</p>

---

An agent that spawns on your codebase can read every line of it. It cannot read
your tracker, your wiki, or the thread where you decided six months ago that
sessions must never be self-contained JWTs. So it writes plausible code that
breaks a rule nobody wrote down where it could be found.

Ank puts that layer in the repository, attached to the code it constrains, in a
format an agent consumes in one call and under 2000 tokens. No server, no
daemon, no account — two kinds of file and a CLI.

## Quick start

**Install.** Take a binary from the [latest release][releases] — Linux
(`x86_64-musl`), macOS (Apple silicon) or Windows (`x86_64`), each with a
`.sha256` beside it — and put `ank` on your `PATH`. Or build it:

```
cargo install --git https://github.com/haksolot/ank ank-cli
```

Ank needs **git 2.34 or newer**: claims are git refs, and it checks at startup.

**Run the loop.** In any git repository:

```
ank init                      # creates .ank/, once
ank context                   # what binds here, and what is free to take
ank claim <id>                # takes the task, freezes its criterion by hash
ank log "<what you learned>"  # renews the claim; working is what holds it
ank done                      # runs the verifiers itself and writes the proof
```

**Hand it to an agent.**

```
npx skills add haksolot/ank
```

[**Getting started**](docs/getting-started.md) walks all of that with real
output, including the two refusals a fresh repository will give you.

## What it looks like

A decision that constrains code, and a unit of work with what would prove it
finished. Both are markdown with YAML frontmatter, flat in `.ank/`:

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

```yaml
---
id: TASK-8f3a91c2d4e7
type: task
title: Migrate auth to opaque sessions
status: open
scope:
  - src/auth/**
blocked_by: [TASK-51c2a7f0b3d9]
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

## Why it works this way

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
ratification commit — and `ank check` compares. Editing a criterion to unblock
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

## Documentation

| If you want to | Read |
|---|---|
| use it, from install to a first finished task | [Getting started](docs/getting-started.md) |
| write a tool that reads or writes `.ank/` | [The file format](docs/format.md) |
| know why it is shaped this way, or need the normative answer | [The specification](docs/ank-spec-v1.1.md) |
| work on ank itself | [CLAUDE.md](CLAUDE.md), and the section below |

The specification is the source of truth. It argues the design and settles every
question the other documents defer to it; it is not a tutorial, and the first two
exist so that it does not have to be one.

## Status

Pre-v1, and the CLI runs on Linux, macOS and Windows.

**One command surface.** Every verb is available to every caller, and ank refuses
on state — a claim held elsewhere, a blocked task, a missing proof — never on who
is asking. `ank help` prints them all in one flat listing; `ank help <verb>`
answers about one — which is why no number appears here. Every verb the
specification specifies ships today, and `crates/ank-cli/tests/skill.rs` compares
the two lists in both directions, so a verb that stops shipping fails the suite
unless it is declared missing there, by name and with its task.

What an agent is *taught* is a smaller set, and that is what is frozen: the loop
above, plus `new`, `find` and `release`. The freeze is on `skill/SKILL.md`, which
is loaded on every session and therefore costs tokens on every call — not on the
dispatch table. An agent that types `ank graph` gets the graph; it was simply
never told the verb existed, and being untold is not being refused.

This repository is built by dogfooding its own format. The development plan lives
in `.ank/` — a DAG of tasks through `blocked_by`, and the decisions in
`.ank/adr/` — and the tool reads, claims and closes its own tasks. `ank check`
validates the corpus on every CI run.

## Working on ank

```
cargo build --release        # target/release/ank
cargo test                   # full suite, format conformance included
cargo fmt --check
ank check                    # validates .ank/: parse, round-trip, references
```

`.ank/` is opaque to an agent, the way `.git/` is: reach it with `ank show <id>`
for an entity whole, `ank find` to list, `ank context` to learn what binds. The
tool knows what the files do not — the context budget, the frozen criterion, who
holds which claim — and a `PreToolUse` hook in [`.claude/`](.claude/) refuses the
direct route rather than trusting anyone to remember. A human with an editor
keeps every power they had; `ank check` remains what notices.

`crates/ank-core/tests/golden/` is the format conformance suite, reusable by any
third-party tool: `valid/` must round-trip byte for byte once normalised,
`invalid/` must be rejected with the expected error. One valid file is in CRLF on
purpose and must come back in LF — the format is read in either and written in
one.

```
crates/ank-core   parser and data model — the reference implementation of the format
crates/ank-cli    the `ank` binary
docs/             the specification, getting started, the format
skill/            the bootstrap skill for agents
assets/           the project mark, one file per theme
.ank/             ank's own development plan, in the ank format
```

Conventions for agents are in [CLAUDE.md](CLAUDE.md).

## Licence

GPL-3.0 — see [LICENSE](LICENSE). The copyleft covers the tool's code, not the
format: your `.ank/` files, and the third-party tools that read or write them,
are not derivative works.

[releases]: https://github.com/haksolot/ank/releases/latest
