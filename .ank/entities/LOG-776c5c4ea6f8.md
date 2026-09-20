---
id: LOG-776c5c4ea6f8
type: log
title: Confirmed, repeatedly, which is the only way a race says anything.
created: 2026-09-20T20:12:46Z
author: claude-code/opus-5+flake
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-b9701a228f47
seq: 8
schema: 4
version: 1
---



'cargo test --workspace' with .ank/index.db removed before each run: 12 runs on the tree with all three fixes, then 6 more after the last comment-only edit, 18 of 18 exit 0 and no test failed in any of them. The six runs immediately before WAL, on the same machine and the same procedure, failed 3.

The regression test, concurrent_readers_of_a_cold_corpus_of_this_size_all_answer, pins the first two causes and was seen red for them five times out of five. It does not pin the third and says so in its own doc comment: isolating 'a reader is not blocked by a writer' wants a reader with nothing to write meeting a writer that has plenty, and on one corpus those cannot be arranged -- a cold reader always has the same rebuild to write as everyone else. The repeated suite is what measures that one, which is what the criterion asked for.

TASK-e9dfaf187a1b's test, concurrent_readers_of_one_corpus_all_answer, green, and so is TASK-4111dfae8a87's readers_of_a_warm_corpus_take_no_write_lock.

'ank check' exit 0, 'cargo fmt --check' exit 0.

One consequence left outside this diff and filed rather than done quietly: WAL keeps index.db-wal and index.db-shm beside the database and SQLite removes them on a clean close, but a verb killed mid-write leaves them, and init's gitignore line is the literal '.ank/index.db', which matches neither. Measured: 'git status --porcelain -uall' then offers both. TASK-574dee03ba56 carries it. ank-daemon's warm::fingerprint already skips them by prefix, so nothing else in the tree is surprised by the pair.
