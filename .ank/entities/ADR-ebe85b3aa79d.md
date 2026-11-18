---
id: ADR-ebe85b3aa79d
type: adr
slug: a-verifier-may-declare-vars-inherit-env-and-cwd
title: A verifier may declare vars, inherit_env and cwd, its hash covers them, and ank tells it which task it proves
created: 2026-11-18T15:01:38Z
author: claude-code/plan
status: proposed
scope:
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - .ank/config.yml
  - docs/config-keys.md
constraint: |
  Besides run, timeout and inputs, a verifier may declare vars, a map of names to fixed string values; inherit_env, false or a list of names the caller's environment passes, absent meaning the whole environment as today; and cwd, a directory relative to the repository root that exists inside it; the definition hash covers all six, so changing any one is a verifier change and no run recorded before it is reused. Every run sets ANK_VERIFIER, ANK_BASE (the merge-base with the default branch, empty when none), ANK_TASK (the full id) and ANK_SCOPE (the task's globs, one per line), the last two empty under ank verify; vars may not name an ANK_ variable, and env stays a key the parser refuses by name.
amends: [ADR-7224dea611c1]
schema: 4
version: 1
---

GitHub #787. A verifier inherits the whole environment of whoever ran `ank done`, receives nothing from ank about the task it proves, and cannot declare a variable. Measured in the issue: 49 variables of the caller reach the verifier, none set by ank, and a SECRET_TOKEN of the caller is printed into the kept output.

Amends ADR-7224dea611c1, which stays binding: env is still refused by name, nothing names an image, a platform, a network or a hermeticity, and done still reuses local runs only. What changes is the list of keys a verifier carries and what its hash covers.

The map is called vars and not env, the name the issue proposes, so that env stays free for the execution environment an executor would honour (ADR-d8a328e311b9, GitHub #570), whose own vars sat inside it. inherit_env false passes nothing from the caller, PATH included: a verifier that needs PATH lists it.

Two documents state the opposite today and move with the implementation: SPEC-ff885356f1f9 says a verifier declares nothing but run, timeout, inputs and default; its successor is TASK-2444d0cc6392.
