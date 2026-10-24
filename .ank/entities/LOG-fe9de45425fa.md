---
id: LOG-fe9de45425fa
type: log
title: "Self-review. Criterion: mise step (ci.yml's shape) in npm-smoke and, by YAML alias, publish-npm,"
created: 2026-10-24T10:29:32Z
author: claude-code/opus-5.5+w4182
scope:
  - .github/workflows/release.yml
about: TASK-41822c992809
seq: 5
schema: 4
version: 1
---

 the two jobs that used actions/setup-node, which no longer appears in the file; node from mise install --locked; the npm@latest step and its comment are untouched in the diff; rustc read from all four build logs; dispatch green, nothing published, run id logged. Not exercised by a dispatch: publish-npm (tag-only), so its *mise alias and the registry step that replaces setup-node's registry-url .npmrc are proven only by the workflow parsing and by the identical anchored step running in npm-smoke; and the macOS-X64 digest, since build installs no tool beyond rustup and runs no mise -- the digest is carried for the Intel row and measured three ways (LOG-91089d3220cf). Constraints from ank context .github/workflows/release.yml: ADR-24e3 holds (no permissions change, contents: write still only on publish); ADR-9c69 holds (node through mise, mise pinned by version+sha256, no third-party action); ADR-221a/1ea3/8b30/3397/e1d7 untouched by the diff. Nothing found beyond that.
