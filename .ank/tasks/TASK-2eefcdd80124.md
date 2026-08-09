---
id: TASK-2eefcdd80124
type: task
slug: three-verbs-put-non-json-on-stdout-while-json-is
title: Three verbs put non-JSON on stdout while --json is set
created: 2026-08-09T17:11:56Z
author: claude-code@ank
status: open
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Under --json, every verb writes exactly one JSON document to standard output and nothing else. Measured through the binary on the three known sites -- done progress lines, and the takeover warnings of log and amend -- by capturing stdout alone and requiring it to start with a brace and to contain no line outside the document. A test walks the verbs rather than naming these three, so a line added later to a fourth is caught. What moves off stdout goes to standard error, keeping the information rather than dropping it. cargo test and ank check stay green.
criteria_by: creator
schema: 2
version: 1
---

Found while executing TASK-bfa325e55424, which put its own warning on standard error rather than beside these.

Section 4 says --json is data, and that it stays byte-for-byte what a caller's parser already reads. Three verbs contradict it, and cli.rs already knows: the comment on the style gate names them -- done's progress lines, and the takeover warnings of log and amend -- and treats the situation as a given rather than as a defect.

Measured on a scratch repository with the binary at 5f825cd. Running `ank done <id> --json` put this on standard output:

    running: ok ... ok (0.1s)
    {"task": "TASK-ff695a8a0a39", "status": "done", ...}

A caller parsing stdout as JSON fails on the first line. The failure is not subtle, which is probably why it has survived: anyone who hit it worked around it by reading the last line, and a workaround that works is a defect nobody files.

The information itself is worth keeping -- progress on a long verifier run is exactly what a human wants -- so the fix is to move it to standard error, not to suppress it under --json. That is what TASK-bfa325e55424 did for the constraint-drift warning, and doing the same here would leave one rule instead of two.

The test should walk the surface rather than name the three sites, for the reason TASK-8dd89053fa33 was filed: a criterion that enumerates instances gets satisfied exactly, and the fourth site is added outside it.
