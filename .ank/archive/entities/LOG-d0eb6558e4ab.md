---
id: LOG-d0eb6558e4ab
type: log
title: "PR #406 CI, ubuntu and macos (tui.rs is cfg(unix), so the local Windows suite never ran it)."
created: 2026-09-13T11:48:49Z
author: claude-code/1ce7
scope:
  - crates/ank-cli/src/**
  - crates/ank-core/src/model.rs
  - crates/ank-cli/tests/**
  - docs/format.md
about: TASK-1ce7abea9608
seq: 3
schema: 4
version: 1
---

 Decision on the tui --json total, which went from 2 to 4 with shown still 2: the creation records count, and the golden is updated rather than the reader. Reasons, from the constraints binding crates/ank-tui: ADR-559eebf5c6f5 says log entries are never rows of the list, and the reader's own code (model.rs, listed and Snapshot::total) already drops them from the rows while keeping total as what find said, 'in the corpus', a choice made for work entries before this task existed; ADR-8bd76e8d7c4e has the reader reach the corpus only through the CLI's --json, and ADR-3e6ce108edcd has find answer a program whole. A creation record is an entity of kind log, like a work entry, so it counts toward find's total the way a work entry does. show and log split machinery from the work trace inside one entity's entries, which says nothing about how many entities the corpus holds. Excluding it would mean a second definition of total in the reader and an edit to crates/ank-tui, which this task's scope does not reach. The other failure, a_document_ratified_through_the_reader_is_what_a_shell_accept_makes: the creation record's LOG id and produced hash are computed over each document's own id and instant, so masking the id afterwards cannot make them agree. The test now masks them by shape (LOG- and produced followed by 12 hex characters) and still compares the ratified anchor byte for byte. Checked by compiling masked_records standalone and running it on the two documents CI printed: equal after masking, and ade43771e7ed kept in both ratified and constraint+scope.
