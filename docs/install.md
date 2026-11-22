# Install

Two separate acts: the binary, which is the tool, and the skills, which teach an
agent how to use it. Installing one leaves the other where it was, unless you
ask the binary to place the skills.

The short version is two lines:

    npm install -g @haksolot/ank     # the binary for your platform
    npx skills add haksolot/ank      # the skills, for your agent

Everything below is for the cases those two do not fit.

## What you need

- **git 2.34 or newer.** Claims live in git refs, and Ank checks the version at
  startup.
- **`sh`.** Verifiers run under `sh -c` on all three operating systems. On
  Windows it comes with Git for Windows, so requiring git makes it free.
- **Rust 1.95 or newer**, only if you build from source.

## The binary

Three routes install ank, and only three: each is a command served from this
repository and derived from a published release, and none of them owes anything
to a registry with a gatekeeper of its own.

**npm**, on any machine that has node, and the one to reach for behind a
firewall that blocks downloading a bare executable but lets a registry through:

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binary is inside the package: one package per platform, installed through
`optionalDependencies`, with no `postinstall` step fetching anything. A
`postinstall` download would die behind the very firewall this channel exists
to cross, after the install looked like it had worked.

It covers `linux x64`, `linux arm64`, `darwin arm64` and `win32 x64`. On
anything else, an Intel Mac for one, the wrapper exits 9 and names `cargo install`.

**`curl | sh`**, on Linux and macOS:

    curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh

It reads your platform from `uname`, fetches the archive and the `.sha256`
published beside it, and refuses before unpacking if the two disagree. On a
platform no release carries, it refuses by name and lists the platforms that
exist.

**A PowerShell one-liner**, on Windows, the same shape as the line above it:

    irm https://raw.githubusercontent.com/haksolot/ank/main/install.ps1 | iex

It verifies the `.sha256` the same way, runs under Windows PowerShell 5.1 as
well as PowerShell 7, and moves a running `ank.exe` aside before writing the new
one, so an upgrade works from a shell that already has ank on its `PATH`.

