# Install

Two separate acts: the binary, which is the tool, and the skills, which teach an
agent how to use it. The skills do not install the binary, and the binary does
not install the skills unless you ask it to.

If you only want the short version, it is two lines:

    npm install -g @haksolot/ank     # the binary for your platform
    ank skills --install             # the skills, for your agent

Everything below is for the cases those two do not fit.

## What you need

- **git 2.34 or newer.** Not a convenience: claims live in git refs, and Ank
  checks the version at startup.
- **`sh`.** Verifiers run under `sh -c` on all three operating systems. On
  Windows it comes with Git for Windows, so requiring git makes it free.
- **Rust 1.95 or newer**, only if you build from source.

## The binary

Three routes install ank and no more (ADR-221aa5da440a): each is a command
served from this repository and derived from a published release, and none of
them owes anything to a registry with a gatekeeper of its own.

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

<!-- replay bare -->

    $ ank --version
    ank 0.8.0 (8310e75, skill 0d916cc3d9a5)

Three components: the version, the commit it was built from, and **the revision
of the skill it was built alongside**. The commit matters the first time you
suspect the binary in your hand is older than the behaviour you are reading
about. The revision answers the same question about the other half: it is the
value `skill/SKILL.md` carries under `metadata.revision`, so an agent that has
loaded the skill holds a string it can compare against the one its tool prints,
and can see for itself that its instructions predate its binary. Worth checking
when the skill came from a clone and the binary came from a release: those are
two points in history, and nothing forces them to be the same one.

The same executable carries every surface. A client that has no shell reaches
the verbs over MCP through `ank mcp`, and the background cache warmer is `ank
watch`: there is nothing further to install for either (ADR-1ea31c2f3c5a).
[The MCP server](mcp.md) says how a client is configured.

### Where the binary you run comes from

Worth stating once, because it costs time exactly where nobody expects it: **a
globally installed `ank` tracks the published release, not the tree you have
checked out.** The two are the same file only on the day of a release.

For most repositories that is the whole story: you are using Ank, not changing
it, and the published binary is the one you want. It matters when you are
working *on* a repository whose corpus is written by a binary newer than yours:
somebody else's release, or your own tree if you are contributing to Ank
itself. Then the tool managing the work and the tool being changed are different
versions, and both print the same `ank 0.8.0`. Only the commit separates them,
which is why `--version` carries it.

Contributors hit the sharper form of this. Building from source puts a binary in
`target/`, and running that one is what tests a change. But on Windows a
running executable cannot be relinked, so a command that rebuilds the tree while
`target/debug/ank` is the process running it fails on the lock. The habit that
avoids it is to copy the built binary somewhere outside `target/` and run the
copy, which means the binary managing the work drifts from the tree the moment
the tree moves. Rebuild and re-copy it after a merge, or accept that it answers
about the code it was built from.

One half of this the tool diagnoses on its own. A corpus whose entities declare
a schema newer than the binary reads is refused entity by entity, so every verb
that lists would answer short of them without a word; instead each says so
first:

