---
id: LOG-6ec6c0ac7b27
type: log
title: "Measured: new test red before the fix (copy_tree copied index.db.12, -wal, -shm), green after. Then"
created: 2026-10-18T22:16:32Z
author: claude-fixture@vmi
scope:
  - crates/ank-cli/tests/cli.rs
about: TASK-e4f578737ab6
seq: 1
schema: 4
version: 1
---

 ANK_TEST_COLD_ROUNDS=20 on the cold-readers test while a loop ran 233 'ank find --all --json' on the workspace corpus, its index.db.12-wal and -shm present in the copied source: 20/20 rounds, 0 refused of 8, test green in 123s. The CI failure was never a refusal: rounds 1-6/7 were 0 refused, the panic was NotFound in the fixture's copy.
