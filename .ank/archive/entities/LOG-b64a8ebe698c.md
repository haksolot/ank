---
id: LOG-b64a8ebe698c
type: log
title: Measured on this tree, ank 0.7.0 (50f4b39). ls crates/ prints six directories -- ank-cli,
created: 2026-08-31T04:10:04Z
author: claude-code/opus-5+corpus
scope:
  - .ank/entities/**
about: TASK-58ccbc4683ef
seq: 0
schema: 4
version: 1
---

 ank-contract, ank-core, ank-daemon, ank-mcp, ank-tui -- and cargo metadata --no-deps names exactly the same six packages, so the directory listing is not a superset of the workspace. ADR-85e6bbb195b8 says 'the crates are ank-core and ank-cli', which names two of six.

Every other clause re-measured and holds: the binary is ank (ank --version prints 'ank 0.7.0 (50f4b39, skill d25cedf8fe35)'); the state directory is .ank/; git for-each-ref finds refs/ank/remote 311, refs/ank/proof 186, refs/ank/watch 148 and refs/ank/claims 8, all under refs/ank/*; the identity variable is ANK_AGENT (ank status prints 'identity claude-code/opus-5+corpus (ANK_AGENT)'); and 'ankor' occurs zero times in tracked files outside .ank/. Inside the corpus ank find ankor returns only historical anchors -- log entries already written and the done rename task -- which is the exception the constraint itself carves out.
