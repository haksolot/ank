---
id: TASK-bcdc32d08947
type: task
slug: ank-config-refuses-a-peer-declared-by-url-and-na
title: ank config refuses a peer declared by URL, and names the override
created: 2026-09-27T19:03:45Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/peer_url_refusal.rs
blocked_by: [TASK-e95c729e5d43]
done_criteria: |
  Through the binary: ank config peers.<name> <value> refuses at exit 1, writing nothing, a value shaped as a URL -- a scheme followed by :// (https://, ssh://, file://, git://) or the scp form user@host:path -- and the refusal names the per-reader override of ADR-da2819aef598 as the command to use instead. The same holds for ank config --user when it writes a peer override. A relative path, an absolute path, and on Windows a drive-letter path such as C:/src/b, are still accepted. config.yml is byte-for-byte unchanged after every refusal. A test in crates/ank-cli/tests/peer_url_refusal.rs asserts each case through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

Implements the write-time refusal of ADR-96fe1f9d619a.

TASK-c6d184d238e1 measured why it is needed: with two corpora declaring each
other by path, `ank context` serves the peer's ADR; set the peer's
back-declaration to a `file://` URL and `ank config` accepts it, then the ADR
disappears from the reader's context with no warning at all. A URL is read as a
path, the path is "not a corpus", and the binding degrades in silence.

Refusing at write time is the place where the mistake is still visible to the
one who made it. Waits on TASK-e95c729e5d43, which introduces the override the
refusal names and edits config.rs too.

Mind the scp form: `user@host:path` must be refused, but a Windows path with a
drive letter must not, and `repo::is_peer_name` already explains why a single
letter before a colon is a drive and never a name.
