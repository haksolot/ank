---
id: ADR-ed3e14d0f991
type: adr
slug: one-live-claim-per-identity-is-per-corpus-and-th
title: One live claim per identity is per corpus, and the claims held elsewhere are named
created: 2026-08-25T16:52:42Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/claim.rs
constraint: |
  The one live claim per identity of section 3 is per corpus, and it is stated rather than left to be discovered: refs/ank/* is per repository, so an identity holds at most one live claim in each corpus it works in, and a refusal that reached across corpora would be an arbitration the refs cannot carry. Nothing refuses on a claim held in another corpus.
  
  What a caller gets instead is the fact. Where the reader can see another corpus without being told about one -- a corpus declared in corpora.yml, or the further corpora a protocol surface was addressed with -- claim names the live claims that identity already holds elsewhere, with the corpus each is in, and takes the task anyway. A corpus the reader was not told about is not searched for and is never named. This is ADR-052accd6e3b2's rule applied across a boundary instead of across a scope: the fact is stated at claim time and never refused.
schema: 4
version: 1
---

SPEC-183dd2eb4dc0 left this open in writing, and said why it had to be settled
somewhere: "The one-claim-per-identity rule is silently per repository, since the
refusal reads the refs of the resolved root, so one identity holds a claim in
every repository at once and nothing notices -- doubtful by the rule's own
reasoning, which is that a caller already holding one is not available."

The behaviour is measured and not inferred: one identity took a claim in two
corpora a second apart, and each `status` reported its own with no mention of
the other.

## Why the rule stays per corpus

The objection is real -- an agent holding five leases in five repositories is
not available five times over -- and the answer to it cannot be a refusal. A
refusal has to be arbitrated, and the only mechanism this project trusts for
arbitration is a compare-and-swap on a ref that both parties can reach.
`refs/ank/*` is per repository by ADR-4e7c and ADR-a1de673043b4, so a
cross-corpus refusal could only be enforced by whatever process happened to see
both corpora at once. Two such processes would not see each other, which is an
arbitration that holds until the moment it matters.

## Why it is named rather than counted

ADR-052accd6e3b2 met the same shape and answered it once already: two live
claims whose scopes intersect are named at claim time and never refused, because
the tool refuses on state and the state here is somebody else's business to
weigh. Availability is exactly that kind of fact. A person or an agent taking a
sixth task while five leases are outstanding should be told, in the answer to the
command they typed, and then allowed to proceed.

**And it is named only where the reader was already told where to look.** Nothing
walks a filesystem for a corpus (ADR-621a7fd96ce1, ADR-96174f1ac2b7), so a
single-corpus caller with no declaration sees exactly what it sees today: this
adds a line where a reader has declared others, and adds nothing anywhere else.
