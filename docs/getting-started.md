# Getting started

By the end of this page your repository holds one binding constraint, one
finished task, and a proof that Ank wrote after running the verification
itself. It takes about ten minutes, and nothing here asks you to read the
specification.

Every command and every output below was run against a fresh repository. Where
the tool refuses, the refusal is shown as it appears.

## What you need

- **git 2.34 or newer.** Not a convenience: claims live in git refs, and Ank
  checks the version at startup.
- **`sh`.** Verifiers run under `sh -c` on all three operating systems. On
  Windows it comes with Git for Windows, so requiring git makes it free.
- **Rust 1.95 or newer**, only if you build from source.

## Install

Put `ank` on your `PATH`. [agents.md](agents.md) carries every route with its
trade-offs (a release binary and its checksum, npm, or building from source)
and the shortest of them is one line of npm. Whichever you took, check it
answers:

<!-- replay bare -->

    $ ank --version
    ank 0.8.0 (8310e75, skill 0d916cc3d9a5)

It prints the version, the commit it was built from, and the revision of the
skill it was built alongside. The commit matters the first time you suspect the
binary in your hand is older than the behaviour you are reading about. The
revision answers the same question about the other half: it is the value
`skill/SKILL.md` carries under `metadata.revision`, so an agent that has loaded
that file can compare two strings it already holds and see that its
instructions predate its tool.

### The protocol surface is a verb of the same binary

A client that has no shell to run verbs in reaches the same verbs over MCP, and
there is nothing further to install for it: the surface is **`ank mcp`**, a verb
of the executable you just put on your `PATH` (ADR-1ea31c2f3c5a). Every route
carries one file, so a route cannot deliver the CLI and fail to deliver the
surface.

It speaks JSON-RPC over stdio and the client spawns it, which means it is
configured rather than started. Three configurations, and each of them is
pasted rather than derived.

**Claude Code**, one line, which writes the entry for you:

    claude mcp add ank -- ank mcp --repo /path/to/your/repo

