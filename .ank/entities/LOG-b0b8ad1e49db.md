---
id: LOG-b0b8ad1e49db
type: log
title: "log split measured: 'discrepancy: the criterion assumes merge=union and .gitattributes declares"
created: 2026-09-20T17:43:34Z
author: claude-code/opus-5+4eef
scope:
  - docs/format.md
about: TASK-4eef0864be46
seq: 3
schema: 4
version: 1
---

 none, which is measurable' is exactly 100 characters, so ank log stores it whole in title with an empty body -- the documented example does not split at all. A 111-character message splits at character 97 (the last space at or before 100), title 'discrepancy: the criterion assumes merge=union is configured for the log path, and .gitattributes', body a newline, ' declares none', a newline.
