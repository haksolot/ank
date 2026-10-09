---
id: TASK-cae5c8ec9a69
type: task
slug: the-reader-draws-its-listings-beside-the-detail
title: The reader draws its listings beside the detail on a wide terminal
created: 2026-10-09T12:31:37Z
author: clausse@macbook-air-de-clausse
status: in_progress
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
blocked_by: [TASK-3c1622d65f0d]
done_criteria: |
  At a width of at least DASHBOARD columns (one constant in crates/ank-tui/src/view.rs), one frame draws the claims, entities and queue panels stacked in a left column and the detail panel on the right, each with its digit and name on its title; exactly one panel is marked as focused by a character on its title and by its border set, and the cursor marker appears only in the focused panel. Below that width the frame draws one panel at a time, as before. Tab, BackTab and the digits move the focus between panels without taking any panel off a wide frame; Enter on a row opens it into the detail panel and Back returns the focus to the listing it came from. On a wide frame the claims and the queue are read during the opening, after the first frame has been drawn and after find has answered, and moving the cursor spawns no verb. A tap in a panel focuses that panel and selects the row under it. Tests in crates/ank-tui/tests drive the session at 160x45 and at 80x24 and assert each of these; the tests that pinned ADR-559eebf5c6f5's single region are rewritten against ADR-ac6be1ebe9aa.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Step two of the dashboard ADR-ac6be1ebe9aa proposes. The panels already render one at a time through App::panel(focus, listing, area); this draws several of them, decides the rectangles in App::arrange, and reads status and review during the opening so the panels are not empty.
