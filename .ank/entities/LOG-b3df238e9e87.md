---
id: LOG-b3df238e9e87
type: log
title: "measured: 'ank init' in an empty repository writes .gitattributes, .gitignore and a pointer line in"
created: 2026-09-20T17:44:14Z
author: claude-code/opus-5+5e6a
scope:
  - CONTRIBUTING.md
  - SECURITY.md
  - .github/pull_request_template.md
  - .github/ISSUE_TEMPLATE/**
about: TASK-5e6aabcddbb0
seq: 6
schema: 4
version: 1
---

 AGENTS.md at the root, outside .ank/. SECURITY.md's in-scope bullet reads 'a path where ank writes outside .ank/ and the refs under refs/ank/', which makes init's own documented behaviour a reportable vulnerability. The hook SECURITY.md cites as 'this repository's own working example' was deleted in 264636c along with all of .claude/.
