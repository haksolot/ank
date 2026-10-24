---
id: LOG-a701162edbe4
type: log
title: "discrepancy (measured): run 1, on this machine with no mise.local.toml, failed exactly one test,"
created: 2026-10-24T08:53:23Z
author: agent-ce8d
scope:
  - mise.toml
  - CONTRIBUTING.md
about: TASK-ce8dbe52f357
seq: 4
schema: 4
version: 1
---

 human::tests::accept_runs_on_a_feature_branch, because git tag -s asked for the passphrase of ~/.ssh/gh_signing (984/1676 ran before fail-fast, 983 passed, exit 100). That's the user's signing key, not the TMPDIR: with CONTRIBUTING's documented git-ignored mise.local.toml + target/nosign.gitconfig, run 2 is 1676/1676 green. 'Clean checkout' holds for a git config with no passphrase-only signing key.
