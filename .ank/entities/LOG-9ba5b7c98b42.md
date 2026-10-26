---
id: LOG-9ba5b7c98b42
type: log
title: "Swept at dcc3c1bc: 5 citations in 4 tracked files (verbs.rs x2, hooks.rs, tests/skill.rs,"
created: 2026-10-26T13:51:35Z
author: claude-code/opus-5.5+014b
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/hooks_install.rs
about: TASK-014bbfca8fb6
seq: 1
schema: 4
version: 1
---

 tests/hooks_install.rs) re-pointed to ADR-376bed7fa39a; git grep -nE 'ADR-24ef' outside .ank/ prints nothing (exit 1). Each sentence cites the decision for hooks being in the surface, a flag being no new surface, or what hooks installs, all three restated by the successor, so no sentence was rewritten. Accepted citers read: ADR-35d714834ed3 and ADR-bafbd28498cc cite it only to say they leave the surface untouched, which stays true. SPEC-79012ba18769 (the CLI surface, section 4) lists ADR-24efa7543a16 in references: and is the document that fixes the flags, so --peers changes what it describes: TASK-ae6c94125af5 and TASK-35af3b803257 must bring a superseding revision of section 4 (ADR-1f70 wants any short form declared there before code moves). The references: entry is re-pointed after the accept by ank amend --reference, outside this task's scope.
