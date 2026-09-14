---
id: ADR-1556aaffe0c5
type: adr
slug: freshness-is-decided-by-stat-before-hash-and-a-s
title: Freshness is decided by stat before hash, and a stat may only say unchanged
created: 2026-09-14T06:38:09Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/index.rs
constraint: |
  The content hash stays the record of freshness: the index holds one per entity file and a file whose hash diverged is reindexed. Beside it the index records the file's mtime, size and inode, and the instant of its own last write, read from the same filesystem clock. On open, a file is hashed unless all three match what the index holds and its mtime is strictly older than that last write. A stat may only say unchanged, never changed: a mismatch, an unavailable field, or an mtime inside the racy window falls back to the hash. check reindexes fully and takes no shortcut.
schema: 4
version: 1
---

Measured on 2026-09-14, release build, this corpus of 1921 entity files:
every verb that opens the index reads and SHA-256s every file on every open,
which is the whole of what `graph --json` costs -- 0.05 s at 960 files,
0.09 s at 1921, 0.16 s at 3842. mtime and size are never consulted
(`Index::scan`, `crates/ank-cli/src/index.rs`). The cost is linear in the
corpus and paid by a verb that answers about one task.

## The rule, and why it is git's

git's index has the same problem and the same answer: a stat that matches
is trusted, and the one case a stat cannot see -- a rewrite in the same
second leaving the same size -- is closed by the racy rule. A file whose
mtime is not older than the index's own write is one the index cannot vouch
for, so it is hashed. That predicate is stated here rather than left to the
implementation, because a freshness rule that is wrong on one side makes
every verb lie about the corpus, and a lying `find` is worse than a slow one
(the reasoning of ADR-f3d1dea65d84, applied to the index rather than to
`status`).

## What does not move

SPEC-aa4b §6 says the index stores a content hash per file and compares the
files against those hashes. It still does. The stat is a pre-check that may
skip a hash and can never replace one, and deleting `index.db` stays safe.
`check` reads the corpus because that is its answer; it takes no shortcut.

## The risk, named

A filesystem with a coarse mtime widens the racy window, and the fallback
is the hash: wider window, more hashing, never a stale answer. An inode is
unavailable on some Windows filesystems; an unavailable field is a
mismatch, and a mismatch hashes.
