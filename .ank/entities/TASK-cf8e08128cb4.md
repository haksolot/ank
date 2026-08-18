---
id: TASK-cf8e08128cb4
type: task
slug: ank-is-installable-from-the-aur
title: ank is installable from the AUR
created: 2026-08-16T03:36:21Z
author: claude-code/opus-5
status: closed
scope:
  - packaging/aur/**
  - .github/workflows/publish-aur.yml
blocked_by: []
done_criteria: |
  packaging/aur carries a PKGBUILD template rendered from a tag, naming the released archive and the sha256 the release publishes. A workflow triggered on a published release renders it, regenerates .SRCINFO, and pushes both to the AUR under an SSH key held in Actions secrets. A CI job in an Arch container builds the rendered package with makepkg, installs it with pacman -U, and asserts ank --version prints the released version. Nothing in packaging/aur is edited by hand between releases: the job derives the file. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 4
---

**This task exists because of ADR-782a3556cf2d and must not be claimed before it
is accepted.** On the AUR a package *is* a git repository, so there is no shape
in which the PKGBUILD lives in this tree alone. Under the letter of
ADR-e3cb36646d77 that made Arch packaging forbidden — not by anybody's judgement
but by a sentence written about skills. The superseding decision draws the line
where it belongs: a registry a release is pushed into is not a satellite
somebody maintains.

What keeps that honest is derivation, and the criterion says so. The AUR
repository is a push target whose content is a function of a tag. The day
somebody edits it there instead, the drift the old ADR forbade has arrived
through the door the new one opened, and no `check` in this corpus can see it —
because it is not in this corpus.

The name is free: neither `ank` nor `ank-bin` exists on the AUR, measured
2026-08-15. The package installs a prebuilt binary, so the conventional name is
`ank-bin`; that is a choice to make and to record rather than to assume.
