---
id: LOG-da31ebf85ffe
type: log
title: cargo test --workspace is not green on this machine, and it is not this task that made it red.
created: 2026-08-13T22:12:14Z
author: claude-code/72ba
scope:
  - assets/**
  - npm/ank/package.json
about: TASK-72baa24eef8f
schema: 3
version: 1
---

 a_shallow_clone_cannot_explain_a_dead_scope_and_says_so_instead_of_faulting fails at crates/ank-cli/tests/cli.rs:129, and it fails identically on 2215a78 with every change of this task stashed -- the tree this branch is based on, untouched. This task adds no Rust, so it cannot reach that test. Root cause, measured rather than guessed: clone_of builds file:///{path} at cli.rs:7621, and git 2.54.0.windows.1 refuses that form, reporting the URL as the literal path /C:/Users/... Probed side by side on real Windows paths: file://C:/... clones and writes .git/shallow, file:///C:/... fails, and a plain path clones but drops --depth with the warning the helper's own comment predicts. So the third slash before the drive letter is the whole defect, and the fix is to not emit it on Windows. CI is green on all three platforms for the run that merged this test, so the runner image carries a git that still accepts the form -- which makes this a latent red rather than a broken test, and it turns red on its own the day the image updates. Filed as its own task rather than fixed here: it is outside this task's declared scope, and cli.rs is being edited by three other live sessions.
