---
id: TASK-530f27a4aeb3
type: task
slug: a-push-refused-before-it-reaches-the-remote-s-up
title: A push refused before it reaches the remote's update, such as a missing credential, is not reported as a lost race
created: 2026-10-23T14:43:14Z
author: claude-code/opus-5.5+b896
status: done
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Through the binary, against a remote that answers ls-remote but refuses the push before any ref is updated (no credential, or a URL readable and not writable), with no porcelain status line for the ref and the remote ref still equal to the lease: claim, log, done, release and close never report the task held or moved by anyone, never fetch the remote record over the local write, and report the failure with the command that diagnoses it; done completes locally as it does for an unreachable remote. The case is decided in an ADR amending ADR-aaee5307b34c, whose clause keeping the ls-remote reading alone when no status line exists is what reads it as a lost race today. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@93d219b
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/efd22f28d064@03e4c38
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w3917
  - type: test
    ref: local/9a9c0c2fb196@03e4c38
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w3917
  - type: test
    ref: local/4638e0f758a4@fd43ec5
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w582f
  - type: test
    ref: local/e32bf5fd3d37@93d219b
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/c5dc120150eb@93d219b
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/dd5acff41466@93d219b
    tree: scope/806c972d089c
    criteria: 89d5663caf66
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Measured on 2026-10-23 while closing TASK-b896bfbc9c8b. With GIT_CONFIG_GLOBAL pointing at a file without the credential helper, git push to https://github.com/haksolot/ank exits 128 with 'fatal: could not read Username' and prints no porcelain status line; ls-remote still answers because the repository is public, and the remote ref equals the lease. push_ref then returns Refused{holds}, claim::push maps it to Cas::Lost and fetches the remote record over the local one, and ank done ran every verifier green before exiting 4 'moved while it was being completed', leaving the task file written done while the ref stayed a claim. ADR-aaee5307b34c keeps the ls-remote reading alone for a push with no status line, which is exactly this case: nobody won, the ref is where the lease said it was. That clause needs a decision before code, hence an ADR first.
