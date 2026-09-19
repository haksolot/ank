---
id: LOG-bd099b629f6e
type: log
title: The two scripts cannot move a cursor the same way. install.sh uses ESC[12A and ESC[K; install.ps1
created: 2026-08-24T23:02:19Z
author: claude-code/opus-5+install-logo
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-bbab313a4fea
seq: 2
schema: 4
version: 1
---

 uses $Host.UI.RawUI.CursorPosition, because Windows PowerShell 5.1 in conhost does not enable virtual terminal processing for its own output and would print ESC[12A as glyphs -- on precisely the shell that file claims as its floor. The consequence for the proof: install.ps1 emits no escape sequence on any run, so the escape assertion is a floor on Windows and the frame itself has to be asserted absent beside it.
