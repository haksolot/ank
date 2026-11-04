---
id: LOG-f8ae78ba4702
type: log
title: "Reproduced on git 2.47.3: a fixture 'git commit' runs 'git maintenance run --auto --quiet --detach'"
created: 2026-11-02T14:16:54Z
author: claude-code/opus-5.5+task-a836-peer-default-branch
scope:
  - crates/ank-cli/tests/peer_default_branch.rs
about: TASK-60d1ad73701a
seq: 1
schema: 4
version: 1
---

 (GIT_TRACE), which takes .git/objects/maintenance.lock in the background after commit returns, so the snapshot of api caught it during the next ank verb (CI run 41457009716, claim and show). GIT_TRACE over a run of the suite: 18 maintenance processes with the fixture as of 6edcdc3e, 0 with maintenance.auto=false and gc.auto=0 in each scratch repo (the settings cli.rs's Repo::new already uses). Snapshot assertion unchanged; 5 consecutive runs green.
