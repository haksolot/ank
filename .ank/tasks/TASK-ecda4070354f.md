---
id: TASK-ecda4070354f
type: task
slug: nothing-connects-the-binary-s-identity-to-the-sk
title: Nothing connects the binary's identity to the skill's
created: 2026-08-04T05:17:24Z
author: seanl@sean-laptop
status: open
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  A reader holding only the binary and the installed skill can tell whether the two agree, with no repository and no network. The binary prints the SKILL.md revision it was built alongside, that value is derived at build time from skill/SKILL.md rather than typed, and tests/skill.rs proves the printed value and the file agree. Asserted through the binary.
criteria_by: creator
schema: 2
version: 1
---

TASK-548c518cb705 made the binary say what it is. TASK-b495234f192c made the skill say which revision it is. Neither says anything about the other, so telling a stale installed skill from a current one still needs a third value from somewhere -- the repository, a release note, a person who remembers.

Closing it is cheap because both halves exist. The build script already stamps the commit through rev-parse; hashing skill/SKILL.md in the same place costs one more line and no new dependency, and the value is derived rather than kept, which is the property that made the marker itself worth having.

What that buys is the check the original failure needed: an agent that has loaded a SKILL.md and can run ank --version compares two strings it already holds and learns, offline, that its instructions predate its tool.

The design question for whoever claims this: where the value belongs. ank --version prints one line and section 4 says so in as many words, so a second line is a specification change before it is a code change (ADR-63b59c5c26f7). ank status is the other candidate and answers where am I, which is arguably the same question.
