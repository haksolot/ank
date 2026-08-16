---
id: LOG-d8bad17ad542
type: log
title: "discrepancy: the criterion asks a CI job to assert ank --version prints the released version on"
created: 2026-08-16T04:56:58Z
author: claude-code/0f86
scope:
  - install.sh
about: TASK-0f86494ecae7
seq: 1
schema: 3
version: 1
---

 both macOS architectures. The latest release is v0.2.0 and carries no x86_64-apple-darwin archive: the Intel row landed in release.yml after that tag, so no published release can be installed on an Intel Mac until the next one. Measured with gh release view v0.2.0 --json assets: six assets, none of them ank-0.2.0-x86_64-apple-darwin.tar.gz. The install job therefore asks the release what it carries before it asserts anything, and takes one of two branches per row: carried, so the install must succeed and ank --version must print the released version; not carried, so the script must refuse with exit 3 naming the archive it looked for and install nothing. The branch is chosen from the release rather than written as a literal, so the Intel row flips to the full assertion the day a release carries it, with no edit. The rest of the path is proven on all three architectures against a staged release served from localhost, which is what makes resolve, verify, unpack, install and the PATH advice testable on an architecture no release carries yet.
