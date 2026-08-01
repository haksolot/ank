---
id: ADR-3859eb46bdc3
type: adr
slug: the-agent-surface-is-eight-verbs-show-included
title: The agent surface is eight verbs, show included
created: 2026-08-01T01:54:59Z
status: accepted
scope:
  - crates/ank-cli/**
  - skill/**
constraint: |
  The agent surface is exactly: context, claim, log, done, new, find, release, show. No verb is added to it without superseding this ADR in turn. Any other new functionality lands on the human side or in the format.
supersedes: ADR-2f8a61c04b7d
ratified: 42e9d5147281
schema: 1
version: 2
---

ADR-2f8a61c04b7d said no verb is ever added. This adds one, so it is superseded rather than amended: an accepted constraint is not edited in place, and the agent role could not do it anyway.

The reason is a hole found by trying to close `.ank/` to direct reads. `context` serves the criterion and the constraints under the budget of section 5; it never serves a task body. The bodies are where the reasoning lives -- why an approach was rejected, why a task came back once already, which trap cost the last agent an hour. With `cat` forbidden and `show` human, an agent had no way left to inherit any of it, and SKILL.md said `cat` is your `show` precisely because `show` was out of reach.

`show` is the only unbounded reader in the system, and that is the argument that put it on the human side. The argument was about output size. Section 5 answers size with a budget and a truncation notice that says what it cut. Withholding the reasoning entirely is the worse trade.

Loop rather than off-loop: reading the task you just claimed is part of executing it, not a detour around it. The order in `help` says so -- context, claim, show, log, done.

Eight is not a slope. The constraint names the surface as eight and requires a succession for any ninth, which is what keeps the freeze a decision rather than a formality any task can spend. The clause is the whole point of superseding instead of editing: the next verb costs an ADR and a human signature, not a commit.
