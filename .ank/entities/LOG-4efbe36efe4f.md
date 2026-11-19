---
id: LOG-4efbe36efe4f
type: log
title: No test in scope can hold SECURITY.md or the spec text (doc_replay.rs is outside the scope); the
created: 2026-11-19T00:06:49Z
author: claude-code/opus-5.5+w6-authority-docs
scope:
  - SECURITY.md
  - docs/signing.md
  - docs/check.md
  - .ank/entities/SPEC-*.md
about: TASK-2444d0cc6392
seq: 2
schema: 4
version: 1
---

 proof is the declared suites plus ank check, and the content is measured against hooks.rs (PreToolUse matcher includes Bash, shell_call refuses only under deny), verify.rs (definition_hash: run, timeout, then inputs/vars/inherit_env/cwd each only when declared; default uncovered; ANK_TASK, ANK_VERIFIER, ANK_SCOPE, ANK_BASE set after vars) and docs/signing.md (require_signature refusals).
