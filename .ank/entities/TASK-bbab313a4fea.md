---
id: TASK-bbab313a4fea
type: task
slug: the-ank-logo-is-drawn-to-a-terminal-and-to-nothi
title: The ank logo is drawn to a terminal, and to nothing else
created: 2026-08-24T21:59:19Z
author: claude-code/opus-5+planning
status: in_progress
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
blocked_by: []
done_criteria: |
  Both installers draw the ank logo as animated ASCII art before the download starts, and only when a terminal is attached. Piped, redirected, run under a CI runner or run with the disabling flag, they emit no frame and no escape sequence at all. The frames are held in the scripts themselves, so neither installer gains a network request it does not already make, and install.sh stays POSIX sh with no bashisms. install.yml asserts on all three platforms that a non-interactive run emits no escape sequence and that the installed binary answers --version. The disabling flag appears in the usage text and in the comment block that lists the exit codes. cargo test is green.
criteria_by: creator
schema: 4
version: 3
---

The visible half of ADR-5fbd99bf6fd5, and the half that carries its whole risk:
an animation is escape sequences, and escape sequences in a log file, a CI
transcript or a piped install are noise somebody has to clean up.

**So the criterion is written as an absence, not a presence.** It is easy to
assert that frames appear on a terminal, and that assertion would have passed for
every naive implementation that also spat control characters into `install.log`.
What has to be proved is the other branch: nothing at all when nobody is looking.
That is what `install.yml` gains, on all three platforms, because a runner is
precisely a machine with no terminal and it is where this fails first.

**In the scripts, not fetched.** `install.sh` exists for the machine that has
busybox `ash` and a firewall, and an installer that reaches for a logo over the
network before it has reached for the binary is an installer with a second way to
fail before doing anything useful. The art is bytes in the file.

The `assets/` directory already holds `ank.svg` and `ank-dark.svg`. They are the
reference for the shape; nothing here reads them at run time.
