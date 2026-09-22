#!/bin/sh
#
# The tasks the attest job anchors: read `ank check --json` on stdin, print one
# task id per line.
#
#   $ ank check --json | sh .github/scripts/unanchored.sh
#
# A script rather than a filter inline in ci.yml, so the workspace suite can run
# it over a corpus it built and hold it to what the binary prints
# (crates/ank-cli/tests/attest_ids.rs). Inline, the selection was tested by
# nothing, and it drifted: it matched one of the two wordings check gives this
# finding and silently skipped the other (TASK-44e6b39c13d6).
#
# Two wordings, one finding, emitted by the same rule in human.rs:
#   done with no test proof: nothing external anchors it
#   done with no attested test proof: 'test:<ref>' was submitted, not attested
# The second is a task somebody closed on a test reference they typed. It is
# exactly as unanchored as the first, and the job has to anchor both.
#
# POSIX sh and not bash, called as `sh <path>` by the job and by the test alike:
# on ubuntu `sh` is dash, so a bashism would pass on macOS and break the job.
set -eu

jq -r '
  .findings[]
  | select(.message | test("^done with no (attested )?test proof"))
  | .subject'
