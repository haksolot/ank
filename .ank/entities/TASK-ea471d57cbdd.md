---
id: TASK-ea471d57cbdd
type: task
slug: the-repository-stops-being-a-pi-package-and-the
title: The repository stops being a pi package, and the npm package ships no copy of the skills
created: 2026-10-25T08:12:15Z
author: claude-code/plan
status: in_progress
scope:
  - package.json
  - npm/**
  - .github/scripts/npm-assemble.sh
  - .github/workflows/release.yml
  - assets/make-card.mjs
  - assets/ank-card.png
  - docs/install.md
  - .gitignore
blocked_by: []
done_criteria: |
  Under ADR-33f2888f4ac7: package.json at the repository root is absent; npm/ank/package.json has no pi key, no pi-package keyword, and no skills/ entry in files; npm-assemble.sh copies no skill and writes no npm/ank/skills/; release.yml has no step or check that expects skills/ inside the npm tarball, and the npm smoke still installs the package and runs ank --version; assets/make-card.mjs and assets/ank-card.png are absent; .gitignore no longer names npm/ank/skills/; docs/install.md has no pi section and no pi install line; git grep -I -iw pi over tracked files, excluding .ank/, crates/ and .github/ISSUE_TEMPLATE/, prints nothing; cargo test and ank check are green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 4
---

Everything removed here exists only for pi; ADR-33f2888f4ac7 has the
inventory. The binary already carries the skills (ADR-e1d750884b82), so an npm
user loses nothing: `ank skills --install` places them.

The release smoke job at release.yml ~412-443 hashes skills/<name>/SKILL.md
inside the packed tarball; that check goes, the install-and-run smoke stays.
Rehearse release.yml before the next tag (docs/releasing.md).

`pi@host` in golden.rs and the bug template is an identity string, not the
harness; leave it.
