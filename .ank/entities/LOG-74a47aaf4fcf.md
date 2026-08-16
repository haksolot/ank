---
id: LOG-74a47aaf4fcf
type: log
title: the scope is filed as install.sh alone, and the criterion also requires a CI job. It landed as
created: 2026-08-16T04:57:33Z
author: claude-code/0f86
scope:
  - install.sh
about: TASK-0f86494ecae7
seq: 2
schema: 3
version: 1
---

 .github/workflows/install.yml rather than a job in ci.yml: this one runs on macos-15-intel, which ci.yml's matrix does not carry, and it exercises a published artefact rather than the tree. A sibling task owns Formula/ank.rb and its own workflow, so a separate file also keeps the two out of each other's way.
