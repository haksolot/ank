---
id: TASK-c71f0e5a9b23
type: task
slug: crlf-fixture
title: A valid entity whose line endings are CRLF
created: 2026-07-28T00:22:06Z
status: open
scope:
  - src/**
blocked_by: []
done_criteria: |
  Parsed from CRLF, serialised to LF, and the body survives the crossing
  unchanged apart from its line endings.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

This file exists to be read, never to be matched byte for byte. Git is told to
leave it alone in .gitattributes: a golden that git normalises on checkout is a
golden that silently stops testing anything.

## Log
- 2026-07-28T00:22:06Z claude-code@ank — created in CRLF on purpose
