---
id: LOG-b8d46dcfff0b
type: log
title: The criterion holds and its stated cause does not. On this machine, git 2.54.0.windows.1, the test
created: 2026-08-14T17:32:35Z
author: claude-code/143a
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-143a310de8b6
seq: 0
schema: 3
version: 1
---

 passes as it stands: the three-slash form is accepted. It is refused only when MSYS_NO_PATHCONV or MSYS2_ARG_CONV_EXCL is set, and either one alone is enough. Measured four ways from PowerShell, with no MSYS shell between the caller and git, on a freshly seeded repository under TEMP: neither variable set, file:///C:/... exit 0 and file://C:/... exit 0; MSYS_NO_PATHCONV=1 alone, file:/// exit 128 'does not appear to be a git repository', file:// exit 0; MSYS2_ARG_CONV_EXCL=* alone, the same split; both set, the same split. The two-slash form is shallow in every case that succeeds. Through the suite rather than a probe: cargo test --test cli -- a_shallow_clone_cannot_explain_a_dead_scope passes alone in 3.72s with the variables unset, and fails alone in 0.57s with MSYS_NO_PATHCONV=1 as the only change, with the same message the task reports. So git-for-Windows normally rewrites the three-slash drive-letter form, either variable disables that rewriting, and git then reads the literal /C:/... as a path. What the earlier reports set to remove a confound is what created the failure -- both ran their probe under MSYS_NO_PATHCONV=1 and read the git version as the cause. The task remains worth doing: the two-slash form is the one that does not depend on an environment variable an agent's shell may export, which is exactly how two sessions saw red where a third saw green. Alternative weighed and not taken: git clone --no-local honours --depth without any URL and would be immune by construction, but the frozen criterion says clone_of builds a URL git accepts, so the URL stays and only its form changes.
