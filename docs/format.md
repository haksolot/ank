# The file format

For anyone writing a tool that reads or writes `.ank/`: an editor plugin, an
exporter, a linter, a second implementation.

The format is the specification, and the CLI is a reference implementation of
it rather than a gatekeeper. Nothing here needs `ank` to be installed or asks
your tool to call it.

**This document is not normative.** Section 3 of the specification is: *The data
model*, one of the ten `spec` documents in `.ank/` that `ank find --type spec`
lists. Where the two disagree the specification is right and this page is a bug. What you will find here instead
is the mechanical half a writer has to reproduce exactly, the field order, the
emission rules and the quoting predicate, which the specification states as
properties rather than as a list, plus a pointer to the section that argues each
one.

Two things settle a disagreement in practice, in this order: the specification,
then `crates/ank-core/tests/golden/`, which is the suite your implementation can
run against.

## Layout

    .ank/
      config.yml             repository settings and named verifiers
      allowed_signers        public keys allowed to ratify (§8), versioned
      entities/TASK-<hex>.md
      entities/ADR-<hex>.md
      entities/SPEC-<hex>.md
      entities/LOG-<hex>.md  one entry of the work trace, written once
      log/<ID>.md            the previous shape of the trace, read and never written
      index.db               derived, disposable, belongs in .gitignore, never a source of truth

Flat, deliberately: attachment happens through the `scope` field, not through
location (§3). A file's name is its id; nothing resolves through the directory
tree.

**One directory for every kind.** The kind is already in the id prefix, which is
already in the file name, so a per-kind subdirectory would state it a third time
and the only thing a third copy can do is disagree with the first two (§6). The
path is computed from the id with no lookup: every entity is at
`.ank/entities/<ID>.md`, whatever its kind, log entries included.

**An entity's entries are a query, not a path.** They are the entities of kind
`log` whose `about` names it, so finding them means reading the directory, or
your own index, where the previous shape let you compute one address. That is the
one thing this layout gives up, and it is deliberate (§3).

The layout is **fixed, not configured**. A layout read from `config.yml` would
mean your tool has to parse the configuration before it can find a file, and the
conformance suite at the end of this page would stop being something anybody can
run against a directory.

**The previous layouts, and the window for them.** Corpora written before the
flat directory are at `tasks/TASK-<hex>.md` and `adr/ADR-<hex>.md`, with the log
inside the task body; corpora written between that revision and this one carry
the trace as one file per entity under `.ank/log/`. A reader **must accept all of
them**; a writer **must never produce them**. A corpus holding several layouts is
one corpus, and nothing in it is counted twice: if an id resolves in both, decide
and document which wins rather than silently preferring one, and an entity's
entries are the union of the two sources. `ank check` reports a corpus still in a
previous shape as a signal, not a fault, naming the command that moves it: such a
corpus parses, round-trips and answers every verb.

This dual read is a **window, not a feature**. It exists for the release across
which an existing corpus moves, and a new tool has no reason to write anything
but the flat layout.

**Identifiers** are `TASK-`, `ADR-`, `SPEC-` or `LOG-` followed by exactly 12
hexadecimal characters, lowercase on output and accepted in either case on
input. They hash the act of creation (timestamp, identity, title, entropy)
never the content, so they survive every edit (§3). A tool that resolves short
prefixes must require at least 4 hex characters and must fail on an ambiguous
one, listing the candidates. Guessing is the one behaviour the format rules out
by name.

## The shape of a file

Markdown with YAML frontmatter, UTF-8 without BOM, LF line endings:

    ---
    <frontmatter>
    ---
    <body>

The delimiters are exact: the file begins with `---\n`, and the frontmatter ends
at the first `\n---\n`. Everything after that separator is the body, kept
verbatim, byte for byte. The body is free-form markdown and carries no
convention at all: the one that used to live there, the log, is an entity of its
own now.

**Unknown fields are rejected**, not ignored. That is what turns a typo like
`priorty:` into an error instead of a silent loss, and it is the reason the
`schema` rule in the next section exists at all.

## Fields, in canonical order

Canonical form is a **fixed field order**, and a serializer that emits the right
fields in the wrong order produces a non-canonical file. The order is not
alphabetical and not negotiable; it is this.

