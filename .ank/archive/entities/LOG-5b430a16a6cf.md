---
id: LOG-5b430a16a6cf
type: log
title: Two honest fixes weighed. Restoring ank-mcp to check-version.sh's loop would re-erect a rule whose
created: 2026-08-25T21:59:31Z
author: claude-code/opus-5+surface-version
scope:
  - crates/ank-mcp/**
  - .github/scripts/check-version.sh
about: TASK-ae64d1c5678d
seq: 0
schema: 4
version: 1
---

 stated justification (the release ships ank-mcp as a file) died with ADR-1ea31c2f3c5a, and would leave two numbers agreeing by rule -- one more of the 'nine more chances to be careful than anyone gets right forever' the script itself names. Taking the other: the surface reports the binary's version, the crate's own number becomes internal, and one number reaches a reader. Where it comes from: Address is what the dispatch already hands the surface because the surface cannot compute it (exe, repo), and Address::exe IS current_exe(), so ank-cli's CARGO_PKG_VERSION handed down is by construction the number that same process prints for --version. No link to ank-cli (tests/dependencies.rs), no parse of a spawned --version.
