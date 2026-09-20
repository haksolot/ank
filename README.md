

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
npm install -g @haksolot/ank     # the wrapper, and the binary for your platform beside it
ank skills --install             # the skills out of the binary, handed to npx skills add
```

The skill is not the binary. The package carries six `SKILL.md` for an installer
that reads them, and `npm install -g` places none of them for an agent: the
second line does that. It writes the six out of the executable, which needs no
network, and then hands the directory to `npx skills add`, which does. So what
an agent receives is the build's own copy, and only the placing of it goes near
a registry. Needs **git 2.34 or newer**; the npm route needs **Node 18 or
newer**.

Two other routes install the same release, and there are no more than these
three:

```sh
curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh   # Linux and macOS
irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1 | iex        # Windows
```

Both fetch the archive and the `.sha256` published beside it and refuse before
unpacking if the two disagree. [Handing ank to an agent][agents] has what each
one covers, and the honest answer when none of the three fits.

```sh
ank update --check               # the running version and the latest release; installs nothing
ank update                       # the latest release, through the route that placed this binary
```

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

Six verbs carry the loop, which is the group `ank help` prints first: `ank context`
for what binds here and what is takeable, `ank claim` to take a task and freeze its
criterion, `ank show` for the entity whole, `ank log` while you work, `ank done` to
finish with a proof that `ank check` can verify afterwards, and `ank release` to hand
a task back with the reason recorded.
[Getting started][start] walks all of it with real output, from `ank init` onward.

Installing puts one executable on your `PATH`, and every surface ank has is a
verb of it. A client with no shell reaches the same verbs over MCP through
`ank mcp`, with no second file to fetch or discover, and [getting started][start]
carries the configuration to paste, for Claude Code, Claude Desktop and Cursor.

`ank tui` is the same corpus full-screen, for a human at a terminal. `ank watch`
keeps the corpora you declare warm so the next `ank` answers sooner, and nothing
depends on it running.

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
