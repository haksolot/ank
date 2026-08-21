---
id: LOG-f0d60a0be355
type: log
title: Done, and the shape of it is smaller than expected because write_back already took text. The named
created: 2026-08-21T22:52:22Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/edit.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-353036d7972f
seq: 0
schema: 4
version: 1
---

 path parses nothing new: it clones the entity, sets the named fields, renders canonical text, and hands that to the same write_back the editor path uses. So the two meet check_id, check_frozen, changed_fields and the store's compare-and-swap at one place, which is what makes the pairing test possible rather than aspirational.

Three flags and no fourth: --title, --body, --constraint. --criteria, --scope, --blocked-by and --reference are deliberately absent because amend owns them, and a second way to write one field is the defect this corpus keeps removing rather than adding.

The pairing test taught me something I would have asserted wrongly. The two paths do not produce byte-identical stderr, and it is right that they do not: the editor path appends where it kept the text that was typed, because a refusal after twenty minutes of typing must not discard it, and a flag value is still in the caller's shell. So the test compares the refusal as a prefix and then asserts the remainder is exactly that note. A blunt assert_eq would have been either a false failure or, once weakened, an assertion that checked nothing.

One deviation from my own criterion, stated rather than hidden. It says the surface goldens gain the flags and nothing else. They also gain a reworded summary, three notes, and a second refusal. Checked verb by verb: exactly one verb declaration moved, edit, and CONTRACT_VERSION did not. The additions are not drift, they are the declaration becoming true: TASK-84cf7 exists because a verb silent on what it refuses is a defect, so declaring the new refusal is required and not optional, and a summary still reading "opens an entity in EDITOR" for a verb that usually does not would be a help text the tool knows to be false. Reverting them to satisfy the letter would have shipped a lie.

SPEC-f353359663d5 supersedes the CLI surface with three edits: the one-line summary, the commands block, and one paragraph under the verb. The existing sentences are untouched word for word.
