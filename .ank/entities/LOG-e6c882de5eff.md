---
id: LOG-e6c882de5eff
type: log
title: "Measured 2026-09-13, scratch corpus on this tree (ank built from 81330f5): ank init, ank new task"
created: 2026-09-13T09:42:34Z
author: claude-code/fable-5.1+planning
scope:
  - crates/ank-cli/**
about: ADR-52bb0da2023a
seq: 0
schema: 4
version: 1
---

 --no-verify, ank amend --scope, commit; then a second task written whole by heredoc into .ank/entities/TASK-aaaaaaaaaaaa.md imitating canonical form, and the first task's title changed by sed without touching version. ank check: the sed edit is reported as a signal (content is 49adc3a82e47 where the last write left b31ea7b33bd0: it was edited outside the CLI, which is legal and leaves no entry); the heredoc entity gets only the written-by-an-agent-and-read-by-no-human signal, identical to the legitimate one. exit 0, 2 tasks, 5 signals. Creation by hand is invisible today.
