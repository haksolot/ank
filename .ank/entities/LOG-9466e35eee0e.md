---
id: LOG-9466e35eee0e
type: log
title: What the counts above say about the two signals, 2026-08-01 to 2026-09-15, 326 task branches.
created: 2026-09-15T11:06:41Z
author: claude-code/opus-5+history-counted
scope:
  - .ank/entities/**
about: TASK-96bc559cf6d8
seq: 4
schema: 4
version: 1
---



Scope overlap is the one that costs. Among claims that overlapped in time, every git conflict on a file a scope could name fell in a pair whose scopes intersect: 10 of 10 under the trace window, 17 of 17 under the base window. The only 3 conflicts outside are one early-August entity file (#50 against #54) that no scope named. No disjoint pair conflicted on code, and the 14 conflicts agents actually resolved on their branches are all accounted for. Its noise is also measured: 15 of its 27 lines (trace window) or 29 of 47 (base window) preceded neither a conflict nor a red first run, 56% and 62%, and the file behind most of those silent lines is crates/ank-cli/tests/cli.rs. Scope intersection does not predict a red first run: 3 of 27 intersecting pairs against 10 of 62 disjoint ones.

The corpus-drift condition, measured at the moment a branch opens, is noise in this window. 19 of 326 branches were behind main's .ank/ at the agent's earliest trace, 41 at the first commit, 55 at PR creation. 18 of the 19 were one or two corpus commits behind, the largest gap was 6 by any of the three moments, and 1 of the 19 had a red first run against 19 of the 307 level branches. The 4 of those 19 that later resolved a conflict did so on a file another task also edited, which the overlap signal already names. This counts only drift at opening; drift that builds up while an agent works is what 'ank status' reports at any moment, and nothing in git records when it was read.
