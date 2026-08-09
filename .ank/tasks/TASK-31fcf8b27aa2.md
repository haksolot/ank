---
id: TASK-31fcf8b27aa2
type: task
slug: the-guide-still-says-init-does-not-write-the-git
title: The guide still says init does not write the gitignore rule, and it does
created: 2026-08-09T15:53:53Z
author: claude-code@ank
status: open
scope:
  - docs/getting-started.md
blocked_by: []
done_criteria: |
  docs/getting-started.md no longer tells the reader to run 'echo .ank/index.db >> .gitignore', and no longer says init does not write the rule. The count in 'Two edits to make now' matches what follows it. What the page says about init writing .gitignore agrees with the init output it shows earlier on the same page. No other file changes, and cargo test stays green.
criteria_by: creator
schema: 2
version: 1
---

Found while executing TASK-7af1cd875d9a and left out of it, because that task's criterion is about config.yml and this is a different staleness in the same file.

The page shows 'wrote .gitignore' in the ank init output, and then thirty lines later says init 'does not yet write the rule for you' and asks the reader to append it by hand. Both cannot be true. Measured on a fresh repository with the binary at d0ccc24: init writes a .gitignore containing '.ank/index.db', which is also what section 9 of the specification says it does.

So the second of the page's 'Two edits to make now' is not an edit anyone has to make, and the count is wrong with it. A reader who follows the instruction gets a duplicate line rather than an error, which is why this has survived: it costs nothing visible and quietly teaches that the tool does less than it does.
