---
id: TASK-6de3f29911bd
type: task
slug: the-package-manager-channels-are-dismantled
title: The package-manager channels are dismantled
created: 2026-08-19T16:21:40Z
author: claude-code/5
status: open
scope:
  - .github/workflows/publish-brew.yml
  - .github/workflows/publish-scoop.yml
  - .github/workflows/publish-apt.yml
  - .github/workflows/publish-winget.yml
  - .github/workflows/release.yml
  - .github/scripts/apt-repo.sh
  - .github/scripts/brew-formula.sh
  - .github/scripts/winget-manifests.sh
  - Formula/**
  - bucket/**
  - packaging/**
  - docs/agents.md
blocked_by: [TASK-091023648de0]
done_criteria: |
  The tree carries no publish-brew, publish-scoop, publish-apt or publish-winget workflow, no Formula/, bucket/ or packaging/ directory, and none of apt-repo.sh, brew-formula.sh, winget-manifests.sh. release.yml creates the release under the workflow's default GITHUB_TOKEN, grants contents: write to the publish job and to no other, and carries no step naming RELEASE_TOKEN or asserting channel runs. PR microsoft/winget-pkgs#418653 is closed unmerged and the haksolot/winget-pkgs fork is deleted. The GitHub Pages site of this repository no longer serves an apt Release file at /deb/dists/stable/Release. gh secret list names none of APT_GPG_PRIVATE_KEY, WINGET_TOKEN, RELEASE_TOKEN. The install section of docs/agents.md names exactly three routes: npm, curl | sh, and the PowerShell one-liner. cargo test and ank check are green.
criteria_by: creator
schema: 3
version: 1
---

Executes ADR-221aa5da440a, ADR-8b3045cf11db and ADR-24e306277bd4. Do not
claim before those three are ratified: until then ADR-768374fe6076 still
binds release.yml to the RELEASE_TOKEN this task removes, and
ADR-782a3556cf2d still scopes the directories this task deletes.

What each clause of the criterion is about:

The winget exit must happen before Microsoft merges. PR 418653 is open at
planning time; merged, it would leave a Haksolot.Ank 0.3.0 in the winget
registry forever, pointing at release assets this project still hosts but a
channel nobody maintains. Close the PR with a comment saying the package is
withdrawn, then delete the fork WINGET_TOKEN pushed to.

The Pages site exists only for the apt tree: publish-apt.yml is its sole
deployer and install.sh is served from raw.githubusercontent.com, not Pages.
An apt source list left on a user's machine must stop resolving an index
signed by a key whose private half is about to be deleted; deploying an empty
tree (or disabling Pages, which is one gh api call) both satisfy the
criterion, and the choice belongs beside the change.

The secrets go last, after the workflows that name them are gone, so no
intermediate commit leaves a workflow red on a missing secret.

docs/agents.md carries the brew, scoop, apt and winget sections and the
"channels beyond npm" sentence; docs/getting-started.md names only npm and
the release page and needs nothing. TASK-6704242b47f3 owns the skill-copy
question inside release.yml's package step and is deliberately not folded in
here: its scope intersects this one, so do not hold both claims at once
unless you take them in sequence.
