# Handing ank to an agent

The skill says *how* to use ank; it does not install the binary. Those are two
separate acts, and this page covers both: the routes that reach an agent, and
the binary channels the two commands in the README do not cover.

If you only want the short version, it is in the README: `npm install -g
@haksolot/ank` for the binary, `npx skills add haksolot/ank` for the skill.
Everything below is for the cases those two do not fit.

## The skill

Six plain markdown files, one per skill. Each is the only copy that exists in
git, and every route below points at it rather than holding one of its own, so
no route can fall behind it.

    ank           ../skill/SKILL.md           the contract
    ank-plan      ../skill/plan/SKILL.md      interview a goal into ADRs and tasks
    ank-drift     ../skill/drift/SKILL.md     audit decisions against the code
    ank-loop      ../skill/loop/SKILL.md      work the backlog autonomously
    ank-tdd       ../skill/tdd/SKILL.md       drive an implementation test-first
    ank-diagnose  ../skill/diagnose/SKILL.md  work a defect back to its cause

[`../skill/SKILL.md`](../skill/SKILL.md) is the one an agent loads by default,
and it is self-sufficient: why ank is shaped as it is, the verbs grouped by the
moment each is used, and the rules that are not negotiable. It names the other
five so an agent reaching for an activity knows what to load, and never
depends on them being installed. The five carry a policy each and are loaded
when the activity calls for them (ADR-e4a5a8873fe3).

**What that costs a session is the frontmatter, not the page.** The projection
below runs from `~50` to `~80 tok` always-on per skill, and that is a skill's
`name` and `description`; a body is read when its skill is invoked. The ceiling
on the bodies is kept anyway, because the by-hand route at the end of this page
copies whole files into whatever a harness loads, and some harnesses load all of
it every session, so it bounds the worst route rather than the measured one.

One convention it carries is worth knowing before you watch an agent follow it:
**`.ank/` is opaque to an agent, the way `.git/` is.** Reading goes through
`ank show`, `ank find` and `ank context`; writing goes through the verbs. The CLI
knows what the files do not: the context budget, the frozen criterion, who holds
which claim. A human with an editor keeps every power they had.

### The `skills` CLI

Detects what you run (Claude Code, Codex, Cursor, OpenCode and some thirty more)
and links each one to a single copy. Ask it what it found before you let it
install:

    $ npx skills add haksolot/ank --list
    Source: https://github.com/haksolot/ank.git
    Repository cloned
    Found 6 skills

    Available Skills
    Ank
      ank
        Read a repository's tasks and binding constraints, claim work, and
        finish it with proof. Use when working in a repo that has a .ank/
        directory.
      ank-diagnose
        Work a defect back to its cause before changing anything, and close it
        with a regression test. ...
      ank-drift
        Audit the decisions in .ank/ against the current code and report what
        no longer holds. ...
      ank-loop
        Work through the open tasks in .ank/ without supervision, one claim at
        a time. ...
      ank-plan
        Interview a goal into decisions and tasks recorded in .ank/. ...
      ank-tdd
        Drive an implementation test-first, red before green, against a claimed
        task's frozen criterion. ...

    Use --skill <name> to install specific skills

Drop `--list` to install them all, or name one with `--skill`.

This is the widest route and the least anchored one. It finds the skill through
its own recursive scan rather than through a manifest, `skill/` not being one of
the directories it looks in by name, so it works because the fallback works. If
a future version of that CLI narrows its search, this is the route that breaks
first, and the hand copy below is the answer.

### Claude Code, as a plugin

This repository serves as its own marketplace:

    /plugin marketplace add haksolot/ank
    /plugin install ank@ank

`claude plugin details ank` then tells you what it costs, which is the question
worth asking of anything loaded on every session:

    Tasks and architecture decisions in your repo, behind one CLI any coding
    agent can call.

    Component inventory
      Skills (6)  ank, ank-diagnose, ank-drift, ank-loop, ank-plan, ank-tdd

    Projected token cost
      Always-on:   ~411 tok   added to every session

    Per-component (rounded)
      component     always-on  on-invoke
      ank                 ~50      ~2.1k
      ank-plan            ~70       ~830
      ank-drift           ~80       ~540
      ank-loop            ~70      ~1.3k
      ank-tdd             ~60      ~1.3k
      ank-diagnose        ~70      ~1.9k

