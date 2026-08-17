<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/ank-dark.svg">
<img src="assets/ank.svg" alt="" width="88" height="88"></picture></p>

<h1 align="center">ank</h1>

<p align="center"><strong>The stupid coordination tool</strong><br>
Tasks and architecture decisions in your repo, behind one CLI any coding agent can call.</p>

<p align="center"><a href="https://github.com/haksolot/ank/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/haksolot/ank/actions/workflows/ci.yml/badge.svg"></a>
<a href="https://github.com/haksolot/ank/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/haksolot/ank"></a>
<a href="LICENSE"><img alt="Licence" src="https://img.shields.io/badge/licence-Apache--2.0-blue"></a></p>

```sh
npm install -g @haksolot/ank     # the binary
npx skills add haksolot/ank      # the skill, into whichever agent you run
```

Needs **git 2.34 or newer**. Every other route is in [handing ank to an agent][agents].

---

An agent that spawns on your codebase can read every line of it. It cannot read
your tracker, your wiki, or the thread where you decided six months ago that
sessions must never be self-contained JWTs. So it writes plausible code that
breaks a rule nobody wrote down where it could be found.

Ank puts that layer in the repository, attached to the code it constrains, and
serves it through one command surface. `.ank/` is opaque to an agent, the way
`.git/` is: not a directory to grep, a CLI to call.

## The loop

What applies here, and what is takeable:

```console
$ ank context src/auth/

CONSTRAINTS (1 active)
  ADR-f848  Sessions are opaque, never self-contained JWTs

TASKS (1)
  TASK-2352  [open] Rotate a session on privilege change

> ank claim TASK-2352 to start
```

Taking it freezes the criterion by hash, so editing it later to get unstuck
unblocks nothing:

```console
$ ank claim TASK-2352
claimed TASK-235214d1655d rotate-a-session-on-privilege-change -> HEAD
```

The same verb now answers about the one task you hold, and serves the rule that
binds it **in full** rather than by title:

```console
$ ank context

TASK-2352  Rotate a session on privilege change

DONE_CRITERIA
  Logging in at a higher privilege level issues a new identifier and invalidates the old one.

CONSTRAINTS (1 active)
  ADR-f848  A session identifier is opaque and resolved server-side. No claim travels in the token.
```

Then work. Logging what you learn is also what holds the claim, and `done` runs
the verifiers the task declares rather than taking your word for the result:

```console
$ ank log "the old identifier has to be invalidated, not just replaced"
logged LOG-3c812d240fe3 on TASK-235214d1655d

$ ank done --proof commit:97fdae7
proof recorded: commit -> 97fdae7
TASK-235214d1655d -> done
```

`ank check` is the verb a pipeline runs: parse, round-trip, references, frozen
fields, orphaned claims. It exits **0** healthy, **8** on faults, **9** when the
environment failed rather than the work.

[Getting started](https://github.com/haksolot/ank/blob/main/docs/getting-started.md) walks all of it, from `ank init` onward.

<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/demo-dark.gif">
<img src="assets/demo.gif" alt="A terminal session: ank context serves a constraint saying every refusal must name the command that fixes it; ank graph shows which task is takeable; a task is claimed and its criterion frozen; the code written next produces exactly that message; ank done runs the declared verifier and records a hashed proof."></picture></p>

## Why it works this way

**Scope, not hierarchy.** Constraints and work are two planes joined only by globs.
A rule written last year binds work created today, and a glob is verifiable against
the filesystem where a label is not.

**Nobody declares themselves done.** An agent that reports its own result can simply
be wrong, so `ank done` runs the verifiers itself and records what ran, hashed.

**Freezing is verifiable, not defended.** The CLI cannot stop you editing a file and
does not pretend to. Frozen fields are anchored by a hash the editor does not
control, and `ank check` compares.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Undo, history and recovery
are git's. There is nothing to run.

One call is bounded at 8000 characters by default, roughly 2000 tokens. Behind it
four kinds of entity, all plain markdown with YAML frontmatter: an **ADR** that
constrains code, a **spec** that describes without binding, a **task** with what
would prove it finished, and a **log entry**, written once.

## What it is not

- **Not a tracker.** No cycles, estimates, velocity, roadmap or burndown.
- **Not a wiki.** Only what is actionable or binding for an agent goes in.
- **Not a security boundary.** It protects against drift, not against an attacker.

## Documentation

| If you want to | Read |
|---|---|
| go from install to a first finished task | [Getting started](https://github.com/haksolot/ank/blob/main/docs/getting-started.md) |
| hand it to an agent, whichever one you run | [Handing ank to an agent][agents] |
| build a tool on top of ank | [Integrating with ank](https://github.com/haksolot/ank/blob/main/docs/integrating.md) |
| write a tool that reads or writes `.ank/` | [The file format](https://github.com/haksolot/ank/blob/main/docs/format.md) |
| know how it compares to RAG, a wiki, or OKF | [How ank compares](https://github.com/haksolot/ank/blob/main/docs/alternatives.md) |
| open a pull request | [Contributing](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md) |
| report a vulnerability | [Security policy](https://github.com/haksolot/ank/blob/main/SECURITY.md) |

The specification is the source of truth and lives as ten accepted `spec` entities
in `.ank/`: `ank find --type spec` lists them, `ank show <id>` prints one whole.
This repository dogfoods its own format, so the plan is in there beside them.

Linux, macOS and Windows. The version is `0.x` deliberately: the loop and the exit
codes are specified and an agent can branch on them today, while the storage format
is not, and a major version is a promise about exactly that. Contributions are under
a [Code of Conduct](https://github.com/haksolot/ank/blob/main/CODE_OF_CONDUCT.md).

## Licence

Apache-2.0, whole. See [LICENSE](LICENSE). Every crate, the binary as distributed
and every channel that declares a licence say the same thing, so your `.ank/` files,
the tools that read them, and anything you build on top are yours. Ank was
GPL-3.0-only until 0.3.0 and the change is **prospective**: a release you already
received stays available to you under GPL-3.0.

[agents]: https://github.com/haksolot/ank/blob/main/docs/agents.md
