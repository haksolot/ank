---
id: TASK-58e55ec52d7d
type: task
slug: a-git-clone-of-the-repository-installs-into-pi
title: A git clone of the repository installs into pi
created: 2026-08-08T16:17:19Z
author: seanl@sean-laptop
status: open
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
version: 1
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
