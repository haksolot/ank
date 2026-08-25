---
id: LOG-c7ecac8f471d
type: log
title: The criterion asks for a test asserting the three copies of the prompts are character for character
created: 2026-08-25T04:01:46Z
author: claude-code/opus-5+adopt-prompts
scope:
  - install.sh
  - install.ps1
  - docs/getting-started.md
about: TASK-567084d21d2b
seq: 0
schema: 4
version: 1
---

 equal, and no test file is inside the task scope: install.sh, install.ps1 and docs/getting-started.md are the three files that carry the prose, and none of them can assert anything. crates/ank-cli/tests/ is where skill.rs already asserts conformance of prose the repository ships, so the drift test belongs beside it as crates/ank-cli/tests/adopt.rs. Amending the scope rather than writing outside it.