A kind is declared **once**, as a row of a registry: the name written in `type`,
the id prefix, the status values, which fields are required and which optional,
and the canonical order (§3). The two tables below are that registry written out.
Reproduce them as data, a table your serializer walks, rather than as one
emitter per kind; the order is the single thing most easily lost by rewriting two
straight-line emitters as a generic loop, and it is what the round-trip rests on.

An **unknown kind is rejected naming the kind**, not naming the first field it
happens to carry. Inside a known kind, an unknown field is still rejected. The two
refusals answer different questions: `priorty:` in a `task` is a typo, and
`type: epic` is a document your tool does not know how to read.

### Task

| # | Field | Emission | Notes |
|---|---|---|---|
| 1 | `id` | bare | `TASK-<12 hex>` |
| 2 | `type` | bare | always `task` |
| 3 | `slug` | scalar | optional, omitted when absent; cosmetic, never resolved on |
| 4 | `title` | scalar | |
| 5 | `created` | scalar | ISO 8601, always UTC with the `Z` suffix |
| 6 | `author` | scalar | optional; absent means the entity predates the field |
| 7 | `status` | bare | `open` \| `in_progress` \| `done` \| `closed` |
| 8 | `scope` | block sequence | mandatory, never empty; globs |
| 9 | `blocked_by` | flow list | always emitted, `[]` when empty |
| 10 | `done_criteria` | literal block | optional |
| 11 | `criteria_by` | bare | `creator` \| `claimer`; invalid without `done_criteria` |
| 12 | `verify` | flow list | omitted when empty |
| 13 | `proof` | block sequence of maps | omitted when empty |
| 14 | `verified` | block sequence of maps | optional, omitted when empty |
| 15 | `schema` | integer | |
| 16 | `version` | integer | |

A `proof` entry emits its own keys in order: `type`, `ref`, then `tree`,
`criteria`, `verifier` and `via`, each omitted when absent. `type` is one of
`test`, `commit`, `human-review`, `assertion`. `via` is one of `verifier`,
`attested`, `submitted`, the route by which the entry arrived, and its absence
means the entry was written before the field existed, never a fourth route.

A `verified` entry emits `by`, then `at`. Both are required in an entry that
exists at all, an entry missing either being rejected, while the list itself is
optional on every kind.

### ADR

| # | Field | Emission | Notes |
|---|---|---|---|
| 1 | `id` | bare | `ADR-<12 hex>` |
| 2 | `type` | bare | always `adr` |
| 3 | `slug` | scalar | optional |
| 4 | `title` | scalar | |
| 5 | `created` | scalar | ISO 8601, UTC |
| 6 | `author` | scalar | optional |
| 7 | `status` | bare | `proposed` \| `accepted` \| `superseded` |
| 8 | `scope` | block sequence | mandatory, never empty |
| 9 | `constraint` | literal block | mandatory |
| 10 | `see` | scalar | optional |
| 11 | `supersedes` | bare | optional, an entity id |
| 12 | `ratified` | scalar | optional; set by `accept` (see below) |
| 13 | `verified` | block sequence of maps | optional, omitted when empty |
| 14 | `schema` | integer | |
| 15 | `version` | integer | |

### Spec

An ADR without its `constraint`, and the absence is what makes it a kind of its
own: a spec describes where an ADR binds, so nothing in it is ever injected into
an agent's context (§3).

| # | Field | Emission | Notes |
|---|---|---|---|
| 1 | `id` | bare | `SPEC-<12 hex>` |
| 2 | `type` | bare | always `spec` |
| 3 | `slug` | scalar | optional |
| 4 | `title` | scalar | |
| 5 | `created` | scalar | ISO 8601, UTC |
| 6 | `author` | scalar | optional |
| 7 | `status` | bare | `proposed` \| `accepted` \| `superseded` |
| 8 | `scope` | block sequence | mandatory, never empty; what the document governs |
| 9 | `references` | flow list | optional, omitted when empty; entity ids |
| 10 | `supersedes` | bare | optional, an entity id |
| 11 | `ratified` | scalar | optional; set by `accept`, over the body and `scope` |
| 12 | `verified` | block sequence of maps | optional, omitted when empty |
| 13 | `schema` | integer | |
| 14 | `version` | integer | |

