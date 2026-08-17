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

<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/demo-dark.gif">
<img src="assets/demo.gif" alt="A terminal session: ank context serves a constraint saying every refusal must name the command that fixes it; ank graph shows which task is takeable; a task is claimed and its criterion frozen; the code written next produces exactly that message; ank done runs the declared verifier and records a hashed proof."></picture></p>

Four verbs carry the loop: `ank context` for what binds here and what is takeable,
`ank claim` to take a task and freeze its criterion, `ank log` while you work, and
`ank done` to finish with a proof that `ank check` can verify afterwards.
[Getting started][start] walks all of it with real output, from `ank init` onward.

## What it is not

- **Not a tracker.** No cycles, estimates, velocity, roadmap or burndown.
- **Not a wiki.** Only what is actionable or binding for an agent goes in.
- **Not a security boundary.** It protects against drift, not against an attacker.

## Documentation

| If you want to | Read |
|---|---|
| go from install to a first finished task | [Getting started][start] |
| hand it to an agent, whichever one you run | [Handing ank to an agent][agents] |
| build a tool on top of ank | [Integrating with ank](https://github.com/haksolot/ank/blob/main/docs/integrating.md) |
| write a tool that reads or writes `.ank/` | [The file format](https://github.com/haksolot/ank/blob/main/docs/format.md) |
| know how it compares to RAG, a wiki, or OKF | [How ank compares](https://github.com/haksolot/ank/blob/main/docs/alternatives.md) |
| open a pull request | [Contributing](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md) |
| report a vulnerability | [Security policy](https://github.com/haksolot/ank/blob/main/SECURITY.md) |

The specification is the source of truth and lives as ten accepted `spec` entities
in `.ank/`: `ank find --type spec` lists them, `ank show <id>` prints one whole.

Linux, macOS and Windows. The version is `0.x` deliberately: the loop and the exit
codes are specified and an agent can branch on them today, while the storage format
is not, and a major version is a promise about exactly that. Contributions are under
a [Code of Conduct](https://github.com/haksolot/ank/blob/main/CODE_OF_CONDUCT.md).

## Licence

Apache-2.0. See [LICENSE](LICENSE).

[agents]: https://github.com/haksolot/ank/blob/main/docs/agents.md
[start]: https://github.com/haksolot/ank/blob/main/docs/getting-started.md
