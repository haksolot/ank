---
id: LOG-1557cd5ef49b
type: log
title: Amended the scope once more with docs/ank-spec-v1.1.md. Section 13 said 'The version is checked at
created: 2026-08-12T22:32:17Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/ank-spec-v1.1.md
about: TASK-65017ea098f2
seq: 5
schema: 3
version: 1
---

 startup' and section 7 said 'an uninitialised repository exits with code 9' without qualification -- both direct contradictions of ADR-9307, and my own change is what turns them from true into false. Fixed the two, stating the per-verb rule and the enumeration in section 13 and qualifying section 7 with 'a coordinating verb'. Left the broader section 6 prose alone: TASK-62136e8c2b69 owns the discovery walk, which section 6 has never described, and folding it in here would take that task's subject without its criterion.
