---
id: LOG-03b0fc1790eb
type: log
title: "Experiment 4, one repository, several URLs: https://github.com/haksolot/ank and"
created: 2026-09-27T17:04:51Z
author: claude-code/opus-5.5+c6d1
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
about: TASK-c6d184d238e1
seq: 4
schema: 4
version: 1
---

 https://github.com/haksolot/ank.git both ls-remote to 74e4ca61 HEAD; git@github.com:haksolot/ank.git and ssh://git@github.com/haksolot/ank both fail from this machine (Host key verification failed, no ssh key configured). Two readers of one corpus commit one URL string, and which spelling works depends on each reader's credentials: a URL is as machine-dependent as the relative path it was meant to replace, just along a different axis (auth instead of layout).
