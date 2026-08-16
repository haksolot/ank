---
id: TASK-b693dc062cab
type: task
slug: ank-is-installable-with-winget
title: ank is installable with winget
created: 2026-08-16T03:35:49Z
author: claude-code/opus-5
status: done
scope:
  - packaging/winget/**
  - .github/workflows/publish-winget.yml
blocked_by: []
done_criteria: |
  packaging/winget carries the three manifest files winget requires, rendered for a version with the installer URL and the sha256 the release publishes. A workflow triggered on a published release renders them and opens a pull request against microsoft/winget-pkgs under a token the repository holds. A CI job on Windows validates the rendered manifests with winget validate and installs from them locally, asserting ank --version prints the released version. The pull request is opened and never merged by this repository: the registry reviews it. cargo test --workspace and ank check stay green.
criteria_by: creator
proof:
  - type: commit
    ref: 387382f0fd1db85f4142a86b6aeae7c91fb42c10
    criteria: f85663ec0578
    via: submitted
schema: 3
version: 3
---

winget's registry is `microsoft/winget-pkgs`, a central index that accepts a
manifest and holds no authority over it — the shape ADR-782a3556cf2d names as a
registry rather than a satellite, and the same shape npm already has.

**Validate and install locally before opening anything.** A manifest is only
proved by an install, and the pull request goes to somebody else's repository:
an invalid one costs a reviewer's time and comes back days later. `winget
validate` and a local install from the rendered files are what make the request
worth opening.

The merge is theirs. A job that waited on it, or retried it, would be a pipeline
waiting on a human it does not employ.
