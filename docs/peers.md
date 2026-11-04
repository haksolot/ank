# Several repositories

One `.ank/` per repository stays authoritative: a corpus belongs to the code it
constrains. Several repositories still share decisions, a backend and the client
that calls it being the usual pair, and ank lets one corpus read another without
ever writing to it (ADR-a1de673043b4). This page walks through the whole of it
with two repositories side by side, `/work/api` and `/work/web`, in the order
you meet it: a declaration at both ends, the same peers from every worktree, the
override for a layout that differs, naming an entity of a peer, the peer's
default branch as the one place it is read, the peer constraints under a claim,
the view across repositories, and what never crosses.

Every block below is replayed against the binary in two scratch repositories.

## A binding needs a declaration at both ends

<!-- replay peers dir=/work ANK_AGENT=human:marie
$ mkdir -p /work/api/src /work/web/src
$ cd /work/api && git init -q && echo 'export const routes = []' > src/routes.ts && git add -A && git commit -q -m "the api"
$ cd /work/web && git init -q && echo 'export const client = {}' > src/client.ts && git add -A && git commit -q -m "the client"
$ cd /work/api && ank init && ank config default_branch main
$ cd /work/web && ank init && ank config default_branch main
-->

The decision lives in the repository that governs: `api` declares `web` as a
peer, by a path relative to its own root, and scopes its ADR into it with
`<peer>:<glob>`.

<!-- replay peers -->

    $ cd /work/api && ank config peers.web ../web
    peers.web (unset) -> ../web
    $ cd /work/api && ank new adr --title "Every request carries a trace id" \
        --scope "web:src/**" \
        --constraint "Every call the web client makes to the API sends an X-Trace-Id header."
    created ADR-351131160961 Every request carries a trace id

<!-- replay peers
$ cd /work/api && git add -A && git commit -q -m "adr: trace ids"
-->

<!-- replay peers -->

    $ cd /work/api && ank accept 3511
    accepted ADR-351131160961 -> 5c88658

<!-- replay peers
$ cd /work/api && git add -A && git commit -q -m "accept: trace ids"
-->

That is one end, once committed: another repository reads `api` from the git
objects of its default branch (ADR-09b985a5627e). Inside `web`, nothing binds yet, because `web` has not said it
reads `api`, and a corpus is never discovered from the filesystem:

<!-- replay peers -->

    $ cd /work/web && ank context src/

    > no ready tasks in scope

<!-- replay peers
$ cd /work/web && git add -A && git commit -q -m "the corpus"
-->

`check` in `api` names the missing half. The end that resolves its own
declaration reads the other end's `config.yml` at its default branch, sees no
peer there resolving back, and prints the command that completes the binding.
It is a signal and exits 0, since the repair lives in the other repository:

<!-- replay peers -->

    $ cd /work/api && ank check --kind peer-half-declared
    signal: peers.web: peer 'web' declares no peer resolving back to this corpus, so ADR-351131160961 binds nothing there (ank --repo ../web config peers.api ../api)
    check: ok — 0 tasks, 1 adr, 1 signal(s)

`check` in `web` says the same from the other side when only `web` declares.

The governed repository declares the governing one, and the constraint is
served where the code it binds lives, by `context` and by `scope` alike:

<!-- replay peers -->

    $ cd /work/web && ank config peers.api ../api
    peers.api (unset) -> ../api
    $ cd /work/web && ank context src/

    CONSTRAINTS (1 active)
      ADR-351131160961@api  Every request carries a trace id

    > no ready tasks in scope
    $ cd /work/web && ank scope src/client.ts
    src/client.ts

    ADR (1)
      ADR-351131160961@api  [accepted] Every request carries a trace id

So the direction is: the governing corpus names the governed one in its scope,
through its own `peers.<name>`; the governed corpus names the governing one in
its `peers.<name>` to read it. `ank help config` says the same in one line.

### Every worktree reads the same peers

<!-- replay peers
$ cd /work/web && git add -A && git commit -q -m "peers: api"
-->