or `.mcp.json` at the root of the repository, which is the form that travels
with the tree and reaches everyone who clones it:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**Claude Desktop**, in `claude_desktop_config.json` (`~/Library/Application
Support/Claude/` on macOS, `%APPDATA%\Claude\` on Windows), which has no project
directory of its own and so has nothing else to go on:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**Cursor**, in `.cursor/mcp.json` beside the repository, or `~/.cursor/mcp.json`
for every project at once:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**`command` is `ank` and `mcp` is the first argument, in all three.** If you
have a configuration written against `ank-mcp`, that is the one line to change:
releases up to 0.6.0 placed a second executable by that name and no route places
one any more, so a client still naming it gets `command not found` rather than a
wrong answer.

**`--repo` is written out in all three, and that is the point of showing them.**
A client spawns the server in whatever directory it happens to be in, and with
no `--repo` the server takes that directory. The failure mode is not an error:
it is a process quietly speaking for a corpus nobody meant, or for none. The
configuration is also where it is named rather than something a call may
override, and a call that tries to pass `--repo` itself is refused by name:

<!-- replay mcp
$ ank init
-->

    --> {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ank_status","arguments":{"repo":"/tmp"}}}
    <-- {"jsonrpc":"2.0","id":3,"error":{"code":-32602,"message":"--repo belongs to the server: name a corpus with the corpus argument, by the identity ank status --json prints, never by a path"}}

A path with no corpus under it is refused before any client is listening, rather
than after, so it reaches a person rather than a log:

<!-- replay mcp dir=/tmp -->

    $ ank mcp --repo /tmp
    error[1]: no .ank/ found from /tmp
      -> ank init

Every verb the CLI dispatches is a tool on that surface, under the name
`ank_<verb>`, and what a call gets back is the document `--json` returns with
the exit code beside it. [integrating.md](integrating.md) says what the surface
is, for somebody building against it. The rest of this page uses the shell,
which is what an agent that has one should use.

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
exit 7 and names `cargo build` instead.

## Initialise a repository

The walk from here to the end of the page was run twice against a fresh
repository, once with a remote named `origin` and once with none, and the two
runs differ in exactly one line, named where it appears.

**Three files are in the tree before anything starts**, because every scope and
every verifier below is pointed at one of them. These are their exact contents,
one line each, and they are written out because two of the hashes further down
are hashes of them:

    src/auth/session.ts   export function createSession(id) { return store.put(id) }
    tests/auth.sh         exit 0
    README.md             # auth service

What matters is that all three exist. A scope matching no file and a verifier
whose command is not there are the two ways this walk ends in red, and neither
says anything until `ank done`, twenty commands later.

From the root of that repository:

<!-- replay walk ANK_AGENT=human:marie
$ git init -q --bare ../origin.git && git remote add origin ../origin.git
$ mkdir -p src/auth tests && echo 'export function createSession(id) { return store.put(id) }' > src/auth/session.ts
$ echo 'exit 0' > tests/auth.sh && echo '# auth service' > README.md
$ git add -A && git commit -q -m "the service"
-->

    $ ank init
    created .ank/entities
    wrote .ank/config.yml
    wrote .gitattributes
    wrote .gitignore
    pointer added to AGENTS.md
    refspec added: +refs/ank/*:refs/ank/*

**The last line is the one a repository with no `origin` does not print**:
there is no remote to add a refspec to, so `init` reports five effects instead
of six and the other five are identical. Re-running changes nothing either way:
`init` is idempotent and says `already initialised, nothing to do`.

One directory is created, not one per kind: entities live flat in
`.ank/entities` whatever they are (ADR-c9f9d0d6f05d), and a task, an ADR, a
spec and a log entry are told apart by a field rather than by a folder. The
`.gitattributes` line keeps `.ank/` in LF on checkout: on Windows git would
otherwise convert back to CRLF everything the tool has just written, on every
clone. The `.gitignore` line is `.ank/index.db`, the derived SQLite index: it
is rebuilt from the files whenever it is missing, so committing it would only
track a binary that every command rewrites. The refspec is what makes claims
travel, since hosts do not fetch non-standard refs on their own.

Both git files are appended to, never replaced, so an existing `.gitignore`
keeps everything already in it.

One edit to make now, before the first real command.

**Name your default branch.** It is the one key `init` leaves unset on purpose:
it runs where the reference branch is not known yet, and writing `main` there
would be exactly the guess the tool refuses everywhere else. `.ank/config.yml`
is written through the CLI rather than by hand:

<!-- replay walk -->

    $ ank config default_branch main
    default_branch (unset) -> main

Without it, Ank looks for `refs/remotes/origin/HEAD`, and a repository with no
remote has none. It refuses rather than guessing, and here is what the verb two
sections down would have said (`06d2` is the ADR you write there):

<!-- replay unset ANK_AGENT=human:marie
$ ank init
$ ank new adr --title "Opaque sessions rather than stateless JWT" --scope "src/auth/**" --constraint "Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store."
created ADR-06d29e727d24 Opaque sessions rather than stateless JWT
-->

    $ ank accept 06d2
    error[9]: default branch indeterminable (default_branch absent from .ank/config.yml, refs/remotes/origin/HEAD absent)
      -> git remote set-head origin -a
      -> or ank config default_branch <name>

That is the refusal the run without an `origin` gets, and it is why the key is
set here rather than later. A clone sets `refs/remotes/origin/HEAD` for you, so
the run with an `origin` resolves the branch without the key and the command
above is a no-op it never needed; setting it anyway costs one line and removes
the difference between the two.

**Coordination between clones needs a remote named `origin`, and nothing
else.** Whether claims travel is not configured: a repository with an `origin`
pushes every claim to it as a compare-and-swap, and one without keeps them as
local refs. GitHub is not required. A bare repository reachable over `file://`
or `ssh` on the same network is a remote named `origin` like any other: `claim`
reads it first, so the second clone to take a task is refused with code 4 and
the holder named, and the push settles a race the read misses. Without one,
`git worktree`s of a single clone are still arbitrated, because they share
`refs/ank/`. **Two clones with no common origin are not arbitrated: both claims
of one task succeed, both agents work, and nothing reports it**, not `status`,
not `check`, not later.

## The two kinds this page uses

Flat in `.ank/`, markdown with YAML frontmatter. Four kinds exist and this page
needs two of them; the other two are a **spec**, normative text that describes
rather than binds, and a **log entry**, written once and never transitioned.

An **ADR** is a decision that constrains code. Its `constraint` is the one
field injected into an agent's context, so it is short and imperative; the body
holds the reasoning and costs nothing at injection time.

A **task** is a unit of work. Its `done_criteria` says what would prove it
finished.

Both carry a `scope`: a list of globs, and the only thing that joins the two
kinds. There is no epic, no parent, no label. "Everything about the auth
migration" is answered by `ank context src/auth/`. The full field list is in
[format.md](format.md); you do not need it to work through this page.

## Write the first constraint

<!-- replay walk -->

    $ ank new adr --title "Opaque sessions rather than stateless JWT" \
        --scope "src/auth/**" \
        --constraint "Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store."
    created ADR-06d29e727d24 Opaque sessions rather than stateless JWT

It is created `proposed`, which means visible but not binding. Promotion goes
through one command, and that command is the only one in the tool that makes a
git commit:

<!-- replay walk -->

    $ git add -A && git commit -m "adr: opaque sessions"
    $ ank accept 06d2
    accepted ADR-06d29e727d24 -> 9c45c50

Short prefixes work everywhere an id is accepted; an ambiguous one is an error
listing the candidates, never a guess. The commit it produced carries the hash
of `constraint` and `scope` at acceptance:

<!-- replay walk
$ git log -1 --format=%B
-->

    ratify ADR-06d29e727d24

    constraint+scope: c5d4f3478ad5
    by: human:marie

That hash is the anchor. Editing the constraint afterwards does not change it,
which is how `ank check` notices. Sign your commits and list the key in
`.ank/allowed_signers` if you want the anchor to be verifiable by others; with
no signing configured, `check` says so rather than pretending.

`accept` runs on the default branch only, and there is no flag around it. A
constraint ratified on a feature branch would bind on that branch alone.

## Declare a verifier, then the task that uses it

A task names verifiers; it never carries a shell command. The definitions live
in `.ank/config.yml`, under the repository's own review, and they go in through
the same verb: writing a `run` for a name the file does not carry is how a
verifier is declared:

<!-- replay walk -->

    $ ank config verifiers.auth-tests.run "sh tests/auth.sh"
    verifiers.auth-tests.run (unset) -> sh tests/auth.sh
    $ ank config verifiers.no-jwt.run "! grep -rq jwt.verify src/auth/"
    verifiers.no-jwt.run (unset) -> ! grep -rq jwt.verify src/auth/

which is what the file then carries:

<!-- replay walk part
$ cat .ank/config.yml
-->

    verifiers:
      auth-tests:
        run: sh tests/auth.sh
      no-jwt:
        run: "! grep -rq jwt.verify src/auth/"

`ank config --unset verifiers.no-jwt` takes one back out. Reading a key says
where the value comes from, this repository or a default the tool resolved:

<!-- replay walk -->

    $ ank config verifiers.auth-tests.run
    sh tests/auth.sh
    $ ank config verifiers.auth-tests.timeout
    10m (default)

The distinction is the reason writing is line surgery rather than a round-trip
through a YAML serializer. Your comments, blank lines, key order and quoting
survive a write, and a key you never set is never written out: an unset key
follows the tool, a written one is pinned here, and a serializer would quietly
convert every one of the first kind into the second.

Declare them first. A task that names a verifier `config.yml` does not know is
refused at creation:

<!-- replay undeclared
$ ank init
-->

    $ ank new task --title "Migrate auth" --scope "src/auth/**" --verify auth-tests
    error[7]: no verifier 'auth-tests' in .ank/config.yml
      -> ank config verifiers.auth-tests.run "<command>"

With the definitions in place:

<!-- replay walk -->

    $ ank new task --title "Migrate auth to opaque sessions" \
        --scope "src/auth/**" \
        --criteria "The auth tests pass and no reference to jwt.verify remains in src/auth/" \
        --verify auth-tests --verify no-jwt
    created TASK-820d259af6a7 Migrate auth to opaque sessions

A composite criterion is mechanised by several verifiers, not by one that
covers half of it. All of them must pass.

### The default list, and declining it

`--verify` names a task's verifiers one at a time, which is the right amount of
ceremony for a verifier that suits one perimeter and the wrong amount for the
suite every task in the repository has to pass. A verifier marked `default`
joins every task written afterwards:

<!-- replay walk -->

    $ ank config verifiers.no-jwt.default true
    verifiers.no-jwt.default false (default) -> true

which lands beside the `run` it belongs to, and nowhere else in the file:

<!-- replay walk part
$ cat .ank/config.yml
-->

    verifiers:
      auth-tests:
        run: sh tests/auth.sh
      no-jwt:
        run: "! grep -rq jwt.verify src/auth/"
        default: true

The read form is the same as any other key, and an unmarked verifier answers
`false (default)` -- the tool's default for the `default` key, which is the one
place the word does double duty:

<!-- replay walk -->

    $ ank config verifiers.no-jwt.default
    true
    $ ank config verifiers.auth-tests.default
    false (default)

A task created now with no `--verify` of its own carries `verify: [no-jwt]` in
its frontmatter, without anybody naming it. `--verify` replaces that list
rather than adding to it, so a task that names its own verifiers gets exactly
those.

**`--no-verify` is the third possibility, and it is a judgement, not a
shortcut.** It writes a task with no `verify:` at all:

<!-- replay walk -->

    $ ank new task --title "Say in the README what a session is now" \
        --scope "README.md" \
        --criteria "The README describes opaque sessions and names no JWT" \
        --no-verify
    created TASK-51c2a0f6d418 Say in the README what a session is now

That is the task whose `done` refuses without `--proof`, further down. Use it
where no declared verifier can settle the criterion -- prose a person has to
read, a behaviour only a published release answers -- and where that is
genuinely the case, say so when you write the task, so the empty list is
visible in its diff. A task that reaches `done` with an empty `verify:` nobody
decided on closes on a proof nothing ran.

## Orientation

`ank context` is the first call, and the only one you have to remember. With no
argument it covers the whole repository; with a path it covers that perimeter.

<!-- replay walk -->

    $ ank context src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Opaque sessions rather than stateless JWT

    TASKS (1)
      TASK-820d  [open] Migrate auth to opaque sessions

    > ank claim TASK-820d to start

The other task is not here, and that is the two planes doing their work: its
scope is `README.md`, which `src/auth/` does not cover. Nothing labelled it out
of this perimeter; the glob did.

**Before a claim a constraint is one line, and that line is its title.** What
is being answered here is which perimeter to enter, and a survey that spent its
whole budget on constraint text would answer it worse. The constraint itself
arrives with the claim, below, where the perimeter is settled and the rule is
what you are about to be held to; `ank show 06d2` prints it whole at any time.
Constraints come first in both forms, and once you are working they are never
truncated. The output ends with the next command, as every output here does.

## Claim

<!-- replay walk -->

    $ ank claim 820d
    claimed TASK-820d259af6a7 migrate-auth-to-opaque-sessions -> HEAD

Three things happened. The task moved to `in_progress`. A claim ref appeared at
`refs/ank/claims/TASK-820d259af6a7`, which is what arbitrates two people or two
agents reaching for the same task: git's compare-and-swap, one winner. And the
`done_criteria` was frozen: its hash went into the claim record, where the
file's editor cannot reach it.

The freeze is the rule worth internalising. Editing the criterion to make the
work fit does not unblock anything; `done` compares against the recorded hash
and refuses, and `check` reports the divergence. If the criterion is genuinely
wrong, hand the task back with `ank release --reason "<why>"` and say so.

`claim` also sets HEAD, so the following commands need no id. One claim at a
time, per person and per agent.

### One identity per session

Claiming a second task while you already hold one is refused:

<!-- replay walk -->

    $ ank claim 51c2
    error[7]: human:marie holds a live claim on TASK-820d259af6a7 (expires in 30m)
      -> ank release --reason "<why>"   (a second session on this machine sets its own ANK_AGENT)

If you meant it, the first way out is the one to take: finish the task you hold
or hand it back. If the refusal surprises you, it is almost certainly two
terminals: `$ANK_AGENT` unset resolves to `<user>@<hostname>`, so two sessions
on one machine are the same agent as far as the refs can tell, so they would see
each other's claims and renew them. Give every concurrent session an identity
of its own:

<!-- replay walk -->

    $ ANK_AGENT=human:marie-2 ank claim 51c2
    claimed TASK-51c2a0f6d418 say-in-the-readme-what-a-session-is-now -> HEAD

**Write it typed.** An identity that goes into an entity says what kind of
actor it is (ADR-3877fef1d662): `human:<id>` is a person, `<producer>/<version>`
an agent -- `claude-code/opus-5`, and a suffix after `+` for one session of it
-- and `process:<id>` something automated, which is the form the pipeline at the
end of this page uses. The convention is what lets `check` say that an entity
was written by an agent and read by no human; the fallback
`<user>@<hostname>` carries no type and so answers that question with nothing.
It is a signal and not a wall -- anyone can type `human:` in front of a model
-- and what it buys is that the ordinary case is legible.

That is why the refusal names the identity rather than calling you its holder:
under a shared identity, the session being refused may have claimed nothing at
all. Identity is declared, never proved, and it is deliberately not bound to the
session: a PID or a TTY in it would mean losing your claim to a restarted
terminal. Parallel agents, each with its own `ANK_AGENT`, are the supported
case; one ref per task is what arbitrates them.

An expired claim is not a live one, so this never stands between you and a task
whose lease ran out, yours or anybody's.

How parallel sessions assemble into one change, with a branch per task, `done` as
a local proof and integration as a task of its own, is in [agents.md](agents.md),
under "Parallel work and integration".

Run `ank context` again and the output inverts: no other task, the full
criterion, and the constraints matching this task's scope.

<!-- replay walk -->

    $ ank context

    TASK-820d  Migrate auth to opaque sessions

    DONE_CRITERIA
      The auth tests pass and no reference to jwt.verify remains in src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store.

`ank show 820d` gives you the entity whole, frontmatter, body and log, which
is where the reasoning behind a task lives.

## Work, and log what you learn

<!-- replay walk -->

    $ ank log "jwt.verify removed from session.ts"
    logged LOG-6b0f39d7a4c1 on TASK-820d259af6a7

The log is a work trace, not proof. Write to it when you discover something,
not when you finish: it renews the claim, so working is what keeps the lock and
there is no heartbeat command to remember. A claim lasts 30 minutes by default;
if it expires because a build ran long, the task stays `in_progress` and you
re-acquire silently, provided nobody took it over.

**Each entry is an entity of its own**, which is why the line names one. The
task's own file is not touched at all, no frontmatter and no version bump, and two
agents writing at once write two files, so there is nothing for a merge to
resolve. Any kind may be logged against, and a subject with no work left to
arbitrate asks for no claim: an ADR, a spec, and a task already `done` or
`closed`. A claim is what arbitrates work, so holding it is required exactly
where there is work to arbitrate, on a task that is `open` or `in_progress`.
That is what lets a correction reach a task after it is settled, which is the
one place a wrong entry used to have nowhere to go. Name the task when the entry
goes on a finished one: HEAD never points at one.

`ank log 820d` with no message reads the entries back, newest first, and needs
no claim. A message too long for a line is printed elided there; `ank show`
on the entry's own id prints it whole, and `--json` always carries it whole.

## Finish

<!-- replay walk -->

    $ ank done
    running: auth-tests ... ok (0.0s)
    running: no-jwt ... ok (0.0s)
    proof recorded: auth-tests@94a1f671c577 -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    proof recorded: no-jwt@791cc818d0ad -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    TASK-820d259af6a7 -> done

This is the point of the tool. Ank ran the verifiers itself and wrote what
actually ran: one proof entry each, carrying the hash of the verifier
definition that executed, the HEAD commit, and a hash of the scope files'
content at that moment. Nobody reported their own result. Never set
`status: done` by hand: a status written by the party being measured measures
nothing.

Four of those hashes are content and not identity, so they are the same on your
machine as on this page. `94a1f671c577` and `791cc818d0ad` are the two verifier
definitions; `e3b0c44298fc` is what each of them printed, which is nothing; and
`18d14da584ab` is `src/auth/session.ts` as the section above wrote it. The
commit is the one your tree is on, so that one is yours.

The other task takes the other branch. It was written `--no-verify`, so there
is no verifier to produce anything and `--proof` becomes mandatory -- the
session holding it is the second one, from the identity section above:

<!-- replay walk -->

    $ ANK_AGENT=human:marie-2 ank done
    error[5]: proof required to move TASK-51c2a0f6d418 to done
      -> ank done --proof commit:<sha>

The proof types are `commit`, `test`, `human-review` and `assertion`, and what
separates them is not local versus hosted but **who controls the environment**.
A CI reference is out of the agent's reach and guarantees the most;
`commit:<sha>` is checked with git; a local test proves what ran in a tree the
agent could have altered; `assertion:"..."` guarantees nothing and is marked
weak, which is what keeps it from quietly becoming the default path.

**The type is half the answer, and the entry records the other half.** Every
proof carries `via`: `verifier` when Ank ran the verifier itself, `attested`
when it arrived on `refs/ank/proof/<id>`, `submitted` when a caller typed it,
because a run reference is the strongest thing in that list when a pipeline
wrote it and the weakest when somebody typed it. Typing `--proof
test:<run-id>` is still accepted and still recorded; what it does not do is
clear the `done with no test proof` signal, which stays until a pipeline
anchors the run below. Entries written before the field carry no `via` and are
read exactly as they were.

If a verifier fails or times out, the transition is refused and the task stays
where it is.

## Commit, and what `check` says until you do

**Ank never commits, except `accept`.** Everything else writes files and leaves
them in your working tree, so the corpus travels through your normal review
like any other change.

Before you commit, `check` has something to say:

<!-- replay walk -->

    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: TASK-820d259af6a7: finished on another branch, main has not caught up
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 6 entity file(s) differ from main: this checkout does not carry the corpus the default branch does (git merge main)
    check: ok — 2 tasks, 1 adr, 4 signal(s)

Four signals and no fault, so exit 0. The first is this walk being one person:
you wrote the ADR and you ratified it, and `check` says so rather than deciding
what it means. The last counts what your working tree has that `main` has not,
and it is the same fact as the second, said about the corpus rather than about
one task.

The second signal is the mechanism doing its job. `status: done` lives in the
file, therefore on your branch alone until the merge, and during all that time
the task would look free to everyone else. The claim ref is not deleted at
`done`: it becomes a completion ref, and anyone who tries to claim the task is
refused with the commit and the branch named. Commit, and the ref is pruned:

<!-- replay walk -->

    $ git add -A && git commit -m "the migration is done"
    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 3 hot entities are cold and belong in .ank/archive/entities/, with every entry about them (ank archive)
    └── ank archive --dry-run lists them and moves nothing
    pruned refs/ank/claims/TASK-820d259af6a7
    check: ok — 2 tasks, 1 adr, 3 signal(s)

**That is the green this page set out to reach**, with an `origin` and without:
three signals, no fault, exit 0. The new one is the corpus noticing that three
log entries are cold -- an entry is cold with its subject, and their subject is
a task that is done (ADR-467ce7e9cda1):

<!-- replay walk unordered -->

    $ ank archive --dry-run
    LOG-3b92d9719950  created (version 0 to 1, produced 4cc65f12691c)
    LOG-6b0f39d7a4c1  jwt.verify removed from session.ts
    LOG-f234f9b08f06  done, proof test:local/e3b0c44298fc@9c45c50 test:local/e3b0c44298fc@9c45c50
    3 cold, nothing moved (--dry-run): ank archive moves them

It lists and moves nothing, which on a corpus this size is the right answer.

`check` is what you put in CI. It validates parsing, byte-for-byte round-trip,
`blocked_by` references, frozen fields against their anchors, and it prunes the
coordination plane. **Exit 8 means findings; signals exit 0**, because a signal is
something a reader should see, not a failure.

## When a command refuses

The exit code carries the meaning so a script can route without parsing
anything, and the message always ends with the exact command to run next.

Ten codes, and [the exit-code reference](exit-codes.md) lists all of them. That
page is generated from `crates/ank-contract/src/exit.rs`, where they are declared
once and read by every call site, so it cannot fall behind the binary.

Two of them are worth a sentence each. **Code 9** says the environment is
broken, not that your work is wrong: fix the machine, not the code. **Code 1**
is the one with no reaction of its own to prescribe -- it is what a
mistyped command and an unparseable file both get, and a script that routes on
it is guessing. The `ank mcp --repo /tmp` refusal above is a 1.

## Running ank in a pipeline

Read this part before the recipes, because the recipes are the easy half. The
whole integration surface is two things: **an exit code, and `--json`**. Learn
those and you can write the pipeline for a CI system nobody here has heard of.

`ank check` is the verb a pipeline runs. It exits:

- **0**, the corpus is healthy. Signals exit 0 too: they are observations, not
  faults, and reddening a build over one teaches a team to stop reading `check`.
- **8**, findings. This is the failure a pipeline exists to catch.
- **9**, the environment and not the corpus: git too old, `sh` missing, the default
  branch indeterminable. Nothing is wrong with the files, and a pipeline that
  collapses 9 into "the check failed" sends somebody to fix sound work.

`--json` is opt-in on every verb and is what you parse. It carries no colour and
no layout, and it stays byte-for-byte what your parser already reads.

That is the contract. Everything below is the CI system's own syntax around it.

**Check out the whole history, and on GitHub that is one line.** `ank check`
walks it: it reads the ratification commit that anchors a frozen constraint,
and it asks where a scope that matches nothing went, a rename or a deletion.
`actions/checkout` fetches one commit by default, and a corpus read from one
commit does not report itself unreadable -- it reports itself unverified, at
exit 0. Measured on this repository at 8310e75: a `--depth 1` clone answers
`check: ok — 440 tasks, 61 adr, 728 signal(s)` where the full clone of the
same commit answers 663, and 71 of those extra signals read
`ratified, but no ratification commit is reachable: the freeze cannot be verified`,
each noting that `the history here is shallow, so where it went cannot be
verified (git fetch --unshallow)`. Green, and every frozen constraint
unchecked, which is a worse failure than the red it replaces:

    - uses: actions/checkout@v5
      with:
        fetch-depth: 0

Every host spells the depth its own way, and the thing to carry across is the
property rather than the key: `check` needs the history that reaches the
ratification commits, and a shallow checkout is silent about what it could not
read.

**A bare shell**, which is the recipe the other two wrap:

    #!/bin/sh
    code=0
    ank check || code=$?
    case $code in
      0) ;;
      8) echo "ank check: findings, see above" >&2; exit 1 ;;
      9) echo "ank check: environment unavailable, not a corpus failure" >&2; exit 2 ;;
      *) echo "ank check: unexpected exit $code" >&2; exit 1 ;;
    esac

