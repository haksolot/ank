---
id: TASK-0f86494ecae7
type: task
slug: curl-sh-installs-ank-on-any-linux-and-macos
title: curl | sh installs ank on any Linux and macOS
created: 2026-08-16T03:34:51Z
author: claude-code/opus-5
status: open
scope:
  - install.sh
blocked_by: [TASK-054fd964221f]
done_criteria: |
  install.sh resolves the platform, downloads the matching archive from the latest GitHub release or a version given to it, verifies the published .sha256 before unpacking, installs the binary to a directory on PATH or names the one to add, and refuses with a readable message on an unsupported platform rather than installing nothing silently. A CI job runs it on Linux and on both macOS architectures and asserts ank --version prints the released version. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 2
---

The specification has promised this since v1 and it does not exist. It matters
more than the packaged channels rather than less: the binary is a static musl
build with no libc dependency, so one script covers every Linux distribution
that will never have a native package.

**The checksum is not decoration.** The release already publishes a `.sha256`
beside every archive, and a script that downloads a binary over the network and
runs it without checking the hash it was given is a supply chain with a hole in
the middle. Verify before unpacking, and fail loudly.

An unsupported platform must be a refusal that names what it looked for. A
script that ends in silence leaves the caller with no binary and no idea why.
