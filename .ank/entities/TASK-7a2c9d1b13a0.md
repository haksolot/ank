---
id: TASK-7a2c9d1b13a0
type: task
slug: the-stale-binary-hint-names-an-install-that-rein
title: The stale-binary hint names an install that reinstalls the same build
created: 2026-08-21T23:59:29Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  The next step a corpus-ahead-of-binary warning names is one that resolves the state it describes, and where no released version carries the schema found, it does not send the reader to reinstall the version they already have. The message distinguishes the two cases it can be in and says which: a release exists that reads the corpus, and the install command is the answer; or none does, and the answer is to build from the tree or wait for a release, said in those words. A test drives the binary on a corpus one schema ahead and asserts the sentence against both cases rather than against whichever holds today. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

Found in use, not by reading. `ank show` on this corpus printed:

    warning: corpus at schema 4, this binary reads 3: 8 entities left out of every listing
      -> the binary is older than the corpus: ank --version names the build,
         npm install -g @haksolot/ank replaces it

The install named would have fetched `0.5.0`, which is the build that had just
refused. Schema 4 landed on the default branch after the 0.5.0 tag, so no
published version reads this corpus at all, and the two binaries even report the
same version string: `0.5.0 (7bc34f8)` installed against `0.5.0 (52c59da)` built
from the tree, distinguishable only by the commit in parentheses.

**§4 asks a message to name the command to run next**, and this one names a
command that returns the caller exactly where they were. That is worse than
naming none: a reader who follows it concludes the tool is broken rather than
that their copy is old, because the remedy visibly did nothing.

**The half that works should be kept.** Naming the build with the commit is what
let the two copies be told apart at all, and the count of entities left out of
the listing is what makes the warning actionable rather than vague. What is
wrong is one clause.

**The condition is knowable at the point of printing**, which is what makes this
fixable rather than a caveat: the binary knows the schema it reads and the
version it is, and a released version that reads schema N either exists or does
not. What it must not do is guess, or print advice that is true only when
somebody remembered to ship.

Worth separating from this: whether a release carrying schema 4 should ship soon
is a decision for the human who tags them, and every installed copy is blind to
this corpus until one does.