**GitHub Actions:**

    - name: ank check
      run: |
        code=0
        ank check || code=$?
        case $code in
          0) ;;
          8) exit 1 ;;
          9) echo "::notice::ank could not run: environment"; exit 2 ;;
          *) exit 1 ;;
        esac

**GitLab CI:**

    ank:check:
      script:
        - |
          code=0
          ank check || code=$?
          case $code in
            0) ;;
            8) exit 1 ;;
            9) echo "ank could not run: environment"; exit 2 ;;
            *) exit 1 ;;
          esac

Three vendors, one contract, and the third one is a shell script in a YAML file
like the other two.

`ank check || code=$?` and never a bare `ank check` followed by `case $?`: a
GitHub Actions `run:` block is `bash -e`, so the bare form aborts the step on
exit 8 and the routing you wrote is never reached. The `|| code=$?` form is what
survives `set -e`, which is why it is in all three rather than in the one that
needs it.

**There is no `--format github`, and there never will be.** Annotations,
folding markers and job summaries are one vendor's protocol, and putting them in
the binary would couple the tool to a company. A pipeline that wants annotations
pipes `--json` into whatever produces them, which is exactly the arrangement
that lets the fourth vendor work without anybody shipping a release for it.

### Anchoring a run

`done` records what ran on the machine that ran it. That is a local claim, and a
pipeline can anchor the same task to a run anybody can re-read:

    ank attest <id> --proof test:<run-id> --detached

