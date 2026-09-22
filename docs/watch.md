# The watcher

`ank watch` is a background process that keeps the derived index of the corpora
you declare current, so the `ank` you run finds a cache it does not have to
rebuild. It is a verb of the same one executable every route installs
(ADR-1ea31c2f3c5a), so every installation already has it -- and running one is
still nobody's condition for anything, which is the statement below about
nothing depending on it. Everything else worth knowing about it as an
integrator is what it refuses to be (ADR-4b45f344344f).

## It keeps a cache warm, and answers nothing

**It is not a surface.** No socket, no protocol, no query of its own, and no
subset of the verbs. There is nothing here to ask: a caller that wants an answer
runs the CLI, or talks to [`ank mcp`](mcp.md). A watcher answering the three
questions a dashboard finds convenient would be the curated subset
ADR-fd98f4bc6dea refuses, reached from the other direction, and it would be a
third dispatch path in a project that has spent its history reducing to one. It
does *tell* you when a corpus it watches changes, on a stream described below,
and that is push and never pull: it says what moved, it says nothing about what
moved, and there is still nothing to connect to.

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
`watch.yml`, beside the `corpora.yml` of ADR-96174f1ac2b7 in the reader's
configuration directory ([Environment
variables](environment.md#where-the-readers-configuration-lives) says where).
It lives outside every repository, and it is keyed on the repository identity of
ADR-621a7fd96ce1 rather than on a path:

    schema: 1
    # Seconds between two mirrors of refs/ank/claims/*. Optional; 60 when omitted.
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
`index.db` and a mirror of `refs/ank/claims/*`.** The mirror lands in
`refs/ank/watch/origin/claims/*`, a tracking namespace of the watcher's own, and
carries the remote's claims alone: a mirrored proof is read by nobody, so none
is fetched (ADR-4b45f344344f). No branch, no tag, no working tree, no index of
git's, and no `refs/ank/claims`. It takes no claim, holds none on anybody's
behalf, and renews none -- a claim is renewed by working, not by reporting
(ADR-0bb7ea8991bc). A fetch that fails is a line on stderr and never an exit
code: the watcher keeps watching, and a dead network downgrades what it offers
rather than stopping it.

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

**Where it is.** `events.jsonl`, beside `watch.yml` in the same directory. One
file for every corpus the watcher was handed; each line says which corpus it is
about.

**What a line is.** One JSON object, one line, newline-terminated:

    {"schema":1,"corpus":"<root commit>","change":"entities"}
    {"schema":1,"corpus":"<root commit>","change":"refs"}

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
(ADR-4b45f344344f). What changed is on the stream; what is now true of it is what
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

## What to bind to

- **Do not bind to `ank watch`.** It answers nothing, and it is optional by
  construction. Write your integration against the CLI or the protocol surface,
  and let the watcher make those answers arrive sooner where somebody chose to
  run one.
- **You may bind to `events.jsonl`**, and it is a narrow license: it tells you a
  corpus changed so you can stop asking on a timer. Every answer still comes from
  the CLI, and your integration has to work with no stream at all, because most
  installations have none.
