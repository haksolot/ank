---
id: LOG-f9f05649b4ff
type: log
title: "NO_COLOR measured through a pseudo-terminal, ank status: on a tty 538 bytes with 22 escape"
created: 2026-09-20T17:55:01Z
author: claude-code/opus-5+58f9
scope:
  - docs/integrating.md
  - docs/agents.md
about: TASK-58f946514828
seq: 5
schema: 4
version: 1
---

 sequences; NO_COLOR=1 gives 448 bytes and 0; NO_COLOR= (empty) gives 538 and 22, so the empty value is not an opt-out; TERM=dumb gives 0; down a pipe 0 either way. ANK_UPDATE_REPOSITORY measured against a local bare clone tagged v0.9.0 and v0.7.0: ank update --check --json answered {"contract":1,"current":"0.8.0","latest":"0.9.0","newer":true}, and none against the same repository with no tags.
