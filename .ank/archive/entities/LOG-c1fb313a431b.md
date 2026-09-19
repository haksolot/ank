---
id: LOG-c1fb313a431b
type: log
title: "self-review before close. Against the criterion: install.sh runs \"${install_dir}/ank\" skills"
created: 2026-09-13T15:01:20Z
author: claude-code/37b0
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-37b018fee1d3
seq: 2
schema: 4
version: 1
---

 --install < /dev/null >&2 and install.ps1 runs & $Ank skills --install with $destination passed in; both print nothing of their own on exit 0 and 'ank skills --install exited N' otherwise; grep -c 'npx skills add' is 0 in both scripts and each install.yml job now fails if it reappears; install.yml stages a second release (9.9.8) carrying ank built from the tree, the accept case asserts the stub npx was handed <tmp>/ank-skills-* by the verb, exit 0 and the binary in place, and the nonode case compares what followed the answer to the verb run by hand on the same PATH. Decline/Enter/EOF/flag cases are unchanged apart from installing the built binary. Nothing else in either installer changed: --help line, the offer function, one call-site argument. Against the constraints (ank context on install.sh, install.ps1, .github/workflows/install.yml: ADR-e1d7, ADR-5fbd, ADR-1ea3, ADR-221a, ADR-8b30): the question still reads /dev/tty and [Console]::ReadLine after install, the offer stays wrapped so no exit code changes, one executable is still installed; nothing found. Against 0.7.0, which refuses the verb with exit 1, the installers report 'ank skills --install exited 1' and exit 0, which is the failure branch the rehearsal proves.
