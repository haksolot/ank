---
id: LOG-9a80124cc6ad
type: log
title: federation needs two declarations, not one, and section 7 is why. The reader declares the peer,
created: 2026-08-15T08:45:37Z
author: claude-code/federation-read
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-13e802e46050
schema: 3
version: 1
---

 which is what lets it open that corpus at all; the peer declares the reader, because a scope entry peer:glob is resolved through the declarations of the corpus that wrote the scope. Each half is reviewed where it is written -- "I read that corpus" here, "this decision binds that corpus" there -- and an entry resolving to some third repository binds nothing, which is what makes the entry mean the same thing wherever it is read. The criterion says "declares one the peer of the other"; the test declares both, and the mutual pair is what the shape requires rather than an extra.
