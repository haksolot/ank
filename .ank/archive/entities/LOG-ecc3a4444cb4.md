---
id: LOG-ecc3a4444cb4
type: log
title: Credential shape chosen by the maintainer, the strategy having already been chosen by the
created: 2026-08-18T18:09:11Z
author: claude-code/opus-5
scope:
  - .github/workflows/release.yml
  - .github/workflows/publish-brew.yml
  - .github/workflows/publish-scoop.yml
  - .github/workflows/publish-apt.yml
  - .github/workflows/publish-winget.yml
about: TASK-2078ab116f63
seq: 1
schema: 3
version: 1
---

 criterion: a fine-grained PAT scoped to this repository with Contents: write, held as RELEASE_TOKEN, used by the publish job of release.yml for gh release create and for nothing else. The GitHub App was priced and declined -- it buys a token that expires in an hour at the cost of an App to create, install and hold two secrets for, and this repository already carries a PAT of the same shape (WINGET_TOKEN, LOG-386ff0c2aa25), so the second credential is the same kind of object the maintainer already renews rather than a new class of thing. The four publish workflows are unchanged: they already trigger on release: published and already read github.event.release.tag_name, which is exactly what the ruled-out chaining would have forced them to stop doing.
