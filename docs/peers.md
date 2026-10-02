# Several repositories

One `.ank/` per repository stays authoritative: a corpus belongs to the code it
constrains. Several repositories still share decisions, a backend and the client
that calls it being the usual pair, and ank lets one corpus read another without
ever writing to it (ADR-a1de673043b4). This page walks through the whole of it
with two repositories side by side, `/work/api` and `/work/web`, in the order
you meet it: a declaration at both ends, the override for a layout that differs,
naming an entity of a peer, and what never crosses.

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

That is one end. Inside `web`, nothing binds yet, because `web` has not said it
reads `api`, and a corpus is never discovered from the filesystem:

<!-- replay peers -->

    $ cd /work/web && ank context src/

    > no ready tasks in scope

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

## When the committed path is wrong here

<!-- replay peers
$ cd /work/web && git add -A && git commit -q -m "peers: api"
-->

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
