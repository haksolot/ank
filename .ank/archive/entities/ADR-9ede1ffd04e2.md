---
id: ADR-9ede1ffd04e2
type: adr
slug: one-surface-and-policy-lives-above-the-tool
title: One surface, and policy lives above the tool
created: 2026-08-01T18:29:38Z
author: seanl@sean-laptop
status: superseded
scope:
  - crates/ank-cli/**
  - skill/**
  - docs/ank-spec-v1.1.md
constraint: |
  The CLI exposes one surface: every verb is available to every caller, and the CLI refuses on state, never on identity. The only hard authority line is the signed ratification commit produced by accept. Who uses which verb is policy, and policy lives above the binary: SKILL.md documents the loop for agents and its content is frozen, harness hooks enforce where enforcement is real, roles in config.yml remain advisory. ank help presents the loop first, the rest layered.
supersedes: ADR-3859eb46bdc3
ratified: 5355952b8cd1
schema: 2
version: 3
---

ADR-3859eb46bdc3 froze the agent surface at eight verbs and sent everything else to the human side. This supersedes it because the split it protected was never a boundary: the CLI told callers apart by ANK_AGENT, a variable the caller sets itself, and section 8 of the specification already names the consequence, permissions are advisory. A wall whose bricks are self-declared identity is a sign, not a wall.

What the eight-verb freeze actually protected is a token budget: SKILL.md is loaded permanently, so the surface an agent reads about must stay small. That protection moves to where it operates. The content of SKILL.md is frozen at the loop (context, claim, show, log, done, new, find, release), and growing what SKILL.md teaches costs a succession of this ADR, exactly as growing the verb list did.

Enforcement moves to where it is real. A harness hook that blocks a verb cannot be talked out of it by an environment variable; the PreToolUse hook already guarding .ank/ in this repository is the working example. The binary keeps refusing on state, frozen hash diverged, task blocked, proof missing, accept off the default branch, because state refusals apply to everyone equally and are the refusals that carry guarantees. The signed ratification commit remains the single hard line of authority, and it is a proof requirement, not a role check.

Rejected: keeping two surfaces and adding the new human verbs to the human side only. It preserves a fiction at the cost of the git property this project borrows everywhere else, one surface with policy above it. Rejected: dropping roles from config.yml entirely. They are three lines, they catch honest drift, and an unknown identity defaulting to least privilege is a sane default; they are kept and named what they always were, advisory.

Consequence: verbs serving human ergonomics, status, edit, graph, scope, the interactive form of new, the read form of log, enter the one surface without ceremony. What an agent is taught remains the loop; what a human may type is everything.
