---
id: LOG-aeb78b9368b0
type: log
title: "measured: ank frobnicate exits 1 (error[1] unknown command); ank help nosuchverb exits 2 (error[2]"
created: 2026-09-22T18:44:45Z
author: claude-code/opus-5+0e77
scope:
  - .github/workflows/**
about: TASK-0e77f30ae410
seq: 1
schema: 4
version: 1
---

 no such verb) -- release.yml's step runs the latter, comment now names it and both codes. No ADR records the MSRV floor (ank find --type adr: no msrv/rust/toolchain/floor/dependency match); ci.yml's bare 'ADR:' now cites TASK-973e9dc3f9ce, which decided it. rust-version is read from manifests by both msrv jobs (steps 'the declared MSRV' and 'the minor below the declared MSRV'); failure message no longer asks to update a pin. Duplicate 9-line comment in release.yml: 2 occurrences -> 1 (grep -c). Both workflows parse as YAML.
