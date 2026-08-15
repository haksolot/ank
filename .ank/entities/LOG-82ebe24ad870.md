---
id: LOG-82ebe24ad870
type: log
title: The fixtures run on ubuntu alone, not on the test matrix. Two reasons, and the second is the one
created: 2026-08-13T05:01:34Z
author: claude-agent-c
scope:
  - .github/workflows/ci.yml
about: TASK-adf11c12c480
schema: 3
version: 1
---

 that decided it: ubuntu is the platform of use, since the version job in release.yml runs there; and on windows-latest git checks out with autocrlf, so a .sh under .github/scripts/ arrives in CRLF and bash refuses it. Keeping these scripts in LF is a .gitattributes decision about the tree, outside this task's scope. Falsified in both directions before committing: a check-version.sh that always exits 0 turns four fixtures red, and a JSON parser anchored on the wrong indentation turns all five red with 'the file's shape moved' rather than passing on an empty read.
