---
id: LOG-c0dee80f4112
type: log
title: "Self-review. Against the criterion: TMPDIR on ${XDG_CACHE_HOME:-$HOME/.cache}/ank-test -> mise.toml"
created: 2026-10-24T08:53:46Z
author: agent-ce8d
scope:
  - mise.toml
  - CONTRIBUTING.md
about: TASK-ce8dbe52f357
seq: 6
schema: 4
version: 1
---

 run lines; created if absent -> mkdir -p, measured from an absent dir; outside repo/target and on disk -> df -T ext4, path under $HOME; every listed test runs and passes -> 1676 listed, 1676 run and passed, exit 0 (LOG-3995c96e1bb1); mise.toml names the dir beside why target/ cannot -> the comment above [tasks.test]; CONTRIBUTING says where -> the paragraph after the mise run list. No hunk outside those; dropping GIT_CEILING_DIRECTORIES removes a workaround that only applied to target/. Against the constraints, re-run over mise.toml and CONTRIBUTING.md: ADR-8c12a38531e3 (active) holds, since the suite still runs under cargo nextest run; ADR-9c69bd47ce23 (proposed, implemented under authorisation) holds, since the task stays generic and the machine's no-sign recipe stays in the git-ignored mise.local.toml, uncommitted. Nothing found against either.
