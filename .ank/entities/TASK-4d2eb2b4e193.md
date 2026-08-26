---
id: TASK-4d2eb2b4e193
type: task
slug: the-bindings-are-one-table-and-the-reader-reads
title: The bindings are one table, and the reader reads it back
created: 2026-08-26T17:07:10Z
author: claude-code/opus-5+reader-redesign
status: open
scope:
  - crates/ank-tui/**
blocked_by: []
done_criteria: |
  crates/ank-tui/src/bindings.rs declares every binding of the reader once -- its key, its aliases, the command it runs, the word it is called by, the group it belongs to, the focus it is admitted in, and the CLI verb it spells where it spells one -- and keys::typed, App::actions and the key list are all computed from it rather than written beside it. Driven on a pseudo-terminal, the built binary answers ? with a list that names every binding the table declares and names nothing the table does not. Every row the table marks as a verb names a verb ank help --json declares, and a row naming one it does not fails the suite rather than reaching a screen.
criteria_by: creator
schema: 4
version: 1
---
