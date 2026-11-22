---
name: ank
description: Read a repository's tasks and binding constraints, claim work, and finish it with proof. Use when working in a repo that has a .ank/ directory.
metadata:
  revision: "56f07b7dad23"
---

# ank

This repository keeps its tasks and architecture decisions in `.ank/`, reached
through one CLI. Before anything else, run `ank skills ank` and read what it
prints: that is the contract, served by the binary you are about to use, so it
is always the one that binary obeys.

A claimed task may name its method: `ank claim` prints it after the
criterion, and `ank skills <name>` prints any of them again.

This file holds nothing a release changes. The binary is separate: releases
carry one for Linux, macOS and Windows.
