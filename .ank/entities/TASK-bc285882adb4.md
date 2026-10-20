---
id: TASK-bc285882adb4
type: task
slug: docs-ratifying-md-names-accept-reanchor-as-the-w
title: docs/ratifying.md names accept --reanchor as the way a commit anchor moves into a ref
created: 2026-10-20T19:15:42Z
author: claude-code/orchestrator
status: done
scope:
  - docs/ratifying.md
blocked_by: []
done_criteria: |
  docs/ratifying.md, in its section on anchors made before the ref, names ank accept <id> --reanchor as the command that moves a legacy ratify <id> commit anchor into refs/ank/ratify/<id>, says that it re-signs under whoever runs it and rewrites no history, and lists the refusals the binary gives for it (a ref already carries the document, the file has moved from the commit's anchor, no such commit reachable in this clone), each matching the binary's own help or output.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@9ec2b64
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/79b9
  - type: test
    ref: local/cb4f14e9747c@632bc03
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-core@11c0192e3520
    via: verifier
  - type: test
    ref: local/5aad4158c5b3@632bc03
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
  - type: test
    ref: local/72996a5e0b4b@9ec2b64
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/79b9
  - type: test
    ref: local/d145e4443a66@9ec2b64
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
    reused: local
    by: claude-code/79b9
  - type: test
    ref: local/00e51342c602@9ec2b64
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
    reused: local
    by: claude-code/79b9
  - type: test
    ref: local/ec3dafde58a3@632bc03
    tree: scope/099d0aed43ac
    criteria: c13b7c7a769b
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

PR #658 (TASK-4b25843e6151) added `accept --reanchor` and noted in its body that the docs, out of its scope, should mention it. PR #655 had just rewritten docs/ratifying.md, whose section "Anchors made before the ref" is where the flag belongs.
