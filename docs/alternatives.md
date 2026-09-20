# How ank compares

Ank is not the first curated layer over a codebase. Here is where it differs, and
what each comparison costs it.

## Against retrieval

RAG answers *what looks relevant to this query*; `ank context src/auth/` answers
*what applies to this path*. The first ranks by similarity, the second is a
set-membership test, and for a constraint that is the whole point, because a rule
that should have bound but ranked seventh did not bind, and nothing says so. No
embeddings, no index service, no re-indexing per commit.

The cost is real: somebody has to have written the constraint and its scope. Ank
does not replace search over a corpus nobody curated; it replaces the wiki page
that was never found.

## Against an LLM-maintained wiki

Karpathy's [pattern][wiki] is three layers (raw sources, a wiki the model owns, a
schema file) and three operations: ingest, query, lint. Ank has that shape and
differs on what the middle layer *is*.

A wiki page is derived from sources and can be regenerated if lost; an ADR records
a choice that has none. Somebody decided opaque sessions over JWTs, and the record
is the only copy, and nothing can re-ingest its way back to it. Hence hash-anchored
and signed at ratification rather than rewritten, and a lint that is mechanical
rather than a model re-reading for contradictions.

## Against OKF

Google's [Open Knowledge Format][okf] is the closest relative, reached
independently: markdown with YAML frontmatter, no server, no SDK, identity carried
by the path, the format as the contract so producer and consumer stay swappable.
Its v0.2 trust signals are the same instinct as ank's proofs, and ank has adopted
two outright: the actor convention and `verified`.

One axis diverges, deliberately on both sides. OKF tells consumers never to reject
a document for what it lacks; ank rejects an unknown field. OKF optimises for
knowledge crossing organisations, where a rejected document is knowledge lost; ank
optimises for a criterion that cannot be quietly weakened, where an
accepted-but-malformed file is a rule that silently stopped applying.

## Against a process-skills workflow

[Matt Pocock's skills][pocock] are the richest example: two dozen prompts
covering the whole cycle, interviewing the human until the spec is precise,
cutting it into tickets, driving the implementation test-first, reviewing before
merge. The skill is the method, and the agent is held to it while it works.

Ank ships methods too, and holds the other end of them. Six skills travel
inside the binary: the contract every agent loads, and five siblings teaching
one activity each, plan, drift, loop, tdd and diagnose. A task may name any of
the five. `ank new --method tdd` writes the designation into it, `ank context`
names it and the sibling to load beneath the criterion once the task is
claimed, `ank log --method tdd` records that it fired, and `ank skills` counts
designations against firings per method. A name no sibling carries is refused
at exit 7, when the task is written rather than when the work starts.

What no verb does is enforce one. `done` never reads `method`: the criterion
is frozen by hash at claim, `done` runs the declared verifiers itself instead
of believing a report, and the proof records the route by which it arrived. A
verifier measures the tree and not the process, which is the point
(ADR-e4a5a8873fe3). An agent graded on its process learns to fake the process.
An agent graded on the tree has to change the tree.

So the method is taught and the outcome is measured, and the cost is the mirror
of retrieval's: a designation nobody honours is a line of frontmatter, and only
the tree says whether the work was done. A team that wants a practice ank does
not ship writes a skill for it, and ank still verifies only the outcome.

## What it costs to run

Two numbers rather than an adjective. The six skills cost about 280 tokens in
every session, which is their frontmatter: the `name` and `description` a harness
keeps loaded whether or not a skill is ever invoked, the body being read only when
it is. That is 38 tokens for the contract and 41 to 57 for each sibling, counted
with `cl100k_base` over the six `SKILL.md`. And orientation is bounded at 8000
characters.

[okf]: https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf
[pocock]: https://github.com/mattpocock/skills
[wiki]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
