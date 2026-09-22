<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/ank-dark.svg">
<img src="assets/ank.svg" alt="" width="88" height="88"></picture></p>

<h1 align="center">ank</h1>

<p align="center"><strong>The stupid coordination tool</strong><br>
Tasks and architecture decisions in your repo, behind one CLI any coding agent can call.</p>

<p align="center"><a href="https://github.com/haksolot/ank/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/haksolot/ank/actions/workflows/ci.yml/badge.svg"></a>
<a href="https://github.com/haksolot/ank/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/haksolot/ank"></a>
<a href="LICENSE"><img alt="Licence" src="https://img.shields.io/badge/licence-Apache--2.0-blue"></a></p>

An agent can read every line of your code, but not your tracker, your wiki, or
the thread where you decided that sessions must never be self-contained JWTs.
Ank keeps those decisions and the work in `.ank/`, attached to the code they
constrain, and serves them through one CLI.

<p align="center"><picture>
<source media="(prefers-color-scheme: dark)" srcset="assets/demo-dark.gif">
<img src="assets/demo.gif" alt="A terminal session: ank context serves a constraint saying every refusal must name the command that fixes it; ank graph shows which task is takeable; a task is claimed and its criterion frozen; the code written next produces exactly that message; ank done runs the declared verifier and records a hashed proof."></picture></p>

## Install

```sh
npm install -g @haksolot/ank     # the binary for your platform
ank skills --install             # the skills, for your agent
```

Linux, macOS and Windows; needs git 2.34 or newer. The shell installers and
every other route are in [Install][install]. The version is `0.x` on purpose: the loop and
the exit codes are specified, the storage format is not yet.

## The loop

```sh
ank context <path>    # what binds this perimeter, and what is takeable
ank claim <id>        # take a task and freeze its criterion
ank show <id>         # the entity whole
ank log "<message>"   # what you learned, while you work
ank done              # run the declared verifiers and record the proof
ank release --reason "<why>"   # hand the task back
```

[The quickstart][start] walks it from `ank init` onward, with real output.

## What it is not

- **Not a tracker.** No cycles, estimates, velocity or roadmap.
- **Not a wiki.** Only what is actionable or binding for an agent goes in.
- **Not a security boundary.** It protects against drift, not against an attacker.

## Documentation

The documentation is published at <https://haksolot.github.io/ank/>, built from
[`docs/`](https://github.com/haksolot/ank/tree/main/docs) on every merge.

- **Using ank**: [Install][install], [the quickstart][start], [proof and
  verifiers](https://haksolot.github.io/ank/proof.html), [claims and
  identity](https://haksolot.github.io/ank/claims.html), [reading `ank
  check`](https://haksolot.github.io/ank/check.html), [CI](https://haksolot.github.io/ank/ci.html),
  [multi-agent work](https://haksolot.github.io/ank/multi-agent.html)
- **Reference**: [exit codes](https://haksolot.github.io/ank/exit-codes.html),
  [the file format](https://haksolot.github.io/ank/format.html),
  [environment variables](https://haksolot.github.io/ank/environment.html),
  [the specification](https://haksolot.github.io/ank/specification.html)
- **Integrating**: [the machine surface](https://haksolot.github.io/ank/integrating.html),
  [the MCP server](https://haksolot.github.io/ank/mcp.html),
  [the watcher](https://haksolot.github.io/ank/watch.html)
- **Maintaining**: [ratifying](https://haksolot.github.io/ank/ratifying.html),
  [releasing](https://haksolot.github.io/ank/releasing.html),
  [CI jobs](https://haksolot.github.io/ank/ci-jobs.html)
- [Contributing](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md) · [Security](https://github.com/haksolot/ank/blob/main/SECURITY.md) · [Code of Conduct](https://github.com/haksolot/ank/blob/main/CODE_OF_CONDUCT.md)

## Licence

Apache-2.0. See [LICENSE](LICENSE).

[install]: https://haksolot.github.io/ank/install.html
[start]: https://haksolot.github.io/ank/quickstart.html
