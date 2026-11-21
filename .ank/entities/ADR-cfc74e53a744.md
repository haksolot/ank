---
id: ADR-cfc74e53a744
type: adr
slug: one-skill-is-installed-a-stub-and-the-binary-ser
title: One skill is installed, a stub, and the binary serves the contract and every method
created: 2026-11-21T20:15:13Z
author: haksolot@vmi3223161
status: accepted
scope:
  - skill/**
  - crates/ank-cli/**
  - crates/ank-contract/**
  - .claude-plugin/**
  - docs/**
constraint: |
  One skill is installed: skill/SKILL.md, a stub. It names when to load it and the one command that prints the contract, ank skills ank, and restates no rule and no verb, so a release leaves no installed copy stale. The contract and each method (plan, drift, loop, tdd, diagnose, herdr) are files under skill/ that the binary embeds and no channel installs. ank skills <name> prints one whole; where the contract lists verbs the binary prints the table ank help prints, and the contract names herdr only where HC_TERMINAL_ID is set. ank help ends on the line naming ank skills ank, and ank skills --install writes the stub alone. claim prints the designated method's text after the criterion, and context names it with the command that prints it. A method is policy and never dispatch: no verb enforces one, no verifier inspects the route, done measures the tree. No method asks for a load record; log --method and the counts ank skills reports stay for the entries already written. Every file under skill/ declares metadata.revision as the hash of its body, recomputed by test, and stays within 180 lines and 1500 words; accept is described and never invited.
amends: [ADR-a8f9c603a0e7, ADR-e1d750884b82, ADR-2fc9b88cc0bd]
supersedes: ADR-e4a5a8873fe3
ratified: e70b724ca903
anchor: 461883ac2f7fb4dcd30a74c89890656e2688f902
verified:
  - by: claude-code/fable-5.2+ratify
    at: 2026-11-21T22:13:08Z
schema: 4
version: 2
---

Issue #833. Supersedes ADR-e4a5a8873fe3, whose system was plural and installed; amends ADR-a8f9c603a0e7 on the load record, ADR-e1d750884b82 on what --install writes, and ADR-2fc9b88cc0bd on herdr being an installed sibling. What each says beyond those points stands.

Measured on 1dbd58ff, 2026-11-21:

- `ank skills` in this corpus: tdd designated 102, diagnose 51, fired 0 for both, since the counter landed on 2026-09-13. The four other siblings are never designated, so their use is unreadable.
- On the maintainer's machine, five of the six installed skills carry a revision the binary no longer has, and ank-herdr is absent. The installed ank-tdd predates the `ank log --method` instruction: the counter could not have moved. An installed copy goes stale at every release and nothing says so.
- Seven files, 36 KB; the contract restates the verb table `ank help` prints.

So the text moves to where it cannot be stale: the binary that runs. The installed file keeps the one thing only it can do, which is trigger, and hands over to a command. The method arrives at the moment it applies, printed by `claim`, where the agent is already reading its criterion.

Decided with the maintainer on 2026-11-21: a stub and not a generated full copy; `ank skills <name>` and no new verb, so ADR-89c1e7146fe5 is untouched; the load record is no longer asked for; herdr is a topic the binary serves, since ADR-8b3045cf11db forbids a second source. `log --method` is left accepted because stale installed siblings in the wild still call it.

Order: the citations of the superseded ADR are re-pointed first (ADR-3b6ba766a42e refuses the accept otherwise), then a human accepts, then the tasks that change behaviour.
