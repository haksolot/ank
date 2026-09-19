---
id: LOG-74fce6a8c2da
type: log
title: Verified every command and output against a fresh scratch repository rather than composing them,
created: 2026-08-09T15:53:42Z
author: claude-code@ank
scope:
  - docs/getting-started.md
about: TASK-7af1cd875d9a
seq: 0
schema: 3
version: 1
---

 which is what the page's own opening promises. The verb writes exactly the YAML the guide already showed: 'verifiers: {}' is promoted to a block, roles and identities are untouched, and no-jwt's run comes back double-quoted because it opens with '!'. Found a second staleness in the same file and left it out: 'init does not yet write the rule for you' at line 98 contradicts 'wrote .gitignore' in the init output at line 64, and init does write '.ank/index.db' -- confirmed on the scratch repo. That makes 'Two edits to make now' wrong too. Outside this criterion, filed separately.
