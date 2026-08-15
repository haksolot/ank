---
id: LOG-ce54ee81a7da
type: log
title: "discrepancy: the criterion assumes the current helper builds file:/// and strips the path's leading"
created: 2026-08-15T23:39:57Z
author: claude-code/5052
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-5052971b8e9c
seq: 2
schema: 3
version: 1
---

 slash, and that a run with MSYS_NO_PATHCONV set fails against it. Measured: origin/main already builds the two-slash form. crates/ank-cli/tests/cli.rs carries a file_url() helper, one line, format!("file://{}", path.replace('\', "/")), landed by TASK-143a310de8b6 with an argument longer than the one on fix/clone-url-msys-pathconv. Both runs above are green, so there is no failure to put on record against the tree as it stands. The rest of the criterion is finished: the form is argued where the helper builds it, and what fix/clone-url-msys-pathconv still holds that the merged comment does not is carried over.
