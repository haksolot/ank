---
id: LOG-5462df47d3af
type: log
title: "npm channel implemented as four packages in the esbuild layout: the wrapper @haksolot/ank plus one"
created: 2026-08-05T05:16:25Z
author: seanl@sean-laptop
scope:
  - .github/workflows/release.yml
  - npm/**
  - docs/**
about: TASK-79bb5c779a59
seq: 0
schema: 3
version: 1
---

 package per target, listed as optionalDependencies with os and cpu, binaries embedded, no postinstall download. The bare name ank was taken on the registry before this project existed (a one-version test module published over a year ago), so the package is scoped; ank-cli and ankor are taken too. The Linux package deliberately declares no libc: the musl build is static and runs on glibc, and libc: [musl] would make npm skip it on Debian, which is most installs. The wrapper forwards the child exit code unchanged and reserves 9 for its own failures. Verified locally on Windows: npx ank --version matches the release binary byte for byte, ank help nosuchverb comes back as 2 through the wrapper, and a missing platform package gives a self-correcting error at 9. Two things measured rather than assumed: ank context outside a repository is code 1 and not 9, so the CI assertion uses the unknown verb instead; and the build job now uploads the loose binary beside the archives, because extracting a zip on the Windows runner would need unzip, which its bash does not have. NPM_TOKEN is not set on the repository yet, so publish-npm will fail until the maintainer adds it; the smoke job runs on workflow_dispatch and proves the pipeline without publishing.
