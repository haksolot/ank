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

    $ ank --version
    ank 0.1.3 (f573bc3, skill 3f350ad26459)

It prints the version, the commit it was built from, and the revision of the
skill it was built alongside. The commit matters the first time you suspect the
binary in your hand is older than the behaviour you are reading about. The
revision answers the same question about the other half: it is the value
`skill/SKILL.md` carries under `metadata.revision`, so an agent that has loaded
that file can compare two strings it already holds and see that its
instructions predate its tool.

### Where the binary you run comes from

Worth stating once, because it costs time exactly where nobody expects it: **a
globally installed `ank` tracks the published release, not the tree you have
checked out.** The two are the same file only on the day of a release.

For most repositories that is the whole story: you are using Ank, not changing
it, and the published binary is the one you want. It matters when you are
working *on* a repository whose corpus is written by a binary newer than yours:
somebody else's release, or your own tree if you are contributing to Ank
itself. Then the tool managing the work and the tool being changed are different
versions, and both print the same `ank 0.2.0`. Only the commit separates them,
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

    $ ank find
    warning: corpus at schema 4, this binary reads 3: 1 entity left out of every listing
      -> the binary is older than the corpus: ank --version names the build, npm install -g @haksolot/ank replaces it
      TASK-0000  [open] Ordinary task

It warns and still answers, because the entities this build does understand are worth
having, and a corpus mid-migration is a real state rather than a broken one.

**The second line is one of two, and which one depends on whether a release can
help.** The schema a published version reads is stamped into the binary at build
time, from the newest tag's own source, so the message names the road that
actually resolves the state rather than the one that sounds like it does. Where
no release reads the corpus -- a schema that landed on the default branch after
the last tag, which is the ordinary case for a contributor -- it says so and
sends you to the tree instead:

      -> no release is known to read schema 4: ank --version names the build, build from the tree or wait for a release

Naming the install there would fetch the build that had just refused, and a
reader who follows advice that visibly does nothing concludes the tool is broken
rather than that their copy is old.

The other half, an old binary reading an old corpus, is not detectable: nothing
in the files says a newer format exists. That one is `--version` and the
paragraphs above.

## Initialise a repository