`--detached` writes the proof to `refs/ank/proof/<id>` and touches no file, so
the pipeline produces **no commit**: it needs no write access to the branch and
cannot race the merge. It is still a write to the remote, though, and on GitHub
that is a permission: this repository's `ci.yml` declares `contents: read` for
the workflow and the `attest` job is the only one that raises it, to the one
thing it writes.

    permissions:
      contents: write

**Which ids.** `check` is the work list, and nothing diffs `.ank/` to build it.
A finished task with no external anchor carries the signal
`done with no test proof: nothing external anchors it`, so the ids are the
subjects of that finding and `--json` is what a pipeline reads them out of:

    ids=$(ank check --json | jq -r '
      .findings[]
      | select(.message | startswith("done with no test proof"))
      | .subject')

**Fetch the proofs already written first**, before that `check` runs.
`actions/checkout` fetches history and not `refs/ank/*`, so a job that skips
this reads a corpus in which nothing is anchored and re-attests every finished
task, on every push, forever. Measured on run 33285805350: 199 ids listed and
336 seconds spent pushing refs that were already there, against 16 genuinely
unproved in a clone that carries them.

    git fetch origin '+refs/ank/proof/*:refs/ank/proof/*'

One direction and read-only. A refspec matching nothing is not an error, so a
corpus with no proofs yet passes through untouched.

**Run it on the default branch and on a push, and nowhere else.** The signal is
gated on the task appearing done on the default branch, so on a feature branch
straight after `done` there is nothing to anchor and this job would build for
two minutes to be told so. A `pull_request` event runs on a merge commit no
branch carries, and a fork's token cannot push a ref at all, which would fail
the job for a reason about the event rather than about the corpus:

    if: >-
      github.event_name == 'push' &&
      github.ref == format('refs/heads/{0}', github.event.repository.default_branch)

The identity doing the attesting is typed like any other, and a pipeline is
`process:`:

    env:
      ANK_AGENT: process:github-actions

The proof is a ref, so it has to reach the remote to be worth anything, and
because the ref is the whole of what this verb produces, a push that did not
land is a failure and not a warning: `attest --detached` exits **9** and names
the push to run. Nothing special is needed to notice it, which is the point:

    ank attest "$id" --proof "test:$RUN_ID" --detached

`--json` still reports `"pushed"`, so an integration that prefers to read the
flag reads the same fact. What it must not do is read the flag *instead* of the
code, because the two now say the same thing. This repository's own `ci.yml`
reads the code.

Attest on every run without worrying about the ref: it grows with facts and
never with runs. A fact is a proof type, the criteria hash it was attested
against and the identity attesting it, and a second run of the same fact
replaces the first, so `show` lists the latest run. A corpus whose proof refs
were written before that rule carries one entry per run, and `check` names each
such ref with the command that rewrites it, one ref by name:

    ank attest <id> --compact --detached

It keeps one entry per fact, adds nothing and pushes that one ref. Never push
`refs/ank/*` with a wildcard to do the same: from a worktree, that force-reverts
the attestations a pipeline wrote.

## Handing the loop to an agent

Five routes install the same files (the binary itself, the `skills` CLI, the
Claude Code plugin, `pi`, and copying one markdown file by hand) and they are
walked with their real output in [agents.md](agents.md), along with `$ANK_AGENT`
and what changes when more than one agent works the same repository.

The shortest of them is the binary you just installed, which carries the skills
its build read:

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
printed.

Run inside a corpus, the same verb prints that catalogue and then a second
block, reporting how each sibling is used. A task can name the sibling its work
calls for with `ank new task --method tdd`, and a sibling that opens under a
claim writes an entry saying so with `ank log --method tdd`, titled with its
name and kept apart from the work trace. On a corpus of three tasks -- one
designating `tdd` whose holder loaded it, one designating `diagnose` whose
holder never did, and one designating nothing where `tdd` was loaded anyway --
the block reads:

<!-- replay methods part
$ ank init
$ id=$(ank new task --title "Designates tdd" --scope "**" --criteria "c" --no-verify --method tdd | cut -d' ' -f2) && ANK_AGENT=a/1 ank claim $id && ANK_AGENT=a/1 ank log --method tdd
$ id=$(ank new task --title "Designates diagnose" --scope "**" --criteria "c" --no-verify --method diagnose | cut -d' ' -f2) && ANK_AGENT=b/1 ank claim $id
$ id=$(ank new task --title "Designates nothing" --scope "**" --criteria "c" --no-verify | cut -d' ' -f2) && ANK_AGENT=c/1 ank claim $id && ANK_AGENT=c/1 ank log --method tdd
$ ank skills
-->

    METHODS
    diagnose  designated 1  fired 0  undesignated 0
    drift     designated 0  fired 0  undesignated 0
    loop      designated 0  fired 0  undesignated 0
    plan      designated 0  fired 0  undesignated 0
    tdd       designated 1  fired 1  undesignated 1

`designated` counts the tasks naming the sibling, `fired` those of them carrying
its entry, and `undesignated` its entries on tasks naming none. `ank skills
--json` carries the same counts as integers. The numbers are read from the
corpus, printed to whoever asks, and sent nowhere; outside a corpus the verb
prints the catalogue alone.

`ank skills --install` writes those six files to a temporary directory and hands
it to `npx skills add`, which detects what you run and installs them for it,
without asking and without cloning anything:

    ank skills --install

It prints two lines of its own, the directory it wrote and the `npx` command it
runs; everything after them is the `skills` CLI's, and
[agents.md](agents.md) shows a whole run.

On a machine that has node and no ank, the same skills come from the repository
instead:

    npx skills add haksolot/ank

That one installs the skill, not the binary. The skill teaches one page,
and it is loaded on every session, which is why its content is deliberately
small.

## Adopting ank where there is already code

Everything above happened in an empty directory, which is the one repository
nobody has. `ank init` on two years of history leaves you with a corpus and no
content, and the question it raises is not how to write an ADR: it is which
decisions this code has already made, and which of them were worth writing down
all along. That reading is an agent's work, and these are the three prompts that
ask for it. Both installers offer to print the same three, character for
character, and a test holds the three copies together.

Run this once, at the root of the repository:

    ank init

Then paste each prompt into your agent, one at a time, reading what comes back
before you send the next. They follow the three moments of an adoption: what the
code already decided, what it still owes, and whether the answer holds.

**What the code already decided.** This is the one you judge the tool on. A
constraint you recognise on sight is a constraint that was true and unwritten,
and the reason it now has an id is that the next agent reads it without being
told.

<!-- adopt-prompts:begin -->

    Read this repository and write, as ank ADRs, the decisions its code
    has already made: the ones a newcomer would break without knowing
    they existed. One ADR per decision, each with a scope glob covering
    the files it binds and a constraint stated as a rule. Leave them
    proposed; I ratify them myself.

They land `proposed`, which binds nobody. `ank review` lists what is waiting, and
`ank accept <id>` stays yours: a signed act, on the default branch, one at a
time. An agent proposes and says it is waiting.

**What it still owes.** Intentions scattered across TODOs, an issue tracker and a
README are not work an agent can take. A scope and a criterion are, which is what
`ank claim` freezes and `ank done` measures.

    Read the TODOs, the open issues and the README of this repository,
    and turn what they promise into ank tasks. Give each one a scope
    glob and a done_criteria a test could settle, and use blocked_by
    only where a task genuinely waits on another.

**Whether the answer holds.** The third is the uncomfortable one, and it is meant
to be: a constraint the code already breaks is the ordinary result of writing
down what was implicit, not a sign the first prompt went wrong. What you want out
of it is the list, before anybody starts repairing.

    Run ank check and ank review here, then read every ADR back against
    the code its scope matches. Tell me which constraints the code
    already breaks and which scopes match no file, and change nothing
    until I have read your answer.

<!-- adopt-prompts:end -->

None of the three is a step the tool waits on. A corpus with nothing but the ADRs
from the first prompt is already worth `ank context`, and the other two can wait
until you want them.

## Why it works this way

You have now done the loop once, which is the right moment for the four claims
underneath it. Each one is a property of the tool rather than a convention you
are asked to keep.

**Scope, not hierarchy.** Constraints and work are two planes joined only by
globs. A rule written last year binds work created today, and a glob is
verifiable against the filesystem where a label is not. There is no epic, no
parent and no rollup to keep in step.

**Nobody declares themselves done.** An agent that reports its own result can
simply be wrong, so `ank done` runs the verifiers itself and records what
actually ran, hashed. That is why the previous section refused a `done` with no
proof rather than trusting the caller.

**Freezing is verifiable, not defended.** The CLI cannot stop you editing a file
and does not pretend to. Frozen fields are anchored by a hash the editor does not
control, and `ank check` compares. Editing a criterion to unblock yourself
unblocks nothing; it makes the divergence visible.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Undo, history and
recovery are git's, and there is no server to run.

One call is bounded at 8000 characters by default, roughly 2000 tokens, which is
the constraint every one of those choices is paid for by: what `context` serves
has to fit in a context window beside the code.

## Where to go next

- [agents.md](agents.md): the five routes that reach an agent, the three that
  install the binary, and what running several agents actually requires.
- [format.md](format.md): the file format and canonical form, for anyone
  writing a tool that reads or writes `.ank/`.
- [alternatives.md](alternatives.md): how this compares to retrieval, an
  LLM-maintained wiki, and OKF.
- The specification, the source of truth for everything above: the `spec`
  documents in `.ank/`, read with `ank find --type spec` and `ank show <id>`.
  They argue the design; they are not a tutorial.
- `ank help` lists every verb, `ank help <verb>` answers about one.
