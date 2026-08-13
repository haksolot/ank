---
id: ADR-01b6dd05f0db
type: adr
slug: the-ank-directory-is-reached-only-through-the-cl
title: The .ank directory is reached only through the CLI
created: 2026-08-01T01:58:41Z
status: accepted
scope:
  - .ank/**
constraint: |
  Entities in .ank/ are read and written through the ank CLI only. Reading is ank show, ank find and ank context; writing is ank new, claim, log, done, release, attest, close and accept. No agent opens, greps, lists or edits a file under .ank/ directly. This constrains the agent, not the tool: a human with an editor keeps every power they had, and check remains what notices.
ratified: ea7020bae33d
schema: 1
version: 2
---

`.ank/` should be as opaque to an agent as `.git/` is. Nobody opens `.git/refs/heads/main` to read a branch; they run `git rev-parse`. The reason is not secrecy, it is that the tool knows things the file does not: the budget of section 5, the truncation notice, the freeze anchored where the file's editor cannot reach it, what is claimable and by whom.

Reading directly is now strictly worse than asking. `ank show <prefix>` returns the entity byte for byte, frontmatter and body, for tasks and ADRs alike. `ank find --status open` lists without needing a query. `ank context` serves the constraints in full and says so when it truncates. Measured on 2026-07-31: not one read of an entity requires opening a file.

The write side was the real gap, and it is closed. `ank new` writes a task that needs no hand finishing, verifiers and body included (TASK-bc214fd815b2). `ank attest` performs the one write section 3 permits after `done` (TASK-1f4f7b57039b). `ank new adr --supersedes` writes the field nothing could write (TASK-cc1a2fb317f3), and `accept` performs the succession that field announces (TASK-24b9456d8ec7). One act still has no command -- adding a `blocked_by` to a task that already exists -- and that is a plan someone else owns, which belongs to the human surface.

**This is not the CLI becoming a gatekeeper.** ADR-6b3f19e08a24 says no frozen field may rely on the CLI refusing a write, and nothing here changes that: the freezes stay anchored by hash in artifacts the file's editor does not control, and stay verifiable by anyone. What is constrained is the agent's harness. The distinction matters because the alternative -- making ank enforce it -- is the design that ADR rejected.

Stated here rather than only in SKILL.md because `ank context` serves an ADR whose scope matches, at the top of every session, to every agent in every repository. That is the mechanism this project built for exactly this. SKILL.md said the opposite until now, in as many words, and an agent that followed the file it was given was right to.
