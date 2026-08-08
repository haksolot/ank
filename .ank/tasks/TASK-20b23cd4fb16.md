---
id: TASK-20b23cd4fb16
type: task
slug: the-npm-package-is-a-pi-package-skill-included
title: The npm package is a pi package, skill included
created: 2026-08-08T16:17:02Z
author: seanl@sean-laptop
status: done
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
proof:
  - type: test
    ref: "31268193741"
    criteria: fefb7a7b8347
schema: 2
version: 5
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

## Log
- 2026-08-08T16:54:00Z seanl@sean-laptop — Verified locally end to end. npm-assemble.sh copies skill/SKILL.md into npm/ank/skills/ank/SKILL.md; npm publish --dry-run lists it at 3.2kB among five files; the tarball assertion matches sha256 9e4161fb... on both sides. Negative direction proved too: repacked with skills/ removed, the same assertion exits 1, so the step fails when the skill is absent rather than passing vacuously. pi accepts the package -- pi install ./npm/ank succeeds and pi list shows it -- but pi could not be made to print its resolved skills on this machine, so what is proved is that the manifest is accepted, not that the skill loads. Two findings outside this criterion. First, npm 11.14.1 refuses the dry-run the smoke job runs: 0.0.0-smoke is a prerelease and needs --tag, and even 0.0.0 is refused as lower than the published 0.1.2. CI passes today on the runner's npm 10, so this is latent, and it fails closed. Second, git checkout -- npm/ to undo the version stamping also reverts hand edits to the same file; assemble in a throwaway tree or restore the version field alone.
- 2026-08-08T17:02:09Z seanl@sean-laptop — The new smoke step ran on all three runners via a workflow_dispatch of release.yml on the branch (run 31268193741): 'the wrapper carries the skill' is success on ubuntu, macos and windows. This is what a PR's ci could not tell us -- release.yml fires only on a tag or a dispatch, and the step is bash using tar -xzOf and sha256sum, which is exactly the kind of thing that is right on one platform and wrong on another.
- 2026-08-08T17:05:07Z seanl@sean-laptop — done, proof test:31268193741
