# The machine surface

For someone writing a tool that reads or drives an ank corpus, who has never
seen this repository: a board, an editor plugin, a dashboard, an agent harness,
anything that reads a corpus and shows it to somebody. A pipeline needs less
than this, and [Running ank in CI](ci.md) is the whole of it. A client with no
shell reaches the same verbs through [the MCP server](mcp.md), and [the
watcher](watch.md) is the optional process that tells a reader a corpus moved.

What costs such a reader real time to discover is below.

## The entry point is `ank help --json`

Not this document, and not the source. The surface describes itself, and the
description is generated from the same table the binary dispatches from, so it
cannot fall behind what the binary does.

<!-- replay help -->

    $ ank help --json | cut -c1-64
    {"contract":1,"verbs":[{"name":"context","usage":"ank context [<

One verb, whole, is the shape of every entry:

<!-- replay help -->

    $ ank help close --json
    {"contract":1,"verbs":[{"name":"close","usage":"ank close <id>","summary":"closes a task that will never be done; --reason is mandatory","group":"shape the work","flags":[{"name":"--reason","short":null,"takes_value":true,"repeatable":false},{"name":"--json","short":"-j","takes_value":false,"repeatable":false},{"name":"--quiet","short":"-q","takes_value":false,"repeatable":false},{"name":"--repo","short":"-r","takes_value":true,"repeatable":false},{"name":"--worktree","short":null,"takes_value":true,"repeatable":false}],"notes":["the ref is not the whole product: a push the remote refuses leaves the write standing in this clone, and the verb exits 0"],"refuses":[{"code":7,"when":"no --reason: a closure nobody explained is one nobody can reopen"},{"code":2,"when":"no such entity, or the prefix matches more than one"},{"code":1,"when":"a flag this verb does not take, or a value the parser cannot read"},{"code":3,"when":"the entity moved between the read and the write: redo context, somebody else wrote"},{"code":9,"when":"git is absent or older than 2.34: an environment to repair, not work that failed"},{"code":6,"when":"the task is already closed or already done: neither is a state this verb moves out of"}],"returns":[{"when":null,"fields":[{"name":"contract","type":"number","nullable":false},{"name":"task","type":"string","nullable":false},{"name":"status","type":"string","nullable":false},{"name":"claim_revoked","type":"boolean","nullable":false}]}]}]}

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
are stable, and `ank help --json` publishes which verb returns which. The table
is [the exit-code reference](exit-codes.md), generated from the enum that
declares them.

Two of them are the ones a loop must handle. **3** means "somebody moved, read
again". **4** means "take something else".

**6 and 7 are two codes on purpose.** In 6 the state forbids what you asked; in 7
the thing you asked for is legal and something it depends on is absent. `accept`
off the default branch is a 7, because the promotion is legal and the place is
not. A client that conflates them reacts wrongly to one of the two.

**9 is not a failure of the work.** git absent or too old, `sh` missing,
`$EDITOR` unset, a default branch that cannot be determined. Collapsing it into
"the command failed" sends somebody to fix sound code.

**1 has no reaction of its own to prescribe.** It is what a mistyped command
and an unparseable file both get, and a script that routes on it is guessing.

Every refusal names the exact command to run next, on stderr, and that is
stable too:

<!-- replay state
$ ank init
$ mkdir src && echo 'fn main() {}' > src/main.rs
$ ANK_AGENT=tool/1.0 ank new adr --title "Readers go through the CLI" --scope "src/**" --constraint "Read the corpus through the CLI, never through .ank/."
created ADR-57715ae64348 Readers go through the CLI
$ ANK_AGENT=tool/1.0 ank new task --title "The parser reads a corpus without opening a file" --scope "src/**" --criteria "The parser reads every file." --no-verify
created TASK-6da126c832be The parser reads a corpus without opening a file
$ ANK_AGENT=tool/1.0 ank amend TASK-6da1 --criteria "A caller reads every entity through the CLI."
amended TASK-6da126c832be done_criteria
$ ANK_AGENT=tool/1.0 ank claim TASK-6da1
claimed TASK-6da126c832be the-parser-reads-a-corpus-without-opening-a-file -> HEAD
$ ANK_AGENT=tool/1.0 ank log "the layout is not the contract"
logged LOG-c0f96bc669ae on TASK-6da126c832be
$ grep -l "^records: edit" .ank/entities/*.md
.ank/entities/LOG-e6c24bc5f3e2.md
-->

    $ ank show TASK-9999
    error[2]: entity not found: TASK-9999
      -> ank find TASK-9999

    $ ank claim TASK-6da1
    error[4]: TASK-6da126c832be held by tool/1.0 (expires in 30m)
      -> ank context

## A refusal leaves stdout empty, and a warning may not

Under `--json` a refusal writes **nothing at all** to stdout. Not an error
document, not an empty object: zero bytes. The message and its hint go to
stderr, the code goes to the exit status, and that is the whole answer.
Measured across four codes -- `show TASK-9999` (2), `done` with no claim held
(6), `close` with no `--reason` (7), `accept` off a default branch (9) -- stdout
was 0 bytes every time. So parse stdout only once the code says 0; a parse error
on a refusal is a client reading the wrong stream.

**A warning is the other case, and it does not all go to one stream.** There are
two kinds and the split is deliberate.

**The warnings about the corpus are in the document**, under a `warnings` array
of strings, with stderr left empty. Four verbs carry the field -- `context`,
`claim`, `log` in its appending form, and `release` -- and `ank help --json` is
where to read which, rather than this list:

<!-- replay overlap
$ ank init
$ mkdir src && echo 'fn main() {}' > src/main.rs
$ ank new task --title "One" --scope "src/**" --criteria "c" --no-verify
created TASK-efd813eedb23 One
$ ank new task --title "Two" --scope "src/**" --criteria "c" --no-verify
created TASK-0e6148ab8b03 Two
$ ANK_AGENT=tool/1.0 ank claim TASK-efd8
claimed TASK-efd813eedb23 one -> HEAD
-->

    $ ANK_AGENT=tool/2.0 ank claim TASK-0e61 --json
    {"contract":1,"task":"TASK-0e6148ab8b03","holder":"tool/2.0","expires":"2026-09-20T18:15:44Z","warnings":["tool/1.0 holds TASK-efd813eedb23, overlapping on src/**"]}

An intersecting claim is named and never refused (ADR-052accd6e3b2), so the fact
has to reach a caller somewhere it will be read, and under `--json` that is the
document rather than a stream a parser was told to ignore. The array is present
and empty when there is nothing to say, so a client reads it unconditionally.

**The warnings about the refs are on stderr, in both modes**, because they are
not the answer: a write whose ref did not reach the remote leaves the document
and the exit code exactly as they would have been. `done`, `release` and `close`
each owe one. Against an unreachable remote, with stderr sent to a file of its
own, `done` put the document on stdout and exited 0:

<!-- replay unreachable
$ ank init && ank config default_branch main
$ git add -A && git commit -q -m corpus && git remote add origin ../nowhere.git
$ git rev-parse --short HEAD
8db4465
$ ank new task --title "t" --scope "**" --criteria "c" --no-verify
created TASK-277368641a6e t
$ ank claim TASK-2773
-->

    $ ank done --proof commit:8db4465 --json 2>stderr.txt; echo "exit $?"
    {"contract":1,"task":"TASK-277368641a6e","status":"done","commit":"8db44652564828e480ea7e5be3768b14f9c03893","branch":"main","proofs":1}
    exit 0

and the warning on stderr:

<!-- replay unreachable -->

    $ cat stderr.txt
    warning: claim not pushed: it holds in this clone only, and another clone can take the same task

So read stderr, and do not assume it is empty on success -- but do not look
there for what the document already carries.

## The global flags

`ank help --json` carries every flag of every verb, so none of this is a list to
maintain by hand. Four flags are on nearly every verb, and what each one does is
worth stating once.

**`--json`, short `-j`**, on all 29 verbs. One line on stdout, never coloured.

**`--quiet`, short `-q`**, on all 29 verbs, and it *empties* stdout rather than
shortening it: `ank check` printed 340 bytes on one corpus, `ank check --quiet`
printed 0, and both exited 0. What is left is the exit code, which is the point
-- a caller that only routes on the code pays for no output at all. It does not
silence a refusal: `ank show TASK-9999 --quiet` still puts `error[2]` on stderr
and still exits 2. `--json` wins over it, so `--quiet --json` is still a
document.

**`--repo <path>`, short `-r`**, on 27 verbs: which corpus. `init` refuses it by
name, and `watch` takes its corpora from [its own declaration](watch.md).

**`--worktree <path>`, no short form**, on every verb but `watch`: which *tree*
that corpus is anchored to. `--repo` says where `.ank/` is; `--worktree` says
where a scope glob is confronted, where a path argument resolves, where a
verifier runs, and where a `commit:` proof is looked up (ADR-9e56318631f3). The
two are equal unless you separate them, and separating them shows: one corpus
whose single task is scoped `src/**`, checked against a tree that has `src/`,
reported 3 signals; checked against a tree that does not, 4, the extra one being
`scope 'src/**' matches no file yet`. That is the flag a tool wants when one
`.ank/` sits above several checkouts. A path that is not a directory is refused
at exit 1, naming the confusion the refusal exists for:

<!-- replay state -->

    $ ank status --worktree /nope/nope
    error[1]: --worktree /nope/nope is not a directory
      -> --worktree names the tree the corpus is anchored to, not its corpus

The remaining short forms belong to one verb or two and are read from `ank help
<verb> --json` rather than from here: `-c` for `--criteria`, `-b` for
`--blocked-by`, `-v` for `--verify`, `-p` for `--proof`, `-t` and `-s` for
`find`'s `--type` and `--status`, `-l` for `context --limit`, `-u` for `config
--unset`.

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

  <!-- replay state -->

      $ cat .ank/entities/LOG-c0f96bc669ae.md
      ---
      id: LOG-c0f96bc669ae
      type: log
      title: the layout is not the contract
      created: 2026-08-26T00:22:04Z
      author: tool/1.0
      scope:
        - src/**
      about: TASK-6da126c832be
      seq: 2
      schema: 4
      version: 1
      ---

  An entry carrying `records` is **machinery** rather than work: written by a
  verb that changed the entity's content, not by the agent holding it. This
  task carries two: the `create` record `new` wrote at its birth, and an `edit`,
  because the criterion in the file below was amended before a claim froze it.
  The edit's message is the whole of that accounting:

  <!-- replay state -->

      $ cat .ank/entities/LOG-e6c24bc5f3e2.md
      ---
      id: LOG-e6c24bc5f3e2
      type: log
      title: done_criteria (version 1 to 2, replaced b1f3aa97873c, produced 83947c872580)
      created: 2026-08-26T00:22:04Z
      author: tool/1.0
      scope:
        - src/**
      about: TASK-6da126c832be
      seq: 1
      records: edit
      schema: 4
      version: 1
      ---

Read the file alone and here is what you see:

<!-- replay state -->

    $ cat .ank/entities/TASK-6da126c832be.md
    ---
    id: TASK-6da126c832be
    type: task
    slug: the-parser-reads-a-corpus-without-opening-a-file
    title: The parser reads a corpus without opening a file
    created: 2026-08-26T00:22:04Z
    author: tool/1.0
    status: in_progress
    scope:
      - src/**
    blocked_by: []
    done_criteria: |
      A caller reads every entity through the CLI.
    criteria_by: creator
    schema: 4
    version: 3
    ---

`in_progress`, `version: 3`, and not one word about who is holding it, when the
lease expires, what they have learned, or what the two versions before this one
were. Ask the CLI instead and the same task answers whole:

<!-- replay state -->

    $ ank show TASK-6da1 --json
    {"contract":1,"id":"TASK-6da126c832be","coordination":"claimed by tool/1.0","blocked_by":[],"unblocks":[],"detached_proofs":[],"log_total":1,"log_shown":1,"log":[{"id":"LOG-c0f96bc669ae","timestamp":"2026-08-26T00:22:04Z","who":"tool/1.0","message":"the layout is not the contract","records":null}],"machinery":[{"id":"LOG-3a51d0c2b7e4","timestamp":"2026-08-26T00:22:04Z","who":"tool/1.0","message":"created (version 0 to 1, produced 5b0e7c93d1a2)","records":"create"},{"id":"LOG-e6c24bc5f3e2","timestamp":"2026-08-26T00:22:04Z","who":"tool/1.0","message":"done_criteria (version 1 to 2, replaced b1f3aa97873c, produced 83947c872580)","records":"edit"}],"content":"---\nid: TASK-6da126c832be\ntype: task\nslug: the-parser-reads-a-corpus-without-opening-a-file\ntitle: The parser reads a corpus without opening a file\ncreated: 2026-08-26T00:22:04Z\nauthor: tool/1.0\nstatus: in_progress\nscope:\n  - src/**\nblocked_by: []\ndone_criteria: |\n  A caller reads every entity through the CLI.\ncriteria_by: creator\nschema: 4\nversion: 3\n---\n"}

`coordination` came from the ref. `log` and `machinery` came from the log
entities, split on `records`: the work trace is what a holder wrote and is what
the budget is spent on, the machinery is what the verbs wrote and is listed
under it, so a task edited eight times does not answer "what did the last holder
learn" with eight mechanical lines. `records` is `null` on a work entry, and a
client reading `log` alone still sees the field. `log_total` and `log_shown`
count the work trace and never the machinery. `content` is the file, byte for
byte, so nothing is lost by going through the verb.

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

<!-- replay help -->

    $ ank help check
    ank check [<path>]
      the mechanical invariants: parse, round-trip, references, frozen fields, orphaned claims; prunes the claim refs it finds stale, so it writes
      global:   -j, --json -q, --quiet -r, --repo <v> --worktree <v>
      note:     exit 8 means findings; a signal alone leaves it 0
                the only verb that prunes refs/ank/claims: orphans, and completion refs whose task is done or closed on the default branch
      refuses:  the path names nothing inside this repository (1)
                the corpus carries at least one fault; a signal alone leaves the code at 0 (8)

A dashboard refreshing every thirty seconds must not call it. `ank status` and
`ank find` are what a poll uses; `check` is the verb a human or a pipeline runs
deliberately.

**`ank show` is not a poll either, and for a different reason: it renews a
claim** when its subject is the task the caller holds, and so does `context`.
A screen nobody is sitting at would keep an abandoned claim alive all night, and
every other agent would go on reading the task as held. Which verbs move the
lease, measured, is [Claims and identity](claims.md#the-lease-and-what-renews-it).
Poll `status` and `find`; call `context` and `show` when somebody is actually
working.

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

What a finding means, fault or signal, is [Reading ank check](check.md). Under
`--json` each one carries its level, its subject and its message:

<!-- replay state -->

    $ ank check --json
    {"contract":1,"faults":0,"signals":4,"tasks":1,"adr":1,"hot_files":6,"plane_bytes":173,"pruned":[],"findings":[{"level":"signal","subject":"ADR-57715ae64348","message":"written by an agent and read by no human","note":[],"charge":[]},{"level":"signal","subject":"TASK-6da126c832be","message":"written by an agent and read by no human","note":[],"charge":[]},{"level":"signal","subject":"allowed_signers","message":"no ratification key declared: permissions are advisory, not enforced (§8)","note":[],"charge":[]},{"level":"signal","subject":"coordination","message":"default branch indeterminable, completion refs neither pruned nor judged (ank config default_branch <name>)","note":[],"charge":[]}]}

## The conformance suite is offered to you

Two sets of fixtures in this repository are yours to reuse. The first says so in
its own header ("any third-party tool that claims to read or write the format
can reuse the `tests/golden/` directory") and the second is offered here, which
is the only place it is said:

- **`crates/ank-core/tests/golden/`**: the file format. Valid files that must
  round-trip byte for byte in canonical form, and invalid ones with the error
  each must produce. If you are writing a parser in another language, this is
  what tells you it is right, and [the file format](format.md) is what it is checking
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
  coloured, and a refusal leaves stdout empty rather than putting a shape there
  your parser has to tell apart. Warnings split: the ones about the corpus are a
  `warnings` array inside that document, the ones about a ref that did not reach
  the remote are on stderr in both modes.
- **Bind to the exit code**, never to the wording of an error. The message and
  the hint are written for a person to read and may be improved; the code is
  the contract.
- **Bind to `ank help --json`** for what a verb accepts and returns, rather than
  to a list you maintain. A list maintained by hand is a list that will disagree,
  and the disagreement surfaces on your side, days later, as a bug you cannot see
  from there.
- **One corpus is addressed at a time**, by `--repo <path>` on the CLI and by
  the `corpus` argument over MCP, so a tool holding several addresses each on
  its own. One process may hold several; nothing merges them. Claims are per
  repository, and nothing merges the claim spaces of two clones, because
  `refs/ank/*` cannot carry such an arbitration.
- **Do not poll a verb that renews a claim.** `context` and `show` over the held
  task move the lease; `status` and `find` do not, and they are what a refresh
  is for.
- **Do not bind to `ank watch`**, and bind to `events.jsonl` only as
  [the watcher](watch.md#what-to-bind-to) says.
