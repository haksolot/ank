---
id: LOG-585bc2fe03be
type: log
title: "How a first accept keeps its tag without completing, measured: write_ratification does update-ref,"
created: 2026-11-20T15:36:38Z
author: claude-code/opus-5.5+task-4937-accept-ratify-ref
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/accept_tag.rs
about: TASK-4937d03297e9
seq: 2
schema: 4
version: 1
---

 push, THEN store.write. Any store.write failure after the push leaves the ref on both sides and the file proposed, with no rollback. Reproduced: a stale .ank/entities/.<id>.md.lock -> accept waits 10s, exit 1 'lock ... still held after 10s', after: local=remote=2d78e9e, file sha unchanged, status proposed. Same path for a VersionConflict (another writer between prepare and write) and, on Windows, a rename refused because another process (tui, editor, AV) holds the file. Also unguarded: git::remote(cwd)? and git::push_ref(...)? erroring (spawn failure) between update-ref and the rollback, and a kill between update-ref and the file write. The reporter's ref was local-only, which these post-push paths do not produce; a local-only leftover with no rollback needs push_ref/remote to return Err, or a kill/Ctrl-C between update-ref and push. Not fixed here (scope is the leftover handling); the fix makes every one of these recoverable by a second accept.
