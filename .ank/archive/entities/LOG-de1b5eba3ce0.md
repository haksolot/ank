---
id: LOG-de1b5eba3ce0
type: log
title: "discrepancy: assumed that mise x -- cargo nextest runs the lockfile's cargo-nextest whatever the"
created: 2026-10-24T08:45:51Z
author: haksolot@vmi3223161
scope:
  - .ank/config.yml
about: TASK-791de95bdd2d
seq: 3
schema: 4
version: 1
---

 caller's PATH; measured that cargo resolves subcommands from $CARGO_HOME/bin before PATH. With PATH=<rustup proxies only>:~/.local/bin:/usr/bin:/bin and mise x prepending .local/share/mise/installs/github-nextest-rs-nextest/0.9.148, 8 of 8 samples of cargo-nextest processes whose cwd was this worktree had exe /home/haksolot/.cargo/bin/cargo-nextest (CARGO=~/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/cargo). Both are 0.9.148 here, so the run is green, but a contributor with an older cargo-installed nextest runs that one. Kept the criterion's literal form; filed a follow-up.
