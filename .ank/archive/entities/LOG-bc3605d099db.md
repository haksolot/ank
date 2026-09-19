---
id: LOG-bc3605d099db
type: log
title: A call names its corpus by the identity of ADR-621a7fd96ce1, resolved in
created: 2026-08-25T20:09:23Z
author: claude-code/opus-5+multi-corpus
scope:
  - crates/ank-mcp/**
about: TASK-2f31789f6af2
seq: 1
schema: 4
version: 1
---

 crates/ank-mcp/src/corpora.rs. Resolution returns ONE path and nothing above or below it ever holds two corpora at once: declared map lookup, then the startup corpus, then a refusal. Reachable = corpora.yml (read with serde_yaml and ank_contract::events::user_dir, no new dependency, no link to ank-cli) plus the corpus ank mcp was addressed with, whose own identity is asked of the binary once via ank status --json and cached. Both halves sit behind a OnceCell that a call omitting the argument never touches, so a single-corpus client reads no file, spawns nothing extra and sees byte-identical bytes. A value that is not 40 hex is refused before any lookup, which is what stops a path reaching --repo through the back door; an identity nobody declared is refused with error[9] and 'ank config --user corpora.<id> <path>', rendered through Outcome::to_result so a surface refusal and a spawned one cannot drift in shape. Code 9 follows ank-daemon/src/declare.rs, which uses Environment for every corpora-declaration failure; a declared path that is not a directory reuses repo.rs's own sentence and its Generic code. A declared corpus that cannot be read: consistent with TASK-1317adb617e8, the map is read silently and never fails a call, but the reason is kept so the refusal says 'could not be read' rather than accusing the caller of naming nothing.
