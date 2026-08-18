---
id: LOG-1b742efde8c6
type: log
title: Not closable here, and it was known before the work started. The criterion is measured on a tag --
created: 2026-08-18T18:21:20Z
author: claude-code/opus-5
scope:
  - .github/workflows/release.yml
  - .github/workflows/publish-brew.yml
  - .github/workflows/publish-scoop.yml
  - .github/workflows/publish-apt.yml
  - .github/workflows/publish-winget.yml
about: TASK-2078ab116f63
seq: 3
schema: 3
version: 1
---

 'gh run list names the four with event=release', the three manifests carrying the new version on the default branch, the winget submit job having run -- and none of that can be observed until two acts only the maintainer can perform: gh secret set RELEASE_TOKEN with a fine-grained PAT on this repository, Contents: write, and then a tag pushed. Everything in this tree that the criterion depends on is written and verified: cargo fmt --check passes, cargo test --workspace is 584 passed 0 failed, and the two new scripts were falsified against fixtures rather than merely read.

Recorded here rather than left to the next holder to rediscover: the first run under RELEASE_TOKEN is also the first exercise of the channels job, and if the four do start while --branch fails to match the tag, that job is red on a release that actually worked. The diagnostic it prints separates the two cases by name, and the fix in that case is this file, not the token.

Also discovered, and not in this perimeter: closing TASK-cf8e08128cb4 took check from one fault to three, because a closed task's dead scopes are reported at fault severity forever. Filed as TASK-305cf978d37d rather than repaired by amending a closed task's scope, which would silence check by making the entity lie about what it was about.
