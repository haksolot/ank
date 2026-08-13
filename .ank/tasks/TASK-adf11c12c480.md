---
id: TASK-adf11c12c480
type: task
slug: the-version-check-is-exercised-on-every-push-not
title: The version check is exercised on every push, not only on a tag
created: 2026-08-13T04:32:59Z
author: claude-agent-c
status: done
scope:
  - .github/workflows/ci.yml
blocked_by: [TASK-91462ace35fd]
done_criteria: |
  ci.yml runs .github/scripts/check-version-fixtures.sh on every push and pull request, so a change that breaks check-version.sh is red on the branch that made it rather than on the next tag. It runs the fixtures alone and never compares the tree against a version: there is no tag on a push, and a check that invented one would fail every branch between releases.
criteria_by: creator
proof:
  - type: test
    ref: "31668935610"
    criteria: 4279599f37c9
schema: 2
version: 4
---

Discovered while implementing TASK-d33024e7b98a.

The release workflow now refuses to build when the tag and the manifests
disagree, and it exercises the check against fixtures before trusting its
verdict. Both run on a tag or a dispatch and nowhere else, which leaves the
check itself untested between releases: an edit to `check-version.sh`, or to
the shape of any of the seven manifests it parses, is only discovered by the
run that was supposed to be gated by it.

`ci.yml` is where the fixtures belong. They were left out of TASK-d33024e7b98a
for one reason only: that file was inside the perimeter another task held while
the work was done, and two agents writing one workflow is a conflict rather
than a decision.

What ci.yml must not do is compare the tree against a version. A push carries
no tag, so there is nothing to compare with, and the tree's own agreement is
already asserted by the first fixture -- which reads the version out of
`crates/ank-cli/Cargo.toml` and holds every other literal to it.

## Log
- 2026-08-13T05:01:34Z claude-agent-c — The fixtures run on ubuntu alone, not on the test matrix. Two reasons, and the second is the one that decided it: ubuntu is the platform of use, since the version job in release.yml runs there; and on windows-latest git checks out with autocrlf, so a .sh under .github/scripts/ arrives in CRLF and bash refuses it. Keeping these scripts in LF is a .gitattributes decision about the tree, outside this task's scope. Falsified in both directions before committing: a check-version.sh that always exits 0 turns four fixtures red, and a JSON parser anchored on the wrong indentation turns all five red with 'the file's shape moved' rather than passing on an empty read.
- 2026-08-13T05:06:16Z claude-agent-c — done, proof test:31668935610
