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

Every release carries a static binary for three targets —
`x86_64-unknown-linux-musl`, `aarch64-apple-darwin` and
`x86_64-pc-windows-msvc` — each with a `.sha256` beside it. Take the archive for
your platform from the [releases page](https://github.com/haksolot/ank/releases/latest),
check the hash, unpack it, and put `ank` on your `PATH`.

To build instead:

    cargo install --git https://github.com/haksolot/ank ank-cli

That puts `ank` in `~/.cargo/bin`. From a clone, `cargo build --release` leaves
it at `target/release/ank`. Either way, check it answers:

    $ ank --version
    ank 0.1.1 (bc59636)

It prints the version and the commit it was built from. That second half
matters the first time you suspect the binary in your hand is older than the
behaviour you are reading about.

## Initialise a repository

From the root of a git repository:

    $ ank init
    created .ank/tasks .ank/adr
    wrote .ank/config.yml
    wrote .gitattributes
    pointer added to AGENTS.md
    refspec added: +refs/ank/*:refs/ank/*

Five effects, and re-running changes nothing — `init` is idempotent and says
`already initialised, nothing to do`. The `.gitattributes` line keeps `.ank/`
in LF on checkout: on Windows git would otherwise convert back to CRLF
everything the tool has just written, on every clone. The refspec is what makes
claims travel, since hosts do not fetch non-standard refs on their own.

Two edits to make now, before the first real command.

**Name your default branch.** Open `.ank/config.yml` and add one line:

    default_branch: main

Without it, Ank looks for `refs/remotes/origin/HEAD`, and a repository with no
remote has none. It refuses rather than guessing:

    $ ank accept 06d2
    error[9]: default branch indeterminable (default_branch absent from .ank/config.yml, refs/remotes/origin/HEAD absent)
      -> git remote set-head origin -a
      -> or add "default_branch: <name>" to .ank/config.yml

**Ignore the index.** `.ank/index.db` is a derived SQLite cache, rebuildable
from the files and safe to delete at any time. It does not belong in git, and
`init` does not yet write the rule for you:

    echo '.ank/index.db' >> .gitignore

## The two kinds of file

Flat in `.ank/`, markdown with YAML frontmatter, and only two of them.

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
in `.ank/config.yml`, under the repository's own review:

    verifiers:
      auth-tests:
        run: sh tests/auth.sh
      no-jwt:
        run: "! grep -rq jwt.verify src/auth/"

Declare them first. A task that names a verifier `config.yml` does not know is
refused at creation:

    $ ank new task --title "Migrate auth" --scope "src/auth/**" --verify auth-tests
    error[7]: no verifier 'auth-tests' in .ank/config.yml
      -> declare it under verifiers: in .ank/config.yml

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
agents reaching for the same task — git's compare-and-swap, one winner. And the
`done_criteria` was frozen: its hash went into the claim record, where the
file's editor cannot reach it.

The freeze is the rule worth internalising. Editing the criterion to make the
work fit does not unblock anything; `done` compares against the recorded hash
and refuses, and `check` reports the divergence. If the criterion is genuinely
wrong, hand the task back — `ank release --reason "<why>"` — and say so.

`claim` also sets HEAD, so the following commands need no id. One claim at a
time, per person and per agent.

Run `ank context` again and the output inverts: no other task, the full
criterion, and the constraints matching this task's scope.

    $ ank context

    TASK-820d  Migrate auth to opaque sessions

    DONE_CRITERIA
      The auth tests pass and no reference to jwt.verify remains in src/auth/

    CONSTRAINTS (1 active)
      ADR-06d2  Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store.

`ank show 820d` gives you the entity whole — frontmatter, body and log — which
is where the reasoning behind a task lives.

## Work, and log what you learn

    $ ank log "jwt.verify removed from session.ts"
    logged on TASK-820d259af6a7

The log is a work trace, not proof. Write to it when you discover something,
not when you finish: it renews the claim, so working is what keeps the lock and
there is no heartbeat command to remember. A claim lasts 30 minutes by default;
if it expires because a build ran long, the task stays `in_progress` and you
re-acquire silently, provided nobody took it over.

`ank log 820d` with no message reads the log back instead of writing, and needs
no claim.

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
`status: done` by hand — a status written by the party being measured measures
nothing.

A task with no `verify` takes the other branch, and `--proof` becomes
mandatory:

    $ ank done
    error[5]: proof required to move TASK-0ff108d8e2ca to done
      -> ank done --proof test:<ci-run-ref>

The proof types are `commit`, `test`, `human-review` and `assertion`, and what
separates them is not local versus hosted but **who controls the environment**.
A CI reference is out of the agent's reach and guarantees the most;
`commit:<sha>` is checked with git; a local test proves what ran in a tree the
agent could have altered; `assertion:"..."` guarantees nothing and is marked
weak, which is what keeps it from quietly becoming the default path.

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
coordination plane. **Exit 8 means findings; signals exit 0** — a signal is
something a reader should see, not a failure.

## When a command refuses

The exit code carries the meaning so a script can route without parsing
anything, and the message always ends with the exact command to run next.

| Code | Meaning |
|---|---|
| 2 | entity not found, or ambiguous prefix |
| 3 | version conflict — re-read and retry |
| 4 | task unavailable — held by someone else, or finished on another branch |
| 5 | proof missing or invalid |
| 6 | illegal transition, or a frozen field diverged |
| 7 | missing prerequisite — no criterion, task blocked, `accept` off the default branch |
| 8 | `check` found something |
| 9 | environment — git too old, `sh` missing, default branch indeterminable |

Code 9 is the one to read carefully: it says the environment is broken, not
that your work is wrong. Fix the machine, not the code.

## Handing the loop to an agent

    npx skills add haksolot/ank

That installs the skill, not the binary. The skill teaches one page: the loop
`context → claim → show → log → done`, the three off-loop verbs `new`, `find`
and `release`, and the rules that are not negotiable. It is loaded on every
session, which is why its content is deliberately small.

One convention it carries is worth knowing before you see an agent follow it:
**`.ank/` is opaque to an agent, the way `.git/` is.** Reading goes through
`ank show`, `ank find` and `ank context`; writing goes through the verbs. The
CLI knows what the files do not — the context budget, the frozen criterion, who
holds which claim. A human with an editor keeps every power they had.

## Where to go next

- [format.md](format.md) — the file format and canonical form, for anyone
  writing a tool that reads or writes `.ank/`.
- [ank-spec-v1.1.md](ank-spec-v1.1.md) — the specification, and the source of
  truth for everything above. It argues the design; it is not a tutorial.
- `ank help` lists every verb, `ank help <verb>` answers about one.
