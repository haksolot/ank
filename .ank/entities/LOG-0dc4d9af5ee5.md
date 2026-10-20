---
id: LOG-0dc4d9af5ee5
type: log
title: Measured the claims docs/ratifying.md now makes, in a scratch repo with a bare remote. (1) With
created: 2026-10-20T16:23:30Z
author: claude-code/1c35
scope:
  - .github/workflows/ci.yml
  - docs/**
  - CLAUDE.md
  - CONTRIBUTING.md
  - skill/**
about: TASK-1c350fc2670f
seq: 2
schema: 4
version: 1
---

 refs/ank/ratify/ADR-34772d1d1d66 present and the file rewritten to status: proposed, context src listed it under 'CONSTRAINTS (1 active)' and check signalled 'ratified by refs/ank/ratify/ADR-34772d1d1d66, but .ank/entities/ADR-34772d1d1d66.md still reads proposed: the file has not caught up'; with the ref deleted, context listed 'PROPOSED (1, non-binding)'. (2) origin set to a missing path: accept ADR-58dadd44df5e printed 'error[9]: refs/ank/ratify/ADR-58dadd44df5e did not reach origin, and nothing was kept', exit 9, file sha1 c6057ea0 before and after, no ref written. (3) Plumbing recipe: hash-object, tag -a, update-ref, tag -d, then 'git push --force-with-lease=refs/ank/ratify/<id>: origin refs/ank/ratify/<id>' created the ref (exit 0); the same push once the local ref pointed elsewhere was '! [rejected] ... (stale info)', exit 1. Out of scope and filed as TASK-79b9b5195c8e: docs/exit-codes.md is generated from crates/ank-contract/src/exit.rs, which still gives accept off the default branch as the example of exit 7. The criterion names skill/ank-plan/SKILL.md; the file is skill/plan/SKILL.md (the skill is published as ank-plan), edited there. skill.rs's the_skill_describes_accept_without_inviting_it asserts 'default branch' anywhere in SKILL.md: it stays green through 'the default branch is where work arrives', no longer through the accept paragraph.
