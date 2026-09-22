---
id: LOG-fe4c13f98f02
type: log
title: fix v2, HEAD alone judges an archived file HEAD carries, index for uncommitted moves. Scratch repo,
created: 2026-09-22T18:02:59Z
author: claude-code/opus-5+77df
scope:
  - crates/ank-cli/tests/**
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/human.rs
about: TASK-77df1446260f
seq: 5
schema: 4
version: 1
---

 4 archived LOG entries committed: clean exit 0; one byte appended exit 8 (1 fault); rm index.db* then two runs exit 8 naming archive/entities/LOG-3c6e75b026a0.md; git checkout -- it, exit 0 with the index rebuilt over the edited bytes still in place. GIT_TRACE on check: 15 git processes, of them 5 cat-file and 2 hash-object; the new path is one cat-file of HEAD's archive tree and one hash-object --stdin-paths, whatever the archive count.
