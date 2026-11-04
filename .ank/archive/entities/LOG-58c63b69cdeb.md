---
id: LOG-58c63b69cdeb
type: log
title: Session-start pointer named `ank skills install`, which the binary refuses (exit 1, 'skills'
created: 2026-10-25T15:08:38Z
author: claude-code/opus-5.5+w582f
scope:
  - crates/ank-cli/src/hooks.rs
  - crates/ank-contract/src/env.rs
  - docs/environment.md
  - crates/ank-cli/tests/hooks_guard.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/hooks_run.rs
  - crates/ank-contract/tests/environment_page.rs
about: TASK-582f6758fb0b
seq: 7
schema: 4
version: 1
---

 accepts 0); fixed to `ank skills --install`. Regression: hooks_run::every_command_the_pointer_names_is_one_the_binary_accepts extracts every backticked ank command from additionalContext and checks it against ank help <verb> (flags from the flags/global lines, positional slots and <a|b> alternatives from the usage line). Watched it red on the old text: 'ank skills takes 0 argument(s), the pointer gives 1'.
