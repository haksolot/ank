---
id: LOG-842cec47fc6d
type: log
title: "Measured: bash -c with the entry as written on Windows (\"\\\\?\\C:\\...\\ank.exe\") reports '\\?\\C:\\...:"
created: 2026-11-20T13:46:55Z
author: haksolot@vmi3223161
scope:
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/tests/hooks_install.rs
  - crates/ank-cli/tests/hooks_status.rs
about: TASK-021d53d80c1d
seq: 1
schema: 4
version: 1
---

 No such file or directory' and exit 127, the reported failure. Fix: for_shell strips \\?\ (\\?\UNC\ -> \\) and turns separators to /, applied by exe() on Windows only. Tests: 3 unit tests on for_shell run on every platform; hooks_install::the_installed_command_runs_through_the_shell_the_harness_uses runs both installed commands with sh -c and expects exit 0; ours()/this_binary() now spell the expected form instead of reusing fs::canonicalize. Linux: hooks unit 5/5, integration hooks_ 47/47 green; Windows to be proven by PR CI.
