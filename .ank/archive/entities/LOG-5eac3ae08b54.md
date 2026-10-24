---
id: LOG-5eac3ae08b54
type: log
title: "container measurement: docker unavailable to this user (docker.service inactive, socket"
created: 2026-10-17T16:30:46Z
author: claude-code/opus-5.5+4952
scope:
  - crates/ank-tui/tests/terminal/mod.rs
about: TASK-4952001e9596
seq: 11
schema: 4
version: 1
---

 root:docker, sudo needs a password), so the ank-tui suite ran in a bubblewrap container instead: bwrap --unshare-all --share-net with a fresh devpts (--dev /dev, /dev/pts empty but ptmx at start), its own pid namespace (shell pid 2), user namespace mapping uid 1000 to 0, tmpfs /tmp, root filesystem read-only. cargo test -p ank-tui inside it: 281 lib passed, 123 integration passed, 0 failed. Not the rust:1.98-bookworm image of the original report: same kernel, Arch userland, cargo 1.99. On the hang of 2026-10-14: the inherited masters were measured and were not its cause (LOG-6205c); the cause measured is the ptsname(3) crossing, carried by TASK-adea229723df.
