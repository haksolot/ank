---
id: ADR-bb9ccdba9be7
type: adr
slug: accept-rescope-re-signs-the-scope-of-a-ratified
title: accept --rescope re-signs the scope of a ratified entity whose constraint did not change
created: 2026-10-24T17:39:48Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-contract/src/verbs.rs
constraint: |
  ank accept <id> --rescope is a human act, signed where the repository signs, that ratifies the scope an accepted ADR or spec now declares: amend may edit an accepted entity's scope, which check reports as not caught up until the rescope; the new tag replaces refs/ank/ratify/<id> with compare-and-swap and carries the new anchor, the previous anchor and the previous scope list; accept refuses unless the current constraint (or body) hashed with the previous scope equals the previous anchor, and check verifies the same and faults on a mismatch; the identifier, the status and every citation stay.
amends: [ADR-3094538d831e, ADR-5e47007a75be]
schema: 4
version: 1
---

## Context

A ratification anchors the hash of an ADR's constraint and scope (a spec's body
and scope) in a signed tag at refs/ank/ratify/<id> (ADR-5e47007a75be). Measured
2026-10-24 in a lab repository: after `git mv` of the one file an accepted ADR
governs, `ank amend --scope` refuses (exit 6, the scope is anchored) and `ank
accept` refuses (exit 6, already ratified); both point at a supersession. When
the code moves and the decision does not, the only way to move the perimeter is
a new identifier, a new signature and every citation re-pointed.

The perimeter is part of what a human ratified, so moving it must stay a human,
signed act. What it need not be is a new decision.

## Decision

1. **`ank accept <id> --rescope`** ratifies the scope an accepted ADR or spec
   now declares, and nothing else. It is a human act, signed where the
   repository signs, like accept.
2. Before rescoping, the scope is edited on the accepted entity through `ank
   amend --scope/--drop-scope`, which stops refusing on an accepted entity and
   instead leaves it as a file that has not caught up, which check reports
   until the rescope is signed.
3. The new tag replaces refs/ank/ratify/<id> with compare-and-swap, pointing at
   the blob as written, and its message carries the new anchor, the previous
   anchor and the previous scope list. accept refuses unless the hash of the
   current constraint (or body) with the previous scope list equals the
   previous anchor: the decision itself provably did not change. Anything else
   is a supersession.
4. check verifies a rescoped ratification on the same terms: the signature,
   the new anchor against the file, and the previous anchor recomputed from the
   current constraint and the previous scope. A mismatch is a fault.
5. The identifier, the status and every citation stay as they are.

## Consequences

- ADR-5e47007a75be named accept --reanchor as the one act that moves an anchor
  into a ref; rescope is a second act, on a ref that already exists.
- ADR-3094538d831e proposed a supersession for a dead scope on an accepted ADR;
  for a rename the proposal becomes accept --rescope.
- SPEC-df28 (the CLI surface), SPEC-9796 (the tag message) and SPEC-a459 (how
  check reaches a ratification) state the old rule and are superseded with the
  new one, before or with the signature.

## Rejected

- Letting amend change an accepted scope with no signature: exactly what the
  anchor exists to stop.
- Keeping the previous tag object under a second ref to verify the chain: the
  previous anchor and scope in the new message prove the constraint unchanged
  without one.
- A supersession with a carried-over constraint: the cost measured above, for
  no change in the decision.
