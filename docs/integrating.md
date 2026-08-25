# Integrating with ank

For someone writing a tool that reads or drives an ank corpus, who has never
seen this repository.

[getting-started.md](getting-started.md) already covers the **pipeline** case,
and covers it well: run `ank check`, route on the exit code, parse `--json`. If
that is what you are building, read the "Running ank in a pipeline" section there
and stop. This document is for what a pipeline does not need: a board, an
editor plugin, a dashboard, an agent harness, anything that reads a corpus and
shows it to somebody.

Four things cost such a reader real time to discover, and all four are below.

## The entry point is `ank help --json`

Not this document, and not the source. The surface describes itself, and the
description is generated from the same table the binary dispatches from, so it
cannot fall behind what the binary does.

    $ ank help --json | cut -c1-64
    {"contract":1,"verbs":[{"name":"context","usage":"ank context [<

One verb, whole, is the shape of every entry:

    $ ank help close --json
    {"contract":1,"verbs":[{"name":"close","usage":"ank close <id>","summary":"closes a task that will never be done; --reason is mandatory","group":"shape the work","flags":[{"name":"--reason","short":null,"takes_value":true,"repeatable":false},{"name":"--json","short":"-j","takes_value":false,"repeatable":false},{"name":"--quiet","short":"-q","takes_value":false,"repeatable":false},{"name":"--repo","short":"-r","takes_value":true,"repeatable":false}],"notes":[],"refuses":[{"code":7,"when":"no --reason: a closure nobody explained is one nobody can reopen"},{"code":2,"when":"no such entity, or the prefix matches more than one"}],"returns":[{"when":null,"fields":[{"name":"contract","type":"number","nullable":false},{"name":"task","type":"string","nullable":false},{"name":"status","type":"string","nullable":false},{"name":"claim_revoked","type":"boolean","nullable":false}]}]}]}

So a client can discover, without reading a line of Rust: every verb, its flags
and their short forms, the states it refuses on **with the code each returns**,
and the fields of the document that comes back.

**`returns` is a list, because a verb may answer two questions.** `config <key>`
reads and `config <key> <value>` writes; `log <id>` reads and `log <id> <message>`
appends; `show` over a task carries the `blocked_by` edges and over an ADR does
not, since a document carrying them empty would be answering a question nobody
asked. Each shape names the call that returns it in its `when`, which is `null`
where the verb has only one.

**`returns` is flat, with the path in the name.** A nested field appears as
`tasks` followed by `tasks.id` and `tasks.title`, in the order the document emits
them. No key in any document contains a dot, so a client that wants the tree
splits on one character. The reason it is flat rather than nested is that this
document describes its own output too, and a description that recursed into
itself would not terminate.

The type vocabulary is six words: `string`, `number`, `boolean`, `string[]`,
`object`, `object[]`. `nullable` is separate from the type and is load-bearing:
`null` and `""` are different answers, and a client that treats a nullable string
as a string breaks on the first detached HEAD it meets.

## Every document carries the contract version

    "contract": 1

It leads every `--json` document, and it is the field to read before deciding you
can read the rest.

Within one version a document may **gain** a field, and may never lose, rename or
retype one. So parse leniently, because an unknown field is not a breaking change
and your parser must not refuse one, and treat a change of this number as the
signal to look again.

It is not the version of the binary. `ank --version` says which build is in hand;
this says which shapes came out of it, and a release that changes no document
leaves it untouched.

## The exit codes

The semantics are in the code so a caller can route without parsing output. They
are stable, and `ank help --json` publishes which verb returns which.

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | generic error |
| 2 | entity not found, or an ambiguous prefix |
| 3 | version conflict, re-read and retry |
| 4 | the task is unavailable: held by another agent, or finished on another branch |
| 5 | a proof is missing or invalid |
| 6 | the act is illegal from the state the entity is in, or a frozen field diverged |
| 7 | a prerequisite is missing |
| 8 | `check` or `review` found something |
| 9 | the environment, not the corpus |

Two of them are the ones a loop must handle. **3** means "somebody moved, read
again". **4** means "take something else".

**6 and 7 are two codes on purpose.** In 6 the state forbids what you asked; in 7
the thing you asked for is legal and something it depends on is absent. `accept`
off the default branch is a 7, because the promotion is legal and the place is
not. A client that conflates them reacts wrongly to one of the two.

**9 is not a failure of the work.** git absent or too old, `sh` missing,
`$EDITOR` unset, a default branch that cannot be determined. Collapsing it into
"the command failed" sends somebody to fix sound code.

Every refusal names the exact command to run next, on stderr, and that is
stable too:

    $ ank show TASK-9999
    error[2]: entity not found: TASK-9999
      -> ank find TASK-9999

    $ ank claim TASK-a0a7
    error[4]: TASK-a0a7a75fbb9c held by tool/1.0 (expires in 30m)
      -> ank context

## A task's state is not in its file

This is the one that costs the most time, because a tool that gets it wrong
under-reports **silently**.

The file is the entity. The *state* is the file together with three things that
are not in it:

- `refs/ank/claims/<id>`, who holds the task and until when. A claim lives in a
  git ref and never in the file, so two clones arbitrate through the remote
  rather than through a field somebody has to merge.
- `refs/ank/proof/<id>`, proofs a pipeline attested without making a commit.
- the **log entities** whose `about` names the task, one file per entry, stored
  beside the entities and not inside them:

      $ cat .ank/entities/LOG-bc49fad834f1.md
      ---
      id: LOG-bc49fad834f1
      type: log
      title: the layout is not the contract
      created: 2026-08-17T19:09:30Z
      author: tool/1.0
      scope:
        - src/**
      about: TASK-a0a7a75fbb9c
      seq: 0
      schema: 3
      version: 1
      ---

Read the file alone and here is what you see:

    $ cat .ank/entities/TASK-a0a7a75fbb9c.md
    ---
    id: TASK-a0a7a75fbb9c
    type: task
    slug: the-parser-reads-a-corpus-without-opening-a-file
    title: The parser reads a corpus without opening a file
    created: 2026-08-17T19:09:19Z
    author: tool/1.0
    status: in_progress
    scope:
      - src/**
    blocked_by: []
    done_criteria: |
      A caller reads every entity through the CLI.
    criteria_by: creator
    schema: 3
    version: 2
    ---

`in_progress`, and not one word about who is holding it, when the lease expires,
or what they have learned. Ask the CLI instead and the same task answers whole:

    $ ank show TASK-a0a7 --json
    {"contract":1,"id":"TASK-a0a7a75fbb9c","coordination":"claimed by tool/1.0","blocked_by":[],"unblocks":[],"detached_proofs":[],"log_total":1,"log_shown":1,"log":[{"id":"LOG-bc49fad834f1","timestamp":"2026-08-17T19:09:30Z","who":"tool/1.0","message":"the layout is not the contract"}],"content":"---\nid: TASK-a0a7a75fbb9c\ntype: task\n…"}

`coordination` came from the ref. `log` came from the log entities. `content` is
the file, byte for byte, so nothing is lost by going through the verb.

**So read through the CLI, not through the directory.** Not as a matter of taste:
a reader that walks `.ank/` is reading one of the three sources and will report a
held task as free.

If you do read the files, whether from a viewer with no binary to call or a parser
in another language, then read all three, and read the refs correctly: most of them are in
`.git/packed-refs` rather than under `.git/refs/`, and a reader that walks only
the loose ones finds almost none of them.

## `ank check` writes. Do not poll it

It prunes the claim refs it finds stale: orphans, and completion refs whose task
is `done` or `closed` on the default branch. The binary says so itself:

    $ ank help check
    ank check [<path>]
      the mechanical invariants: parse, round-trip, references, frozen fields, orphaned claims; prunes the claim refs it finds stale, so it writes
      global:   -j, --json -q, --quiet -r, --repo <v>
      note:     exit 8 means findings; a signal alone leaves it 0
                the only verb that prunes refs/ank/claims: orphans, and completion refs whose task is done or closed on the default branch

A dashboard refreshing every thirty seconds must not call it. `ank status`,
`ank find` and `ank show` are what a poll uses; `check` is the verb a human or a
pipeline runs deliberately.

**Two planes, and only one of them is precious.** What `check` prunes is the
**coordination** plane, the refs that say who holds what, and losing a ref
there loses a fact nothing else carries. Separately, every verb that reads the
corpus may write a **disposable** one: a SQLite index beside the files, which
stores a content hash per file and reindexes whatever diverged when it is opened.
That is why an entity edited by hand or arrived through `git checkout` shows up
on the next read with no reindex command to forget. Deleting that index is always
safe, and it is never the source of truth. But it does mean a "read" verb
touches the disk, which is worth knowing before you point twenty pollers at one
working tree.

It is also one of the two verbs that walk git history, `review` being the other
and sharing the same inspection, to say where a dead scope went. That makes both
of them slower than a read, and it is a second reason not to put either on a
timer. Only `check` prunes, so only `check` writes; but neither is a poll.

Exit 8 is findings, meaning faults. Signals leave it 0, and that is deliberate:
reddening a build over an observation teaches a team to stop reading `check`.

    $ ank check --json
    {"contract":1,"faults":0,"signals":4,"tasks":1,"adr":1,"pruned":[],"findings":[{"level":"signal","subject":"ADR-4bed8b557e89","message":"written by an agent and read by no human","note":[],"charge":[]},{"level":"signal","subject":"TASK-a0a7a75fbb9c","message":"written by an agent and read by no human","note":[],"charge":[]},{"level":"signal","subject":"allowed_signers","message":"no ratification key declared: permissions are advisory, not enforced (§8)","note":[],"charge":[]},{"level":"signal","subject":"coordination","message":"default branch indeterminable, completion refs neither pruned nor judged (ank config default_branch <name>)","note":[],"charge":[]}]}

## The conformance suite is offered to you

Two sets of fixtures in this repository are yours to reuse. The first says so in
its own header ("any third-party tool that claims to read or write the format
can reuse the `tests/golden/` directory") and the second is offered here, which
is the only place it is said:

- **`crates/ank-core/tests/golden/`**: the file format. Valid files that must
  round-trip byte for byte in canonical form, and invalid ones with the error
  each must produce. If you are writing a parser in another language, this is
  what tells you it is right, and [format.md](format.md) is what it is checking
  against.
- **`crates/ank-cli/tests/golden-json/`**: the machine surface. One fixture per
  document the CLI returns, captured from the process rather than from a
  function, so what they pin is what leaves the binary. If you are writing a
  client, these are the exact bytes to write it against.

Both are plain files in a public repository. Copy them into your own suite; a
shape that changes here without its fixture changing is a failing test on our
side, which is what makes them worth copying.

## The protocol surface is the same verbs, over MCP

A client with no shell reaches ank through `ank mcp`, a verb of the one
executable every route installs (ADR-1ea31c2f3c5a). There is no second file to
fetch, sign or discover: what the CLI dispatches is what the surface serves,
because they are the same file.

The configuration is `command` naming the binary and `mcp` as its first
argument, and it is pasted rather than derived:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

`--repo` is written out because a client spawns the server in whatever
directory it happens to be in, and with no `--repo` the server takes that
directory -- which is a process quietly speaking for a corpus nobody meant, or
for none, rather than an error anyone sees.
[getting-started.md](getting-started.md) carries the same block per client, for
somebody installing rather than integrating. If you hold a configuration
written against a second executable named `ank-mcp`, releases up to 0.6.0
placed one and no route places one any more: the change is that one line.

What the surface *is* belongs here, because four properties of it are
load-bearing and none of them is visible from a tool list.

**Every verb `COMMANDS` carries, generated from that table.** Not a curated
subset, under any protocol (ADR-fd98f4bc6dea). It is the same table `ank help
--json` is generated from, walked: the summary becomes the tool description, the
refusals and their exit codes are written into it so a client can read what a
call will refuse before making it, and the flags become the input schema. One
tool per verb, whatever the table carries, named `ank_<verb>` because a bare
`context` collides with every other server a client has loaded and `ank context`
is not a legal tool name. Positionals arrive as `arguments`, an array of strings,
exactly as they sit on the command line; flags arrive under their own names with
the leading dashes stripped. Nothing in the server names a verb, so the two
surfaces cannot disagree about what exists.

**One process is addressed with one corpus, at startup.** `--repo` is resolved
once, there, and that value is what a call naming no corpus of its own is given,
so a server cannot drift between corpora while a client holds a claim in one. A
call that passes `--repo`, `--json` or `--quiet` is refused by name rather than
being allowed to contradict the process it is talking to:

    {"jsonrpc":"2.0","id":3,"error":{"code":-32602,"message":"--repo belongs to the server: name a corpus with the corpus argument, by the identity ank status --json prints, never by a path"}}

Nothing is hidden by that and nothing is curated: every verb takes exactly the
arguments the table gives it, and what is withheld is three flags that are the
server's own. A deployment over several repositories is several servers,
addressed separately, presented together by whatever sits above them. It is the
answer `refs/ank/*` forces, and the same one federation gets (ADR-a1de673043b4).

**A refusal is the CLI's refusal, and it carries the CLI's exit code.** The
surface spawns `ank`; it does not link it. So a refusal on state is not
re-derived here, it is inherited, hint and all, and it comes back as a result
rather than as a protocol error, because the request was well formed and the
answer is no:

    {"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"error[2]: entity not found: TASK-9999\n  -> ank find TASK-9999"}],"isError":true,"exitCode":2,"stderr":"error[2]: entity not found: TASK-9999\n  -> ank find TASK-9999"}}

`exitCode` is present on every call including a successful one, so a client that
branches on it never has to tell absence from zero; `stderr` is carried
separately for the reason warnings live there in the first place. The two error
channels stay apart: a JSON-RPC error means the *request* was wrong, a result
with `isError` means the *corpus* said no. A client that conflates them reports
its own bug as a state of your repository.

**No claim is taken that the CLI would not have taken in that clone.** Every
claim goes to `refs/ank/claims/<id>` in that repository, arbitrated by the same
compare-and-swap against the same remote. The server holds no claim on a
client's behalf, renews none for anybody, and pools no clients under one
identity: one stdio server serves one client, so one process is one caller. It
writes under a typed process identity, `ank-mcp/<version>`, unless `$ANK_AGENT`
names one, so a deployment that already names its agents keeps naming them.

`accept` is a tool here like every other verb, because a generated surface
curates nothing out. Being reachable over a protocol changes nothing about it:
it still refuses off the default branch, with no way around it.

## The watcher keeps a cache warm, and answers nothing

`ank watch` is a background process that keeps the derived index of the corpora
you declare current, so the `ank` you run finds a cache it does not have to
rebuild. It is a verb of the same one executable every route installs
(ADR-1ea31c2f3c5a), so every installation already has it -- and running one is
still nobody's condition for anything, which is the statement below about
nothing depending on it. Everything else worth knowing about it as an
integrator is what it refuses to be (ADR-a22cd3196529).

**It is not a surface.** No socket, no protocol, no query of its own, and no
subset of the verbs. There is nothing here to ask: a caller that wants an answer
runs the CLI, or talks to `ank mcp`. A watcher answering the three questions a
dashboard finds convenient would be the curated subset ADR-fd98f4bc6dea refuses,
reached from the other direction, and it would be a third dispatch path in a
project that has spent its history reducing to one. It does *tell* you when a
corpus it watches changes, on a stream described below, and that is push and
never pull: it says what moved, it says nothing about what moved, and there is
still nothing to connect to.

**Nothing depends on it.** Every verb gives the same output and the same exit
code with it stopped; its absence is never an error, and no installation route
makes running it a condition of using ank. The installation without a watcher is
the one every CI runner, every container and every agent has, so it is the
normal one, made slower rather than made lesser. Stopping it is always safe, and
`stopping_the_daemon_changes_no_verbs_output_and_no_verbs_exit_code` in its
suite is what keeps that true.

**Nothing it serves is believed over the files.** The index is a cache the CLI
rebuilds from a content hash per `.ank/` file at read time, so a listing off a
warm index and a listing off no index are the same bytes. The watcher does not
compute that listing and holds no copy of it: it spawns `ank` and asks for a
read, which is what leaves the index current. It is a cache warmer, so a poll it
misses costs latency and never correctness.

**It watches what you declared, and looks for nothing.** The declaration is
`watch.yml`, beside the `corpora.yml` of ADR-96174f1ac2b7 and under the same
directory rule -- `%APPDATA%\ank` on Windows, `$XDG_CONFIG_HOME/ank` elsewhere,
falling back to `$HOME/.config/ank`. It lives outside every repository, and it
is keyed on the repository identity of ADR-621a7fd96ce1 rather than on a path:

    schema: 1
    # Seconds between two mirrors of refs/ank/*. Optional; 60 when omitted.
    fetch: 60
    watch:
      # The key is the root commit, which `ank status --json` prints under
      # "corpus". One checkout, or a list of them.
      4f0b8c2d1e6a39572c84ab0d6f31e75c9a2b48d0: /home/me/work/ank
      9c31ea77b04d5f2681ac3e095b7d4f60a8213ce5: /home/me/work/other

Two worktrees of one repository are two paths under one key, and therefore one
watched corpus -- which is the whole reason the key is not the path. A key that
is not a root commit is refused by name, a checkout filed under another
repository's identity is refused with both identities, and a directory carrying
no `.ank/` is refused rather than searched around: `ank watch --list` prints
what would be watched without watching anything, and `ank watch --where` prints
where the declaration is read from.

**The only things it writes into a repository are that repository's own
`index.db` and a mirror of `refs/ank/*`.** The mirror lands in
`refs/ank/watch/origin/*`, a tracking namespace of the watcher's own: no branch,
no tag, no working tree, no index of git's, and no `refs/ank/claims`. It takes
no claim, holds none on anybody's behalf, and renews none -- a claim is renewed
by working, not by reporting (ADR-0bb7ea8991bc). A fetch that fails is a line on
stderr and never an exit code: the watcher keeps watching, and a dead network
downgrades what it offers rather than stopping it.

**What the mirror buys is one line of `ank status`.** `refs/ank/claims/*` in a
clone is whatever somebody last fetched by hand, so on a parc of clones the
`elsewhere` section reports who held what an hour ago and has no way to say so.
`status` reads the mirror beside its own plane and reports both as one list,
with the local record winning wherever they carry the same task. No other verb
reads it, and none may: an installation with a watcher and one without have to
be one product. That is asserted rather than promised, in
`a_claim_a_watcher_mirrored_is_reported_by_status_and_by_nothing_else`, which
compares every listing verb byte for byte with the mirror present and absent.

## A change becomes an event, and the stream is yours to follow

The watcher appends a line when a corpus it watches changes, and any program may
follow it. That is the one thing it offers a consumer, and it is offered as a
file rather than as a connection: there is nothing to bind to, nothing to
negotiate, and nothing you can ask it. Several readers follow the same bytes
without the watcher knowing any of them exist.

**Where it is.** `events.jsonl`, beside the `watch.yml` above and under the same
directory rule -- `%APPDATA%\ank` on Windows, `$XDG_CONFIG_HOME/ank` elsewhere,
falling back to `$HOME/.config/ank`. One file for every corpus the watcher was
handed; each line says which corpus it is about.

**What a line is.** One JSON object, one line, newline-terminated:

    {"schema":1,"corpus":"4f0b8c2d1e6a39572c84ab0d6f31e75c9a2b48d0","change":"entities"}
    {"schema":1,"corpus":"4f0b8c2d1e6a39572c84ab0d6f31e75c9a2b48d0","change":"refs"}

- `schema` is the shape of the line, and it is **not** the contract version that
  `--json` documents carry: the two move for different reasons. Within a schema a
  line may gain a field and may never lose, rename or retype one. Skip a line
  whose schema you do not know rather than guessing at it.
- `corpus` is the repository identity of the watched corpus -- the root commit,
  which `ank status --json` prints under `"corpus"`. Never a path, and no path is
  carried beside it: a corpus reached by two paths is one corpus, and a field
  naming one would be an invitation to key on it. Two checkouts of one corpus
  changing produce two lines carrying the same identity, and the answer to both
  is the same one read.
- `change` says what moved. `entities` is "a file under that corpus's `.ank/`
  was written, added or removed"; `refs` is "the watcher's mirror of the remote's
  `refs/ank/*` moved", which is how a claim taken in a clone you cannot see
  reaches you. The vocabulary is closed at those two today and may gain a word.

**What a line is not.** It carries no title, no status, no body, no identifier
and no entity content of any kind, and it never will: an event that carried the
new state of a task would save you a call and would make the watcher a source of
corpus data that nothing generated from the verb table ever validated
(ADR-a22cd3196529). What changed is on the stream; what is now true of it is what
the CLI answers, and `no_event_carries_entity_content_a_reader_would_get_from_the_cli`
asserts the absence rather than promising it. An event also never says what to do
about itself. There is one sensible thing to do, which is to read the corpus
again, and the stream does not presume to say so.

**How to follow it.** Open the file, remember the offset you have read to, and
read the bytes past it whenever you like. Three rules and they are the whole
protocol:

- Consume **whole lines only**. The watcher writes one line per call, but a
  reader that took a half-written one would repaint on a corpus it could not
  name.
- If the file is **shorter than your offset**, the watcher started it over and
  you read from the beginning again. The stream is news and not a log: nothing is
  anchored in it, nothing hashes over it, so it is bounded rather than kept, and
  what you missed while you were not running is missed whatever the bound is.
- If the file is **not there**, no watcher has ever run for this reader. That is
  not an error and not a degraded mode: read the corpus when your person asks, as
  every installation without a watcher does. If it appears later, follow it from
  its beginning.

**What it does not license.** Following the stream is not a second way into the
corpus, and it must not become one. `ank tui` follows it and still reaches every
byte it shows by running the CLI with `--json`, because the event says a corpus
moved and nothing more. And an event is a repaint, never a write: the reader
answers one by running `status` and `find`, and deliberately not `show`, which
renews the lease when the id is the task the caller holds (ADR-0bb7ea8991bc). A
screen nobody is sitting at is told the corpus changed all night and keeps
nobody's claim alive; `an_event_repaints_the_list_and_renews_no_claim` is what
holds that true.

## What binds and what does not

- **Bind to `--json`**, never to the human output. One line, stdout only, never
  coloured, and warnings go to stderr precisely so your parser keeps reading what
  it already read.
- **Bind to the exit code**, never to the wording of an error. The message and
  the hint are written for a person to read and may be improved; the code is
  the contract.
- **Bind to `ank help --json`** for what a verb accepts and returns, rather than
  to a list you maintain. A list maintained by hand is a list that will disagree,
  and the disagreement surfaces on your side, days later, as a bug you cannot see
  from there.
- **`--repo <path>` addresses a corpus**, so a tool holding several addresses
  each on its own. Claims are per repository: nothing merges the claim spaces of
  two clones, because `refs/ank/*` cannot carry such an arbitration.
- **Do not bind to `ank watch`.** It answers nothing, and it is optional by
  construction. Write your integration against the CLI or the protocol surface,
  and let the watcher make those answers arrive sooner where somebody chose to
  run one.
- **You may bind to `events.jsonl`**, which is the one exception and a narrow
  one: it tells you a corpus changed so you can stop asking on a timer. Every
  answer still comes from the CLI, and your integration has to work with no
  stream at all, because most installations have none.
