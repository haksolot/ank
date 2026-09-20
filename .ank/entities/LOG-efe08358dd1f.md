---
id: LOG-efe08358dd1f
type: log
title: the two frontmatter diagnostics measured against a binary built from this tree (5e12fac), not the
created: 2026-09-20T18:40:52Z
author: claude-code/opus-5+4eef
scope:
  - docs/format.md
about: TASK-69eaddc84993
seq: 6
schema: 4
version: 1
---

 one on PATH: unterminated-frontmatter.md gives "unterminated frontmatter: no closing '---' after the opening one" and no-frontmatter.md gives "missing frontmatter: the file must start with '---'". The ank on PATH is 0.8.0 at 298c01a, which predates TASK-ccb787dc807f and reports both as missing frontmatter -- measuring the new row against it would have documented the defect as the behaviour.
