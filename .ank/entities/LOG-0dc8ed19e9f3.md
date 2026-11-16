---
id: LOG-0dc8ed19e9f3
type: log
title: Scope widened before the first edit. The four-row list of platform packages is written in
created: 2026-11-16T08:38:27Z
author: claude-code/opus-5.5+task-68a8-release
scope:
  - .github/workflows/release.yml
  - .github/workflows/install.yml
  - install.sh
  - npm/**
  - docs/install.md
  - docs/releasing.md
about: TASK-f754268004d2
seq: 1
schema: 4
version: 1
---

 .github/scripts/npm-assemble.sh (packages, target_of, wrapper pins) and .github/scripts/check-version.sh (pin and version compare), and check-version-fixtures.sh copies every manifest into its fixture, so a fifth package left out of either fails the release or the version gate. SECURITY.md names the three platform packages in the npm scope paragraph. A cargo test holding the lists together (wrapper PACKAGES, optionalDependencies, npm/ directories, npm-assemble, check-version, publish loop, release.yml build rows vs install.sh supported_lines) goes in a new crates/ank-cli/tests/release_rows.rs, registered in crates/ank-cli/tests/main.rs. install.yml has no mise step; release.yml npm-smoke needs a Linux-ARM64 mise digest: e90904144ba4377c5fbe385f429830282c68f3b4f9c690bf6c5a2ae9b74a809b, from the GitHub asset digest and SHASUMS256.txt of v2026.10.17, which agree.
