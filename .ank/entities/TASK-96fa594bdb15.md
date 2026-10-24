---
id: TASK-96fa594bdb15
type: task
slug: ank-init-writes-requires-ank-as-the-running-vers
title: ank init writes requires_ank as the running version
created: 2026-10-24T13:37:23Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/init_requires_ank.rs
  - crates/ank-cli/tests/main.rs
blocked_by: [TASK-109153abe8e0]
done_criteria: |
  Through the binary, in crates/ank-cli/tests/init_requires_ank.rs: ank init in an empty scratch repository writes .ank/config.yml at schema 2 with requires_ank equal to the version ank --version prints first; ank init --at <path> does the same in the declared corpus; ank check on the fresh corpus exits 0 with no fault; an init over an existing corpus does not change an existing requires_ank; each case asserts the file content and the exit code.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@fcad678
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/d836ef2a6c57@81830b6
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/b1a88b420555@81830b6
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/40818d1479a8@81830b6
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1091
  - type: test
    ref: local/990b14680eca@fcad678
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/fc89f4a5376b@fcad678
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/43856ce7e7d7@fcad678
    tree: scope/c1e79be0a5bc
    criteria: 0f5531e35e74
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Applies ADR-756e8b0fe416 once it is ratified; do not claim before. Disjoint from the warning task (cli.rs, human.rs), so the two can run in parallel once the key exists.
