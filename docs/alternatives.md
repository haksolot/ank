# How ank compares

Other tools curate a layer over a codebase too. Here is where ank differs from
each, and what each comparison costs it.

## Against retrieval

RAG answers *what looks relevant to this query*; `ank context src/auth/` answers
*what applies to this path*. The first ranks by similarity, the second is a
set-membership test, and for a constraint that is the whole point, because a rule
that should have bound but ranked seventh did not bind, and nothing says so. No
embeddings, no index service, no re-indexing per commit.

The cost is real: somebody has to have written the constraint and its scope. Ank
replaces the wiki page nobody found, and a corpus nobody curated still needs
search.

## Against an LLM-maintained wiki

Karpathy's [pattern][wiki] is three layers (raw sources, a wiki the model owns, a
schema file) and three operations: ingest, query, lint. Ank has that shape and
differs on what the middle layer *is*.

A wiki page is derived from sources and can be regenerated if lost; an ADR records
a choice that has none. Somebody decided opaque sessions over JWTs, and the record
is the only copy, and nothing can re-ingest its way back to it. Hence hash-anchored
and signed at ratification rather than rewritten, and a lint that is mechanical,
where the wiki's lint is a model re-reading for contradictions.

## Against OKF

Google's [Open Knowledge Format][okf] is the closest relative, reached
independently: markdown with YAML frontmatter, no server, no SDK, identity carried
by the path, the format as the contract so producer and consumer stay swappable.
Its v0.2 trust signals are the same instinct as ank's proofs, and ank has adopted
two outright: the actor convention and `verified`.

One axis diverges, deliberately on both sides. OKF tells consumers to accept a
document whatever it lacks; ank rejects an unknown field. OKF optimises for
knowledge crossing organisations, where a rejected document is knowledge lost; ank
optimises for a criterion that cannot be quietly weakened, where an
accepted-but-malformed file is a rule that silently stopped applying.

## Against a process-skills workflow

[Matt Pocock's skills][pocock] are the richest example: prompts covering
the whole cycle, interviewing the human until the spec is precise,
cutting it into tickets, driving the implementation test-first, reviewing before
merge. The skill is the method, and the agent is held to it while it works.

Ank ships methods too, and holds the other end of them. Seven skills travel
inside the binary, built from `skill/`: the contract every agent loads, in
`skill/SKILL.md`, and six siblings beside it, in `skill/*/SKILL.md`. Five of
them teach one activity of the work each, in `skill/plan`, `skill/drift`,
`skill/loop`, `skill/tdd` and `skill/diagnose`, and `skill/herdr` teaches the
agent orchestrating the others from a herdr pane. A task may name any
sibling, and planning designates one of the five. `ank new --method tdd` writes the designation into it, `ank context`
names it and the sibling to load beneath the criterion once the task is
claimed, `ank log --method tdd` records that it fired, and `ank skills` counts
designations against firings per method. A name no sibling carries is refused
at exit 7, when the task is written.

No verb enforces one. `done` leaves `method` unread: the criterion is frozen
by hash at claim, `done` runs the declared verifiers itself and records what
they returned, and the proof records the route by which it arrived. A verifier
measures the tree, whatever process produced it, which is the point
(ADR-e4a5a8873fe3). An agent graded on its process learns to fake the process.
An agent graded on the tree has to change the tree.

So the method is taught and the outcome is measured, and the cost is the mirror
of retrieval's: a designation nobody honours is a line of frontmatter, and only
the tree says whether the work was done. A team that wants a practice ank does
not ship writes a skill for it, and ank still verifies only the outcome.

## What it costs to run

Two numbers, each beside its source. The seven skills cost 334 tokens in every
session, which is their frontmatter: the `name` and `description` a harness keeps
loaded from the start, the body being read when a skill is invoked. That is 38
tokens for the contract and 41 to 57 for each sibling, counted with `cl100k_base`
over the seven `SKILL.md` under `skill/` by this command, run at the root of the
repository with `tiktoken` installed:

```sh
python3 -c 'import glob,re,tiktoken; e=tiktoken.get_encoding("cl100k_base"); n=[len(e.encode("".join(re.findall(r"^(?:name|description): .*\n", open(p).read().split("---\n")[1], re.M)))) for p in ["skill/SKILL.md", *sorted(glob.glob("skill/*/SKILL.md"))]]; print(n, sum(n))'
```

And orientation is bounded at 8000 characters, the `context_budget` that
`.ank/config.yml` sets and that [the config keys](config-keys.md) list as its
default.

[okf]: https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf
[pocock]: https://github.com/mattpocock/skills
[wiki]: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
