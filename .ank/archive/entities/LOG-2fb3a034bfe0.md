---
id: LOG-2fb3a034bfe0
type: log
title: "Measured at f470b0e1: ank archive --dry-run reported 316 cold (219 on 1dbd58ff, the rest made cold"
created: 2026-11-22T01:26:38Z
author: claude-code/fable-5.2+2e3b
scope:
  - .ank/**
about: TASK-2e3bf025d141
seq: 1
schema: 4
version: 1
---

 by the closes of this wave run with a binary that predated ADR-302efb9c5e40); ank archive moved 316; --dry-run then says nothing is cold; ank check exits 0 with no cold signal. Hot files in .ank/entities: 1019, archive: 3248.
