---
id: LOG-c456a84abb9f
type: log
title: drift audit 2026-08-31, re-measured and holds. A task file was rewritten with CRLF throughout (260
created: 2026-08-31T07:54:06Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-core/**
  - docs/**
about: ADR-63b59c5c26f7
seq: 0
schema: 4
version: 1
---

 bytes to 277, 17 CR). 'ank show' read it unchanged; 'ank amend --scope docs/**' rewrote it and left 0 CR and 272 bytes, so CRLF is read and never written and the normalisation happens on first rewrite. The field-parity half was measured too: all 24 field names the kind registry declares in crates/ank-core/src/registry.rs -- about author blocked_by constraint created criteria_by done_criteria id proof ratified records references schema scope see seq slug status supersedes title type verified verify version -- appear in the ten accepted spec documents, none with a zero count.
