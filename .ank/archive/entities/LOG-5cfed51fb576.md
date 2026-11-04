---
id: LOG-5cfed51fb576
type: log
title: Measured from the macos-latest job logs of run 41449785197 and the ten green ci runs of main before
created: 2026-11-02T13:39:23Z
author: claude-code/opus-5.5+task-pty-table-macos
scope:
  - .config/nextest.toml
  - crates/ank-tui/tests/terminal/**
about: TASK-9d158e54cd7f
seq: 1
schema: 4
version: 1
---

 it (41446773602 41436798274 41434214165 41431298977 40721866364 40528298941 40522297835 40514476407 40511485852 40507460208), by the PASS/FAIL end stamp minus the duration of every ank-tui::integration test. Failing run: terminal::every_terminal_opened_at_once_is_told_its_own_slave ran 13:14:35.7-13:15:32.0 (56.4s) and itself logged 102 posix_openpt refusals, all errno -6, with ceiling 511 and at most 8x7=56 masters held; graph::the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing ran 13:15:16.8-13:15:20.7, inside that window, and died on the same errno -6. 15 pty tests overlapped the stress window in that run. The ten green runs: stress 45.0-58.7s, refused 46/78/67/81/66/74/102/69/101/99 opens, 12-20 other ank-tui::integration tests overlapping it every time, none failed. So on macOS the table is refused at 56 of 511 held on every run of the stress test (the churn of 8x1000 opens, not the live count, is what macOS runs out of), and every pty test of another group can land in that window; the pty-serial group does not exclude the stress test, which is in no group. One failure in 11 runs is the lottery of an open landing on a refused instant with no retry. Next: give the stress test every test thread under the ci profile so nothing runs beside it, and read the refusal count of a run where it runs alone.
