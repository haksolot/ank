---
id: LOG-eb63f2ac1f68
type: log
title: "Clauses: (1) in a --depth 1 clone that fetched +refs/ank/*:refs/ank/*, check judges the signature"
created: 2026-10-20T16:40:04Z
author: claude-code/8afa
scope:
  - crates/ank-cli/tests/**
about: TASK-8afa3d0a6e25
seq: 1
schema: 4
version: 1
---

 of the ref-ratified ADR (verify-tag counted in GIT_TRACE, no fault, no 'not verified', no unverifiable signal on it); (2) a legacy commit-anchored ADR in the same corpus is the ratification-unverifiable signal and never a ratification-altered fault; (3) exit 0, and the clone is still shallow after the refs fetch. Reading taken for 'reports verified': check prints nothing for a Trusted signature (human.rs check_anchor), so verified is observed as the verify-tag process plus the absence of every anchor and signature finding on that id. Test goes in tests/ratify_ref.rs, reusing Corpus and crate::legacy_anchor::commit.