An agent works in a linked worktree of its own, and a worktree can sit
anywhere: `../api` from `/work/trees/trace` would name `/work/trees/api`. A
relative peer path resolves against the root of the main checkout, the parent
of the git common directory, so every worktree of `web` reaches `api` with no
override (ADR-a14dabe4b259):

<!-- replay peers -->

    $ cd /work/web && git worktree add -q ../trees/trace -b trace
    $ cd /work/trees/trace && ank context src/

    CONSTRAINTS (1 active)
      ADR-351131160961@api  Every request carries a trace id

    > no ready tasks in scope

## When the committed path is wrong here

`peers.api ../api` in `config.yml` is a convention: the layout every clone is
expected to follow. On a machine where `api` was checked out somewhere else, the
peer is not a corpus, and the binding disappears with a warning that names the
fix:

<!-- replay peers -->

    $ mv /work/api /work/backend
    $ cd /work/web && ank context src/
    warning: peer 'api' at ../api is not a corpus, answered without it (ank config --user peers.98204934184e14045a2158a2bd0f23586745d560.api <path>)

    > no ready tasks in scope

The answer is never to edit `config.yml`, which every other clone shares. The
reader overrides the path in their own `corpora.yml`, keyed by the identity of
the corpus that declares the peer (its root commit) and the peer's name
(ADR-da2819aef598):

<!-- replay peers -->

    $ cd /work/web && ank config --user peers.98204934184e14045a2158a2bd0f23586745d560.api ../backend
    peers.98204934184e14045a2158a2bd0f23586745d560.api ../backend
    $ cd /work/web && ank context src/

    CONSTRAINTS (1 active)
      ADR-351131160961@api  Every request carries a trace id

    > no ready tasks in scope

The override is read wherever the declaration is read, including when a peer
resolves one of its own declarations back to the reader, and it is never
committed.

A peer is a path to a checkout that already exists on this disk, never a URL: no
verb clones, fetches or caches one (ADR-96fe1f9d619a). `ank config` refuses a
value shaped as a URL when it is written, and names the override instead:

<!-- replay peers -->

    $ cd /work/web && ank config peers.api git@github.com:acme/api.git
    error[1]: 'git@github.com:acme/api.git' is a URL: a peer is a path to a checkout on this disk, and ank never clones or fetches one
      -> clone it where you want it, then ank config --user peers.98204934184e14045a2158a2bd0f23586745d560.api <path>

## Naming an entity of a peer

An entity whose home is a peer is printed `<id>@<peer>`, and every verb that
reads an identifier takes the same form:

