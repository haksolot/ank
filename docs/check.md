# Reading ank check

`ank check` validates the corpus: every file parses and round-trips byte for
byte, every `blocked_by` and reference resolves, every frozen field still
matches its anchor, and every claim ref still means something. It is the verb
you put in CI, and the one to run before adding to a corpus, because it says what
is already known to be wrong.

It answers in two levels, and the difference is the whole of how to read it.

- **A fault** is something wrong with the corpus: a file that does not parse, a
  frozen criterion edited under a live claim, a ratification whose anchor no
  longer matches. Any fault makes `check` exit **8**.
- **A signal** is something a reader should see and is not a failure: a
  decision ratified by its own author, a task finished on a branch that has not
  merged, a scope that matches no file yet. Signals leave the exit code at
  **0**, because reddening a build over an observation teaches a team to stop
  reading `check`.

Exit **9** is neither: the environment, not the corpus -- git too old, the
default branch indeterminable. The full table is [the exit-code
reference](exit-codes.md).

## Signals on a finished task, before and after the merge

The repository here is the one [the quickstart](quickstart.md) ends with: one
ADR accepted by the person who wrote it, one task finished with `ank done`, and
nothing committed since.

<!-- replay check ANK_AGENT=human:marie
$ git init -q --bare ../origin.git && git remote add origin ../origin.git
$ mkdir -p src/auth tests && echo 'export function createSession(id) { return store.put(id) }' > src/auth/session.ts
$ echo 'exit 0' > tests/auth.sh && git add -A && git commit -q -m "the service"
$ ank init && ank config default_branch main
$ ank new adr --title "Opaque sessions rather than stateless JWT" --scope "src/auth/**" --constraint "Do not introduce self-contained JWTs for user auth. Every session goes through the Redis store."
created ADR-06d29e727d24 Opaque sessions rather than stateless JWT
$ git add -A && git commit -q -m "adr: opaque sessions" && ank accept 06d2
$ ank config verifiers.auth-tests.run "sh tests/auth.sh"
$ ank new task --title "Migrate auth to opaque sessions" --scope "src/auth/**" --criteria "The auth tests pass" --verify auth-tests
created TASK-820d259af6a7 Migrate auth to opaque sessions
$ ank claim 820d && ank log "jwt.verify removed from session.ts" && ank done
-->

    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: TASK-820d259af6a7: finished on another branch, main has not caught up
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 4 entity file(s) differ from main: this checkout does not carry the corpus the default branch does (git merge main)
    check: ok — 1 tasks, 1 adr, 4 signal(s)

Four signals and no fault, so exit 0, and each line starts with its subject.

- **Ratified by its own author.** You wrote the ADR and you ratified it, and
  `check` says so rather than deciding what it means: a solo maintainer does
  that legitimately, and a team may want to know.
- **Finished on another branch.** `status: done` lives in the file, so on your
  branch alone until the merge. The claim ref became a completion record at
  `done`, which is what tells every other tree the task is taken ([Claims and
  identity](claims.md#when-done-the-claim-becomes-a-completion)).
- **No ratification key declared.** Nobody has said whose signature makes a
  ratification binding, so the corpus says it runs on trust. [Signing
  keys](signing.md) is how that line goes away.
- **Entity files differ from main.** The same fact as the second line, said
  about the corpus rather than one task: what your checkout carries that the
  default branch does not.

Commit, and the completion record is pruned:

<!-- replay check -->

    $ git add -A && git commit -m "the migration is done"
    $ ank check
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 3 hot entities are cold and belong in .ank/archive/entities/, with every entry about them (ank archive)
    └── ank archive --dry-run lists them and moves nothing
    pruned refs/ank/claims/TASK-820d259af6a7
    check: ok — 1 tasks, 1 adr, 3 signal(s)

**`check` writes, and that line is why.** It is the only verb that prunes
`refs/ank/claims`: orphans, and completion records whose task is `done` or
`closed` on the default branch. Everything else it does is read-only, but a
verb that writes is not a verb to poll.

The new signal is the corpus noticing that three log entries are cold. An entry
is cold with its subject, and their subject is a task that is done
(ADR-467ce7e9cda1). The line under it is the command that answers it, and that
command reads before it moves:

<!-- replay check unordered -->

    $ ank archive --dry-run
    LOG-3b92d9719950  created (version 0 to 1, produced 4cc65f12691c)
    LOG-6b0f39d7a4c1  jwt.verify removed from session.ts
    LOG-f234f9b08f06  done, proof test:local/e3b0c44298fc@9c45c50
    3 cold, nothing moved (--dry-run): ank archive moves them

It lists and moves nothing, which on a corpus this size is the right answer.
When the move is worth making, it is a human's decision landing by pull request:
[Ratifying and archiving](ratifying.md#archiving-what-is-cold).

## A fault

A frozen field edited behind the tool is the fault a corpus is most likely to
meet. Claim a task, then change its criterion in the file:

<!-- replay check
$ ank new task --title "Say in the README what a session is now" --scope "src/auth/**" --criteria "The README names no JWT" --no-verify
created TASK-51c2a0f6d418 Say in the README what a session is now
$ ank claim 51c2
$ f=.ank/entities/TASK-51c2a0f6d418.md && sed 's/names no JWT/names nothing/' "$f" > x && mv x "$f"
-->

    $ ank check; echo "exit $?"
    error: TASK-51c2a0f6d418: done_criteria diverges from the claim (claimed 03c659e46adf, now ef485fff585a)
    signal: ADR-06d29e727d24: ratified by its own author (human:marie)
    signal: TASK-51c2a0f6d418: content is 3eff92338baa where the last write left 2e29f326854e: it was edited outside the CLI, which is legal and leaves no entry
    signal: allowed_signers: no ratification key declared: permissions are advisory, not enforced (§8)
    signal: corpus: 2 entity file(s) differ from main: this checkout does not carry the corpus the default branch does (git merge main)
    signal: corpus: 3 hot entities are cold and belong in .ank/archive/entities/, with every entry about them (ank archive)
    └── ank archive --dry-run lists them and moves nothing
    check: 1 fault(s) — 2 tasks, 1 adr, 5 signal(s)
    exit 8

A fault prints as `error:`, first, and the summary line counts it. The second
signal on the task is the same edit seen from the other side: its content no
longer hashes to what the last verb wrote, which alone would be legal. Nothing
refused the edit, and nothing could: the file is yours. What the edit cannot do
is make the hash recorded in the claim agree with it. The two criterion hashes
are content, so they are the same on your machine: `03c659e46adf` is the
criterion as written, `ef485fff585a` as edited. `done` refuses and `check` exits
8 until the criterion is put back or the task is handed back with `ank release
--reason`, which ends the freeze and leaves only the signal. Freezing is
verifiable, not defended.

## What check reads beyond the files

`check` walks git history. It finds the ratification commit that anchors each
accepted decision by its subject, and it asks where a scope that matches nothing
went, a rename or a deletion. A checkout that carries one commit does not make
`check` fail: it makes it report every frozen decision unverifiable, at exit 0,
which is why a pipeline has to fetch the whole history ([Running ank in
CI](ci.md)).

`ank check <path>` narrows the report to the entities whose scope covers that
path. `--json` carries every finding with its level, subject and message, and
it is what a script reads; [the machine surface](integrating.md) shows the
document.
