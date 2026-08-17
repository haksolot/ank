---
id: ADR-372b82af1ec7
type: adr
slug: a-protocol-surface-is-a-generated-full-surface-p
title: A protocol surface is a generated full-surface passthrough, or it does not exist
created: 2026-08-17T05:14:01Z
author: claude-code/2.1.233+integration-contract
status: accepted
scope:
  - crates/ank-cli/src/cli.rs
constraint: |
  A protocol surface exposes every verb the CLI dispatches, generated from the same table the CLI dispatches from, or it does not exist. No curated subset, under any protocol. It refuses on state exactly as the CLI does and never on identity, and it carries the CLI's exit codes as the reason for a refusal. It speaks for exactly one corpus, addressed the way --repo addresses one: claims stay per clone, and no server arbitrates across clones -- a deployment over several repositories is several corpora addressed separately and never one merged claim space. It writes under a typed process identity. It ships from this repository, and it is a sibling binary rather than a verb, so the verb list and skill/SKILL.md are untouched.
supersedes: ADR-1713af205186
ratified: 629f3ae1a72d
schema: 3
version: 4
---

## What this supersedes, and what it keeps

ADR-1713af205186 refused an MCP server, and refused with it "no subset of the
verbs exposed over any protocol". It was right, and it was right for a reason
that has not expired: the proposal it rejected exposed four verbs out of
twenty-two, which rebuilds under a second protocol the agent-surface split that
was abolished once already, and rebuilds the worse half of it -- a caller
reached through the protocol could not amend, could not propose an ADR, could
not check the corpus.

That refusal is kept, in full, as the shape of what is now allowed. What changes
is only that the conditions it wrote for itself are met.

## Its two conditions, answered

**"A full-surface passthrough, generated from the same dispatch table the CLI
uses so it cannot fall behind."**

The verb table moves into a crate that both the CLI and the protocol surface
consume, and the surface is generated from it. Not kept in step by review, not
tested for parity -- generated, so that a verb the table does not carry is a
verb neither surface has, and a verb it carries is a verb both have. This is
also what the decision on the machine surface already requires for `--json`, so
the protocol surface inherits it rather than asking for it.

**"A statement of what it does with claims."**

It does nothing with them that the CLI does not. A server process speaks for one
corpus, named the way `--repo` names one, and every claim it takes is the claim
the CLI would have taken in that clone, on `refs/ank/claims/<id>`, arbitrated by
the same compare-and-swap against the same remote. It does not hold a claim on a
client's behalf, it does not pool clients under one identity, and it does not
merge the claim spaces of two clones -- because `refs/ank/*` is per repository,
and a server that pretended otherwise would be inventing an arbitration the refs
cannot carry.

A deployment over several repositories is therefore several corpora, addressed
separately, presented together by whatever is above them. That is the same
answer ADR-a1de673043b4 gives for federation, and it is the same answer for the
same reason.

## Why a sibling binary and not a verb

The live one-surface decision says the verbs themselves do not change, and what
`skill/SKILL.md` teaches is frozen by revision hash. A twenty-third verb would
cost a second supersession and a second human signature, and would buy nothing:
the requirement is that the surface be generated from the dispatch table, and a
sibling binary in this workspace reads that table directly. It is the stronger
reading of the condition, not the weaker one.

## What is still true

Shell remains the common denominator, and remains what the skill teaches. This
decision does not make a protocol the preferred route for an agent that has a
shell; it makes one exist for a client that has none, which is the narrower and
honest claim the original ADR said was the only real one.