**Where an archive came from.** The `.sha256` beside an archive catches a
truncated or corrupted download, and whoever could replace the archive on the
release page could replace its hash too. So every archive of a release after
0.9.4 also carries a build provenance attestation, which GitHub signs for the
`release.yml` run that built the file. Check one by hand with
[`gh`](https://cli.github.com):

    gh release download v<version> --repo haksolot/ank --pattern 'ank-*-x86_64-unknown-linux-musl.tar.gz'
    gh attestation verify ank-<version>-x86_64-unknown-linux-musl.tar.gz \
      --repo haksolot/ank --signer-workflow haksolot/ank/.github/workflows/release.yml

When a signed-in `gh` is on `PATH`, both installers run that check before
unpacking and refuse an archive that fails it, with exit 4. Each says in one
line which check it made: `verified: sha256 and provenance`, or `verified:
sha256 only` followed by the reason. To refuse any install the check cannot be
made for, pass `--require-provenance` to `install.sh` or `-RequireProvenance` to
`install.ps1`, or set `ANK_REQUIRE_PROVENANCE` for either: it exits 5 without a
signed-in `gh`, and 4 for a release from before the attestations.

Two fallbacks remain when none of the three fits, and this project supports
neither as a route. The [releases
page](https://github.com/haksolot/ank/releases/latest) carries the archives the
installers fetch, one per target with a `.sha256` beside it; unpacking one by
hand is what those installers do for you. And the tree builds:

    cargo install --git https://github.com/haksolot/ank ank-cli

`--git` because nothing publishes to crates.io. That puts `ank` in
`~/.cargo/bin`, and needs Rust 1.95 or newer and a C compiler.

No package manager ships ank. Homebrew, Scoop, apt, winget and the AUR each
carried a channel here or an attempt at one, and each was withdrawn; adding one
back means replacing the decision that withdrew them.

Whichever you took, check it answers:

<!-- replay bare -->

    $ ank --version
    ank 0.8.0 (8310e75, skill 0d916cc3d9a5)

Three components: the version, the commit it was built from, and **the revision
of the skill it was built alongside**. The commit tells you whether your binary
predates the behaviour you are reading about. The revision is the value the installed stub, `skill/SKILL.md`, carries under
`metadata.revision`, so an agent that has loaded the skill can compare it with
the one its tool prints, and see for itself that its instructions predate its
binary. Worth checking when the skill came from a clone and the binary from a
release: those are two points in history, and nothing forces them to match.

The same executable carries every surface. A client with no shell reaches the
verbs over MCP through `ank mcp`, and the background cache warmer is `ank
watch`: there is nothing further to install for either. [The MCP server](mcp.md)
says how a client is configured.

### Where the binary you run comes from

**A globally installed `ank` tracks the published release, not the tree you
have checked out.** The two are the same file only on the day of a
release.

It matters when a corpus is written by a binary newer than yours: somebody
else's release, or your own tree if you contribute to Ank itself. Then the tool
managing the work and the tool being changed both print `ank 0.8.0`, and only
the commit separates them, which is why `--version` carries it.

Contributors hit a sharper form. On Windows a running
executable cannot be relinked, so a command that rebuilds the tree while
`target/debug/ank` runs it fails on the lock. The habit that avoids it is to
copy the built binary outside `target/` and run the copy, which drifts from the
tree as soon as the tree moves. Rebuild and re-copy it after a merge.

One half of this the tool diagnoses on its own. A corpus whose entities declare
a schema newer than the binary reads is refused entity by entity, so every verb
that lists would silently answer short; each says so first:

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

It warns and still answers: the entities this build understands are worth
having, and a corpus mid-migration is a real state.

**The second line is one of two, chosen by whether a release can help.** The
schema a published version reads is stamped into the binary at build time, from
the newest tag's own source, so the message names the road that resolves the
state. Above, no release reads the corpus: the schema landed on the default
branch after the last tag, the ordinary case for a contributor, so it sends you
to the tree. Where a published release does read it, your binary is simply old,
and the line names the install:
`-> the binary is older than the corpus: ank --version names the build, npm install -g @haksolot/ank replaces it`.

Naming the install in the first case would fetch the very build that refused,
and a reader whose advice visibly does nothing concludes the tool is broken.

The other half, an old binary reading an old corpus, is undetectable: nothing in
the files says a newer format exists. That one is `--version`, the update below,
and the paragraphs above.

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

Otherwise it prints the command it hands the install to, runs it, and exits
with that command's code. The installer it pipes is the one tagged with the
release it installs, read from `raw.githubusercontent.com/haksolot/ank/v<version>/`,
so a change on `main` reaches an update only once a release carries it. A
release at or below 0.9.4 still takes the installer on `main`, as before. `--version <v>` installs the release it names, an
older one included. The skills stay where they are: when the new release carries
another skill revision than the binary it replaced, `update` names `ank skills
--install` in one line and leaves running it to you.

Only this verb reaches the network for a release, and only when you run it. A
binary built under a cargo target directory was placed by no route, so `update`
refuses it at exit 7 and names `cargo build`. Where the releases are read from
is [`ANK_UPDATE_REPOSITORY`](environment.md#ank_update_repository).

## The skills

One file is installed, `skill/SKILL.md`, a stub of about twenty lines. It
carries the description that makes a harness load it in a repository holding
`.ank/`, and one instruction: run `ank skills ank` and read what it prints. The
binary serves the rest, so an installed copy holds nothing a release changes.

Seven skills live under `skill/` as plain markdown, each the only copy in git,
and the build embeds them all:

    ank           skill/ank.md       the contract
    ank-plan      skill/plan.md      interview a goal into ADRs, specs and tasks
    ank-drift     skill/drift.md     audit decisions against the code
    ank-loop      skill/loop.md      work the backlog autonomously
    ank-tdd       skill/tdd.md       drive an implementation test-first
    ank-diagnose  skill/diagnose.md  work a defect back to its cause
    ank-herdr     skill/herdr.md     orchestrate several agents from a herdr-core terminal

The contract is why ank is shaped as it is, the verbs grouped by the moment each
is used, and the rules that bind every agent. The five activity policies,
`ank-plan`, `ank-drift`, `ank-loop`, `ank-tdd` and `ank-diagnose`, carry one
activity each, and `ank claim` prints the one a task names after its criterion.
`ank-herdr` is for the agent that runs the others, inside a herdr-core terminal.
What an agent does with them is [Multi-agent work](multi-agent.md).

The routes for the stub:

    ank skills --install                      from the binary you installed, nothing cloned
    npx skills add haksolot/ank               a machine with node and no ank
    /plugin marketplace add haksolot/ank      Claude Code, as a plugin
    by hand                                   copy skill/SKILL.md where your harness loads it

### From the binary

`ank skills` lists the seven skills the build embeds, with the revision each
file declares. The revisions below are the ones this page was written against;
yours are whatever your build read:

<!-- replay bare -->

    $ ank skills
    ank           61e3f7298630  Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
    ank-diagnose  98cd5d5badff  Work a defect back to its cause before changing anything, and close it with a regression test. Use when a claimed task's criterion names a defect in a repository with a .ank/ directory.
    ank-drift     36cf5808e95e  Audit the decisions in .ank/ against the current code and report what no longer holds. Use when asked whether ADRs, specs, or tasks are still accurate, after a milestone, or when the corpus and the code seem to disagree.
    ank-herdr     0bce41153716  Orchestrate several ank agents in parallel, each on its own task, worktree and branch. Use only inside a herdr-core terminal, when asked to fan ank tasks out across agents in a repository with a .ank/ directory.
    ank-loop      1e28b752916a  Work through the open tasks in .ank/ without supervision, one claim at a time. Use when asked to work the backlog, chain tasks, or run autonomously in a repository with a .ank/ directory.
    ank-plan      34bbc5f04464  Interview a goal into decisions and tasks recorded in .ank/. Use when someone brings a feature, change, or problem to plan before implementation in a repository with a .ank/ directory.
    ank-tdd       5c0133123d36  Drive an implementation test-first, red before green, against a claimed task's frozen criterion. Use when implementing a task in a repository with a .ank/ directory.

One line per file: the name, the revision that file declares, and the whole
description. Run inside a corpus, the same verb prints a second block under that
listing, which [Multi-agent work](multi-agent.md#the-methods-a-corpus-uses)
reads.

`ank skills <name>` prints one of them whole, by the name in that first column or
without its `ank-` prefix: `ank skills ank` is the contract, `ank skills tdd` the
test-first method. It prints the same text in any directory. Where the
contract lists verbs it prints the lines `ank help` prints for them, so the table
has one source, and it names `ank-herdr` inside a herdr-core terminal, where
`HC_TERMINAL_ID` is set. `ank help` ends on the line naming it.

`ank skills --install` writes the stub, as `ank/SKILL.md`, into a new directory
under the temporary directory and hands that directory to the `skills` CLI
below, without asking: the flag is the consent. It prints two lines of its own,
`wrote the ank skill to <directory>` and `running: npx skills add <directory>
-y`; everything after them is the `skills` CLI's own report of where it linked
the stub. The `-y` answers that CLI's question of which agents: all of them,
from an agent's session or from your own terminal alike.

`npx` itself goes online to fetch the `skills` CLI the first time it has none
cached. That CLI installs copies into `.agents/skills` of the directory it ran
from, and they outlive the temporary directory. Without `npx` on your `PATH`,
the verb prints the directory it wrote and the command to run later, and exits
0.

### The `skills` CLI

The route for a machine that has node and no ank. It detects what you run
(Claude Code, Codex, Cursor, OpenCode and some thirty more) and links each one
to a single copy. Ask it what it found before you let it install:

    npx skills add haksolot/ank --list

It clones the repository and lists the stub, the one file under `skill/` named
`SKILL.md`. That route installs the stub and leaves the binary to you, and the
stub then asks for a binary to run.

It finds the stub through its own recursive scan, since `skill/` is outside the
directories it looks in by name, so it works because the fallback works. If a
future version of that CLI narrows its search, this route breaks first, and the
hand copy below is the answer.

### Claude Code, as a plugin

This repository serves as its own marketplace:

    /plugin marketplace add haksolot/ank
    /plugin install ank@ank

The manifest lists `./skill` and no other directory, so the plugin carries the
stub alone. Ask what it costs, the question worth asking of anything loaded on
every session:

    claude plugin details ank

It prints the plugin's component inventory and a projected token cost, split
into always-on and on-invoke. Always-on is the stub's description, paid by every
session; on-invoke is its body, paid by the session that wanted it, and the
contract it names is paid for when `ank skills ank` runs. Nothing of the plugin
runs unless you call `ank` yourself, and the counts are estimates that move with
the files and with whatever does the counting.

### By hand

Where no route above fits your harness, copy `skill/SKILL.md` into whatever it
loads. It holds nothing generated, and everything it points at comes from the
binary.

## The Claude Code hook

The binary also serves as a hook handler for Claude Code. One command writes
its entries into your user-level settings:

    ank hooks install claude

It edits `settings.json` in `CLAUDE_CONFIG_DIR`, or `~/.claude/settings.json`
when that variable is unset or empty, and adds two entries. The `SessionStart`
one hands the agent `ank status` and a pointer to the contract at the start of
every session. The `PreToolUse` one runs before the file tools: it keeps the
agent out of `.ank/`, and warns when an edit lands on a file that none of the
agent's claims covers.

Both entries call the binary that installed them by its absolute path, so run
the command again after you move or replace that binary. A second install
leaves the file as it was, every byte it did not add stays where it was, and
this takes out exactly what install wrote:

    ank hooks remove claude

Outside a corpus the hook stays silent and lets every tool call through. Its
two guards, the levels they take and how a launch environment tightens them are
in [Integrating](integrating.md#an-agent-harness-calls-ank-hooks).

Next: [the quickstart](quickstart.md), from `ank init` to a first finished task.
