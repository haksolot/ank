---
id: ADR-c88f99e1c16e
type: adr
slug: a-reference-names-a-document-and-the-reader-foll
title: A reference names a document, and the reader follows its succession
created: 2026-08-21T23:53:42Z
author: claude-code/opus-5
status: accepted
scope:
  - crates/ank-cli/**
constraint: |
  A specification is several documents, one entity each, and each is ratified and anchored on its own. Coherence between them is verified rather than assumed: a reference from one document to another is a declared field, and changing a ratified document is a supersession that leaves a chain. A reference names a document and not a revision of it, so a reference to a superseded entity resolves through that chain to the entity at its end, and check reports one naming an entity that is absent, of a kind a specification does not cite, not yet accepted, or superseded with no successor to follow. Nothing is rewritten to make a reference resolve: the file keeps the identifier its author wrote, and the resolution belongs to the reader. A document that cannot be read alone is not a document, and the decomposition is argued in the corpus rather than derived from section numbers.
supersedes: ADR-5a690829388d
ratified: d55fd5cf73fa
verified:
  - by: seanl@sean-laptop
    at: 2026-08-22T00:10:27Z
schema: 4
version: 2
---

This changes one clause of ADR-5a690829388d and carries the rest of it word for
word. The clause is the one that made `check` report a reference to a superseded
entity, and the reason to change it is measured rather than anticipated.

**The repair is mechanical, named by the tool, and it grows.** Superseding the
CLI surface and storage documents left four citations to re-point. Superseding
the data model and the CLI surface again, hours later, left nine. It grows
because each replacement is cited by more of the corpus than the one before,
which is exactly what re-pointing everything to it accomplishes. Nobody decided
that the number should grow; it is a property of the rule.

**The obvious fix is the wrong one, and ADR-16813b3bcf37 is why.** Making
`accept` re-point every citation in the same act was the first design, and it
would touch nine entities from one ratification. Under the accounting decision
this corpus has just ratified, each of those writes leaves a machinery entry, so
one `accept` would deposit nine of them, and TASK-dfe5a1bb0857 would then count
nine versions asking to be accounted for. The repair automating itself would
pollute the trace the corpus just built to watch itself. A resolution performed
at read writes nothing at all: no amend, no version, no commit, no entry.

**What a `references` entry means is the actual question.** `SPEC-183d297253ac`
does not rest on revision three of the data model. It rests on *the data model*,
and the identifier is how the document is named rather than what is being named.
Read that way, a citation to a superseded revision is not stale, it is a name for
something that has moved, and the chain is how the corpus already records where.
`chain_head` walks it today, cycle detection included, and does so only to phrase
the signal this decision removes.

**Half of this was already true.** The rule as it stands lets a citation off when
the citing document *also* references the end of the chain, which is a reader
following a succession, spelled by hand and stored twice. This generalises what
that exception already concedes.

**What is not weakened.** A reference to something the corpus does not hold stays
a fault; so does one naming a kind a specification may not cite. A reference to a
document not yet accepted stays a signal, because two documents are legitimately
drafted at once. And a chain ending on a superseded entity that nothing replaces
keeps its signal in the words it has: that is a real incoherence, the citing
document rests on something with no living form, and it is not a citation anybody
can refresh.

**What must not be built.** No verb rewrites a stored reference, on any path. The
file keeps the identifier its author wrote, which is the promise the format keeps
everywhere else, and a corpus where two documents cite one thing by two names is
a corpus that reads correctly through the chain rather than a corpus to be
normalised. No refusal is added at write time either: an identifier that resolves
is not an error to be corrected before it is stored.
