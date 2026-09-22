---
id: LOG-7ecc71fd0dcf
type: log
title: "fix v1 (HEAD blob vs hash-object, beside the index digest): edited archived file faults exit 8 on"
created: 2026-09-22T18:02:06Z
author: claude-code/opus-5+77df
scope:
  - crates/ank-cli/tests/**
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/human.rs
about: TASK-77df1446260f
seq: 4
schema: 4
version: 1
---

 two runs with the cache and two after rm index.db. But git checkout -- the file, check still exit 8: the rebuilt index took the edited bytes' digest, so the restored bytes mismatch the cache; rm index.db again gives 0. The cache's verdict depends on when it was rebuilt; for a file HEAD carries, HEAD must decide alone.
