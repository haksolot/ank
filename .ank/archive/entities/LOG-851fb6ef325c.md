---
id: LOG-851fb6ef325c
type: log
title: Measured from the nextest PASS/SLOW lines of the five latest green ci runs on main (39059985353,
created: 2026-10-17T22:03:37Z
author: claude-8cb5
scope:
  - .config/nextest.toml
  - docs/ci-jobs.md
  - .github/workflows/ci.yml
about: TASK-8cb5af6607f5
seq: 1
schema: 4
version: 1
---

 39062128982, 39062687374, 39064047011, 39068855928). Slowest honest test: ank-cli cli::a_failing_verifier_answers_as_it_would_sequentially_and_leaves_nothing_running on windows-latest, 327.623s (39059985353) and 327.381s (39064047011), bimodal: 20.170-20.839s in the other three runs, 8.8-11.5s on ubuntu and macos. The 1000-process scratch-root test scratch::tests::a_root_is_locked_before_a_sibling_sweep_can_see_its_lock_file: windows 46.280-54.325s, macos 47.843-61.582s, ubuntu 17.506-19.479s. Next slowest: log_rewritten::the_git_process_count_is_the_same_for_one_and_for_forty_entries_changed 77.042s windows (39062687374), status_over_a_thousand_entities 63.493s windows. Bound chosen: slow-timeout period 60s, terminate-after 10, so 600s, 1.83x the slowest pass ever seen; a bound under 328s would have turned two of these five green runs red.
