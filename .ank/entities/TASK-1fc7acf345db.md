---
id: TASK-1fc7acf345db
type: task
slug: docs-yml-takes-mdbook-from-mise-instead-of-a-han
title: docs.yml takes mdBook from mise instead of a hand-pinned archive
created: 2026-10-24T07:36:47Z
author: haksolot@vmi3223161
status: done
scope:
  - .github/workflows/docs.yml
blocked_by: [TASK-d7253853996c]
done_criteria: |
  docs.yml installs mise as ci.yml does (pinned version, sha256 per OS, no third-party action) and gets mdBook from mise install --locked; MDBOOK_VERSION and MDBOOK_SHA256 are gone from the workflow; mise x -- mdbook build of docs/ produces a book whose file list equals the one the current workflow produces, compared and recorded with ank log; a pull request run of docs.yml is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@96b0582
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+wdf47
  - type: test
    ref: local/066f8136a9dd@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-core@662d5656b204
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/51456532d207@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-contract@12feb96cb9d5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/8dcdb4163f6d@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-daemon@a6df02a5c56c
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/57fbe31c36eb@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-mcp@426b743c5c6b
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/45258e310c1f@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-tui@5df657e6f69e
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/272c917dbe84@c8f21d0
    tree: scope/f9549ae48eaa
    criteria: 00dc8d92017b
    verifier: test-ank-cli@95de94ea57e3
    via: verifier
schema: 4
version: 3
---

Applies ADR-9c69bd47ce23 once it is ratified; do not claim before.
