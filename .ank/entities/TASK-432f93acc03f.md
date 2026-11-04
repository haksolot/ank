---
id: TASK-432f93acc03f
type: task
slug: a-signature-verdict-this-machine-could-not-check
title: A signature verdict this machine could not check is never cached as final
created: 2026-11-04T09:17:55Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/signature_cache.rs
  - crates/ank-cli/tests/main.rs
blocked_by: []
done_criteria: |
  Through the binary, in a scratch repository whose ratification is a commit signed with a GPG key declared under signers: and whose gpg.program cannot run (gpg missing, git answers N): ank check reports the signature as unchecked; after gpg is available and the key imported, with the index left in place, the next ank check reports it verified and no signature-unchecked signal; the same holds when gpg runs but the keyring lacks the public key (git answers E); a verdict git gives as good, bad, expired or revoked is still read from the cache on the second run without a new git verify process, counted with GIT_TRACE and logged. A test in crates/ank-cli/tests/signature_cache.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@49f3faf
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/8f97b1811043@af02e66
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-64f9-out-of-scope
  - type: test
    ref: local/13209ab16a2c@af02e66
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-64f9-out-of-scope
  - type: test
    ref: local/58851ecff8c4@af02e66
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-64f9-out-of-scope
  - type: test
    ref: local/5a0ef7e38db7@49f3faf
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/ba6c38dc9227@49f3faf
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/c8745fcab81b@49f3faf
    tree: scope/653b21f6f2d4
    criteria: 4b491279c385
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Measured 2026-11-04 on this machine: gpg was not installed, so 293 ratification commits were cached in the index's signatures table (keyed by commit_sha and signers) as N, which check reads as unchecked (47 signatures, one signal). After installing gpg and importing 739A603F...1482554A, git log %G? reported U (good) for them, but ank check kept the signal until the index file was removed; a fresh worktree reported none. The cached verdict depends on the machine's keyring and gpg, which the cache key does not name. The old index is kept at /tmp/claude-1000/-home-haksolot-Projects-ank/ef40602f-bd92-478a-ba61-5ec238a20a21/scratchpad/index.db.12.bak.
