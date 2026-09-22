# The file format

For anyone writing a tool that reads or writes `.ank/`: an editor plugin, an
exporter, a linter, a second implementation.

The format is the specification, and the CLI is a reference implementation of
it rather than a gatekeeper. Nothing here needs `ank` to be installed or asks
your tool to call it.

**This document is not normative.** Section 3 of the specification is: *The data
model*, one of the `spec` documents [the specification](specification.md)
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
      archive/entities/<ID>.md  the cold half, same format, read on demand, never rewritten
      log/<ID>.md            the previous shape of the trace, read and never written
      index.db               derived cache, belongs in .gitignore, never a source of truth

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

**The archive is the second root, and the only other one.** `.ank/archive/` holds
the cold half of the corpus at `archive/entities/<ID>.md`, flat and in the same
format by the same rule as above: one file per entity, the file name is the id,
readable with no parser and no tool. It is a directory rather than a packfile or
a walk of git history because either of those would put a parser, or git,
between a reader and a file.

A tool reading `.ank/` needs three things from it. It is **read on demand and
never by default**: the listing, the prefix resolution and the load of the hot
corpus answer exactly what they answered before the archive existed, so a tool
that ignores it is still correct, only blind to the cold half. It is **resolved
against both roots wherever an id is resolved**, so a reference, a supersession,
a blocker or an entry's subject naming an archived entity names something, and a
scope pointing at `.ank/entities/<ID>.md` for an entity since archived is not
dead; an id present in both roots is one entity, read hot. And **an archived file
is never edited**: it is parsed once when it arrives, the content hash recorded
then is its digest and is never updated, and a file whose bytes stop matching is
a fault rather than a change to take in. That digest is recorded in `index.db`
and in no file, which is the one place the cache is load-bearing; the section on
what is derived says what deleting it costs.

A writer moves files there with `ank archive` and never by hand; the move
commits nothing, and what the hot corpus holds is settled by a human reading a
diff.

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
alphabetical and not negotiable; it is the one the registry declares.

A kind is declared **once**, as a row of a registry: the name written in `type`,
the id prefix, the status values, which fields are required and which optional,
and the canonical order (§3). [Entity fields](entity-fields.md) is that registry
printed, generated from the table the binary reads and writes with: every kind,
every field in order, its emission form, whether it is always emitted, and the
values an enum field takes. Reproduce it as data, a table your serializer walks,
rather than as one emitter per kind; the order is the single thing most easily
lost by rewriting two straight-line emitters as a generic loop, and it is what
the round-trip rests on.

An **unknown kind is rejected naming the kind**, not naming the first field it
happens to carry. Inside a known kind, an unknown field is still rejected. The two
refusals answer different questions: `priorty:` in a `task` is a typo, and
`type: epic` is a document your tool does not know how to read.

### Task

