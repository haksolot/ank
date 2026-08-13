---
id: TASK-ab53a0d5654e
type: task
slug: the-readme-argues-and-everything-mechanical-move
title: The README argues, and everything mechanical moves one link away
created: 2026-08-11T23:12:34Z
author: claude-code@sean-laptop
status: done
scope:
  - README.md
  - docs/agents.md
  - docs/getting-started.md
  - npm/ank/README.md
  - .github/ISSUE_TEMPLATE/config.yml
blocked_by: []
done_criteria: |
  README.md is under 140 lines and carries: the problem, one ank context example,
  exactly two install commands, why it works this way, a section situating ank
  against retrieval, against an LLM-maintained wiki and against OKF, what it is
  not, a documentation table, and the licence. It carries no build instructions,
  no repository tree, no verb listing and no second agent-install block.
  
  docs/agents.md exists and holds the four skill routes, what the skill teaches
  and costs, ANK_AGENT and the one-agent-one-tree rule, and the binary channels the
  two commands do not cover.
  
  Each of these has exactly one home in the tree, verified with git grep: the
  install commands, the five-line loop block, the agent install routes.
  docs/getting-started.md keeps a pointer where its Handing the loop to an agent
  section was, not a copy.
  
  Every link in README.md and docs/agents.md resolves, the relative ones against
  the tree and the absolute ones over the network. The README states nothing the
  tree does not provide: no curl-pipe-sh installer, no Homebrew, no Scoop, no
  winget, no pi gallery image, and no claim that the routes install identical
  bytes.
  
  ank check exits 0 and cargo test --workspace passes.
criteria_by: creator
proof:
  - type: test
    ref: "31546304087"
    criteria: 9cd12d87f35b
schema: 3
version: 4
---

The README is 270 lines and is five documents at once: a pitch, an install
guide, a CLI tour, a contributor guide and a design rationale. A reader after any
one of them wades through the other four.

The length is the symptom. The cause is that the README is the fourth copy of
most of what it says: install instructions live in four places, the five-line
loop block in four, the agent routes in two, and "`.ank/` is opaque like `.git/`"
in six — twice inside the README itself. Trimming without moving the duplication
would push the problem around rather than fix it, which is why the criterion is
about homes rather than about line count alone.

Four claims in the README went stale against the ADRs ratified on 2026-08-11:
"it checks at startup", "one flat listing", "flat in `.ank/`", and the
`PreToolUse` hook that TASK-10b8a29fd853 removes. None of them is corrected —
all four sit in the mechanical detail this task deletes, which is what stops the
README needing another pass when those seven ADRs are implemented.

Four things would be easy to write and are false. There is no `curl | sh`
installer, no Homebrew, no Scoop and no winget — the specification plans for them
and TASK-79bb5c779a59 records that nothing is implemented. `pi.image` is set in
neither manifest, so there is no gallery card; TASK-72baa24eef8f is open for it.
And the current README says all four routes "install the same file", which is
true of the source and false of the bytes: v0.1.3 carries the skill at revision
605f771e1955 while the tree carries 3f350ad26459. The page must be written so
that cutting a tag, or not cutting one, leaves nothing on it false.

The npm package resolves three platform packages and exits 9 on anything else, so
an Intel Mac and a linux arm64 box both fall through. Two commands offered
without naming that gap send those readers into a refusal.

One caution on the sequencing: the README must not ship pointing at a
docs/agents.md that does not exist, which is why both are in one task rather than
two.
