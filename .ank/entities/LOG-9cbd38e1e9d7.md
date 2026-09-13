---
id: LOG-9cbd38e1e9d7
type: log
title: "Review before done. Axis 1, criterion clause by clause: log --method tdd under a claim writes"
created: 2026-09-13T15:13:05Z
author: claude-code/a6c9
scope:
  - crates/ank-cli/src/**
  - crates/ank-core/src/model.rs
  - crates/ank-contract/src/verbs.rs
  - docs/getting-started.md
about: TASK-a6c9d98a38ac
seq: 3
schema: 4
version: 1
---

 records method titled tdd and renews (commands.rs log_method/log_held, entries.rs record_method; cli.rs test checks the entry file and expiry_span 7000-7300); takes no message (positional refused at 1); no claim exit 6 and unknown name exit 7 (skills::method before acting_on); show presents it under EDITS with LOG (1 of 1) counting the trace alone (test); check accepts method (RECORDS_KINDS, test greps for no records finding); skills in a corpus prints METHODS with one line per sibling and three counts and --json carries integers (skills.rs rates/report/document, contract SKILLS_OUT, golden skills.json, scratch corpus of three tasks in test and by hand); getting-started shows the report with the real output measured earlier. Found and removed one hunk no clause asked for: a --quiet branch silencing the skills listing. Axis 2, ank scope over the touched paths returns the context set plus ADR-93d8, which binds pseudo-terminal tests this diff does not add. Found one defect of my own: SKILLS_OUT was inserted between TUI_OUT and its doc comment, moved. Test-side edits outside the task scope: tests/cli.rs, tests/skill.rs (skills_run now runs from its temp dir, since cargo's cwd sits inside this repository's corpus and the listing would carry counts), golden-json skills.json and help.json, the golden count 28 to 29, and ('log','--method') added to SELECTS_AN_ACT in every_flag_the_help_offers_can_be_given_to_the_verb, which failed in the first full run because --method refuses the positional the walk hands log. cfg(unix): the pseudo-terminal suites drive log with a message and never --method, and ank-tui opens no form for log (NEEDS holds close and attest only), so none reaches the changed path beyond the refactor of log_write into log_held, which the Windows suite covers.
