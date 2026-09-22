# Proof and verifiers

A task is finished when `ank done` says so, and `ank done` says so only after it
has run the task's verifiers itself and recorded what ran. This page is where
verifiers come from, how a task gets its list, what `done` writes, and what a
proof is worth depending on who produced it.

The examples run in the repository [the quickstart](quickstart.md) builds: a
`src/auth/session.ts`, a `tests/auth.sh` that exits 0, and a default branch
named `main`.

## A verifier is declared once, in config.yml

A task names verifiers; it never carries a shell command. The definitions live
in `.ank/config.yml`, under the repository's own review, and they go in through
the same verb as every other key: writing a `run` for a name the file does not
carry is how a verifier is declared.

<!-- replay proof ANK_AGENT=human:marie
$ mkdir -p src/auth tests && echo 'export function createSession(id) { return store.put(id) }' > src/auth/session.ts
$ echo 'exit 0' > tests/auth.sh && echo '# auth service' > README.md
$ ank init && ank config default_branch main
$ git add -A && git commit -q -m "the service"
-->

    $ ank config verifiers.auth-tests.run "sh tests/auth.sh"
    verifiers.auth-tests.run (unset) -> sh tests/auth.sh
    $ ank config verifiers.no-jwt.run "! grep -rq jwt.verify src/auth/"
    verifiers.no-jwt.run (unset) -> ! grep -rq jwt.verify src/auth/

which is what the file then carries:

<!-- replay proof part
$ cat .ank/config.yml
-->

    verifiers:
      auth-tests:
        run: sh tests/auth.sh
      no-jwt:
        run: "! grep -rq jwt.verify src/auth/"

`ank config --unset verifiers.no-jwt` takes one back out. Reading a key says
where the value comes from, this repository or a default the tool resolved:

<!-- replay proof -->

    $ ank config verifiers.auth-tests.run
    sh tests/auth.sh
    $ ank config verifiers.auth-tests.timeout
    10m (default)

The distinction is the reason writing is line surgery rather than a round-trip
through a YAML serializer. Your comments, blank lines, key order and quoting
survive a write, and a key you never set is never written out: an unset key
follows the tool, a written one is pinned here, and a serializer would quietly
convert every one of the first kind into the second. Every key a verifier takes
is in [config.yml keys](config-keys.md).

A verifier runs under `sh -c` from the root of the working tree, on all three
operating systems, and passes when it exits 0.

## A task names its list when it is written

Declare the verifiers first. A task that names one `config.yml` does not know is
refused at creation, so a name you misremember fails when you write the task
rather than at the close:

<!-- replay undeclared
$ ank init
-->

    $ ank new task --title "Migrate auth" --scope "src/auth/**" --verify auth-tests
    error[7]: no verifier 'auth-tests' in .ank/config.yml
      -> ank config verifiers.auth-tests.run "<command>"

With the definitions in place, `--verify` names them one at a time, and a
composite criterion is mechanised by several verifiers rather than by one that
covers half of it:

<!-- replay proof -->

    $ ank new task --title "Migrate auth to opaque sessions" \
        --scope "src/auth/**" \
        --criteria "The auth tests pass and no reference to jwt.verify remains in src/auth/" \
        --verify auth-tests --verify no-jwt
    created TASK-820d259af6a7 Migrate auth to opaque sessions

### The default list

`--verify` one at a time is the right amount of ceremony for a verifier that
suits one perimeter and the wrong amount for the suite every task in the
repository has to pass. A verifier marked `default` joins every task written
afterwards:

<!-- replay proof -->

    $ ank config verifiers.no-jwt.default true
    verifiers.no-jwt.default false (default) -> true

which lands beside the `run` it belongs to, and nowhere else in the file:

<!-- replay proof part
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

<!-- replay proof -->

    $ ank config verifiers.no-jwt.default
    true
    $ ank config verifiers.auth-tests.default
    false (default)

A task created now with no `--verify` of its own carries `verify: [no-jwt]` in
its frontmatter, without anybody naming it. `--verify` replaces that list
rather than adding to it, so a task that names its own verifiers gets exactly
those.

### Declining it

**`--no-verify` is the third possibility, and it is a judgement, not a
shortcut.** It writes a task with no `verify:` at all:

<!-- replay proof -->

    $ ank new task --title "Say in the README what a session is now" \
        --scope "README.md" \
        --criteria "The README describes opaque sessions and names no JWT" \
        --no-verify
    created TASK-51c2a0f6d418 Say in the README what a session is now

Use it where no declared verifier can settle the criterion -- prose a person has
to read, a behaviour only a published release answers -- and where that is
genuinely the case, say so when you write the task, so the empty list is
visible in its diff. A task that reaches `done` with an empty `verify:` nobody
decided on closes on a proof nothing ran.

## What done records

<!-- replay proof -->

    $ ank claim 820d
    claimed TASK-820d259af6a7 migrate-auth-to-opaque-sessions -> HEAD
    $ ank done
    running: auth-tests ... ok (0.0s)
    running: no-jwt ... ok (0.0s)
    proof recorded: auth-tests@94a1f671c577 -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    proof recorded: no-jwt@791cc818d0ad -> local/e3b0c44298fc@9c45c50  (scope/18d14da584ab)
    TASK-820d259af6a7 -> done

One proof entry per verifier, carrying the hash of the verifier definition that
executed, a hash of what it printed, the HEAD commit, and a hash of the scope
files' content at that moment. Four of those hashes are content and not
identity, so they are the same on your machine as on this page. `94a1f671c577`
and `791cc818d0ad` are the two verifier definitions; `e3b0c44298fc` is what each
of them printed, which is nothing; and `18d14da584ab` is `src/auth/session.ts`
as the quickstart wrote it. The commit is the one your tree is on, so that one
is yours.

If a verifier fails or times out, the transition is refused and the task stays
where it is.

**A task with no verifier needs a proof you hand it.** There is nothing for
`done` to run, so `--proof` becomes mandatory:

<!-- replay proof -->

    $ ank claim 51c2
    claimed TASK-51c2a0f6d418 say-in-the-readme-what-a-session-is-now -> HEAD
    $ ank done
    error[5]: proof required to move TASK-51c2a0f6d418 to done
      -> ank done --proof commit:<sha>

The reverse is refused too: a task that declares verifiers takes no `--proof`,
because what closes it is what ran and not what somebody typed. Give a proof you
already hold, `commit:<sha>`, and never a run id you would have to wait for.

## What a proof is worth

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
wrote it and the weakest when somebody typed it (ADR-b6b69053a47b). Typing
`--proof test:<run-id>` is still accepted and still recorded; what it does not
do is clear the `done with no test proof` signal, which stays until a pipeline
anchors the run. Entries written before the field carry no `via` and are read
exactly as they were.

`done` records what ran on the machine that ran it, which is a local claim.
[Anchoring a run](ci.md#anchoring-a-run) is how a pipeline adds the external
half after the merge, without a commit.

`done` proves the task in the working tree it ran in. It does not prove the
change merges, or that the combined system works: that gap, and the task that
closes it, are [Multi-agent work](multi-agent.md#parallel-work-and-integration).
