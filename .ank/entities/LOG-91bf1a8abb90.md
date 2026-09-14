---
id: LOG-91bf1a8abb90
type: log
title: Measured what TASK-97fd1992567a will meet, on a scratch clone of this repository (not this corpus,
created: 2026-09-14T12:19:32Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
about: TASK-da978b214eca
seq: 3
schema: 4
version: 1
---

 nothing moved here): the 32 superseded specs moved by hand into .ank/archive/entities/ take check from 0 faults / 479 signals to 31 faults / 443 signals. New faults: 13 'references SPEC-x, which does not exist', 7 'supersedes SPEC-x, which does not exist', 1 '7 log entries are about an entity this corpus does not hold', and 10 dead scopes '.ank/entities/SPEC-x.md' carried by hot log entries about those specs; the prose-identifier signal goes from 73 to 105. Left out of this task on purpose: check's resolutions (check_references, check_succession, check_entries, check_prose_identifiers, blocked_by) read the parsed hot entities plus the unread set, and the unread set doubles as a whole-corpus-read guard (unread.is_empty()), so folding archived ids into it would silence 'nothing supersedes this' claims whenever an archive exists; the archived ids need their own set beside unread. The dead scopes are not a reading question at all: an entry copies its subject's scope, a spec's scope can name its own file, and the ADR keeps entries about a superseded spec hot, so that needs a decision before the verb moves anything.
