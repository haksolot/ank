---
id: ADR-fd98f4bc6dea
type: adr
slug: the-protocol-surface-is-a-verb-of-the-one-binary
title: The protocol surface is a verb of the one binary, and it speaks for the corpora its reader declared
created: 2026-08-25T16:52:02Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-mcp/**
constraint: |
  A protocol surface exposes every verb the CLI dispatches, generated from the same table the CLI dispatches from, or it does not exist. No curated subset, under any protocol. It refuses on state exactly as the CLI does and never on identity, and it carries the CLI's exit codes as the reason for a refusal. It writes under a typed process identity.
  
  It is a verb, ank mcp, and not a sibling binary: ank ships one executable, and crates/ank-mcp is a library linked into it. One executable is not one process. The verb still spawns ank <verb> --repo <corpus> --json per call, so there is no second dispatch path and every refusal remains the one the binary gave.
  
  It may speak for several corpora, and never for a merged one. Every tool carries an optional corpus argument holding the repository identity of ADR-621a7fd96ce1; absent, the call goes to the corpus the process was addressed with at startup. The set a server may reach is what the reader declared in corpora.yml (ADR-96174f1ac2b7) plus that startup corpus, and nothing is discovered: a corpus argument naming an identity the reader did not declare is refused by name. Each corpus is addressed on its own, the way --repo addresses one. Claims stay per clone and no server arbitrates across clones: there is no merged claim space, no claim held on a client's behalf, and no pooling of clients under one identity.
supersedes: ADR-372b82af1ec7
ratified: 7f922fe30d79
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-25T17:19:54Z
schema: 4
version: 2
---

ADR-372b82af1ec7 is kept whole except in two clauses, and both are amended here
rather than in two documents because the format allows one successor per
document and both are edits to the same ratified text.

## The sibling binary, and the reason that expired

That decision chose a sibling binary over a verb, and it named its reason: "The
live one-surface decision says the verbs themselves do not change, and what
`skill/SKILL.md` teaches is frozen by revision hash. A twenty-third verb would
cost a second supersession and a second human signature, and would buy nothing."

Neither half is true any more. The one-surface chain ran
ADR-9ede1ffd04e2 to ADR-c656cbcc33a9 to ADR-e17e1bbd93ff to ADR-f61e2d2c75e8 to
ADR-5dd7b4a9c875 and ends on ADR-91b77f036884, whose constraint states that no
skill's content is frozen by revision hash any more; and the sentence about the
verbs not changing left the live constraint somewhere along that chain.
ADR-8bd76e8d7c4e then added a verb and argued the case in the opposite
direction, for an audience this one shares: a separate executable is invisible
to precisely the people it exists for, and has to be distributed, documented and
discovered as a third thing.

**And the price it avoided was paid anyway, twice.** ADR-e39a44f80e0e exists
only because the second binary reached nobody: `release.yml` built one,
`install.sh` unpacked one, no document mentioned it. That is a rule, a workflow
matrix and an installer branch, all of whose work is to make a second file
arrive beside the first. A verb cannot fail to arrive.

## One executable is not one process

The trade ADR-372b82af1ec7 took deliberately survives untouched, and this is the
clause that keeps it: `ank mcp` spawns `ank <verb>` exactly as `ank-mcp` did.
Linking `ank-cli` into the surface would re-derive every refusal, and anything
re-derived can differ; spawning inherits them by construction. So what is folded
here is the *file*, never the dispatch. The same reading applies to
`crates/ank-tui`, which is linked in and still reaches the corpus only by
running the binary.

## Several corpora, and why that is not a merged claim space

The clause this amends reads "It speaks for exactly one corpus", and its stated
reason is that `refs/ank/*` is per repository, so a server that merged two claim
spaces would be inventing an arbitration the refs cannot carry. That reason
forbids merging. It does not forbid multiplexing, and ADR-621a7fd96ce1 already
permits the shape: a reader may hold several corpora at once and present them
together, each addressed on its own.

So each call names its corpus or takes the default, and every call still becomes
`ank --repo <one corpus>`. Nothing is pooled, nothing is merged, and the ban on
a merged claim space is carried forward in the same words. What a multi-corpus
server does acquire is one identity holding a lease in several corpora at once,
which SPEC-183dd2eb4dc0 left open on purpose; that question is settled by its
own decision and not by this one.

## What is still true

Shell remains the common denominator and remains what the skill teaches. This
does not make a protocol the preferred route for an agent that has a shell.
`skill/SKILL.md` does not move: an agent has `context`, `find` and `show`, and
teaching it a protocol it should not reach for would spend the contract to no
end -- the same reading ADR-8bd76e8d7c4e gave for `tui`.