**Read the two columns as what they are.** Always-on is six descriptions, paid
by every session whether or not anything fires; on-invoke is a body, paid by
the session that wanted it. Splitting the teaching moved cost from the second
column to the first, which is the trade the plural skill system makes: an agent
executing a task no longer loads the planning policy it will not use, and every
session pays a little more to know the policies exist.

### pi

From the registry, or from a clone:

    $ pi install npm:@haksolot/ank
    $ pi install git:github.com/haksolot/ank
    Installed git:github.com/haksolot/ank

The git route clones the repository and reads its `pi` manifest; the npm route
takes the published package, which carries the skill beside the binary.

One thing to expect from the npm route: pi loads resources, it does not put
executables on your `PATH`. The binary is inside the package it installed, and
`ank` will still not be a command you can type until you install it by one of the
routes below.

### By hand

Each skill is one file with nothing generated in it. Where none of the routes
above fits your harness, copy `skill/SKILL.md` into whatever that harness loads
and you have lost nothing: the routes exist to save you a copy, not to add
anything to it. Copy `skill/plan/SKILL.md`, `skill/drift/SKILL.md`,
`skill/loop/SKILL.md`, `skill/tdd/SKILL.md` and `skill/diagnose/SKILL.md`
beside it for the activity policies, or copy none of them and keep the
contract, which stands alone.

## The binary

The README names npm because it is the shortest line that works on most
machines. Three routes install ank and no more (ADR-221aa5da440a): each is a
command served from this repository and derived from a published release, and
none of them owes anything to a registry with a gatekeeper of its own.

**npm**, on any machine that has node, and the one to reach for behind a
firewall that blocks downloading a bare executable but lets a registry through:

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binary is inside the package: one package per platform, installed through
`optionalDependencies`, and no `postinstall` fetches anything. A `postinstall`
download would die behind the very firewall this channel exists to cross, and
would do it after the install looked like it had worked.

It covers `linux x64`, `darwin arm64` and `win32 x64`. On anything else, an
Intel Mac or a linux arm64 box, the wrapper exits 9 and names `cargo install`,
which is the honest answer rather than a silent failure.

**`curl | sh`**, on Linux and macOS:

    curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh

It reads your platform from `uname`, fetches the archive and the `.sha256`
published beside it, and refuses before unpacking if the two disagree. On a
platform no release carries it refuses by name and lists what does exist, rather
than ending in silence.

**A PowerShell one-liner**, on Windows, the same shape as the line above it:

    irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1 | iex

It verifies the `.sha256` the same way, runs under Windows PowerShell 5.1 as
well as PowerShell 7, and moves an `ank.exe` that is currently running aside
instead of failing to overwrite it, so an upgrade works from a shell that
already has ank on its `PATH`.

