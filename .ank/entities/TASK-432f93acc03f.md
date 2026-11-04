---
id: TASK-432f93acc03f
type: task
slug: a-signature-verdict-this-machine-could-not-check
title: A signature verdict this machine could not check is never cached as final
created: 2026-11-04T09:17:55Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/signature_cache.rs
  - crates/ank-cli/tests/main.rs
blocked_by: []
done_criteria: |
  Through the binary, in a scratch repository whose ratification is a commit signed with a GPG key declared under signers: and whose public key the keyring running the test does not hold: ank check reports the signature as unchecked; after the key is imported into that keyring and with the index left in place, the next ank check reports it verified and no signature-unchecked signal; a verdict git gives as good, bad, expired or revoked is still read from the cache on the second run without a new git verify process, counted with GIT_TRACE and logged. A test in crates/ank-cli/tests/signature_cache.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Measured 2026-11-04 on this machine: gpg was not installed, so 293 ratification commits were cached in the index's signatures table (keyed by commit_sha and signers) as N, which check reads as unchecked (47 signatures, one signal). After installing gpg and importing 739A603F...1482554A, git log %G? reported U (good) for them, but ank check kept the signal until the index file was removed; a fresh worktree reported none. The cached verdict depends on the machine's keyring and gpg, which the cache key does not name. The old index is kept at /tmp/claude-1000/-home-haksolot-Projects-ank/ef40602f-bd92-478a-ba61-5ec238a20a21/scratchpad/index.db.12.bak.
