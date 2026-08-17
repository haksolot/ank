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
