# Quickstart

By the end of this page your repository holds one binding constraint, one
finished task, and a proof that Ank wrote after running the verification
itself. It takes about ten minutes. It assumes `ank` answers `ank --version`;
[Install](install.md) gets it there.

Every command and output below was run against a fresh repository, and the
suite replays them against the binary on every change. Each step says what it
does and links to the page that explains it whole.

## Initialise a repository

**Three files are in the tree before anything starts**, because every scope and
every verifier below points at one of them. These are their exact contents, one
line each, since two hashes further down are hashes of them:

    src/auth/session.ts   export function createSession(id) { return store.put(id) }
    tests/auth.sh         exit 0
    README.md             # auth service

All three have to exist. A scope matching no file and a verifier whose command
is missing are the two ways this walk ends in red, and both stay silent until
`ank done`, a dozen commands later.

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
    @AGENTS.md added to CLAUDE.md
    refspec added: +refs/ank/*:refs/ank/*

**Two of those files are for the agents who will work here.** `AGENTS.md`
gains a short block telling them to run `ank context <path>` before editing a
path and to reach `.ank/` through the CLI, and naming `claim`, `done` and
`help`. `CLAUDE.md` gains the line `@AGENTS.md`, which makes Claude Code load
that block. Only the run that creates `.ank/` writes them.

**A repository with no `origin` prints the last line differently**: there is no
remote to add a refspec to, so `init` names itself as the command to run once
there is one. A second `init` after adding the remote picks it up:

<!-- replay bare ANK_AGENT=human:marie
$ echo '# auth service' > README.md && git add -A && git commit -q -m "the service"
-->

    $ ank init
    created .ank/entities
    wrote .ank/config.yml
    wrote .gitattributes
    wrote .gitignore
    pointer added to AGENTS.md
    @AGENTS.md added to CLAUDE.md
    refspec +refs/ank/*:refs/ank/* not added: no remote named origin yet, run this once it exists (ank init)
    $ git init -q --bare ../origin.git && git remote add origin ../origin.git
    $ ank init
    refspec added: +refs/ank/*:refs/ank/*

Otherwise a second `init` writes nothing and says `already initialised,
nothing to do`. Whether you want an `origin` at all is a question about
coordination, answered in [Claims and identity](claims.md#where-claims-travel).

Every entity lives flat in `.ank/entities`, and a field tells a task from an
ADR. The `.gitattributes` line keeps `.ank/` in LF on a Windows checkout. The
`.gitignore` line covers the SQLite index, which every command rebuilds from
the files when it is missing. The refspec makes claims travel, since hosts
fetch only the standard refs on their own. Both git files are appended to, so
an existing `.gitignore` keeps what it had. [The file format](format.md)
describes the whole layout.

**Name your default branch.** It is the one key `init` leaves unset on purpose:
`init` runs before the reference branch is known, and writing `main` there would
be a guess. `.ank/config.yml` is written through the CLI:

<!-- replay walk -->

    $ ank config default_branch main
    default_branch (unset) -> main

Without it, Ank looks for `refs/remotes/origin/HEAD`, which a repository with
no remote lacks, and `check` says what it stopped judging:

<!-- replay unset ANK_AGENT=human:marie
$ ank init
-->

    $ ank check | grep 'default branch'
    signal: coordination: default branch indeterminable, completion refs neither pruned nor judged and log entries not compared against it (ank config default_branch <name>)

A clone sets `refs/remotes/origin/HEAD` for you; setting the key anyway costs
one line and makes both kinds of repository behave alike.

## The two kinds this page uses

Each entity is a markdown file with YAML frontmatter. Four kinds exist and this
page needs two; the others are a **spec**, normative text that describes, and a
**log entry**, written once.

An **ADR** is a decision that constrains code. Its `constraint` is the one
field injected into an agent's context, so it is short and imperative; the body
holds the reasoning and costs nothing at injection time.

A **task** is a unit of work. Its `done_criteria` says what would prove it
finished.

Both carry a `scope`, a list of globs, and the scope is the only thing joining
the two kinds. "Everything about the auth migration" is answered by
`ank context src/auth/`. The full field list is in
[Entity fields](entity-fields.md).

## Write the first constraint

<!-- replay walk -->

    $ ank new adr --title "Opaque sessions over stateless JWT" \
        --scope "src/auth/**" \
        --constraint "Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store."
    created ADR-06d29e727d24 Opaque sessions over stateless JWT

It is created `proposed`: visible, and binding nobody yet. One command
promotes it. It writes the ratification as a tag at `refs/ank/ratify/<id>`,
signed where the repository signs, updates the file, and commits nothing:

<!-- replay walk -->

    $ git add -A && git commit -m "adr: opaque sessions"
    $ ank accept 06d2
    accepted ADR-06d29e727d24 -> 3f0a6b2

Short prefixes work everywhere an id is accepted, and an ambiguous one lists
the candidates. The tag's message carries the hash of `constraint` and `scope`
at acceptance:

<!-- replay walk
$ git cat-file tag refs/ank/ratify/ADR-06d29e727d24 | sed '1,/^$/d'
-->

    ratify ADR-06d29e727d24

    constraint+scope: c5d4f3478ad5
    by: human:marie

That hash is the anchor: edit the constraint afterwards and `ank check`
notices. `accept` runs on any branch, and the file it updates travels with your
next commit. [Ratifying](ratifying.md) covers the act and
[Signing keys](signing.md) how others verify it.

## Declare the verifiers, then the task

A task names verifiers, and their commands live in `.ank/config.yml`. Writing a
`run` for a new name declares one:

<!-- replay walk -->

    $ ank config verifiers.auth-tests.run "sh tests/auth.sh"
    verifiers.auth-tests.run (unset) -> sh tests/auth.sh
    $ ank config verifiers.no-jwt.run "! grep -rq jwt.verify src/auth/"
    verifiers.no-jwt.run (unset) -> ! grep -rq jwt.verify src/auth/

Then the task, naming both:

<!-- replay walk -->

    $ ank new task --title "Migrate auth to opaque sessions" \
        --scope "src/auth/**" \
        --criteria "The auth tests pass and no reference to jwt.verify remains in src/auth/" \
        --verify auth-tests --verify no-jwt
    created TASK-820d259af6a7 Migrate auth to opaque sessions

Every verifier a task names has to pass. The list a task gets when it names
none, and everything else about verifiers, is in [Proof and verifiers](proof.md).

## Orientation

`ank context` is the first call, and the only one you have to remember. With no
argument it covers the whole repository; with a path it covers that perimeter.

<!-- replay walk -->

    $ ank context src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Opaque sessions over stateless JWT

    TASKS (1)
      TASK-820d  [open] Migrate auth to opaque sessions

    > ank claim TASK-820d to start

**Before a claim, each constraint shows its title.** The question here is which
perimeter to enter, and titles answer it in the least space. The full
constraint arrives with the claim, below; `ank show 06d2` prints it at any time.
The output ends with the next command, as every output here does.

## Claim

<!-- replay walk -->

    $ ank claim 820d
    claimed TASK-820d259af6a7 migrate-auth-to-opaque-sessions -> HEAD

The task is `in_progress`, a ref says who holds it, and its `done_criteria` is
frozen by hash. The next commands need no id. How long a claim lasts and why
each session needs its own identity are [Claims and identity](claims.md).

Run `ank context` again and it narrows to this task: its criterion, and the
constraints matching its scope in full.

<!-- replay walk -->

    $ ank context

    TASK-820d  Migrate auth to opaque sessions

    DONE_CRITERIA
      The auth tests pass and no reference to jwt.verify remains in src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store.

`ank show 820d` prints the entity whole: frontmatter, body, where the
reasoning lives, and log.

## Work, and log what you learn

<!-- replay walk -->

    $ ank log "jwt.verify removed from session.ts"
    logged LOG-6b0f39d7a4c1 on TASK-820d259af6a7

The log is a work trace. Write to it when you discover something: each entry
renews the claim, so working is what keeps the lock. `ank log 820d` with no
message reads the entries back, newest first. How they are stored is
[the file format](format.md#the-log).

## Finish

<!-- replay walk -->

    $ ank done
    running: auth-tests ... ok (0.0s)
    running: no-jwt ... ok (0.0s)
    proof recorded: auth-tests@94a1f671c577 -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    proof recorded: no-jwt@791cc818d0ad -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    TASK-820d259af6a7 -> done

This is the point of the tool: Ank ran the verifiers itself and recorded what
ran, so the status is Ank's measurement of the tree. What each hash on those
lines means is in [Proof and verifiers](proof.md#what-done-records).

## Commit, and read `check`

**Ank leaves committing to you.** Its files wait in your working tree and travel
through your normal review. Before you commit, `check` reads them:

<!-- replay walk -->

    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: TASK-820d259af6a7: finished on another branch, main has not caught up
    signal: corpus: 5 entity file(s) differ from main: this checkout does not carry the corpus the default branch does (git merge main)
    signal: signers: no ratification key declared: permissions are advisory, not enforced (§8)
    check: ok — 1 tasks, 1 adr, 4 signal(s)

Commit, and the two signals about uncommitted work go:

<!-- replay walk -->

    $ git add -A && git commit -m "the migration is done"
    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: corpus: 3 hot entities are cold and belong in .ank/archive/entities/, with every entry about them (ank archive)
    └── ank archive --dry-run lists them and moves nothing
    signal: signers: no ratification key declared: permissions are advisory, not enforced (§8)
    pruned refs/ank/claims/TASK-820d259af6a7
    check: ok — 1 tasks, 1 adr, 3 signal(s)

**That is the green this page set out to reach**: signals, no fault, exit 0.
Each line, and the pruned ref, is explained in [Reading ank check](check.md).

When a command refuses, its exit code carries the meaning and its message ends
with the command to run next. The codes are [the exit-code reference](exit-codes.md).

## Starting where there is already code

Everything above happened in a new repository. `ank init` on two years of
history leaves you with an empty corpus, and the question becomes which
decisions this code has already made. That reading is an agent's work, and
these three prompts ask for it. Both installers offer to print the same three,
and a test holds the copies together.

Run this once, at the root of the repository:

    ank init

Then paste each prompt into your agent, one at a time, reading what comes back
before you send the next.

**What the code already decided.** This is the one you judge the tool on. A
constraint you recognise on sight was true and unwritten, and now the next agent
reads it without being told.

<!-- adopt-prompts:begin -->

    Read this repository and write, as ank ADRs, the decisions its code
    has already made: the ones a newcomer would break without knowing
    they existed. One ADR per decision, each with a scope glob covering
    the files it binds and a constraint stated as a rule. Leave them
    proposed; I ratify them myself.

They land `proposed`. `ank review` lists what is waiting, and `ank accept <id>`
stays yours, one at a time.

**What it still owes.** An agent can take work that has a scope and a
criterion, which is what `ank claim` freezes and `ank done` measures. TODOs and
issues become that here.

    Read the TODOs, the open issues and the README of this repository,
    and turn what they promise into ank tasks. Give each one a scope
    glob and a done_criteria a test could settle, and use blocked_by
    only where a task genuinely waits on another.

**Whether the answer holds.** The third is the uncomfortable one. Writing down
what was implicit ordinarily turns up constraints the code already breaks, and
what you want first is the list, before anybody starts repairing.

    Run ank check and ank review here, then read every ADR back against
    the code its scope matches. Tell me which constraints the code
    already breaks and which scopes match no file, and change nothing
    until I have read your answer.

<!-- adopt-prompts:end -->

The tool waits on none of the three. A corpus holding only the ADRs from the
first prompt is already worth `ank context`.

## Why it works this way

You have now done the loop once, which is the right moment for the four
properties underneath it.

**Scope, not hierarchy.** Constraints and work are two planes joined only by
globs. A rule written last year binds work created today, and a glob can be
checked against the filesystem, where a label can only be remembered.

**Nobody declares themselves done.** An agent that reports its own result can
simply be wrong, so `ank done` runs the verifiers itself and records what ran,
hashed.

**Freezing is verifiable.** You can edit any file. Frozen fields are anchored by
a hash the editor cannot reach, and `ank check` compares, so editing a
criterion to unblock yourself only makes the divergence visible.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Undo, history and
recovery are git's, and there is no server to run.

One call is bounded at 8000 characters by default, roughly 2000 tokens: what
`context` serves has to fit in a context window beside the code, and every
choice above pays for that. The normative text behind all of
it is [the specification](specification.md); `ank help` lists every verb, and
`ank help <verb>` answers about one.
