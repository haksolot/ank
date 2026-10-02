---
id: LOG-f7215a8cf9f8
type: log
title: "measured with GIT_TRACE2_EVENT on a scratch repo (.gitignore: build/, build/output.bin on disk),"
created: 2026-10-02T00:44:32Z
author: claude-code/2.0
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5867eb1c06ba
seq: 4
schema: 4
version: 1
---

 one ADR per run: ank check starts 9 git processes for scope build/output.bin, 9 for build/**, 9 for build/absent.bin; the two ignored ones print the git check-ignore -v build/output.bin note, the absent one does not. Suite: 1522 tests green under GIT_CONFIG_GLOBAL=/dev/null; the conteneur's global gpg.ssh.program wrapper makes the 5 signature unit tests fail on upstream/main too, and a stale .ank/index.db signature cache (B verdicts cached under it) made this_repositorys_own_ratifications_are_signed fail until it was regenerated.