The anchor differs from an ADR's in what it covers and in nothing else: a spec
has no field carrying its authority, so `ratified` is taken over the body and
`scope` together. A tool that verifies one hashes the body as it hashes a
`constraint`, under the normalisation below.

`references` names the documents and decisions this one rests on, in the position
`blocked_by` takes on a task: immediately after the perimeter, before the
succession. It is a flow list of entity ids and it is **omitted when empty**, not
written `[]`: a task always states whether it has blockers, and a document that
cites nothing has nothing to state. A reader resolves each entry against the
corpus; what a checker then reports of it is §4's business and not the format's,
and an entry naming a kind other than `spec` or `adr` is a finding there rather
than a parse error here (§3).

### Log entry

| # | Field | Emission | Notes |
|---|---|---|---|
| 1 | `id` | bare | `LOG-<12 hex>` |
| 2 | `type` | bare | always `log` |
| 3 | `slug` | scalar | optional |
| 4 | `title` | scalar | the message, or its head (below) |
| 5 | `created` | scalar | ISO 8601, UTC; the instant of the entry |
| 6 | `author` | scalar | optional; who wrote the entry |
| 7 | `scope` | block sequence | mandatory; the subject's scope as it stood |
| 8 | `about` | bare | mandatory, an entity id of any kind |
| 9 | `seq` | integer | mandatory; rank among that entity's entries, from 0 |
| 10 | `verified` | block sequence of maps | optional, omitted when empty |
| 11 | `schema` | integer | |
| 12 | `version` | integer | |

**No `status`, and that is not an omission**: an entry is written once and has
nothing to transition to, so the registry declares the kind without one and your
parser must not require it. `version` stays, and on this kind it is a detector
rather than a counter: an entry above 1 has been rewritten, which the format
says should not happen.

**Optional fields are omitted, never emitted empty.** An entity with no author
serialises without the key at all. Writing `author:` with nothing after it would
change every older file on its first rewrite, and the round-trip guarantee below
forbids that.

### Actors

Every field naming an actor, `author` and the `by` of a `verified` entry, is
typed: `human:<id>` is a person, `<producer>/<version>` is an agent,
`process:<id>` is an automated process (§3).

**A value that does not match the convention is not a parse error.** Your parser
must accept it; reporting it belongs to a linter, and `ank check` reports the
whole pre-convention set once for the corpus rather than once per file. This is
the one place where being strict would be wrong: the convention postdates most
of the `author` values in an existing corpus, and a parser that refused them
would lock those files out of their own format. Write typed actors; read
anything.

## Emission rules

**Literal blocks** carry the multi-line fields, `done_criteria` and
`constraint`. Two spaces of indent, and the chomping indicator records whether
the value ends in a newline: `|` when it does, `|-` when it does not.

    done_criteria: |
      Auth integration tests pass, and no reference to
      jwt.verify remains in src/auth/

**Flow lists** carry references: `blocked_by: [TASK-51c2a7f0b3d9]`,
`verify: [auth-tests, no-jwt]`.

