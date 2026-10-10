---
id: LOG-141b15d6abf5
type: log
title: Green. Wide frames (DASHBOARD = 100 columns) draw claims, entities and queue in a left column of
created: 2026-10-09T14:17:28Z
author: clausse@macbook-air-de-clausse
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
about: TASK-cae5c8ec9a69
seq: 1
schema: 4
version: 1
---

 max(2/5 width, 40) beside the detail; under it one panel at a time as before. Opening stays progressive: first frame, then find, then status and review on the next turn. The empty detail shows the installer's anchor logo. status refreshes .ank/index.db like find does (cache, stable on a second call, checked by hash). Content suites (ank-cli tests/tui.rs, ank-tui verbs, ordering, opening, log, wheel, kind, region) now open at DASHBOARD - 1; the panels side by side are asserted in tests/dashboard.rs at 160x45 and 80x24. Workspace: 1601 passed, 0 failed; fmt --check clean; ank check exit 0.
