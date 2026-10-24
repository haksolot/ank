---
id: LOG-7fc9caff0528
type: log
title: Re-read the six successor diffs against the criterion, measured by ank show of each predecessor and
created: 2026-10-17T16:24:11Z
author: claude-code/opus-5.5+5f7b
scope:
  - docs/specification.md
  - docs/format.md
about: TASK-5f7b49eaca61
seq: 5
schema: 4
version: 1
---

 successor and diff: frontmatter differs only by id/created/author/status/supersedes/version, the chain-end references already recorded, and schema 3->4 on the implementation spec (written by new spec). Body changes per clause: data model 13 changed hunks (anchor field in both tables and both examples, ref-first then commit-walk fallback, --reanchor); proof 11; implementation 5 (never commits, no exception; tag -s admitted); sync 7 (third family, CAS push, no TTL, never pruned); CLI 12 (branch gate, variable-geometry paragraph and --pr dropped, --reanchor described, exit 7 row trimmed); intent 3 (revision entry s). The check signal quoted in the sync successor exists verbatim at crates/ank-cli/src/human.rs:1291. ank check exit 0; ank review lists the six under PROPOSED.
