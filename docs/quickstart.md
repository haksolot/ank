# Quickstart

By the end of this page your repository holds one binding constraint, one
finished task, and a proof that Ank wrote after running the verification
itself. It takes about ten minutes, and nothing here asks you to read the
specification. It assumes `ank` answers `ank --version`; [Install](install.md)
gets it there.

Every command and every output below was run against a fresh repository, and
the suite replays them against the binary on every change. Where the tool
refuses, the refusal is shown as it appears. Each step says what it does and
links to the page that explains it whole.

## Initialise a repository

**Three files are in the tree before anything starts**, because every scope and
every verifier below is pointed at one of them. These are their exact contents,
one line each, and they are written out because two of the hashes further down
are hashes of them:

    src/auth/session.ts   export function createSession(id) { return store.put(id) }
    tests/auth.sh         exit 0
    README.md             # auth service

What matters is that all three exist. A scope matching no file and a verifier
whose command is not there are the two ways this walk ends in red, and neither
says anything until `ank done`, a dozen commands later.

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
`init` is idempotent and says `already initialised, nothing to do`. Whether you
want an `origin` at all is a question about coordination, and [Claims and
identity](claims.md#where-claims-travel) answers it.

One directory is created, not one per kind: entities live flat in
`.ank/entities` whatever they are (ADR-c9f9d0d6f05d), and a task, an ADR, a
spec and a log entry are told apart by a field rather than by a folder. The
`.gitattributes` line keeps `.ank/` in LF on checkout: on Windows git would
otherwise convert back to CRLF everything the tool has just written, on every
clone. The `.gitignore` line is `.ank/index.db`, the derived SQLite index: it
is rebuilt from the files whenever it is missing, so committing it would only
track a binary that every command rewrites. The refspec is what makes claims
travel, since hosts do not fetch non-standard refs on their own. Both git files
are appended to, never replaced, so an existing `.gitignore` keeps everything
already in it. [The file format](format.md) describes the whole layout.

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

A clone sets `refs/remotes/origin/HEAD` for you, so a repository with an
`origin` resolves the branch without the key; setting it anyway costs one line
and removes the difference between the two.

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
[Entity fields](entity-fields.md); you do not need it to work through this page.

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
which is how `ank check` notices. `accept` runs on the default branch only, and
there is no flag around it: a constraint ratified on a feature branch would bind
on that branch alone. How a ratification reaches a protected default branch is
[Ratifying](ratifying.md), and how others can verify it is [Signing
keys](signing.md).

## Declare the verifiers, then the task

A task names verifiers; it never carries a shell command. The definitions live
in `.ank/config.yml`, and writing a `run` for a name the file does not carry is
how a verifier is declared:

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

A composite criterion is mechanised by several verifiers, not by one that
covers half of it, and all of them must pass. What else a verifier can be told,
the list a task gets when it names none, and the task that deliberately has no
verifier at all are [Proof and verifiers](proof.md).

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

The task moved to `in_progress`, a claim ref now says who holds it, and its
`done_criteria` was frozen by hash where the file's editor cannot reach it.
`claim` also sets HEAD, so the following commands need no id. What a claim is,
how long it lasts, and why a second session needs an identity of its own are
[Claims and identity](claims.md).

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
there is no heartbeat command to remember. Each entry is an entity of its own,
which is why the line names one, and `ank log 820d` with no message reads them
back, newest first. How entries are stored is [the file
format](format.md#the-log).

## Finish

<!-- replay walk -->

    $ ank done
    running: auth-tests ... ok (0.0s)
    running: no-jwt ... ok (0.0s)
    proof recorded: auth-tests@94a1f671c577 -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    proof recorded: no-jwt@791cc818d0ad -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    TASK-820d259af6a7 -> done

This is the point of the tool. Ank ran the verifiers itself and wrote what
actually ran; nobody reported their own result. Never set `status: done` by
hand: a status written by the party being measured measures nothing. What each
hash on those lines is, and what `done` asks for when a task has no verifier,
are [Proof and verifiers](proof.md#what-done-records).

## Commit, and read `check`

**Ank never commits, except `accept`.** Everything else writes files and leaves
them in your working tree, so the corpus travels through your normal review
like any other change. Before you commit, `check` has something to say:

<!-- replay walk -->

    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: TASK-820d259af6a7: finished on another branch, main has not caught up
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 4 entity file(s) differ from main: this checkout does not carry the corpus the default branch does (git merge main)
    check: ok — 1 tasks, 1 adr, 4 signal(s)

Commit, and the two signals about the uncommitted work go:

<!-- replay walk -->

    $ git add -A && git commit -m "the migration is done"
    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 3 hot entities are cold and belong in .ank/archive/entities/, with every entry about them (ank archive)
    └── ank archive --dry-run lists them and moves nothing
    pruned refs/ank/claims/TASK-820d259af6a7
    check: ok — 1 tasks, 1 adr, 3 signal(s)

**That is the green this page set out to reach**, with an `origin` and without:
signals, no fault, exit 0. A signal is something a reader should see and never a
failure; exit 8 is reserved for findings. What each of those lines means, and
why `check` just pruned a ref, is [Reading ank check](check.md).

When a command refuses instead, the exit code carries the meaning so a script
can route without parsing anything, and the message always ends with the exact
command to run next. The codes are [the exit-code reference](exit-codes.md).

## Starting where there is already code

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
actually ran, hashed.

**Freezing is verifiable, not defended.** The CLI cannot stop you editing a file
and does not pretend to. Frozen fields are anchored by a hash the editor does not
control, and `ank check` compares. Editing a criterion to unblock yourself
unblocks nothing; it makes the divergence visible.

**Git does the hard parts.** Claims are git refs, so the compare-and-swap that
arbitrates two agents is the one git already guarantees. Undo, history and
recovery are git's, and there is no server to run.

One call is bounded at 8000 characters by default, roughly 2000 tokens, which is
the constraint every one of those choices is paid for by: what `context` serves
has to fit in a context window beside the code. The normative text behind all of
it is [the specification](specification.md); `ank help` lists every verb, and
`ank help <verb>` answers about one.
