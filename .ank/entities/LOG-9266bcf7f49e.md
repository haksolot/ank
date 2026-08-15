---
id: LOG-9266bcf7f49e
type: log
title: "the 16-thread test surfaced a Windows bug: a lock in delete-pending state returns"
created: 2026-07-28T00:36Z
author: claude-code@ankor
scope:
  - crates/ank-cli/src/store.rs
about: TASK-244a842bc0cc
schema: 3
version: 1
---

 ERROR_ACCESS_DENIED and not ERROR_FILE_EXISTS, so it was fatal instead of retried
