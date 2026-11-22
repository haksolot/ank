---
id: LOG-d96ecf39a1fd
type: log
title: "Measured through the binary (done_and_close_move_the_entries_of_the_task_they_end_into_the_archive):"
created: 2026-11-21T22:37:56Z
author: claude-code/opus-5.5+task-ed79-archive-at-done
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/**
  - skill/**
  - CONTRIBUTING.md
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/archive_digest.rs
about: TASK-ed7990dab03c
seq: 6
schema: 4
version: 1
---

 after done, 0 hot entries about the task and before+1 archived, bytes equal; close the same; attest+amend add exactly 2 archived, 0 hot; a done refused by its verifier (exit 5) leaves the hot set unchanged and archives nothing; check exit 0 with no fault and no 'ank archive' line on the branch and after merge --no-ff into main; ank log --json ids before are a subset of after, plus exactly the closing entry. ank archive now starts 0 git processes for 2 and for 12 done tasks (GIT_TRACE2_EVENT, no trace file written). Repo check after the change: 237 cold entities left hot by tasks finished before the rule, ok. Docs replays: check.md and quickstart.md now print no cold signal after done, and 2 rather than 5 entity files differ from main, the entries having left .ank/entities/.
