# How ank compares

Ank is not the first curated layer over a codebase. Here is where it differs, and
what each comparison costs it.

## Against retrieval

RAG answers *what looks relevant to this query*; `ank context src/auth/` answers
*what applies to this path*. The first ranks by similarity, the second is a
set-membership test — and for a constraint that is the whole point, because a rule
that should have bound but ranked seventh did not bind, and nothing says so. No
embeddings, no index service, no re-indexing per commit.

The cost is real: somebody has to have written the constraint and its scope. Ank
does not replace search over a corpus nobody curated; it replaces the wiki page
that was never found.

## Against an LLM-maintained wiki

Karpathy's [pattern][wiki] is three layers — raw sources, a wiki the model owns, a
schema file — and three operations: ingest, query, lint. Ank has that shape and
differs on what the middle layer *is*.

A wiki page is derived from sources and can be regenerated if lost; an ADR records
a choice that has none. Somebody decided opaque sessions over JWTs, and the record
is the only copy — nothing can re-ingest its way back to it. Hence hash-anchored
and signed at ratification rather than rewritten, and a lint that is mechanical
rather than a model re-reading for contradictions.

## Against OKF

Google's [Open Knowledge Format][okf] is the closest relative, reached
independently: markdown with YAML frontmatter, no server, no SDK, identity carried
by the path, the format as the contract so producer and consumer stay swappable.
Its v0.2 trust signals are the same instinct as ank's proofs, and ank has adopted
two outright — the actor convention and `verified`.

One axis diverges, deliberately on both sides. OKF tells consumers never to reject
a document for what it lacks; ank rejects an unknown field. OKF optimises for
knowledge crossing organisations, where a rejected document is knowledge lost; ank
optimises for a criterion that cannot be quietly weakened, where an
accepted-but-malformed file is a rule that silently stopped applying.

## What it costs to run

Two numbers rather than an adjective. The skill costs about 58 tokens in every
session, which is its frontmatter — the `name` and `description` a harness keeps
loaded whether or not the skill is ever invoked, the body being read only when it
is. And orientation is bounded at 8000 characters.

[okf]: https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf
[wiki]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
