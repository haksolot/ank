---
id: LOG-a6e16fea41bf
type: log
title: "Implemented. git.rs: ratification_at now returns the commit sha with the anchor, signature_of reads"
created: 2026-08-01T20:33:40Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - docs/ank-spec-v1.1.md
about: TASK-d31af22248d9
seq: 1
schema: 3
version: 1
---

 %G? and %GF through rev-list --format (verify-commit collapses no-signature and cannot-check into one exit code, and those are the two states that must stay apart). human.rs: parse_signers, classify_signature, five states. Measured against git rather than assumed: under SSH with an allowed-signers file, G means the principal matched and U means it did not, so git has already done the allowlist check; under OpenPGP git never reads the file and ank matches %GF itself. Detected by fingerprint shape (SHA256: vs hex). Two traps found by measuring: git reports N for a perfectly signed commit when gpg.format is ssh and no allowed-signers file is configured, so with nothing declared ank abstains entirely; and unchecked signatures are counted once for the corpus, not once per ADR, following the rule section 4 already fixed for entities predating author.
