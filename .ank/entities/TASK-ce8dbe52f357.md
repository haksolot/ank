---
id: TASK-ce8dbe52f357
type: task
slug: mise-run-test-is-green-its-tmpdir-lives-outside
title: "mise run test is green: its TMPDIR lives outside the repository"
created: 2026-10-24T08:04:05Z
author: claude-code/opus-5.5+wd725
status: in_progress
scope:
  - mise.toml
  - CONTRIBUTING.md
blocked_by: [TASK-d7253853996c]
done_criteria: |
  mise run test exits 0 on a clean checkout: the workspace suite under nextest runs with TMPDIR on ${XDG_CACHE_HOME:-$HOME/.cache}/ank-test, created if absent, which is outside the repository, outside any cargo target directory and on disk rather than a tmpfs; every test listed under nextest runs and passes, the count run and the exit code are recorded with ank log; mise.toml names the directory beside the reason target/ cannot hold it, and CONTRIBUTING.md says where the test task writes its temporary files.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 3
---

Found working TASK-d7253853996c (LOG-20c188c72f73). Its criterion put the test task's TMPDIR under target/; measured, that makes 14 of 1676 tests fail for two reasons rooted in target/ itself: ank's corpus walk finds this repository's .ank/ above every fixture (3 tests), and ank update refuses any binary under a cargo target directory (10 update:: tests and doc_replay::install_replays). GIT_CEILING_DIRECTORIES covers git discovery only. Planning owes a decision on where the task's TMPDIR lives (a per-user cache directory, or the system temp with the tmpfs caveat this box has); the criterion above assumes outside the repository and lets planning pick the place.
