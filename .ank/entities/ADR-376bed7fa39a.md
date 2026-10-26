---
id: ADR-376bed7fa39a
type: adr
slug: ank-is-one-executable-with-a-frozen-surface-and
title: Ank is one executable with a frozen surface, and the surface gains --peers
created: 2026-10-26T13:28:12Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - Cargo.toml
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/cli.rs
constraint: |
  The surface of ank is one executable, the verbs ank help prints at the commit that ratifies this decision, ank hooks, and the flag --peers on status, find, context and check. A verb, a workspace crate, a protocol, a reader, a background process, or a mechanism that reads or writes across corpora is added by a supersession of this decision and never beside it, and the superseding document names which of the four properties the addition serves: a perimeter that is a path confronted with the tree, a rule that is an entity anchored by hash and served by scope, a close the tool measures by running the declared verifiers, and a corpus that lives whole in the clone with no service, token or network between an agent and it. ank hooks serves the second: it installs, removes and runs the entries an agent harness calls at its own events. --peers serves the second too: status, find and context add each declared peer's answer, read-only at its default branch and addressed <id>@<peer>, claims included; check --peers makes a declared peer that cannot be read a fault rather than a signal. What exists today (tui, mcp, watch, peers, update, hooks) stays as its own decisions state it and gains no verb, subcommand or crate without that supersession.
supersedes: ADR-24efa7543a16
ratified: f1f9ac087e15
anchor: 8c208f861c9c119187fcf1de4c980a92afed7027
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-26T14:15:43Z
schema: 4
version: 2
---

## Context

ADR-24efa7543a16 froze the surface and admits additions only as its
supersession. Two measured gaps of the multi-repository mechanism need one.

**An orchestrator has no view across the repositories it dispatches to.**
Measured on 2026-10-26 at e04c0dad in a pair `api` / `web` with both
declarations in place: `ank status` in `web` reports "1 constraint(s)" while a
peer ADR also binds it, and `elsewhere` lists only `web`'s claims; `ank find
--type task --all` lists only `web`'s tasks; `ank find @api` reaches one peer at
a time. ADR-a1de673043b4 says aggregation happens "above the repositories", and
for a CLI caller nothing is there: the orchestrator walks the repositories
itself and stitches the answers. The MCP server of ADR-fd98 aggregates the
corpora its reader declared, which serves an MCP client and not an orchestrator
driving executants through a shell.

**CI cannot say that it expects the peers.** With the peer absent, every peer
binding degrades to a warning and `check` exits 0, measured above. That is the
right default for a laptop. A CI that checks the sibling out on purpose has no
way to turn a missing peer into the red it should be, so a decision binding two
repositories is never confronted with either tree in CI.

## Decision

The surface gains one flag, `--peers`, on four verbs, and no verb.

- `status --peers`, `find --peers`, `context --peers` add to the local answer
  the answer of each peer this corpus declares, read only and at the peer's
  default branch (ADR-09b985a5627e), each row addressed `<id>@<peer>`. A peer's
  claims are read from its own `refs/ank/*`, read only: they are shown, never
  arbitrated, and claims stay per repository (ADR-a1de673043b4, ADR-ed3e14d0f991).
- `check --peers` makes a declared peer that cannot be read a fault rather than
  a signal. `docs/ci.md` gives the recipe: check the sibling out beside the
  repository, then `ank check --peers`. Ank still clones nothing
  (ADR-96fe1f9d619a).

It serves **a rule that is an entity anchored by hash and served by scope**: the
rules and the work that bind every corpus an orchestrator dispatches into reach
it in one answer, and in CI a rule binding a peer is confronted rather than
quietly dropped.

Everything else of ADR-24efa7543a16 is restated unchanged.

## Rejected

- **Document a loop over `--repo` instead.** It is what every orchestrator does
  today, and each one re-derives the peer resolution, the default-branch read and
  the `id@peer` addressing by hand.
- **A new verb (`ank peers`).** A second way to ask the questions `status`,
  `find` and `context` already answer.