Neither of the two below is a route this project offers; both are named because
they are the honest answer when none of the three fits. The [releases
page](https://github.com/haksolot/ank/releases/latest) carries the archives the
two installers fetch, one per target with a `.sha256` beside it, and unpacking
one by hand is exactly what those installers do for you. And the tree builds:

    cargo install --git https://github.com/haksolot/ank ank-cli

`--git` because nothing publishes to crates.io. That puts `ank` in
`~/.cargo/bin`, and needs Rust 1.95 or newer and a C compiler.

No package manager ships ank. Homebrew, Scoop, apt, winget and the AUR each
carried a channel here or an attempt at one, and every one of them was
withdrawn: the measurements are in ADR-221aa5da440a, and putting one back is a
supersession of that decision rather than an addition beside it.

Whichever you took, check it answers:

    $ ank --version
    ank <version> (<commit>, skill <revision>)

Three components: the version, the commit it was built from, and **the revision
of the skill it was built alongside**. That last one is the value
`skill/SKILL.md` carries under `metadata.revision`, so an agent that has loaded
the skill holds a string it can compare against the one its tool prints, and can
see for itself that its instructions predate its binary. Worth checking when the
skill came from a clone and the binary came from a release: those are two points
in history, and nothing forces them to be the same one.

## One agent, one working tree, one identity

The nominal case is a tree per agent, a clone or a `git worktree`, each on its
own branch.

`$ANK_AGENT` names the session, and falls back to `<user>@<hostname>`. That
fallback is the thing to override: two sessions in one tree with no `ANK_AGENT`
are **one agent** as far as the claim refs are concerned, so they share a claim
instead of arbitrating over it, and the second one is quietly refused work it
should have been given.

    ANK_AGENT=marie-2@laptop ank claim 51c2

Several agents in one tree runs, and it is a degraded mode rather than the
design. What arbitrates properly is a `git worktree` per agent, because every
worktree of a repository shares `refs/ank/`, so the compare-and-swap settles
them. Separate clones are arbitrated only when there is a remote, which is what
the push carries.

Identity here is declared, not proved. `$ANK_AGENT` is set by the caller, so it
records who was working rather than restricting who may. Nothing in ank refuses
on identity; the refusals are on state, and the one hard authority line is the
signed ratification commit `ank accept` produces.

## Parallel work and integration

The section above says who works where. This one assembles the whole run:
several tasks, several agents, one change landing on the default branch.

**Parallelism is derived, not declared.** `blocked_by` is the only relation
between tasks, and it is the only thing that serializes work. Tasks whose
blockers are finished are ready together, and `ank context` computes that
mechanically: every open task in the perimeter, the ready ones first, ordered by
how many other tasks each would unblock. Do not serialize independent tasks
because they belong to the same change. If the order matters, that is a
`blocked_by`; if there is no `blocked_by`, the order is a fiction. `ank graph`
prints the DAG when you want the shape rather than the next move.

**One branch per task.** Each agent claims its task, works in its own tree on
its own branch, and finishes there. `ank done` proves the task in the working
tree it ran in: the verifiers ran against those files and the proof records
their hash. It does not prove the change merges, or that the combined system
works. That gap is held by the refs, not by convention: `done` turns the claim
ref into a completion record naming the commit and the branch, every other tree
answers `finished on another branch (commit …, branch …), not merged here yet`
until the merge lands, and `check` prunes the record only once the default
branch says the task is done.

**Integration is a task.** When several tasks form one change, the whole is
verified the way the parts were: an ordinary task, `blocked_by` each part, with
its own criterion and its own verifiers. This is the spec's model rather than a
workaround: `blocked_by` is a DAG with no rollup precisely because a parent
that completes when its children do is completion without proof, and the seam
between the parts is exactly where integration regressions live. The
integration task becomes ready when the last part finishes; whoever claims it
merges the branches, runs the combined verification, and records `done` like
any other task.

Where the branches meet is git's business, and both shapes are legitimate:

- **Independent tasks merge to the default branch directly.** Two tasks that
  share nothing need no ceremony between them, and no integration task either.
- **A multi-task change goes through an integration branch.** Branch it off the
  default branch, merge each task's branch into it, resolve conflicts there,
  and point the integration task's verification at the combined result. The
  default branch receives one verified change instead of three partial ones.

**What ank will not do.** No verb creates a worktree, names a branch, or merges
one. Tasks, claims, criteria, dependencies and proofs are ank's plane; branches,
worktrees, merges and history are git's, and git is already good at them. The
one place the planes touch is `accept`, which runs on the default branch and
nowhere else.

## Where to go next

- [getting-started.md](getting-started.md): install to a first finished task,
  with real output, including the refusals a fresh repository will give you.
- [format.md](format.md): the file format, for anyone writing a tool that reads
  or writes `.ank/`.
- The specification, the source of truth for everything above, which lives as
  ten `spec` documents in `.ank/`: `ank find --type spec` lists them and
  `ank show <id>` prints one whole.
- `ank help` lists every verb, `ank help <verb>` answers about one.