**Block sequences** carry `scope`, `proof` and `verified`, two spaces of indent:

    scope:
      - src/auth/**
      - src/middleware/session.ts

    verified:
      - by: human:marie
        at: 2026-07-27T09:40:00Z

**Scalars are emitted bare when that is unambiguous and quoted otherwise**,
conservatively: when in doubt, quote. The reference implementation emits a
scalar bare only when all of these hold: it is non-empty; it contains no
newline, no `": "` and no `" #"`; it does not end in `:` or a space; it does not
begin with any of

    - ? : # & * ! | > ' " % @ ` [ ] { } ,

or a space; it is not one of `null`, `~`, `true`, `false`, `yes`, `no`; and it
does not parse as a number. Otherwise it is double-quoted, with `\`, `"` and
newline escaped. Reproducing this predicate exactly is what makes a third-party
writer round-trip.

## The round-trip guarantee

    serialize(parse(x)) == x, byte for byte, when x is in canonical form

Valid but non-canonical input (another acceptable YAML form, superfluous
quotes, CRLF) is read correctly and **normalised on first rewrite** (§3). That
is what lets a human or a third-party tool write a file without knowing the
canonical form, without making that form an authoritative variant.

**CRLF is read, never written.** A parser must accept it; a serializer must not
produce it. A `---\r\n` diagnosed as "missing frontmatter" sends the reader
looking for a delimiter that is right there, so normalise line endings before
splitting, not after. `ank init` writes a `.gitattributes` carrying
`.ank/** text eol=lf`, because on Windows git would otherwise convert back on
every checkout what the tool has just normalised.

## Reading a version you do not know

`schema` is the format version, and a tool declares **a range of versions it
reads**, not a single one.

The current version is **3**, and the reference implementation reads **1 to 3**.
Version 3 carries the log leaving the entity body and the `verified` list with
its typed actors. The flat layout arrived in the same revision and carries no
bump of its own: it moves files, not fields, so a reader that finds the file
finds every field it already knew.

The log is what makes that bump necessary, and it is the case the field exists
for. A reader that does not know the log has left the body opens a task file,
finds no `## Log` section, and shows an empty history for a task that has one,
silently, with nothing reading anywhere as an error. Refusing on the version says
the one true thing before any of that happens.

Older is a promise the format keeps: every field introduced after version 1 is
optional at parse time, and its absence means "written before this existed"
rather than "invalid". A corpus is never migrated by a tool that refuses to read
it.

Newer is refused, and refused **on the version rather than on the first field it
does not recognise**. Since unknown fields are rejected, a tool that checked
only its own version would report a file one version newer as *unknown field
`author`*, and its reader would go hunting for a typo. Naming the version says
the one true thing: this file is newer than this tool. The argument is in §3.

**A new kind carries no bump either**, and needs none: an unknown kind is
rejected naming the kind, which is the same honest refusal by a different
mechanism, so a tool that does not know `spec` or `log` stops on that entity and
says which kind stopped it. The `spec` and `log` kinds are therefore readable at
any version in the range, and the one case no bump could reach, a reader that
opens a task file alone and looks for its entries where an older shape kept them
is covered by continuing to read that shape for one window (§6).

## The log

**The log is neither a section of the body nor a file per entity: an entry is an
entity.** It carries the instant in `created`, the identity in `author`, the
message in `title`, and what it is about in `about`, and it is written once and
never modified. A correction is a new entry naming the one it corrects.

The line grammar has not changed, and it is now how an entry is **printed**
rather than how it is stored: a dash and a space, the timestamp, a space, the
identity, a space, an em dash, a space, the message:

    - 2026-07-26T14:02Z claude-code/1.4.2 — jwt.verify removed from session.ts

So an entry written under either previous shape reads across unchanged and
nothing about it is reinterpreted: only where it lives has moved, twice.

**A message longer than a line is split across `title` and the body, and the
split is lossless.** One line of at most **100 characters** is the whole of the
`title`, and the body is empty. Longer, the title runs to the last space at or
before character 100 and at or after character 50, the limit itself where there
is no such space and the first newline where one comes earlier, and the body is
a newline, the remainder verbatim, a newline.

**The message is the exact concatenation of the two.** The separating space
belongs to the remainder, so joining inserts nothing; recovering the remainder
removes exactly one newline at each end and never trims. Given the message

    discrepancy: the criterion assumes merge=union and .gitattributes declares none, which is measurable

a writer stores

```yaml
title: "discrepancy: the criterion assumes merge=union and .gitattributes declares"
```

with the body `"
 none, which is measurable
"`, and a reader that concatenates
the title with the remainder gets the message back byte for byte. A body that is
not of that shape carries no remainder, and the message is the title alone.

The rule exists because the title is what every lister prints, on every kind: a
2000-character title is one enormous quoted scalar and it is printed in full
wherever entities are listed. **So print the head of the message with a trailing
`…` when there is more**, and let a reader ask for the entry itself to see the
whole. Machine output carries the whole message: a parser reads no page.

Any kind may be logged against, whether a task, an ADR or a spec, and **an entity with no
entries has an empty log, never an error**. Do not write one to record that there
is nothing to record.

**Order an entity's entries by `created`, then `seq`, then the identifier.** All
three are read off the entity; none of them is the file name, the directory
order or anything else outside it.

`seq` exists because a timestamp is not an order. `created` has one-second
resolution, and writing an entry costs a few hundred milliseconds, so several
entries inside one second is the ordinary case: measured on four entries written
about one task, 12 runs of 12 put all four in the same second, and 10 of those
12 came back in the wrong order when the identifier was the only tiebreak: a
hash of the act of creation, which carries no order at all. An append-only file
carried insertion order for free; a set of files does not.

**When you write an entry, set `seq` to one more than the highest `seq` you can
see on the entries already about that subject, or 0 if there are none.** That
requires reading them first, which is a bounded read and the same query you need
to display them. Two writers who cannot see each other will produce the same
value; that is correct rather than broken, since they were concurrent, `created`
separates them when their instants differ, and the identifier settles the rest.
Never treat equal `seq` as a conflict, and never rewrite an entry to renumber
it.

**An entry read out of one of the previous layouts takes the 0-based index of
its line in the file**, which is the order that file recorded. Since `created`
is read first, a file whose lines contradict their own timestamps is reordered
by the timestamps: measured on the reference corpus, one file of 178 stores its
lines newest-first. The guarantee is therefore exact: across distinct instants
the timestamps order the entries, and within one instant the line order does.

**Writing an entry is not a write to the entity it is about.** It writes no
frontmatter there, bumps no `version` there, and touches no file carrying a
frozen field. An entity file changes only on a real transition. That property is
the reason the log left the body, and a tool that records an entry *and* rewrites
the entity has given it up.

**Two entries are two files, which is why there is no merge rule for the log.**
The rule that used to union log sections by timestamp is gone, and the reason
once recorded for dropping it was wrong: git does not union two appends by
itself, it conflicts on them, unless a repository configures `merge=union` for
the path (§7). What has been protecting the corpus all along is one file per
entity, and an entry that is an entity extends that to the trace: there is no
file for two parties to append to.

**One convention lives in the message, and it is where a disproved criterion is
recorded.** A message opening with `discrepancy:` says that the frozen
`done_criteria` of that task rests in part on a false premise, and states what
was measured instead (§3):

    - 2026-08-14T18:16Z claude-code/03fd — discrepancy: the criterion assumes tests/skill.rs passes untouched; two tests there read `ank help`

It is a convention on the message and never on the grammar, `released: <reason>`
being the same kind and older, so it costs no field, no schema bump and no
migration, and every log a corpus already holds stays valid. It changes nothing
mechanically: the criterion is untouched, its hash still anchors it, and `done`
still verifies against that hash. A tool that reads the log should surface such
an entry; none should ever read it as permission to accept less.

The log is a **work trace, not proof**: nothing authoritative is anchored in it,
which is why there is no chained hash over it (§3), and which is what makes an
entry written by a second party harmless. A tool may read entries and may add
them; it should never reorder or rewrite one.

In a corpus in the earliest layout, the log is a `## Log` section at the end of
the task body; in the one between, it is a file per entity under `.ank/log/`,
one line per entry. Both carry the same line grammar. Read them there; write
neither.

## What is derived, and must never be stored

A file says less than the corpus does, on purpose. Four things are computed at
read time and have no field:

- **Blocked.** A task is blocked if and only if at least one of its
  `blocked_by` is not `done`. `closed` does not unblock. There is no `blocked`
  status to go stale.
- **Reverse edges.** What a task unblocks is derived by walking `blocked_by`
  across the corpus. A stored reverse edge is a second copy that can disagree
  with the first.
- **The claim.** Never in the file. See below.
- **The index.** `index.db` is a cache rebuilt from the files; deleting it is
  always safe.

## What lives outside the files

A tool that reads only `.ank/` sees the durable state and none of the
coordination. Two things are deliberately elsewhere, and both matter if your
tool intends to say anything about them.

**Claims are git refs**, one per task, at `refs/ank/claims/<task-id>`. The ref
has two states and the record it points at says which: a `claim` (holder,
expiry, the frozen criterion hash, the hash of applicable constraints) or a
`completed` record (commit, branch, identity, timestamp) written by `done`. A
task that is `in_progress` in the file with no ref behind it is simply one whose
claim expired: legal, and re-claimable (§7).

**The ratification anchor is a commit message.** `accept` writes `ratified:` into
the entity *and* produces a commit whose subject is `ratify <id>` and whose body
carries the anchor. The copy in the commit is the one that counts, because the
copy in the file is written by whoever writes the file. A verifier walks the
entity's own path history with `--full-history` and takes the first such commit
(§3).

**`accept` also records who ran it**, as a `verified` entry naming the typed
actor and the instant. The signature on the ratification commit says that a key
authorised the act, which is true of an agent typing under a cached passphrase as
much as of a human at a keyboard, so the entity carries the actor as well. It is
a record and not a defence: an actor value is declared and never proved, exactly
as `author` is, and what it buys is that an honest ratification leaves a trace a
reader can tell apart. `ank check` reports a decision whose ratifying actor is
its own author as a signal, never as a fault, because a solo maintainer does that
legitimately.

**The key names what was hashed**, and there are two because there are two kinds
that carry an anchor: `constraint+scope: <hash>` on an ADR, `body+scope: <hash>`
on a spec. A spec declares no `constraint`, that absence being what justifies the
kind, so the authority is carried by the whole document, and a commit claiming
`constraint+scope` over one would name a field the file does not have. A reader
accepts either key; a writer writes the one its kind carries.

## The two hashes

Both are SHA-256 over normalised text, displayed as the first 12 hex characters,
and a verifier accepts the short form or the full one.

**Normalisation** is what makes a hash insensitive to editing noise without ever
tolerating a change of meaning: CRLF becomes LF, trailing whitespace is stripped
from each line, trailing blank lines are removed.

- **The criterion freeze**, recorded by `claim`: `hash(normalize(done_criteria))`.
- **The ratification anchor**, recorded by `accept`: the normalised anchored
  text, a newline, then each `scope` glob trimmed and followed by a newline,
  hashed as one buffer. The anchored text is an ADR's `constraint` and a spec's
  body, which is the one place the two anchors differ and what the commit key
  above says.

Freezing is verifiable, not defended (§2). Your tool can rewrite any field in
any file; what it cannot do is make the recorded hash agree afterwards.

## Concurrency

`version` is an integer incremented on every write, and it is an intra-tree
compare-and-swap: read, compare, write, under a file lock, with the write done
atomically (write-then-rename). It protects one working tree: a human and an
agent sharing a checkout. Between clones, git's own compare-and-swap at push
time is what arbitrates (§7).

## Conformance

`crates/ank-core/tests/golden/` is a reusable suite, and it is small enough to
port in an afternoon:

- `valid/`: every file must parse, and re-serialising it must reproduce the
  input byte for byte **once line endings are normalised**. One file,
  `TASK-c71f0e5a9b23.md`, is in CRLF on purpose and must come back in LF; that
  is the only file for which the assertion is against the normalised input
  rather than the bytes on disk. Fixtures at schema 1 and schema 2 are there to
  stay: a file written before a field existed must survive a rewrite unchanged,
  and if one of them moves, the version bump has silently become a migration.
  Every kind carries one: a log entry is an ordinary entity fixture like any
  other, and a fixture in the previous shape, a whole log keyed by the id of
  the entity it belongs to, stays for as long as that shape is read.
- `invalid/`: every file must be **rejected with the right error**, not merely
  rejected. Each case names a distinct failure: no frontmatter, a bad id, an
  unknown schema, an unknown kind, an unknown field inside a known kind, a bad
  status, a type mismatch, an empty scope, an invalid glob, `criteria_by`
  without a criterion, a `verified` entry missing `by` or `at`, an entry naming
  nothing in `about`, and a log line the grammar does not accept. One per kind
  at least, or a kind ships with its strictness untested. The list grows with
  the format; what does not change is that a test asserting only that parsing
  returned *an* error passes for the wrong reason forever.

There is no invalid fixture for a malformed actor. That is deliberate and is the
one place strictness is wrong: the convention is checked, never parsed (above).

The second half is where a permissive implementation is caught. Accepting a file
the format rejects is the failure mode that spreads, because the corpus it
writes still looks fine until something else reads it.

## Where to go next

- The specification's ten `spec` documents in `.ank/`, each declaring in its own
  body which sections it carries: §3 for the data model and
  canonical form, §6 for storage, §7 for the coordination plane, §8 for identity
  and ratification. `ank show <id>` prints one whole.
- [getting-started.md](getting-started.md): if you also want to use the tool.
