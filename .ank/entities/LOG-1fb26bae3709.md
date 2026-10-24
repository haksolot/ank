---
id: LOG-1fb26bae3709
type: log
title: "CONTRIBUTING's mise.local.toml example measured: with that file and target/nosign.gitconfig in"
created: 2026-10-24T07:53:28Z
author: claude-code/opus-5.5+wd725
scope:
  - mise.toml
  - mise.lock
  - mise.local.toml
  - rust-toolchain.toml
  - .gitignore
  - CONTRIBUTING.md
about: TASK-d7253853996c
seq: 5
schema: 4
version: 1
---

 place, 'mise x -- sh' saw GIT_CONFIG_GLOBAL=<root>/target/nosign.gitconfig ({{config_root}} expanded); under it git resolved credential.https://github.com.helper from /home/haksolot/.gitconfig through the include (~ expanded) and user.signingkey to the empty value after it; git status showed mise.local.toml ignored. Both files removed afterwards.