<!-- replay peers -->

    $ cd /work/web && ank show ADR-3511@api
    ---
    id: ADR-351131160961
    type: adr
    slug: every-request-carries-a-trace-id
    title: Every request carries a trace id
    created: 2026-10-02T16:35:33Z
    author: human:marie
    status: accepted
    scope:
      - web:src/**
    constraint: |
      Every call the web client makes to the API sends an X-Trace-Id header.
    ratified: 65824fc735cf
    anchor: 5c88658e1f2b9d0a47c3e6f81b2d94a7c05e3f19
    verified:
      - by: human:marie
        at: 2026-10-02T16:35:33Z
    schema: 4
    version: 2
    ---

    EDITS (1)
    └── 2026-10-02T16:35:33Z human:marie — created (version 0 to 1, produced cd2cc889ba78)

A task can wait on a task of a peer the same way (ADR-c23bef1cc93e). The
backend writes the work its client depends on:

<!-- replay peers -->

    $ cd /work/backend && ank new task --title "Accept X-Trace-Id on every route" \
        --scope "src/**" --criteria "Every route reads X-Trace-Id" --no-verify
    created TASK-e799393f5249 Accept X-Trace-Id on every route

<!-- replay peers
$ cd /work/backend && git add -A && git commit -q -m "task: trace ids"
-->

and the client's task names it in `blocked_by`:

<!-- replay peers -->

    $ cd /work/web && ank new task --title "Send X-Trace-Id from the client" \
        --scope "src/**" --criteria "The client sends X-Trace-Id on every call" \
        --blocked-by TASK-e799393f5249@api --no-verify
    created TASK-8eed823dc16e Send X-Trace-Id from the client

<!-- replay peers
$ cd /work/web && git add -A && git commit -q -m "task: send trace ids"
-->

`claim` treats it as blocked while the peer's task is not done, and points at
the task it waits on:

<!-- replay peers -->

    $ cd /work/web && ank claim 8eed
    error[7]: TASK-8eed823dc16e is blocked by TASK-e799393f5249@api
      -> ank show TASK-e799393f5249@api

**An edge that cannot be read holds.** A reader that cannot see a peer answers
locally and says so; a blocker doing the same would answer "nothing blocks you",
which releases exactly the work the edge exists to hold. Remove the override, so
that `api` is once more not a corpus on this machine, and the claim is still
refused, naming the peer and the command that settles it:

<!-- replay peers -->

    $ cd /work/web && ank config --user --unset peers.98204934184e14045a2158a2bd0f23586745d560.api
    peers.98204934184e14045a2158a2bd0f23586745d560.api (unset)
    $ cd /work/web && ank claim 8eed
    error[7]: TASK-8eed823dc16e is blocked by TASK-e799393f5249@api: peer 'api' at ../api is not a corpus, nothing to read, and an edge that cannot be read holds
      -> ank config --user peers.98204934184e14045a2158a2bd0f23586745d560.api <path>

A missing sibling checkout never unblocks work.

## Read at the peer's default branch

A peer is read from the git objects of its default branch: the local branch
first, `refs/remotes/origin/<branch>` second. The branch its checkout is on, its
uncommitted files and its other worktrees change nothing a reader is served
(ADR-09b985a5627e). Put `api` back where the convention expects it, and accept
a second ADR there on a branch of its own:

<!-- replay peers -->

    $ mv /work/backend /work/api
    $ cd /work/api && git switch -q -c draft
    $ cd /work/api && ank new adr --title "Every response echoes the trace id" \
        --scope "web:src/**" \
        --constraint "The web client logs the X-Trace-Id each response echoes."
    created ADR-57539c2a652d Every response echoes the trace id

<!-- replay peers
$ cd /work/api && git add -A && git commit -q -m "adr: echo"
-->

<!-- replay peers -->

    $ cd /work/api && ank accept 5753
    accepted ADR-57539c2a652d -> 2292f47

<!-- replay peers
$ cd /work/api && git add -A && git commit -q -m "accept: echo"
-->

Accepted and committed, on `draft`, with `draft` checked out. In `web` it binds
nothing yet:

<!-- replay peers -->

    $ cd /work/web && ank context src/

    CONSTRAINTS (1 active)
      ADR-351131160961@api  Every request carries a trace id

    TASKS (1)
      TASK-8eed  [open] Send X-Trace-Id from the client

    > no ready tasks in scope (1 blocked)

It binds once it reaches `main`:

<!-- replay peers unordered -->

    $ cd /work/api && git switch -q main && git merge -q --no-edit draft
    $ cd /work/web && ank context src/

    CONSTRAINTS (2 active)
      ADR-57539c2a652d@api  Every response echoes the trace id
      ADR-351131160961@api  Every request carries a trace id

    TASKS (1)
      TASK-8eed  [open] Send X-Trace-Id from the client

    > no ready tasks in scope (1 blocked)

The same rule settles a `blocked_by`: a peer task releases the edge once it is
done on the peer's default branch, and done on a branch that has yet to merge
holds it. In `api`, Omar claims the task the client waits on, on a branch of
their own:

<!-- replay peers -->

    $ cd /work/api && git switch -q -c task/trace
    $ cd /work/api && ANK_AGENT=human:omar ank claim e799
    claimed TASK-e799393f5249 accept-x-trace-id-on-every-route -> HEAD

## Seeing across repositories

Claims stay per repository, so a plain `status` in `web` sees only the claims of
`web`. `--peers` adds each declared peer's live claims to `elsewhere`, addressed
`<id>@<peer>`, shown and left unarbitrated:

<!-- replay peers -->

    $ cd /work/web && ank status --peers
    branch main (default main)
    drift none, level with main
    identity human:marie (ANK_AGENT)
    no claim
    elsewhere 1 claim(s) by other agents
      TASK-e799393f5249@api Accept X-Trace-Id on every route (human:omar until 2026-11-03T10:19:30Z)
    perimeter the whole repository, 2 constraint(s)
    queue 0 proposal(s), 0 finished elsewhere
    corpus 0 fault(s), 0 signal(s), 1 owing nothing

    > ank context

`find --peers` lists each declared peer's rows after this corpus's own, read at
the peer's default branch together with its claims; `context --peers` does the
same for the ready tasks:

<!-- replay peers -->

    $ cd /work/web && ank find --peers --type task
      TASK-8eed  [open] Send X-Trace-Id from the client
      TASK-e799@api  [claimed:human:omar] Accept X-Trace-Id on every route

## The peer constraints under a claim

Omar finishes on `task/trace`:

<!-- replay peers -->

    $ cd /work/api && echo "// reads X-Trace-Id" >> src/routes.ts && git commit -qam "read X-Trace-Id"
    $ cd /work/api && ANK_AGENT=human:omar ank done --proof commit:$(git rev-parse --short HEAD)
    proof recorded: commit -> 39c7d60
    TASK-e799393f5249 -> done

<!-- replay peers
$ cd /work/api && git add -A && git commit -q -m "done TASK-e799393f5249"
-->

`task/trace` has yet to merge, so in `web` the edge still holds:

<!-- replay peers -->

    $ cd /work/web && ank claim 8eed
    error[7]: TASK-8eed823dc16e is blocked by TASK-e799393f5249@api
      -> ank show TASK-e799393f5249@api

Once it merges, the claim goes through, and it prints every constraint `api`
binds onto the claimed scope in a section of its own, naming the peer and the
default-branch commit it was read at (ADR-50952ba5e226):

<!-- replay peers unordered -->

    $ cd /work/api && git switch -q main && git merge -q --no-edit task/trace
    $ cd /work/web && ank claim 8eed
    claimed TASK-8eed823dc16e send-x-trace-id-from-the-client -> HEAD

    PEER CONSTRAINTS (api at e072acf0b34d, 2 active)
      ADR-57539c2a652d@api  The web client logs the X-Trace-Id each response echoes.
      ADR-351131160961@api  Every call the web client makes to the API sends an X-Trace-Id header.

`context` under the claim prints the same section. The claim records that
commit and the hash of each peer constraint it served, outside the frozen hash
of the criterion, and `done` states any peer constraint added, changed or
retired since, as information: it refuses nothing on that account.

## What never crosses

Reading is the whole of it. Three things stay on their own side of the
boundary:

- **Claims and writes.** A claim is a ref in one repository's `refs/ank/*`, and
  nothing arbitrates across a boundary those refs cannot reach, so claims stay
  per repository. No verb writes to a peer's corpus: not a task, not a log
  entry, not a claim (ADR-a1de673043b4). The task in `api` above is claimed and
  closed in `api`.
- **An unreadable peer as an answer.** An edge whose peer cannot be read is not
  satisfied, as shown above (ADR-c23bef1cc93e).
- **A local glob.** A scope entry with no `<peer>:` prefix is confronted with
  this repository's tree and nothing else: `src/**` in `web` never covers a
  path of `api`, however the two are laid out on disk. Only an entry that names
  a peer reaches into it, through the peer's declaration, which is what keeps a
  binding verifiable where the layout differs (ADR-da2819aef598,
  ADR-96fe1f9d619a).

What crosses is the declaration that a constraint binds a peer, read by the
peer. The ADR keeps exactly one home. The keys themselves are in
[config.yml keys](config-keys.md).