<!-- replay ahead
$ ank init
$ ank new task --title "Ordinary task" --scope "**" --no-verify
$ ank new task --title "Written by a newer ank" --scope "**" --no-verify
$ f=$(grep -l "^title: Written by" .ank/entities/*.md) && sed 's/^schema: 4$/schema: 5/' "$f" > x && mv x "$f"
-->

    $ ank find --type task
    warning: corpus at schema 5, this binary reads 4: 1 entity left out of every listing
      -> no release is known to read schema 5: ank --version names the build, build from the tree or wait for a release
      TASK-c971  [open] Ordinary task

It warns and still answers, because the entities this build does understand are worth
having, and a corpus mid-migration is a real state rather than a broken one.

**The second line is one of two, and which one depends on whether a release can
help.** The schema a published version reads is stamped into the binary at build
time, from the newest tag's own source, so the message names the road that
actually resolves the state rather than the one that sounds like it does. Above,
no release reads the corpus -- a schema that landed on the default branch after
the last tag, which is the ordinary case for a contributor -- so it sends you to
the tree. Where a published release does read it, the binary in your hand is
simply old, and the line names the install instead:
`-> the binary is older than the corpus: ank --version names the build, npm install -g @haksolot/ank replaces it`.

Naming the install in the first case would fetch the build that had just
refused, and a reader who follows advice that visibly does nothing concludes the
tool is broken rather than that their copy is old.

The other half, an old binary reading an old corpus, is not detectable: nothing
in the files says a newer format exists. That one is `--version`, the update
below, and the paragraphs above.

### Updating

`ank update --check` reads the latest release from the repository releases are
published from, with `git ls-remote --tags`, and installs nothing:

<!-- replay update bin=/opt/ank dir=/srv/releases.git ANK_UPDATE_REPOSITORY=/srv/releases.git
$ git init -q --bare /srv/releases.git
$ r=/srv/releases.git && git -C $r tag "v$(ank --version | cut -d' ' -f2)" "$(git -C $r commit-tree -m release "$(git -C $r hash-object -t tree -w --stdin </dev/null)")"
-->

    $ ank update --check
    running  0.8.0
    latest   0.8.0
    up to date

It exits 0 whether or not a newer release exists, because 8 belongs to `check`,
and when one does its last line says `a newer release exists: ank update installs
it`. A script branches on `newer`:

<!-- replay update -->

    $ ank update --check --json
    {"contract":1,"current":"0.8.0","latest":"0.8.0","newer":false}

`ank update` installs that release through the route that placed the binary you
are running: `npm install -g @haksolot/ank@<version>` for a binary inside the npm
package, and otherwise the installer for your platform, told the directory the
binary already sits in. It downloads and unpacks nothing itself, so the checksum
is verified where it always was. At or above the latest release it installs
nothing and says so:

<!-- replay update -->

    $ ank update
    running  0.8.0
    latest   0.8.0
    up to date

Otherwise it prints the command it hands the install to before running it, and
exits with that command's code. `--version <v>` installs the release it names,
an older one included. It never installs the skills: when the release it
installed carries another skill revision than the binary it replaced, it names
`ank skills --install` in one line and leaves running it to you.

Only this verb asks: no other verb checks for a newer release or announces one,
so nothing reaches the network for it until you run `ank update`. A binary built
under a cargo target directory was placed by no route, so `update` refuses it at
exit 7 and names `cargo build` instead. Where the releases are read from is
[`ANK_UPDATE_REPOSITORY`](environment.md#ank_update_repository).

## The skills

Six plain markdown files, one per skill. Each is the only copy that exists in
git. Every route below points at it, or, for the binary, carries the copy its
build read, so no route holds a copy somebody keeps in step by hand.

    ank           skill/SKILL.md           the contract
    ank-plan      skill/plan/SKILL.md      interview a goal into ADRs, specs and tasks
    ank-drift     skill/drift/SKILL.md     audit decisions against the code
    ank-loop      skill/loop/SKILL.md      work the backlog autonomously
    ank-tdd       skill/tdd/SKILL.md       drive an implementation test-first
    ank-diagnose  skill/diagnose/SKILL.md  work a defect back to its cause

[`skill/SKILL.md`](https://github.com/haksolot/ank/blob/main/skill/SKILL.md) is
the one an agent loads by default, and it is self-sufficient: why ank is shaped
as it is, the verbs grouped by the moment each is used, and the rules that are
not negotiable. It names the other five so an agent reaching for an activity
knows what to load, and never depends on them being installed. The five carry a
policy each and are loaded when the activity calls for them (ADR-e4a5a8873fe3).
What an agent does with them once installed is [Multi-agent
work](multi-agent.md).

The routes:

    ank skills --install                      from the binary you installed, nothing cloned
    npx skills add haksolot/ank               a machine with node and no ank
    /plugin marketplace add haksolot/ank      Claude Code, as a plugin
    pi install npm:@haksolot/ank              pi, binary and skill together
    pi install git:github.com/haksolot/ank    pi, from source
    by hand                                   copy skill/SKILL.md where your harness loads it

### From the binary

The build embeds the six files, so the binary in your hand already carries the
skills written for it. `ank skills` lists them, with the revision each file
declares. The revisions below are the ones this page was written against; yours
are whatever `ank --version` names, and printing them is what lets you compare:

<!-- replay bare -->

    $ ank skills
    ank           0d916cc3d9a5  Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
    ank-diagnose  b5d9c0b96462  Work a defect back to its cause before changing anything, and close it with a regression test. Use when a claimed task's criterion names a defect in a repository with a .ank/ directory.
    ank-drift     36cf5808e95e  Audit the decisions in .ank/ against the current code and report what no longer holds. Use when asked whether ADRs, specs, or tasks are still accurate, after a milestone, or when the corpus and the code seem to disagree.
    ank-loop      9f00f607cdb8  Work through the open tasks in .ank/ without supervision, one claim at a time. Use when asked to work the backlog, chain tasks, or run autonomously in a repository with a .ank/ directory.
    ank-plan      83130b664c7e  Interview a goal into decisions and tasks recorded in .ank/. Use when someone brings a feature, change, or problem to plan before implementation in a repository with a .ank/ directory.
    ank-tdd       96d151e6812e  Drive an implementation test-first, red before green, against a claimed task's frozen criterion. Use when implementing a task in a repository with a .ank/ directory.

One line per file: the name, the revision that file declares, and the
description whole, wrapped by nothing. The revisions are the build's, not the
repository's, which is what makes them comparable with the one `--version`
printed. Run inside a corpus, the same verb prints a second block under that
listing, which [Multi-agent work](multi-agent.md#the-methods-a-corpus-uses)
reads.

`ank skills --install` writes them into a new directory under the temporary
directory and hands that directory to the `skills` CLI below. It never asks: the
flag is the consent. It prints two lines of its own, `wrote 6 skills to
<directory>` and `running: npx skills add <directory>`, and everything after
them is the `skills` CLI's. Run from a Claude Code session, that part went on
like this:

    ●   claude-code_2-1-270_agent  Agent detected — installing non-interactively
    ◇  Source: C:\Users\you\AppData\Local\Temp\ank-skills-25808-138072400-0
    ◇  Local path validated
    ◇  Found 6 skills
    ●  Installing all 6 skills
    ...
    ◇  Installed 6 skills
      ✓ .\.agents\skills\ank
        universal: Amp, Antigravity, Antigravity CLI, Cline, Codex +15 more
      ...
      ✓ .\.agents\skills\ank-tdd
        universal: Amp, Antigravity, Antigravity CLI, Cline, Codex +15 more

    └  Done!  Review skills before use; they run with full agent permissions.

Nothing is cloned: the files are the binary's, so they match the build
`ank --version` names rather than whatever the repository holds today. `npx`
itself is not offline, and fetches the `skills` CLI the first time it has none
cached. With an agent detected, the run above installed copies into
`.agents/skills` of the directory it ran from, and they outlive the temporary
directory. Without `npx` on your `PATH`, the verb prints the directory it wrote
and the command to run later, and exits 0.

### The `skills` CLI

The route for a machine that has node and no ank. Detects what you run (Claude
Code, Codex, Cursor, OpenCode and some thirty more) and links each one to a
single copy. Ask it what it found before you let it install:

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

Drop `--list` to install them all, or name one with `--skill`. That route
installs the skills, not the binary.

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

    ank 0.8.0
      Description: Read a repository's tasks and binding constraints, claim work, and finish it with proof.
      Source: ank@ank

    Component inventory
      Skills (6)  ank, ank-diagnose, ank-drift, ank-loop, ank-plan, ank-tdd
      Agents (0)
      Hooks (0)
      MCP servers (0)
      LSP servers (0)

    Projected token cost
      Always-on:   ~409 tok   added to every session

    Per-component (rounded)
      component     always-on  on-invoke
      ank                 ~50      ~2.2k
      ank-plan            ~70       ~930
      ank-drift           ~80       ~540
      ank-loop            ~70      ~1.6k
      ank-tdd             ~60      ~1.3k
      ank-diagnose        ~70      ~1.9k

      On-invoke cost is paid each time a skill or agent fires.
      Token counts are estimates and may differ from actual usage.

The four zeroes are the inventory: the plugin is six skills and nothing else --
no agent, no hook, no server of any kind, so nothing of it runs unless you call
`ank` yourself. The `Description` line is the `ank` skill's own rather than the
one `plugin.json` carries, and the counts are estimates that move with the skill
files and with whatever does the counting.

**Read the two columns as what they are.** Always-on is six descriptions, paid
by every session whether or not anything fires; on-invoke is a body, paid by
the session that wanted it. Splitting the teaching moved cost from the second
column to the first, which is the trade the plural skill system makes: an agent
executing a task no longer loads the planning policy it will not use, and every
session pays a little more to know the policies exist. The ceiling on the bodies
is kept anyway, because the by-hand route below copies whole files into whatever
a harness loads, and some harnesses load all of it every session, so it bounds
the worst route rather than the measured one.

### pi

From the registry, or from a clone:

    $ pi install npm:@haksolot/ank
    $ pi install git:github.com/haksolot/ank
    Installed git:github.com/haksolot/ank

The git route clones the repository and reads its `pi` manifest; the npm route
takes the published package, which carries the skill beside the binary.

One thing to expect from the npm route: pi loads resources, it does not put
executables on your `PATH`. The binary is inside the package it installed, and
`ank` will still not be a command you can type until you install it by one of
the routes above.

### By hand

Each skill is one file with nothing generated in it. Where none of the routes
above fits your harness, copy `skill/SKILL.md` into whatever that harness loads
and you have lost nothing: the routes exist to save you a copy, not to add
anything to it. Copy `skill/plan/SKILL.md`, `skill/drift/SKILL.md`,
`skill/loop/SKILL.md`, `skill/tdd/SKILL.md` and `skill/diagnose/SKILL.md`
beside it for the activity policies, or copy none of them and keep the
contract, which stands alone.

Next: [the quickstart](quickstart.md), from `ank init` to a first finished task.
