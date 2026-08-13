<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/ank-dark.svg">
<img src="assets/ank.svg" alt="" width="88" height="88"></picture></p>

<h1 align="center">ank</h1>

<p align="center"><strong>The stupid coordination tool</strong><br>
Tasks and architecture decisions in your repo, behind one CLI any coding agent can call.</p>

<p align="center"><a href="https://github.com/haksolot/ank/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/haksolot/ank/actions/workflows/ci.yml/badge.svg"></a>
<a href="https://github.com/haksolot/ank/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/haksolot/ank"></a>
<a href="LICENSE"><img alt="Licence" src="https://img.shields.io/badge/licence-GPL--3.0-blue"></a></p>

An agent that spawns on your codebase can read every line of it. It cannot read
your tracker, your wiki, or the thread where you decided six months ago that
sessions must never be self-contained JWTs. So it writes plausible code that
breaks a rule nobody wrote down where it could be found.

Ank puts that layer in the repository, attached to the code it constrains, and
serves it through one command surface. `.ank/` itself is opaque to an agent, the
way `.git/` is — not a directory to grep, a CLI to call.

## What an agent gets

```
$ ank context src/auth/

CONSTRAINTS (2 active)
  ADR-3c7e  Do not introduce self-contained JWTs for user auth.
            Every session goes through the Redis store.
  ADR-8b41  Rate limiting mandatory on every public endpoint.

TASKS (2)
  TASK-8f3a  [claimed:pi@host-3] Migrate auth to opaque sessions
  TASK-51c2  [open] Add secret rotation

> ank claim 51c2 to start
```

One call, and the answer is bounded — 8000 characters by default, roughly 2000
tokens. Behind it two kinds of entity: a decision that constrains code, and a
unit of work with what would prove it finished. Both are plain markdown with YAML
frontmatter, joined to the code by nothing but globs.

## Install

```
npm install -g @haksolot/ank     # the binary
npx skills add haksolot/ank      # the skill, into whichever agent you run
```

Ank needs **git 2.34 or newer**. Every other route — release binaries, `cargo`,
the Claude Code plugin, `pi`, a hand copy, or a platform npm does not carry — is
in [handing ank to an agent][agents].

## Why it works this way

**Scope, not hierarchy.** Constraints and work are two independent planes joined
only by globs. A constraint written last year applies to work created today, and
scope is verifiable where a label is not: a glob is confronted with the filesystem.

**Nobody declares themselves done.** A task names verifiers; `ank done` runs them
itself and records what actually ran, hashed. An agent that reports its own
result can simply be wrong.

**Freezing is verifiable, not defended.** The CLI cannot stop anyone editing a
file and does not pretend to. Every frozen field is anchored by a hash the editor
does not control, and `ank check` compares. Editing a criterion to unblock
yourself unblocks nothing; it makes the divergence visible.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Parallel agents are the
nominal case, not an extension — a worktree per agent, a branch per task,
`blocked_by` for what actually waits; the assembled workflow is in
[handing ank to an agent][agents]. Undo, history and recovery are git's. There
is nothing to run.

## Where this sits

Ank is not the first curated layer over a codebase. Here is where it differs.

**Against retrieval.** RAG answers *what looks relevant to this query*;
`ank context src/auth/` answers *what applies to this path*. The first ranks by
similarity, the second is a set-membership test — and for a constraint that is
the whole point, because a rule that should have bound but ranked seventh did not
bind, and nothing says so. No embeddings, no index service, no re-indexing per
commit. The cost is real: somebody has to have written the constraint and its
scope. Ank does not replace search over a corpus nobody curated; it replaces the
wiki page that was never found.

**Against an LLM-maintained wiki.** Karpathy's [pattern][wiki] is three layers —
raw sources, a wiki the model owns, a schema file — and three operations: ingest,
query, lint. Ank has that shape and differs on what the middle layer *is*. A wiki
page is derived from sources and can be regenerated if lost; an ADR records a
choice that has none. Somebody decided opaque sessions over JWTs, and the record
is the only copy — nothing can re-ingest its way back to it. Hence hash-anchored
and signed at ratification rather than rewritten, and a lint that is mechanical
rather than a model re-reading for contradictions.

**Against OKF.** Google's [Open Knowledge Format][okf] is the closest relative,
reached independently: markdown with YAML frontmatter, no server, no SDK,
identity carried by the path, the format as the contract so producer and consumer
stay swappable. Its v0.2 trust signals are the same instinct as ank's proofs, and
ank has adopted two outright — the actor convention and `verified`. One axis
diverges, deliberately on both sides. OKF tells consumers never to reject a
document for what it lacks; ank rejects an unknown field. OKF optimises for
knowledge crossing organisations, where a rejected document is knowledge lost;
ank optimises for a criterion that cannot be quietly weakened, where an
accepted-but-malformed file is a rule that silently stopped applying.

Efficiency is two numbers rather than an adjective: the skill an agent loads
costs about 58 tokens per session, and orientation is bounded at 8000 characters.

## What it is not

- **Not a tracker.** No cycles, estimates, velocity, roadmap or burndown.
- **Not a wiki.** Only what is actionable or binding for an agent goes in.
- **Not a security boundary.** It protects against drift, not against an attacker.

## Documentation

| If you want to | Read |
|---|---|
| go from install to a first finished task | [Getting started](https://github.com/haksolot/ank/blob/main/docs/getting-started.md) |
| hand it to an agent, whichever one you run | [Handing ank to an agent][agents] |
| write a tool that reads or writes `.ank/` | [The file format](https://github.com/haksolot/ank/blob/main/docs/format.md) |
| know why it is shaped this way | [The specification](https://github.com/haksolot/ank/blob/main/docs/ank-spec-v1.1.md) |
| open a pull request | [Contributing](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md) |
| report a vulnerability | [Security policy](https://github.com/haksolot/ank/blob/main/SECURITY.md) |

The specification is the source of truth; the others exist so it does not have to
be a tutorial. Pre-v1, on Linux, macOS and Windows. This repository dogfoods its
own format: the plan lives in `.ank/`, and the tool reads, claims and closes its own tasks, under a [Code of Conduct](https://github.com/haksolot/ank/blob/main/CODE_OF_CONDUCT.md).

## Licence

GPL-3.0 — see [LICENSE](LICENSE). The copyleft covers the tool's code, not the
format: your `.ank/` files and the tools that read them are not derivative works.

[agents]: https://github.com/haksolot/ank/blob/main/docs/agents.md
[okf]: https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf
[wiki]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
