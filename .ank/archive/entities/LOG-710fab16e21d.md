---
id: LOG-710fab16e21d
type: log
title: what the install job proves, and what only a published release can prove. scoop bucket add clones
created: 2026-08-16T04:34:56Z
author: claude-code/e38b
scope:
  - bucket/ank.json
  - .github/workflows/publish-scoop.yml
about: TASK-e38b9597ee30
seq: 1
schema: 3
version: 1
---

 whatever git URL it is given, so the job has two sources: on release: published it uses the repository URL a user would type, because the manifest job has just pushed the manifest to the default branch; on a pull request there is no such URL yet -- the manifest under test is not on the default branch -- so the checkout is pushed into a bare repository beside it and that is the URL added. Everything downstream is identical and real either way: scoop resolves the manifest, downloads the zip from the GitHub release over the network, verifies the sha256 itself, extracts extract_dir and shims bin, and then ank --version is run. So a pull request proves the manifest installs and the binary names the version; the one hop it cannot prove is that github.com/haksolot/ank serves it as a bucket, and that is proved by the same job on the next release. Verified locally that the parts scoop depends on hold: the release zip wraps its contents in ank-0.2.0-x86_64-pc-windows-msvc/ so extract_dir is right, the ank.exe inside prints 'ank 0.2.0 (c86eeeb, skill 3f350ad26459)' which is why the assertion matches the version token and not the whole line, and the manifest validates against Scoop's own schema.json. Scoop itself is not installed on this workstation and installing it is a change outside the repository, so the install loop was not run here -- it runs on windows-latest in CI.
