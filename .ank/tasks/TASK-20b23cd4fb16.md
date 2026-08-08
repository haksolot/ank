---
id: TASK-20b23cd4fb16
type: task
slug: the-npm-package-is-a-pi-package-skill-included
title: The npm package is a pi package, skill included
created: 2026-08-08T16:17:02Z
author: seanl@sean-laptop
status: open
scope:
  - npm/**
  - .github/scripts/npm-assemble.sh
  - .github/workflows/release.yml
  - .gitignore
blocked_by: []
done_criteria: |
  npm/ank/package.json carries the pi-package keyword, a pi.skills key, and skills/
  in files. .github/scripts/npm-assemble.sh copies skill/SKILL.md to
  npm/ank/skills/ank/SKILL.md, and .gitignore excludes that path, so no copy of the
  skill is committed. The npm-smoke job asserts the packed wrapper contains
  skills/ank/SKILL.md byte-identical to skill/SKILL.md, and fails when it does not.
  Verified by running npm-assemble.sh and npm publish --dry-run locally: the listing
  shows the skill. ank --version still prints skill revision 605f771e1955.
  cargo test and ank check stay green.
criteria_by: creator
schema: 2
version: 1
---

The pi route, and the one that carries discovery: a package published with the
pi-package keyword is listed on pi.dev/packages, which is how pi users find
anything at all -- there is no closed marketplace to submit to.

pi resolves skills from the pi.skills manifest key or, failing that, from a
conventional skills/ directory holding <name>/SKILL.md folders. The wrapper
package is where this belongs rather than a separate package, because then
pi install npm:@haksolot/ank delivers the binary and the skill in one command
instead of asking a user to install two things that must stay in step.

The copy is made at assembly time, never committed. That is the shape
ADR-e3cb36646d77 allows and the shape this script already uses: npm-assemble.sh
copies LICENSE into each package the same way, and .gitignore already carries
npm/ank-*/bin/, npm/*/LICENSE and npm/*/*.tgz for exactly this reason. Follow
those three lines rather than inventing a fourth mechanism.

The smoke assertion is the point of the task, not a nicety. Without it a release
publishes a pi package with no skill in it and nothing says so -- the wrapper
still resolves a binary, npx ank --version still passes, and the failure is
visible only to a pi user weeks later. The job already asserts the version and
the exit code through the wrapper; this is the third assertion of the same kind.

Note that pi does not put the binary on PATH when it installs an npm package: it
loads resources. The skill's Install section already tells a reader the skill is
not the binary, so nothing there needs to change -- and must not, since that body
is frozen and TASK-e70d28a5fba8 already holds the pen on it.
