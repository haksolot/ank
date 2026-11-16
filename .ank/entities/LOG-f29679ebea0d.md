---
id: LOG-f29679ebea0d
type: log
title: "Design, before the first edit. release.yml: the attestation is a job of its own (attest, needs"
created: 2026-11-16T09:39:15Z
author: claude-code/opus-5.5+task-68a8-release
scope:
  - .github/workflows/release.yml
  - install.sh
  - install.ps1
  - crates/ank-cli/src/update.rs
  - crates/ank-cli/tests/cli.rs
  - docs/install.md
about: TASK-1ea826115f7a
seq: 1
schema: 4
version: 1
---

 build, on tags and dispatches) holding id-token: write and attestations: write, because ADR-24e3 keeps contents: write on publish alone and build runs tree code on five machines; it verifies what it attested with gh attestation verify as a positive control, and publish waits on it. Installers: provenance is checked when gh is on PATH and authenticated (gh attestation verify needs the API), with --repo haksolot/ank and --signer-workflow .github/workflows/release.yml; a tag at or below v0.9.4 predates attestations and is sha256 only by construction; --require-provenance / -RequireProvenance refuses with 5 when gh cannot check and 4 when the check fails or the release predates it; a failed verification refuses with 4 whether or not the flag is set. ank update: v<version> above 0.9.4 fetches the installer at refs v<version>, at or below falls back to main, whose flags old tagged scripts may not take. Scope widened: crates/ank-cli/tests/update.rs (where the update tests live; the task named cli.rs), a new crates/ank-cli/tests/install_provenance.rs registered in tests/main.rs (install.sh run end to end against a staged release on a local listener, gh stubbed, PATH a link farm without the real gh), .github/workflows/install.yml (install.ps1 exercised on Windows with a stub gh), crates/ank-cli/tests/golden-json if update help moves.
