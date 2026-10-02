---
id: TASK-5867eb1c06ba
type: task
slug: a-dead-scope-only-git-ignored-files-match-is-a-s
title: A dead scope only git-ignored files match is a signal naming git check-ignore, not a fault
created: 2026-10-01T09:38:46Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Through the binary, in a repository whose .gitignore names build/ and holds build/output.bin on disk: an ADR scoped build/output.bin and an ADR scoped build/** are each reported by ank check as a signal, never a fault, whose text says the matched file is ignored by git, names build/output.bin, and gives the command git check-ignore -v build/output.bin; with no other finding, check exits 0. In the same run an ADR scoped at a path absent from disk stays a fault and check exits 8. Counted with GIT_TRACE at an absolute path, check starts as many git processes on the ignored-only dead scope as on the absent-path dead scope: the ignored match is found by walking the glob's literal prefix on disk, with no git process of its own. ank context build/output.bin still names both ADRs.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Under ADR-3abc4b33153f, carried by SPEC-7edbdf871408. Meant to land inside PR
#523 (issue #522), so that main never holds the state that PR alone produces:
measured on a reproduction, a corpus with an ADR scoped at a generated,
git-ignored file went from `check` exit 0 to exit 8 with the PR, while `scope`
and `context` still named that ADR as governing the file.

Where to hook it: the dead-scope verdict of `scope_verdicts`, after the
rename/deletion walk ADR-3094538d831e already runs. That walk asks a glob
through its literal prefix; this one does the same on disk, stops at the first
file the glob matches, and reads directory entries only, never a file's
content. A dead scope git can name as renamed or deleted keeps that note;
the ignored-only note is for the death git has no history for.

Test idea for the "no content read" half, on unix only and as a supplement to
the criterion, not a substitute: a FIFO under the ignored directory blocks any
reader, so check returning at all proves nothing opened it.
