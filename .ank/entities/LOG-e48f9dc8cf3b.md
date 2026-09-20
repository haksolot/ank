---
id: LOG-e48f9dc8cf3b
type: log
title: "measured: config.yml verifiers take a 'default' key. 'ank config verifiers.no-jwt.default true'"
created: 2026-09-20T17:45:06Z
author: claude-code/opus-5+9592
scope:
  - docs/getting-started.md
about: TASK-959205eb8dee
seq: 2
schema: 4
version: 1
---

 prints 'verifiers.no-jwt.default false (default) -> true'; a subsequent 'ank new task' with no --verify writes 'verify: [no-jwt]'. '--no-verify' writes no verify: at all. Reading an unset one prints 'false (default)'. Neither is taught in getting-started.
