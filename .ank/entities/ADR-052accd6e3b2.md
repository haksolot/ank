---
id: ADR-052accd6e3b2
type: adr
slug: two-live-claims-whose-scopes-intersect-are-named
title: Two live claims whose scopes intersect are named at claim time, and never refused
created: 2026-08-13T16:21:38Z
author: claude-code/2.1.229
status: accepted
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - docs/ank-spec-v1.1.md
constraint: |
  claim names every live claim whose scope intersects the one being taken, with the paths in common, and takes the task regardless. The same computation is what find --free filters on. Scope overlap is a signal and never a refusal: refusing on it would turn a coarse glob into a lock.
ratified: ac8836322753
schema: 3
version: 2
---

Claims coordinate tasks, and at that job they are correct and cheap. Three agents
worked this corpus for a full session with zero contended tasks. What the claim
plane says nothing about is **files**, and that is where the session's real cost
landed.

The default branch moved six times under one agent, across pull requests 91, 97,
98, 101, 102, 104 and 105, and produced two merge conflicts. Both were in
`crates/ank-cli/tests/cli.rs`, and both had the same cause: two agents holding
two disjoint tasks each appended a block of tests to the end of the same file.
Nothing warned either of them. The claims were on disjoint tasks, correctly; the
edits were on one file, and the plane has nothing to say about files.

**The information needed to warn is already in the corpus.** Both tasks declared
`crates/ank-cli/**`. A scope intersection between two live claims is computable
from the entities and the refs, with no network call and no new field. The agent
that filed this finding ranked it as the one change that would have prevented
both of its conflicts.

The second-order damage is why a warning is worth more than it looks: resolving
one textually trivial conflict mechanically dropped a closing brace and the file
stopped compiling. A conflict that is trivial to read is not trivial when a
script resolves it at speed.

**A signal, never a wall, and the reason is measured.** Scope overlap is coarse:
`crates/ank-cli/tests/**` in one claim locks every task that touches any test,
and one held task made five of seven remaining candidates unworkable in a
session. Refusing on that would make the glob a mutex and would push agents to
declare narrower scopes than the truth to get past it -- which is the failure
mode a guard should never have. Naming it costs one line and leaves the decision
where it belongs.

`find --free` is the same computation read from the other side: an agent choosing
work wants the candidates that do not collide, and today it reads seven task
files by hand to find out. One agent did exactly that.

**This is not the tail of the problem, and the ADR says so rather than implying
it is.** The expensive collisions in that session were semantic, not textual: the
same defect written into two files by two agents, in different functions, with no
git conflict at all -- caught by a test and by nothing else. Scope overlap does
not detect that, and ADR-f8a6cf65160e is what names it. What this buys is the
cheap half, for the price of a line.
