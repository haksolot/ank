---
id: TASK-a59ce3bde9cf
type: task
slug: ank-config-refuses-a-zero-duration-and-a-zero-co
title: ank config refuses a zero duration and a zero context_budget, and check faults one written by hand
created: 2026-11-16T08:06:40Z
author: claude-code/plan
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/config-keys.md
  - crates/ank-cli/src/cli.rs
blocked_by: []
done_criteria: |
  ank config refuses the value 0 (0s, 0m, 0h, 0) for claim_ttl_max, claim_ttl_default, verifiers.<name>.timeout, proof.reuse_max_age and context_budget, with the exit code and the shape of message it gives verifier_jobs 0, naming the key, and writes nothing. A .ank/config.yml carrying any of those zero values by hand is a fault ank check reports on .ank/config.yml. docs/config-keys.md states the lower bound of each. Both halves tested through the binary (GitHub #783).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/8e492065b15e@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/50f864a7acc9@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/b615b8935d7e@046a3bf
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-68a8-release
  - type: test
    ref: local/3db3171de773@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/f076f526a084@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/e8372f0eb730@39cfb58
    tree: scope/ed26c0cdd0e9
    criteria: 38d43b89ab09
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

GitHub #783.
