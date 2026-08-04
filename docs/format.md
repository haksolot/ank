# The file format

For anyone writing a tool that reads or writes `.ank/` — an editor plugin, an
exporter, a linter, a second implementation.

The format is the specification, and the CLI is a reference implementation of
it rather than a gatekeeper. Nothing here needs `ank` to be installed or asks
your tool to call it.

**This document is not normative.** Section 3 of
[the specification](ank-spec-v1.1.md) is, and where the two disagree the
specification is right and this page is a bug. What you will find here instead
is the mechanical half a writer has to reproduce exactly — the field order, the
emission rules, the quoting predicate — which the specification states as
properties rather than as a list, plus a pointer to the section that argues each
one.

Two things settle a disagreement in practice, in this order: the specification,
then `crates/ank-core/tests/golden/`, which is the suite your implementation can
run against.

## Layout

    .ank/
      config.yml          repository settings and named verifiers
      allowed_signers     public keys allowed to ratify (§8), versioned
      tasks/TASK-<hex>.md
      adr/ADR-<hex>.md
      index.db            derived, disposable, belongs in .gitignore — never a source of truth

Flat, deliberately: attachment happens through the `scope` field, not through
location (§3). A file's name is its id; nothing resolves through the directory
tree.

**Identifiers** are `TASK-` or `ADR-` followed by exactly 12 hexadecimal
characters, lowercase on output and accepted in either case on input. They hash
the act of creation — timestamp, identity, title, entropy — never the content,
so they survive every edit (§3). A tool that resolves short prefixes must
require at least 4 hex characters and must fail on an ambiguous one, listing the
candidates. Guessing is the one behaviour the format rules out by name.

## The shape of a file

Markdown with YAML frontmatter, UTF-8 without BOM, LF line endings:

    ---
    <frontmatter>
    ---
    <body>

The delimiters are exact: the file begins with `---\n`, and the frontmatter ends
at the first `\n---\n`. Everything after that separator is the body, kept
verbatim, byte for byte. The body is free-form markdown, with one convention
carved out below.

**Unknown fields are rejected**, not ignored. That is what turns a typo like
`priorty:` into an error instead of a silent loss, and it is the reason the
`schema` rule in the next section exists at all.

## Fields, in canonical order

Canonical form is a **fixed field order**, and a serializer that emits the right
fields in the wrong order produces a non-canonical file. The order is not
alphabetical and not negotiable; it is this.

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
| 14 | `schema` | integer | |
| 15 | `version` | integer | |

A `proof` entry emits its own keys in order: `type`, `ref`, then `tree`,
`criteria` and `verifier`, each omitted when absent. `type` is one of `test`,
`commit`, `human-review`, `assertion`.

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
| 13 | `schema` | integer | |
| 14 | `version` | integer | |

**Optional fields are omitted, never emitted empty.** An entity with no author
serialises without the key at all. Writing `author:` with nothing after it would
change every older file on its first rewrite, and the round-trip guarantee below
forbids that.

## Emission rules

**Literal blocks** carry the multi-line fields — `done_criteria` and
`constraint`. Two spaces of indent, and the chomping indicator records whether
the value ends in a newline: `|` when it does, `|-` when it does not.

    done_criteria: |
      Auth integration tests pass, and no reference to
      jwt.verify remains in src/auth/

**Flow lists** carry references: `blocked_by: [TASK-51c2a7f0b3d9]`,
`verify: [auth-tests, no-jwt]`.

**Block sequences** carry `scope` and `proof`, two spaces of indent:

    scope:
      - src/auth/**
      - src/middleware/session.ts

**Scalars are emitted bare when that is unambiguous and quoted otherwise**,
conservatively — when in doubt, quote. The reference implementation emits a
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

Valid but non-canonical input — another acceptable YAML form, superfluous
quotes, CRLF — is read correctly and **normalised on first rewrite** (§3). That
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

Older is a promise the format keeps: every field introduced after version 1 is
optional at parse time, and its absence means "written before this existed"
rather than "invalid". A corpus is never migrated by a tool that refuses to read
it.

Newer is refused, and refused **on the version rather than on the first field it
does not recognise**. Since unknown fields are rejected, a tool that checked
only its own version would report a file one version newer as *unknown field
`author`*, and its reader would go hunting for a typo. Naming the version says
the one true thing: this file is newer than this tool. The argument is in §3.

## The log

One convention inside the body, and the only one: a `## Log` section at the end
of a task file, append-only, one line per entry.

    ## Log
    - 2026-07-26T14:02Z claude-code@host-3 — jwt.verify removed from session.ts

Appending at the end is what makes a log entry a one-line git diff. The log is a
**work trace, not proof**: nothing authoritative is anchored in it, which is why
there is no chained hash over it (§3). A tool may parse it, and may append to
it; it should not reorder or rewrite it.

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
claim expired — legal, and re-claimable (§7).

**The ratification anchor is a commit message.** `accept` writes `ratified:` into
the ADR *and* produces a commit whose subject is `ratify <id>` and whose body
carries `constraint+scope: <hash>`. The copy in the commit is the one that
counts, because the copy in the file is written by whoever writes the file. A
verifier walks the ADR's own path history with `--full-history` and takes the
first such commit (§3).

## The two hashes

Both are SHA-256 over normalised text, displayed as the first 12 hex characters,
and a verifier accepts the short form or the full one.

**Normalisation** is what makes a hash insensitive to editing noise without ever
tolerating a change of meaning: CRLF becomes LF, trailing whitespace is stripped
from each line, trailing blank lines are removed.

- **The criterion freeze**, recorded by `claim`: `hash(normalize(done_criteria))`.
- **The ratification anchor**, recorded by `accept`: the normalised `constraint`,
  a newline, then each `scope` glob trimmed and followed by a newline — hashed
  as one buffer.

Freezing is verifiable, not defended (§2). Your tool can rewrite any field in
any file; what it cannot do is make the recorded hash agree afterwards.

## Concurrency

`version` is an integer incremented on every write, and it is an intra-tree
compare-and-swap: read, compare, write, under a file lock, with the write done
atomically (write-then-rename). It protects one working tree — a human and an
agent sharing a checkout. Between clones, git's own compare-and-swap at push
time is what arbitrates (§7).

## Conformance

`crates/ank-core/tests/golden/` is a reusable suite, and it is small enough to
port in an afternoon:

- `valid/` — every file must parse, and re-serialising it must reproduce the
  input byte for byte **once line endings are normalised**. One file,
  `TASK-c71f0e5a9b23.md`, is in CRLF on purpose and must come back in LF; that
  is the only file for which the assertion is against the normalised input
  rather than the bytes on disk.
- `invalid/` — every file must be **rejected with the right error**, not merely
  rejected. The nine cases each name a distinct failure: no frontmatter, a bad
  id, an unknown schema, an unknown field, a bad status, a type mismatch, an
  empty scope, an invalid glob, and `criteria_by` without a criterion.

The second half is where a permissive implementation is caught. Accepting a file
the format rejects is the failure mode that spreads, because the corpus it
writes still looks fine until something else reads it.

## Where to go next

- [ank-spec-v1.1.md](ank-spec-v1.1.md) — the specification. §3 for the data
  model and canonical form, §6 for storage, §7 for the coordination plane, §8
  for identity and ratification.
- [getting-started.md](getting-started.md) — if you also want to use the tool.
