---
id: TASK-e8da6a00564a
type: task
slug: an-edit-names-its-field-and-close-attest-and-rea
title: An edit names its field, and close, attest and read are reachable
created: 2026-08-26T17:07:37Z
author: claude-code/opus-5+reader-redesign
status: in_progress
scope:
  - crates/ank-tui/**
blocked_by: [TASK-d832452630d2]
done_criteria: |
  The built binary raises, on e, a form whose submission spells ank edit <id> with --title, --body or --constraint, and never a bare ank edit <id>: a named field stands ahead of the search for an editor, and an editor is what this reader must never spawn. close, attest and read are reached from x and from no key, each with the flags its verb requires, and each passes the same confirmation. Every one of the three leaves every byte under .ank/ and every ref under refs/ank/ unchanged when the confirmation is dismissed. Measured with $EDITOR pointed at a script that writes a sentinel the test then finds absent.
criteria_by: creator
schema: 4
version: 4
---
