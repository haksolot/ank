---
id: LOG-ce4e24b5b980
type: log
title: review now carries the signers, and printing them found a second defect.
created: 2026-08-20T16:43:07Z
author: claude-code/opus-5
scope:
  - crates/ank-contract/**
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
about: TASK-8a80b590b356
seq: 0
schema: 3
version: 1
---



The reader itself is small: declared_signers is read once beside the report, never filtered by the perimeter since a signer is a fact about the repository and not about a path, rendered as a MAY RATIFY section beside the queue and as a signers array declared in the contract before it is emitted. The advisory sentence is now one const, NO_RATIFICATION_KEY, used by check as a signal and by review where the section would be: an empty section would read as "declared, and nobody yet", which is not the state SPEC-199de7ac4730 describes.

The defect is in parse_signers, and the golden corpus is what exposed it. It read the entry from the end, key last and keytype before it, which is right for `principal [options] keytype key` and wrong for the line anyone actually writes: an ssh public key carries a trailing comment, so pasting id_ed25519.pub after a principal gives four fields and the last two are the key and the comment. The golden fixture generates its key with -C "ank test", so the moment review printed a type at all it printed `ank`.

It had stayed invisible for two reasons. Nothing displayed these fields, and under gpg.format = ssh it is git and not ank that decides the allowlist, so the parse only had to produce a count. Under OpenPGP it was not invisible at all: `declares` compares the fingerprint against `key`, and against a comment it can only answer no. A gpg entry with a comment would also have been handed to ssh-keygen by git_readable_signers, which is the state TASK-01cc22478782 closed.

Fixed by reading from the front through one helper both readers share: the key type is the first field after the principal that is `gpg` or starts with ssh-, ecdsa-, sk- or webauthn-, the key is the next, the rest is a comment. Options like namespaces="git" match none of those prefixes, so the front reading is safe where the end reading was not, and it survives an option value carrying a space where the old one did not.

Not a widening of the task: the criterion says review names the key type of each principal, and `ank` is not a key type.

One unit test failed on a first run of the binary crate and passed on the next, on the same tree, while a release build was competing for the machine. Not identified and not dismissed; the full workspace run is what decides.
