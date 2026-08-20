---
id: LOG-b3c98d8ba70e
type: log
title: The cache is in, and two existing tests are what made it honest.
created: 2026-08-20T20:56:41Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
about: TASK-dbef284a166c
seq: 0
schema: 3
version: 1
---



First draft opened the index inside the loop, once per ratification, and made check slower than the gpg calls it replaced: a sqlite open is not free and forty-three of them cost more than they saved. The connection is held per repository now.

Second draft held the allowlist hash beside the connection, so declaring a key changed nothing and the verdict cached under the old list went on being served. The test I wrote for exactly that caught it. The connection is kept and the key is recomputed every lookup, which is a small file hashed per call and nothing beside a signature.

Third, and this is the one I did not see coming: a_signature_git_cannot_read_is_reported_rather_than_passed_over failed. The verdict depends on the local signing configuration too, not only on the commit and the allowlist. Setting gpg.format to something git cannot use turns a readable signature into one it refuses to answer about, and the cache reported the old verdict on a machine that could no longer reach it. That is precisely the cache lying about the one anchor section 8 says holds when everything else can be forged, and it was caught by a test written for another reason entirely. Every gpg.* key now goes into the cache key, taken whole rather than named one by one, because which of them matters depends on the format in use.

E is never written, which is the other half: no local public key is a fact about the keyring rather than about the commit, so importing a key changes the answer on the next run instead of leaving a reader wondering.

Measured: check 7.1s to 4.8s warm on this corpus, and zero gpg processes on any run after the first. Cold and warm report identical findings.
