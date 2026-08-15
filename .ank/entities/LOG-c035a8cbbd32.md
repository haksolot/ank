---
id: LOG-c035a8cbbd32
type: log
title: "Three cargo tests failed on a stale target/: the cached test binary carried"
created: 2026-08-01T18:55:25Z
author: seanl@sean-laptop
scope:
  - docs/ank-spec-v1.1.md
about: TASK-ff1c20395929
seq: 1
schema: 3
version: 1
---

 CARGO_MANIFEST_DIR=C:\Users\seanl\Documents\Projects\ankor\crates\ank-cli, the pre-rename path (ADR-85e6). canonicalize() on it raised ERROR_PATH_NOT_FOUND and the three dogfooding tests that reach for this repository's own .ank/ unwrapped it. Cargo did not rebuild because no source changed. cargo clean -p ank-cli -p ank-core is the fix; not a defect and not caused by this task.
