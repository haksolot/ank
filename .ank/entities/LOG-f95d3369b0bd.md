---
id: LOG-f95d3369b0bd
type: log
title: "measured on the built binary: help --json refuses across all 29 verbs carry only codes"
created: 2026-09-20T17:47:07Z
author: claude-code/opus-5+7843
scope:
  - crates/ank-contract/**
  - crates/ank-cli/tests/**
about: TASK-78431b544d01
seq: 3
schema: 4
version: 1
---

 {1,2,4,5,6,7,9} (python over help --json: union = [1,2,4,5,6,7,9]). Code 3 appears in no refuses list; code 8 only in notes for check and review. Reproduced the three gaps: (a) 'ank attest <done-id> --proof bogus' -> error[5] unreadable proof, exit 5, while attest refuses lists only 2 and 9; (b) 'ank check' on a corpus with 1 fault -> exit 8, 'ank review' on the same -> exit 8, neither lists 8; (c) 'ank edit <id>' with an $EDITOR that amends the entity from another process -> error[3] version 4 on disk, 3 expected, exit 3; 'ank done' with a declared verifier that amends the entity mid-run -> error[3] version 5 on disk, 4 expected, exit 3.
