---
id: TASK-ce8dbe52f357
type: task
slug: mise-run-test-is-green-its-tmpdir-lives-outside
title: "mise run test is green: its TMPDIR lives outside the repository"
created: 2026-10-24T08:04:05Z
author: claude-code/opus-5.5+wd725
status: done
scope:
  - mise.toml
  - CONTRIBUTING.md
blocked_by: [TASK-d7253853996c]
done_criteria: |
  mise run test exits 0 on a clean checkout: the workspace suite under nextest runs with TMPDIR on ${XDG_CACHE_HOME:-$HOME/.cache}/ank-test, created if absent, which is outside the repository, outside any cargo target directory and on disk rather than a tmpfs; every test listed under nextest runs and passes, the count run and the exit code are recorded with ank log; mise.toml names the directory beside the reason target/ cannot hold it, and CONTRIBUTING.md says where the test task writes its temporary files.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@96b0582
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/ee9041060d51@96b0582
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/5309865b5940@96b0582
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/ba254b474d6e@39bcc11
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w451a
  - type: test
    ref: local/0e45f722a112@96b0582
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/c74f5dde069b@96b0582
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/30ce1ddb9942@8416afc
    tree: scope/c4dac51af3d0
    criteria: f03326f366b9
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 4
---

Found working TASK-d7253853996c (LOG-20c188c72f73). Its criterion put the test task's TMPDIR under target/; measured, that makes 14 of 1676 tests fail for two reasons rooted in target/ itself: ank's corpus walk finds this repository's .ank/ above every fixture (3 tests), and ank update refuses any binary under a cargo target directory (10 update:: tests and doc_replay::install_replays). GIT_CEILING_DIRECTORIES covers git discovery only. Planning owes a decision on where the task's TMPDIR lives (a per-user cache directory, or the system temp with the tmpfs caveat this box has); the criterion above assumes outside the repository and lets planning pick the place.
