---
id: LOG-8e1c0dd20df0
type: log
title: "released: The criterion I wrote is wrong on one word, found by reading before writing code. It says"
created: 2026-08-24T18:13:14Z
author: claude-code/opus-5-reading
scope:
  - crates/ank-cli/**
  - crates/ank-contract/**
about: TASK-e3370ef322d8
seq: 0
schema: 4
version: 1
---

 the verb appends a reading to any entity that carries verified:, and all four kinds carry the field in the model, log entries included. ADR-25f977377fa0 says a log entry is written once and never modified, a correction being a new entry naming the one it corrects. Meeting the criterion as written would mean writing to a log file a second time, against a ratified constraint.

Nothing else in it moves. Releasing rather than reinterpreting the word, because a criterion read charitably by whoever holds it is not a criterion.
