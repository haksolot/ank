---
id: LOG-2259e1eed0b8
type: log
title: The failure the criterion describes is real, and it is the three-slash form that carries it.
created: 2026-08-15T23:40:58Z
author: claude-code/5052
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-5052971b8e9c
seq: 3
schema: 3
version: 1
---

 Measured by putting that form back into file_url() for the length of one pair of runs and taking it out again. With MSYS_NO_PATHCONV=1: both shallow-clone tests fail, and git prints "fatal: '/C:/Users/seanl/AppData/Local/Temp/ank-cli-it-27340-0' does not appear to be a git repository" for the URL "file:///C:/Users/seanl/AppData/Local/Temp/ank-cli-it-27340-0" -- git read the URL back as a path. With the variable unset, the same two tests pass in 5.44s. So the variable, and not the git version, is what moves the result, and the two-slash form on main is what makes the result not depend on it.
