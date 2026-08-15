# Ank — Specification: index

**This file is an index and carries no rule.** The specification is ten entities
of kind `spec` in `.ank/`, each ratified and anchored on its own, with the
coherence between them verified rather than assumed (ADR-5a690829388d). Where
this page and a document disagree, the document is right — there is nothing here
to disagree with it, and that is the point.

Read one whole:

```
ank find --type spec          # the ten, one line each
ank show SPEC-<id>            # a document, byte for byte
ank context <path>            # the documents governing a perimeter, named
```

## The ten documents

| Carries | Document | Id |
|---|---|---|
| §1, §2, §10, revision record | Intent, principles, and what v1 leaves out | `SPEC-1d5b44efd388` |
| §3 | The data model | `SPEC-acee5d9cb21b` |
| §4, less the two halves below | The CLI surface | `SPEC-c33e07a82cc4` |
| §4 *Presentation* | Presentation: structure for every reader, colour for a terminal | `SPEC-89070ce7f3b8` |
| §4 *Proofs* → *Trust hierarchy*, and §8 | Proof, anchoring and authority | `SPEC-3d12e76d9fa2` |
| §5 and §11 | The attention budget and the constraint lifecycle | `SPEC-6aed60cd3717` |
| §6 | Storage and search | `SPEC-201041998d90` |
| §7 | Synchronisation | `SPEC-183d297253ac` |
| §9 | Bootstrapping, teaching and distribution | `SPEC-fa2f8c49dba4` |
| §12 and §13 | Implementation, and the decisions that bound it | `SPEC-9f510cad4be6` |

A reader arriving with no context reads the first, which rests on nothing. A
reader implementing a third-party tool reads *The data model* and
[format.md](format.md). A reader finishing a first task reads
[getting-started.md](getting-started.md).

**Cross-references keep the section numbers of the monolith**, so a rule that
read `(§7)` still reads `(§7)`. The table above is the map, and it is the reason
the numbers were not rewritten into identifiers: two hundred rewrites are two
hundred chances to lose one, and the numbers now name the parts rather than
positions in a file.

**Each document argues its own boundary in its own body.** The test is the one
ADR-5a690829388d states — *a document that cannot be read without holding another
one open is not a document* — and applying it honestly merged more than it split:
thirteen sections became ten documents, with four merges and two splits, each
argued where it was made.

## Why this path still exists

The file that stood here was the specification; it is gone, and nothing on this
page restates a rule, so there is no second source of truth to drift.

The path survives because deleting it was measured: 58 entities carry
`docs/ank-spec-v1.1.md` in their `scope` — **10 ADRs and 48 finished tasks** —
and a dead scope on either is a `check` **fault**, since git records a deletion
as no rename and there is nothing for the walk to explain. None of the 58 can be
repaired: `amend` refuses a `done` task, and a ratified ADR's scope is anchored
in its ratification commit, so the only route would be superseding ten ratified
decisions to correct a path in their perimeter. A fault nobody can clear
is a finding readers learn to skip, which the specification legislates against in
as many words. Keeping a signpost at the path costs one file that binds nothing
and keeps every one of those scopes truthful: they named where the specification
was, and this is where the specification is described.
