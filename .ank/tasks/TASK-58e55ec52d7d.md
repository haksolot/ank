---
id: TASK-58e55ec52d7d
type: task
slug: a-git-clone-of-the-repository-installs-into-pi
title: A git clone of the repository installs into pi
created: 2026-08-08T16:17:19Z
author: seanl@sean-laptop
status: in_progress
scope:
  - package.json
  - README.md
blocked_by: []
done_criteria: |
  A package.json at the repository root declares the pi manifest -- private, the
  pi-package keyword, and a pi.skills path resolving skill/SKILL.md -- with no
  dependencies and no copy of the skill. The path form is the one pi actually
  accepts, established by running pi install against this working copy and seeing
  the skill load, not by reading the documentation. ank --version still prints skill
  revision 605f771e1955, cargo test and ank check stay green, and no npm publish
  flow reaches this file.
criteria_by: creator
schema: 2
version: 3
---

The second pi route, for a user who installs from source:
pi install git:github.com/haksolot/ank. pi clones the repository and looks for a
manifest or a conventional skills/ directory. This repository has neither, so the
git route finds nothing today, and a root manifest is what opens it.

The open question is the path form, and it decides the file:

  pi discovers skills from a manifest path, or from a conventional skills/
  directory where it "recursively finds SKILL.md folders". Whether a manifest path
  naming ./skill -- a directory holding SKILL.md directly, not skills/<name>/SKILL.md
  -- is accepted is not stated. Try ./skill first. If it is refused, the fallbacks
  are "." for the root, or the file path itself. Do not answer this from the
  documentation; run it.

pi also runs npm install when a package.json exists in a cloned package. With no
dependencies that is a no-op, but confirm it rather than assume it.

private: true, because this manifest exists to be read by pi from a clone and
must never be publishable as a package in its own right. The published package is
@haksolot/ank under npm/ank/, and TASK-20b23cd4fb16 is what makes that one a pi
package too.

A package.json at the root of a Rust repository is an oddity worth accepting
knowingly: it declares one manifest key and nothing else, it pulls in no
dependency, and it is the only way the git route works without moving
skill/SKILL.md -- a move that would break the ADR scopes anchored on skill/**,
the freeze hash in build.rs, and the tests that hold it.

## Log
- 2026-08-08T17:14:37Z seanl@sean-laptop — Both questions answered by running, not by reading. Question one: pi's own loader accepts a path naming a directory that holds SKILL.md directly. Its documented rule is 'if a directory contains SKILL.md, treat it as a skill root and do not recurse further', and calling loadSkillsFromDir from pi's dist against skill/ returns one skill named ank, description intact, zero diagnostics. So pi.skills is ./skill and no skills/<name>/ shape is needed. Question two: pi install ./ succeeds against this working copy, pi list shows it, and nothing is fetched -- npm install at the root is a clean no-op. Two things learned that the task body got wrong. The manifest needs no version field at all: pi installs it and npm is content, so the root manifest adds no sixth place where 0.1.2 is written by hand, which was the reason to hesitate. And private: true is weaker than assumed -- npm 11 ignores it under --dry-run and reports a successful publish of 205 files. It still blocks a real publish, but the guard that matters is that no publish flow here targets the root: every npm publish in release.yml names ./npm/<pkg>, and npm-assemble only ever npm pkg set inside npm/. The name is scoped to @haksolot/ank-pi rather than the bare ank the body proposed, because ank is an existing package on the registry owned by someone else.
