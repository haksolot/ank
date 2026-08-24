---
id: ADR-5fbd99bf6fd5
type: adr
slug: the-installer-greets-a-human-and-is-silent-to-a
title: The installer greets a human, and is silent to a pipe
created: 2026-08-24T21:59:05Z
author: claude-code/opus-5+planning
status: proposed
scope:
  - install.sh
  - install.ps1
constraint: |
  install.sh and install.ps1 animate, ask and teach only when a human is at a terminal. Under curl | sh standard input is the script itself, so a prompt reads from /dev/tty and from nowhere else, and where no terminal is attached the installer asks nothing, draws nothing and installs exactly what it installs with no welcome at all. Every question carries a default that Enter accepts, every offer may be declined, and no offer is a step the installation waits on: the binary is on disk and usable before the first question is asked, and nothing an offer does can change the installer's exit code. A documented flag disables the welcome entirely, and a run with that flag differs in no outcome from an interactive run that declined everything.
schema: 4
version: 1
---

The two installers are 524 and 598 lines that say nothing to the person running
them beyond what failed and what to add to `PATH`. That is a defensible place to
have started, and it is not where this should stop: the moment someone installs
a tool is the one moment they are certainly paying attention, and ank is a tool
whose whole difficulty is knowing what to do with it afterwards.

## The trap this decision exists to name

`curl -fsSL ... | sh` is the documented route, and under it **standard input is
the script**. An installer that calls `read` consumes its own remaining source
and either hangs or executes nothing. This is not a subtlety to remember while
writing the code; it is the reason the rule is recorded as a decision, because
the naive implementation appears to work in every local test where the script is
run from a file, and fails only on the one route people actually use.

`/dev/tty` is the answer, and its absence is the signal: no controlling terminal
means no human, which means a CI runner, a Dockerfile, a provisioning script.
Those must see the installer they see today, to the byte.

## Why the offers cannot be steps

An installation that stops to ask something is an installation that can be
abandoned half-done, and half-done is the worst state for a tool whose next
action is `ank context`. So the ordering is fixed: install, verify, *then*
converse. Whatever the person answers, and whatever the answer turns out to
cost, they already have a working `ank`. This also disposes of the failure mode
where a skill installation that goes wrong takes the exit code of the whole
install with it and a green install is reported red.

## What welcoming is not

Not a menu, not a progress bar over a download that takes a second, not a
newsletter, and not a wizard that must be traversed. Two questions, each
answerable with Enter, and an animation that costs the time it takes to read the
name of the tool. ADR-0c8ab846d262 already settled the underlying split for the
binary -- colour depends on the reader, structure does not -- and this applies
the same reading one layer out, to the script that puts the binary there.
