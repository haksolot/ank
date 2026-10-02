---
id: TASK-a8d3219e84fb
type: task
slug: the-index-is-one-file-per-schema-and-the-archive
title: The index is one file per schema, and the archived digests carry over
created: 2026-10-02T18:08:29Z
author: claude-code/opus-5.5+plan
status: in_progress
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-daemon/src/warm.rs
  - crates/ank-cli/tests/**
  - docs/format.md
  - docs/quickstart.md
  - docs/watch.md
blocked_by: []
done_criteria: |
  Through the binary: a verb run in a corpus writes its index at .ank/index.db.<N>, N the binary's index schema version, and never creates .ank/index.db. A pre-existing .ank/index.db and a .ank/index.db.<M> for another schema M are left byte-identical by every verb, and the binary's own index is rebuilt once and then reused across three runs. When the binary creates its file while a .ank/index.db.<M> of an older schema holds an archived digest, the digest is carried over: an archived file edited before the upgrade still makes ank check exit 8 after it. After a verb runs, git status shows no index file. docs/format.md, docs/quickstart.md and docs/watch.md name the file as .ank/index.db.<N>.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Under ADR-3db9735a7036, carried by SPEC-3252ad5a72e3. Measured during the
2026-10-02 parallel run: a schema bump made the PATH binary and the tree's
binary rebuild each other's `.ank/index.db` in a loop.

The scope is wide on purpose. `DB_FILE` lives in index.rs, but seventeen test
files name `index.db` to force a cold rebuild, and the daemon's warm.rs matches
files by that prefix. The tests should go through one helper rather than
spelling the new name seventeen times.

The digest carry-over is the one part that is not a rename: docs/format.md
explains why the archived digest is the only state the index holds, and why
losing it silently lets an edited archive pass check.