From the root of a git repository:

    $ ank init
    created .ank/tasks .ank/adr
    wrote .ank/config.yml
    wrote .gitattributes
    wrote .gitignore
    pointer added to AGENTS.md
    refspec added: +refs/ank/*:refs/ank/*

Six effects, and re-running changes nothing: `init` is idempotent and says
`already initialised, nothing to do`. The `.gitattributes` line keeps `.ank/`
in LF on checkout: on Windows git would otherwise convert back to CRLF
everything the tool has just written, on every clone. The `.gitignore` line is
`.ank/index.db`, the derived SQLite index: it is rebuilt from the files
whenever it is missing, so committing it would only track a binary that every
command rewrites. The refspec is what makes claims travel, since hosts do not
fetch non-standard refs on their own.

Both git files are appended to, never replaced, so an existing `.gitignore`
keeps everything already in it.

One edit to make now, before the first real command.

**Name your default branch.** It is the one key `init` leaves unset on purpose:
it runs where the reference branch is not known yet, and writing `main` there
would be exactly the guess the tool refuses everywhere else. `.ank/config.yml`
is written through the CLI rather than by hand:

    $ ank config default_branch main
    default_branch (unset) -> main

Without it, Ank looks for `refs/remotes/origin/HEAD`, and a repository with no
remote has none. It refuses rather than guessing:

    $ ank accept 06d2
    error[9]: default branch indeterminable (default_branch absent from .ank/config.yml, refs/remotes/origin/HEAD absent)
      -> git remote set-head origin -a
      -> or ank config default_branch <name>

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

    $ ank new adr --title "Opaque sessions rather than stateless JWT" \
        --scope "src/auth/**" \
        --constraint "Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store."
    created ADR-06d29e727d24 Opaque sessions rather than stateless JWT

It is created `proposed`, which means visible but not binding. Promotion goes
through one command, and that command is the only one in the tool that makes a
git commit:

    $ git add -A && git commit -m "adr: opaque sessions"
    $ ank accept 06d2
    accepted ADR-06d29e727d24 -> 9c45c50

Short prefixes work everywhere an id is accepted; an ambiguous one is an error
listing the candidates, never a guess. The commit it produced carries the hash
of `constraint` and `scope` at acceptance:

    ratify ADR-06d29e727d24

    constraint+scope: c5d4f3478ad5
    by: you@laptop

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

    $ ank config verifiers.auth-tests.run "sh tests/auth.sh"
    verifiers.auth-tests.run (unset) -> sh tests/auth.sh
    $ ank config verifiers.no-jwt.run "! grep -rq jwt.verify src/auth/"
    verifiers.no-jwt.run (unset) -> ! grep -rq jwt.verify src/auth/

which is what the file then carries:

    verifiers:
      auth-tests:
        run: sh tests/auth.sh
      no-jwt:
        run: "! grep -rq jwt.verify src/auth/"

`ank config --unset verifiers.no-jwt` takes one back out. Reading a key says
where the value comes from, this repository or a default the tool resolved:

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

    $ ank new task --title "Migrate auth" --scope "src/auth/**" --verify auth-tests
    error[7]: no verifier 'auth-tests' in .ank/config.yml
      -> ank config verifiers.auth-tests.run "<command>"

With the definitions in place:

    $ ank new task --title "Migrate auth to opaque sessions" \
        --scope "src/auth/**" \
        --criteria "The auth tests pass and no reference to jwt.verify remains in src/auth/" \
        --verify auth-tests --verify no-jwt
    created TASK-820d259af6a7 Migrate auth to opaque sessions

A composite criterion is mechanised by several verifiers, not by one that
covers half of it. All of them must pass.

## Orientation

`ank context` is the first call, and the only one you have to remember. With no
argument it covers the whole repository; with a path it covers that perimeter.

    $ ank context src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store.

    TASKS (1)
      TASK-820d  [open] Migrate auth to opaque sessions

    > ank claim TASK-820d to start

Constraints come first and are never truncated once you are working. The output
ends with the next command, as every output here does.

## Claim

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

    $ ank claim 51c2
    error[7]: marie@laptop holds a live claim on TASK-820d259af6a7 (expires in 24m)
      -> ank release --reason "<why>"   (a second session on this machine sets its own ANK_AGENT)

If you meant it, the first way out is the one to take: finish the task you hold
or hand it back. If the refusal surprises you, it is almost certainly two
terminals: `$ANK_AGENT` unset resolves to `<user>@<hostname>`, so two sessions
on one machine are the same agent as far as the refs can tell, so they would see
each other's claims and renew them. Give every concurrent session an identity
of its own:

    $ ANK_AGENT=marie-2@laptop ank claim 51c2

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

    $ ank context

    TASK-820d  Migrate auth to opaque sessions

    DONE_CRITERIA
      The auth tests pass and no reference to jwt.verify remains in src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store.

`ank show 820d` gives you the entity whole, frontmatter, body and log, which
is where the reasoning behind a task lives.

## Work, and log what you learn

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

    $ ank done
    running: auth-tests ... ok (0.1s)
    running: no-jwt ... ok (0.1s)
    proof recorded: auth-tests@94a1f671c577 -> local/e3b0c44298fc@9c45c50  (scope/b623b24a5777)
    proof recorded: no-jwt@791cc818d0ad -> local/e3b0c44298fc@9c45c50  (scope/b623b24a5777)
    TASK-820d259af6a7 -> done

This is the point of the tool. Ank ran the verifiers itself and wrote what
actually ran: one proof entry each, carrying the hash of the verifier
definition that executed, the HEAD commit, and a hash of the scope files'
content at that moment. Nobody reported their own result. Never set
`status: done` by hand: a status written by the party being measured measures
nothing.

A task with no `verify` takes the other branch, and `--proof` becomes
mandatory:

    $ ank done
    error[5]: proof required to move TASK-0ff108d8e2ca to done
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

    $ ank check
    signal: TASK-820d259af6a7: finished on another branch, main has not caught up
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    check: ok — 1 tasks, 1 adr, 2 signal(s)

The first signal is the mechanism doing its job. `status: done` lives in the
file, therefore on your branch alone until the merge, and during all that time
the task would look free to everyone else. The claim ref is not deleted at
`done`: it becomes a completion ref, and anyone who tries to claim the task is
refused with the commit and the branch named. Commit, and the ref is pruned:

    $ git add -A && git commit -m "TASK-820d is done"
    $ ank check
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    pruned refs/ank/claims/TASK-820d259af6a7
    check: ok — 1 tasks, 1 adr, 1 signal(s)

`check` is what you put in CI. It validates parsing, byte-for-byte round-trip,
`blocked_by` references, frozen fields against their anchors, and it prunes the
coordination plane. **Exit 8 means findings; signals exit 0**, because a signal is
something a reader should see, not a failure.

## When a command refuses

The exit code carries the meaning so a script can route without parsing
anything, and the message always ends with the exact command to run next.

| Code | Meaning |
|---|---|
| 2 | entity not found, or ambiguous prefix |
| 3 | version conflict, re-read and retry |
| 4 | task unavailable, held by someone else or finished on another branch |
| 5 | proof missing or invalid |
| 6 | illegal transition, or a frozen field diverged |
| 7 | missing prerequisite: no criterion, task blocked, `accept` off the default branch |
| 8 | `check` found something |
| 9 | environment: git too old, `sh` missing, default branch indeterminable |

Code 9 is the one to read carefully: it says the environment is broken, not
that your work is wrong. Fix the machine, not the code.

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
cannot race the merge.

The proof is a ref, so it has to reach the remote to be worth anything, and
because the ref is the whole of what this verb produces, a push that did not
land is a failure and not a warning: `attest --detached` exits **9** and names
the push to run. Nothing special is needed to notice it, which is the point:

    ank attest "$id" --proof "test:$RUN_ID" --detached

`--json` still reports `"pushed"`, so an integration that prefers to read the
flag reads the same fact. What it must not do is read the flag *instead* of the
code, because the two now say the same thing. This repository's own `ci.yml`
reads the code.

## Handing the loop to an agent

Four routes install the same file (the `skills` CLI, the Claude Code plugin,
`pi`, and copying one markdown file by hand) and they are walked with their real
output in [agents.md](agents.md), along with `$ANK_AGENT` and what changes when
more than one agent works the same repository.

The shortest of them, which detects what you run and links it to a single copy:

    npx skills add haksolot/ank

That installs the skill, not the binary. The skill teaches one page, and it is
loaded on every session, which is why its content is deliberately small.

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

- [agents.md](agents.md): the four routes that reach an agent, the binary
  channels beyond npm, and what running several agents actually requires.
- [format.md](format.md): the file format and canonical form, for anyone
  writing a tool that reads or writes `.ank/`.
- [alternatives.md](alternatives.md): how this compares to retrieval, an
  LLM-maintained wiki, and OKF.
- The specification, the source of truth for everything above: ten `spec`
  documents in `.ank/`, read with `ank find --type spec` and `ank show <id>`.
  They argue the design; they are not a tutorial.
- `ank help` lists every verb, `ank help <verb>` answers about one.
