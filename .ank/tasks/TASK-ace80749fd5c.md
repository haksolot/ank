---
id: TASK-ace80749fd5c
type: task
slug: the-release-dry-run-passes-on-an-npm-that-would
title: The release dry-run passes on an npm that would refuse it
created: 2026-08-08T16:55:44Z
author: seanl@sean-laptop
status: done
scope:
  - .github/workflows/release.yml
  - .github/scripts/npm-assemble.sh
blocked_by: []
done_criteria: |
  The npm-smoke job's dry-run passes on an npm that refuses an unpublishable
  version. Established by running the job's own command on npm 11 before the fix
  and seeing it fail, and after the fix and seeing it pass, on all three runners.
  The version the smoke job assembles and the flags it passes are chosen so the
  dry-run exercises the same argument publish-npm passes, with no new flag that
  publish-npm does not also carry. cargo test and ank check stay green.
criteria_by: creator
proof:
  - type: test
    ref: "31271789884"
    criteria: 2241bf3237a8
schema: 2
version: 4
---

Found while adding the pi skill to the wrapper (TASK-20b23cd4fb16), by running
the smoke job's own commands locally rather than trusting the green tick.

The job assembles at 0.0.0-smoke and runs:

    npm publish --dry-run --access public "./npm/ank"

npm 11.14.1 refuses it: "You must specify a tag using --tag when publishing a
prerelease version." Assembling at 0.0.0 instead moves the refusal rather than
removing it -- "Cannot implicitly apply the latest tag because previously
published version 0.1.2 is higher". The runners use the npm bundled with node 22,
which does not enforce either, so CI is green today and the trap is dated rather
than absent.

It fails closed, which is the only good news here: a smoke job that cannot dry-run
blocks publish-npm and no tag is spent. The cost is a red release on the day node
22's bundled npm moves, discovered at the moment someone tags, which is precisely
the situation the dry-run step was added to prevent -- v0.1.2 shipped no packages
because nothing had ever run the publish argument.

The shape of the fix is the constraint. Adding --tag to the dry-run and not to
publish-npm would make the smoke job test a command nobody runs, which is how the
v0.1.2 defect happened in the first place. Either both carry it, or the assembled
version is one npm will accept unflagged. Decide that first, then write it once.

## Log
- 2026-08-08T18:36:27Z seanl@sean-laptop — Verified on the runners, both directions. The fix, dispatched on this branch as run 31271789884: 'the publish argument is a path' passes on ubuntu, macos and windows, each reporting npm 12.0.2 -- npm@latest is already past the 11 that broke it, so the pin is what keeps this honest rather than a guess about which npm the runner ships. The negative control, run 31272065545 on a throwaway branch carrying the same npm pin without --tag latest: failure on all three, at that exact step, with 'npm error You must specify a tag using --tag when publishing a prerelease version'. That branch is deleted. So the failure is demonstrated on the runners and not only on a maintainer's machine, and the fix is demonstrated against it.
- 2026-08-08T18:36:29Z seanl@sean-laptop — done, proof test:31271789884