A `proof` entry emits its own keys in order: `type`, `ref`, then `tree`,
`criteria`, `verifier` and `via`, each omitted when absent. The values `type`
and `via` take are listed [with the fields](entity-fields.md#proof-type). `via`
is the route by which the entry arrived, and its absence means the entry was
written before the field existed, never a fourth route.

A `verified` entry emits `by`, then `at`. Both are required in an entry that
exists at all, an entry missing either being rejected, while the list itself is
optional on every kind.

### Spec

An ADR without its `constraint`, and the absence is what makes it a kind of its
own: a spec describes where an ADR binds, so nothing in it is ever injected into
an agent's context (§3).

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

**No `status`, and that is not an omission**: an entry is written once and has
nothing to transition to, so the registry declares the kind without one and your
parser must not require it. `version` stays, and on this kind it is a detector
rather than a counter: an entry above 1 has been rewritten, which the format
says should not happen.

**`records` marks an entry a verb wrote, not a holder.** Absent, the entry is
work. The values known to this build are listed
[with the fields](entity-fields.md#records).

**`edit` and `create` share one grammar**, which carries the versions the write
moved between and the hash of the content it produced. Both of these were
written by the binary:

<!-- replay trace ANK_AGENT=claude-code/1.4.2
$ ank init
$ ank new task --title "Migrate auth" --scope "src/auth/**" --criteria "c" --no-verify
created TASK-5f1e0a9c2b7d Migrate auth
$ ank claim TASK-5f1e
$ ank amend TASK-5f1e --scope "docs/**"
$ for t in '+scope' 'created'; do grep -h "^title: $t" .ank/entities/LOG-*.md | cut -c8-; done
-->

    +scope docs/** (version 2 to 3, replaced 80bdeffde4e7, produced a06b2b863b0e)
    created (version 0 to 1, produced f9b82c19eaec)

`edit` is a change of content outside a status transition, and names the fields
it changed and the hash of the state it replaced. `create` is the record `new`
writes at birth: version 0 is the state before the file existed, and `replaced`
is absent because nothing was. An entry about a log entry never carries
`create`, since an entry is the record.

**`method` carries neither**, and that is why the grammar above is a property of
two values rather than of the field. `ank log --method <name>` writes it to
record that a sibling skill opened under a claim, and the entry's title is the
name alone:

    tdd

A value your reader does not know is read as machinery and never refused.

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
newline escaped.

**"Parses as a number" is Rust's `str::parse::<f64>`**, and naming the parser is
the point: it is wider than YAML's own number grammar, so a writer that
substitutes its own language's parser agrees everywhere except where it matters.
Measured through the binary, these titles come back double-quoted

    inf  INF  infinity  Infinity  nan  NaN  NAN  1e5  +3  .5  1.  007

and these come back bare

    0x10  5_000  1_0  1e

The infinities and `NaN`, in any case, are what another parser will miss;
`5_000` and `0x10` are what it may wrongly catch.

Reproducing this predicate exactly is what makes a third-party writer
round-trip.

## The round-trip guarantee

    serialize(parse(x)) == x, byte for byte, when x is in canonical form

Valid but non-canonical input (another acceptable YAML form, superfluous
quotes, CRLF) is read correctly and **normalised on first rewrite** (§3). That
is what lets a human or a third-party tool write a file without knowing the
canonical form, without making that form an authoritative variant.

**Read is not the same as accepted.** A file that parses but does not round-trip
is a **fault** under `ank check`, reported as `non-canonical form (round-trip
differs)` and exiting 8. Measured by double-quoting one bare `title` in an
otherwise canonical file: `ank show` goes on printing the entity unchanged while
`check` turns red on it. A third-party writer that does not reproduce the
emission rules therefore writes a corpus that reads perfectly everywhere and
fails its own repository's check.

**CRLF is read, never written.** A parser must accept it; a serializer must not
produce it. A `---\r\n` diagnosed as "missing frontmatter" sends the reader
looking for a delimiter that is right there, so normalise line endings before
splitting, not after. `ank init` writes a `.gitattributes` carrying
`.ank/** text eol=lf`, because on Windows git would otherwise convert back on
every checkout what the tool has just normalised.

**CRLF is the one exception to that fault.** When dropping the carriage returns
leaves exactly the canonical form, the content is right and only the checkout is
wrong, so `check` reports a **signal at exit 0** instead, naming `git config
core.autocrlf input`. Measured on one file taken through all three states: LF
and canonical is silent, the same bytes at CRLF give the signal and exit 0, and
a genuinely non-canonical file is the fault whichever line endings it carries.

## Reading a version you do not know

`schema` is the format version, and a tool declares **a range of versions it
reads**, not a single one.

The version the reference implementation writes, and the range it reads, are
printed [with the fields](entity-fields.md).

Version 3 carries the log leaving the entity body and the `verified` list with
its typed actors. The flat layout arrived in the same revision and carries no
bump of its own: it moves files, not fields, so a reader that finds the file
finds every field it already knew. Version 4 carries one field, `records` on a
log entry.

Both bumps are the case the field exists for, and it is the same case twice. A
reader that does not know the log has left the body opens a task file, finds no
`## Log` section, and shows an empty history for a task that has one, silently,
with nothing reading anywhere as an error. A reader that does not know `records`
drops it on the next rewrite, and an entry written to stay out of the work trace
silently rejoins it. Refusing on the version says the one true thing before
either happens.

**A bump is paid for by the builds already distributed**, and it is worth saying
where. Every entity a verb writes carries the version that build writes, so a
corpus edited by a build at 4 becomes unreadable to a build at 3 one entity at a
time: the older build warns once that the schema is ahead and every listing then
answers as if those entities were not there. That is the designed behaviour of
the range and not a side effect of it.

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
identity, a space, an em dash, a space, the message. `ank log` prints each
entry that way, after its short id and two spaces:

<!-- replay trace part
$ ank log "jwt.verify removed from session.ts"
logged LOG-9cfb085b14b3 on TASK-5f1e0a9c2b7d
$ ank log TASK-5f1e
-->

    LOG-9cfb  - 2026-07-26T14:02:11Z claude-code/1.4.2 — jwt.verify removed from session.ts

So an entry written under either previous shape reads across unchanged and
nothing about it is reinterpreted: only where it lives has moved, twice.

**A message longer than a line is split across `title` and the body, and the
split is lossless.** A message of at most **100 characters** is the whole of the
`title`, and the body is empty — 100 itself included, so a message of exactly
that length does not split. Longer, the title runs to the last space at or
before character 100 and at or after character 50, the limit itself where there
is no such space and the first newline where one comes earlier, and the body is
a newline, the remainder verbatim, a newline.

**The message is the exact concatenation of the two.** The separating space
belongs to the remainder, so joining inserts nothing; recovering the remainder
removes exactly one newline at each end and never trims. Given this 111-character
message

    discrepancy: the criterion assumes merge=union is configured for the log path, and .gitattributes declares none

`ank log` stores

```yaml
title: "discrepancy: the criterion assumes merge=union is configured for the log path, and .gitattributes"
```

whose value is 97 characters — the cut is the last space at or before character
100 — with the body

```
\n declares none\n
```

— a newline, the remainder, a newline. The remainder opens with the space that
separated the two words, so a reader that concatenates the title with it gets
the message back byte for byte: 97 + 14 = 111. A body that is not of that shape
carries no remainder, and the message is the title alone.

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

<!-- replay discrepancy part ANK_AGENT=claude-code/03fd
$ ank init
$ ank new task --title "The skill tests pass" --scope "**" --criteria "c" --no-verify
created TASK-2c7a51e0f9b4 The skill tests pass
$ ank claim TASK-2c7a
$ ank log 'discrepancy: the criterion assumes tests/skill.rs passes untouched; two tests there read `ank help`'
$ ank log TASK-2c7a
-->

    LOG-d41c  - 2026-08-14T18:16:03Z claude-code/03fd — discrepancy: the criterion assumes tests/skill.rs passes untouched; two tests there read `ank help`

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
- **The index.** `index.db` is a cache rebuilt from the files, and every query
  it answers is answered again once it has been rebuilt — with one exception,
  which is the next paragraph.

**Deleting `index.db` loses one thing, and it is the one thing in there that is
not derived from the files: the digest an archived file arrived with.** An
archived entity is never edited, and that is enforced by comparing the file's
bytes against the hash recorded when the index first read it. The corpus has
nowhere to keep that hash — the archived file cannot carry a digest of itself —
so it lives in the cache and nowhere else. Measured on a corpus of four archived
entries: appending a line to one of them makes `ank check` exit 8 naming the
file; `rm .ank/index.db` and the same `check` exits 0, on that run and on every
run after it, because the rebuild takes the changed bytes as the digest.

So "deleting it is always safe" is true of everything a reader queries and false
of archived integrity. A tool that intends to say anything about an archived
file's bytes must treat the cache as state, and the only authority that survives
its deletion is git: `git checkout -- .ank/archive/entities/<ID>.md` restores the
file, where a reindex only re-blesses whatever is there.

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
copy in the file is written by whoever writes the file. `ratified` cannot name
the commit, since a commit cannot contain its own identifier, so the subject is
the only pointer there is (§3).

**A verifier finds it by subject, over the whole history, with no path
restriction:**

    git rev-list --full-history HEAD

reading each commit's subject and taking the first that is `ratify <id>`. The
walk is newest-first, so a decision ratified twice answers with the newest.

`--full-history` because path simplification exists to explain a tree's final
state and is free to drop a commit that a merge made redundant; dropping the
ratification would report a perfectly frozen decision as unverifiable. **No path
restriction** because the subject is the key, and a subject is independent of
where the file sits. Measured on ank's own repository, where `ADR-01b6dd05f0db`
was ratified at `.ank/entities/` and later moved to `.ank/archive/entities/`: a
walk restricted to its current path returns no `ratify` commit for it at all,
where the unrestricted walk returns exactly one. A verifier that restricts by
path reports that decision unverifiable, which is the one verdict that looks
like an answer and is not.

One walk answers the whole corpus, and that is also why it is one walk: the
question is asked once per decision and again for every task a decision bears
on, so a search per entity is a process count that grows with the corpus.

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
  text, a newline, then each `scope` glob trimmed and followed by a newline —
  and then **the whole buffer normalised a second time** before it is hashed,
  which removes the newline the last glob was just given. The anchored text is
  an ADR's `constraint` and a spec's body, which is the one place the two
  anchors differ and what the commit key above says.

That second normalisation is the entire difference between a tool that agrees
with `ank accept` and one that does not, so the recipe is worth writing out. The
bytes hashed are

    normalize(text) + "\n" + globs.join("\n")

with **no trailing newline**, the globs trimmed and in the order the file lists
them.

**A worked vector, taken from a real `accept`.** An ADR whose `constraint` is
`Never Y` and whose `scope` is the single glob `src/**`, ratified by the binary,
produced the commit body

<!-- replay vector part
$ ank init && ank config default_branch main
$ ank new adr --title "Y" --scope "src/**" --constraint "Never Y"
$ mkdir src && echo y > src/y && git add -A && git commit -q -m y
$ ank accept $(ls .ank/entities | grep '^ADR-' | cut -c1-8)
$ printf 'Never Y\nsrc/**' | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-12
33045e58af8d
$ git log -1 --format=%B
-->

    constraint+scope: 33045e58af8d

and `printf 'Never Y\nsrc/**' | sha256sum` gives `33045e58af8d…`. Keep the
trailing newline the per-glob rule appears to ask for and the same decision
hashes to `b88d9f81eb08`, which no ratification anywhere carries: an
implementation off by that one byte reports every ADR in the corpus as diverged.
Several globs join the same way — `Rule two.` over `src/**` and `docs/**`
recorded `16a85fda1d17`, which is `printf 'Rule two.\nsrc/**\ndocs/**'`.

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
  input byte for byte **once it is in canonical form**. Two fixtures are not,
  and they are the two shapes the format reads and never writes, so for those
  two the assertion is against the normalised input rather than the bytes on
  disk:

  | Fixture | What it is not canonical in | What the comparison normalises |
  |---|---|---|
  | `TASK-c71f0e5a9b23.md` | CRLF line endings | back to LF |
  | `TASK-9dd8e04b1358.md` | its closing `---` is the last byte | the final newline is put back |

  Exactly one of each, and the suite asserts the counts rather than the files:
  a `.gitattributes` that converted the first, or an editor that added a newline
  to the second, would otherwise leave a green test covering nothing. The second
  shape can only be an entity with an empty body, since a body puts the newline
  after the delimiter by construction.

  **Every version in the reader range carries a fixture**, 1 through 4, and the
  suite fails a bump shipped without one. The old ones are there to stay: a file
  written before a field existed must survive a rewrite unchanged, and if one of
  them moves, the version bump has silently become a migration. Every kind
  carries one too: a log entry is an ordinary entity fixture like any other, and
  a fixture in the previous shape, a whole log keyed by the id of the entity it
  belongs to, stays for as long as that shape is read.
- `invalid/`: every file must be **rejected with the right error**, not merely
  rejected. Seventeen fixtures, each naming a distinct failure:

  | Fixture | What must be refused |
  |---|---|
  | `no-frontmatter.md` | a file with no frontmatter at all |
  | `unterminated-frontmatter.md` | an opening `---` with no closing one |
  | `bad-id.md` | an identifier that is not 12 hex characters |
  | `bad-schema.md` | a schema outside the range, naming the version found |
  | `bad-status.md` | a status the kind does not declare |
  | `bad-glob.md` | a `scope` entry that is not a valid glob |
  | `missing-scope.md` | a `scope` written `[]` |
  | `type-mismatch.md` | `type` disagreeing with the id prefix |
  | `unknown-field.md` | an unknown field inside a known kind |
  | `unknown-kind.md` | a kind the registry does not declare |
  | `criteria-by-without-criteria.md` | `criteria_by` with no `done_criteria` |
  | `bad-proof-via.md` | a `via` outside the closed set |
  | `verified-without-at.md` | a `verified` entry with `by` and no `at` |
  | `spec-with-constraint.md` | a `constraint` on a spec |
  | `log-without-about.md` | a log entry with no `about` |
  | `log-without-seq.md` | a log entry with no `seq` |
  | `log/bad-log-line.md` | a log file holding a line the grammar refuses |

  **Most of them assert on more than the error's type**, and that is where a
  permissive implementation is actually caught. `unknown-kind` must name `epic`,
  not the id prefix and not the first field an unknown kind happens to carry:
  a reader told "invalid identifier" goes hunting for a typo in the hex.
  `bad-schema` must name the version it found. `bad-proof-via`,
  `verified-without-at`, `spec-with-constraint`, `log-without-about` and
  `log-without-seq` must each name the field — `spec-with-constraint` naming
  `constraint` rather than the kind, because the field is the one a spec exists
  in order not to carry. `bad-log-line` must name **line 2**: the file's other
  two lines are entries the grammar accepts, so a fixture whose every line were
  bad would pass for a reader that gave up on the first one.

  The first two rows are one distinction and not two cases of one error, which
  is why both fixtures exist. `no-frontmatter.md` never opens a frontmatter;
  `unterminated-frontmatter.md` opens one and never closes it, and its refusal
  names the **closing** delimiter — *unterminated frontmatter: no closing `---`
  after the opening one*. Telling a reader that a file which plainly starts with
  `---` must start with `---` sends them looking for a delimiter that is right
  there, which is the same hour `---\r\n` already cost. A reader that folds the
  two into one error passes this fixture and fails the person holding the file.

  One case per kind at least, or a kind ships with its strictness untested. The
  list grows with the format; what does not change is that a test asserting only
  that parsing returned *an* error passes for the wrong reason forever.

There is no invalid fixture for a malformed actor. That is deliberate and is the
one place strictness is wrong: the convention is checked, never parsed (above).

The second half is where a permissive implementation is caught. Accepting a file
the format rejects is the failure mode that spreads, because the corpus it
writes still looks fine until something else reads it.

## Where to go next

- [The specification](specification.md): each spec declares in its own body
  which sections it carries -- §3 for the data model and canonical form, §6 for
  storage, §7 for the coordination plane, §8 for identity and ratification.
- [Entity fields](entity-fields.md) and [config.yml keys](config-keys.md): the
  tables, generated from the registry the binary reads with.
- [The quickstart](quickstart.md): if you also want to use the tool.
