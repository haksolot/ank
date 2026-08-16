---
id: LOG-7cb18ada5e06
type: log
title: Three things the winget submission turns on, none of them obvious from the criterion.
created: 2026-08-16T16:27:13Z
author: claude-code/opus-5
scope:
  - packaging/winget/**
  - .github/workflows/publish-winget.yml
about: TASK-b693dc062cab
seq: 0
schema: 3
version: 1
---



The pull request is built through the Git Data API and never by cloning. A fork shares its object store with the repository it came from, so a branch created on haksolot/winget-pkgs can point straight at microsoft/winget-pkgs' current head: no sync, and no clone of a registry that carries hundreds of thousands of manifests to add three files to it. Three blobs, one tree on that base, one commit, one ref update -- and one commit rather than the three the contents API would write, because three commits adding three halves of one manifest is a diff a reviewer has to reassemble.

The token has to be a classic PAT with public_repo. A fine-grained token cannot act on a repository its owner does not own, and opening a request against microsoft/winget-pkgs is exactly that. The submit job says so by name when the secret is missing, because the API's own answer to a missing token here is a 404 that reads like the fork is gone.

RelativeFilePath is inside the directory the archive wraps, verified against the published zip rather than inferred: ank-0.2.0-x86_64-pc-windows-msvc/ank.exe, not ank.exe. Same trap as extract_dir on the Scoop side, and it fails at install time on a user's machine rather than at validation. The derived sha256 also matches the one bucket/ank.json already carries, which is two independent derivations landing on the same number.
