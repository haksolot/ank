---
id: LOG-16a9d3bb356c
type: log
title: "Measured, not assumed: ank attest --detached exits 0 whether or not the ref reached the remote. On"
created: 2026-08-13T06:54:21Z
author: claude-agent-c
scope:
  - docs/getting-started.md
  - .github/workflows/ci.yml
  - CLAUDE.md
about: TASK-2dff950e5d51
schema: 3
version: 1
---

 a scratch corpus with a real file:// remote it prints pushed:true and the ref is there; with the remote pointed at a path that does not exist it prints pushed:false, warns on stderr, and still exits 0. A pipeline reading the exit code would go green having lost the attestation, which is exactly the silence this task says to remove, so the job reads the flag and the docs say why. Two other findings: check gates the 'done with no test proof' signal on the task appearing done on the default branch, so no diff of .ank/ is needed and the job belongs on that branch rather than on every push; and CLAUDE.md never contained an instruction to carry a run id by hand, so half the criterion was already satisfied and only the pointer was added.
