---
id: TASK-69eaddc84993
type: task
slug: the-conformance-list-in-format-md-matches-golden
title: The conformance list in format.md matches golden/invalid file for file
created: 2026-09-20T18:37:17Z
author: claude-code/opus-5+4eef
status: done
scope:
  - docs/format.md
blocked_by: []
done_criteria: |
  The `invalid/` list in the Conformance section of docs/format.md matches crates/ank-core/tests/golden/invalid/ file for file: one row per fixture, no row without a fixture, the stated count equal to the number of files, and each row naming what that fixture must be refused for as crates/ank-core/tests/golden.rs asserts it. The `valid/` bullet beside it names every fixture whose round-trip assertion is against a normalised input rather than the bytes on disk, not only the CRLF one.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/56e3022eb942@5a1a107
    tree: scope/250c2b96d470
    criteria: 6bb233303bf1
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@5a1a107
    tree: scope/250c2b96d470
    criteria: 6bb233303bf1
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---

## Why this exists

TASK-4eef0864be46 rewrote this section on 2026-09-20 and its criterion required
the list to match the directory file for file. It did, when it was written.
TASK-ccb787dc807f landed the same day and added a seventeenth fixture,
`invalid/unterminated-frontmatter.md`, for a frontmatter opened and never
closed. Its scope was `crates/ank-core/tests/**` and three source files; it
never named `docs/format.md`, which is correct. Neither agent could see the
other, both perimeters were respected, and the page came out of the two
landings saying "Sixteen fixtures" over sixteen rows against a directory of
seventeen.

**Nothing turned red.** Measured on this branch: writing "Ninety-nine fixtures"
into that sentence leaves `cargo test -p ank-core --test golden` green, 31 of
31. That is the drift ADR-2b62b9a1fe67 was ratified to stop, caught by a reader
rather than by the suite.

The same landing left a second sentence false. The `valid/` bullet says
`TASK-c71f0e5a9b23.md` is "the only file for which the assertion is against the
normalised input rather than the bytes on disk". TASK-ccb787dc807f added
`valid/TASK-9dd8e04b1358.md`, whose closing `---` is its last byte, and
`canonical_form` in golden.rs puts the newline back before the comparison.
Measured: exactly one fixture carries CRLF and exactly one ends without a
newline, and they are different files. Both sentences come from one landing, so
they are corrected together rather than left for a third task.

TASK-4eef0864be46 is done and its proof anchors a state of the tree, so it is
not reopened and not re-edited. This is a new task, per SPEC-ac4aad6d1edb.

## What makes this unnecessary later

TASK-46d3a4c56bf4 generates the format reference's tables from the kind
registry and from `config.yml`. The conformance list is the same kind of table
over a different source — the directory — and a generated or asserted list ends
this class of correction: a fixture added without a row would fail the suite
instead of waiting for somebody to read the page. TASK-9e80f0e9f5f9 is its
neighbour for the output blocks. Until one of them lands, every fixture added
to `golden/invalid/` silently falsifies this page.

## verify

`verify: [cargo-test, fmt-check]`, the repository defaults, taken deliberately
and not by omission.

`cargo-test` is not vacuous over this perimeter. One test reads
`docs/format.md` — `the_tables_in_docs_format_md_match_the_serializer` in
`crates/ank-core/tests/golden.rs` — and it guards the four `### <Kind>` field
tables. Measured: changing `verify` in the Task table from `flow list` to
`scalar` makes that test fail. So the suite does hold part of the file an edit
here can break, and declaring it is what stops a rewrite of this page from
landing with a broken table.

`cargo-test` does **not** settle this criterion, and that is worth stating
rather than hiding: no test compares the conformance list to the directory, as
the "Ninety-nine fixtures" measurement above shows. The close will therefore be
Ank's statement that the tree is green, not Ank's statement that the criterion
is met; the criterion is met by reading the directory and the table against each
other, which is recorded in the log. An empty `verify:` via `--no-verify` would
have claimed less and proved less, and CLAUDE.md is explicit that an empty list
is a judgement for a criterion no declared verifier can touch at all. Here one
touches the file, so the list is not empty.
