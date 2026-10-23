---
id: TASK-b896bfbc9c8b
type: task
slug: a-push-the-remote-rejects-is-reported-with-its-r
title: A push the remote rejects is reported with its reason, never as a lost race or a holder
created: 2026-10-23T13:29:13Z
author: claude-code/opus-5.5+plan571-552
status: done
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/**
  - docs/claims.md
  - docs/exit-codes.md
  - docs/integrating.md
blocked_by: []
done_criteria: |
  Through the binary, against a local bare remote whose update hook refuses refs/ank/*: ank claim exits 9, its stderr carries the reason git push --porcelain printed for the ref and names no holder, and afterwards no refs/ank/claims/<id> exists in the clone and ank status shows no claim; on a claim pushed before the hook was installed, ank log and ank release exit 0 with a warning carrying that reason, and log never reports the claim taken over. A genuine lost race still exits 4 naming the holder, both when another clone moved the ref first and when a pre-receive hook moves the ref between advertisement and update. An unreachable remote keeps exit 0 and the unsynchronised warning. Counted with GIT_TRACE, a claim refused by the hook runs exactly one git push, and the retry without a witness runs only after a client-side rejected (stale info). No byte of push's stderr is parsed. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/97ab6a9c06be@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/e0f5d00653ee@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/697685577ba4@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/65453ef64776@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/bf594cb8cf52@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
  - type: test
    ref: local/6db50764c351@7b4a67c
    tree: scope/1c2b1e216f3b
    criteria: d7d0efcd925c
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
    reused: local
    by: claude-code/opus-5.5+b896
schema: 4
version: 4
---

Implements ADR-aaee5307b34c (issue #571). The drift audit of 2026-10-23 measured today's three false reports against a refusing update hook: claim exits 4 naming the caller as holder and keeps the local ref, log warns 'taken over while logging', release says 'released' while the remote ref survives. Read the ADR body for the porcelain line format measured on git 2.47.3 and for why a remote refusal is 9 and not 4. A stock hook's reason is always 'hook declined' or 'pre-receive hook declined'; assert on what git printed, never on a vocabulary.
