---
id: LOG-98fb9ec08f13
type: log
title: "Rehearsed the unix step against a staged release on localhost, six pty runs: accept, Enter,"
created: 2026-08-25T03:23:21Z
author: claude-code/opus-5+installer-skills
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-5a2f1b47f204
seq: 3
schema: 4
version: 1
---

 decline, EOT, --no-welcome, node absent. All six exit 0; the prompt appears exactly once except under --no-welcome where it appears not at all, and npx is reached only on accept and Enter.

The negative control is the part worth recording. Dropping 'offer_skills || :' alone does not fail the step, because 'npx ... || offer_code=$?' already absorbs the failure -- the guard is a backstop for anything else in there, not for that. Writing the offer the naive way instead, with npx unguarded, makes accept and enter exit 1 and the step reports three FAILs. So what the step falsifies is the shape somebody would actually write.
