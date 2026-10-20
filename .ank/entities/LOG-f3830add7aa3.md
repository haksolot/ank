---
id: LOG-f3830add7aa3
type: log
title: Provoked every accept --reanchor refusal with binary 2fedfe9e in a scratch repo built after
created: 2026-10-20T21:52:21Z
author: claude-code/bc28
scope:
  - docs/ratifying.md
about: TASK-bc285882adb4
seq: 1
schema: 4
version: 1
---

 ratify_ref.rs (unsigned, origin bare). Measured: (1) ref already carries it -> exit 6 'error[6]: ADR-..6aa is already carried by refs/ank/ratify/ADR-..6aa (tag aeb4db912913): there is no commit anchor left to move', next 'ank show <id>'. (2) depth-1 clone without the ratify commit -> exit 7 'error[7]: no ratify ADR-..6cc commit is reachable from this clone, so there is nothing to re-anchor from', next 'git fetch --unshallow'. (3) constraint edited after the commit -> exit 6 'error[6]: ADR-..6cc has moved from the anchor its commit a215b2409cfe records (ratified 347e054cb514, now 71b07308e3bc): re-anchoring would sign the edit', next 'ank new adr --supersedes <id>'. (4) proposed, never ratified -> exit 6 'error[6]: ADR-..6ee is proposed, and no ratification of it has an anchor to move', next 'ank accept <id>'. Success: 'reanchored ADR-..6cc -> d8af05f (from commit a215b24)', exit 0; tag message carries the same constraint+scope hash, supersedes, by: human:paul (the runner, not the author), reanchors: <full sha>; HEAD unchanged, 3 commits before and after.
