---
id: LOG-d46766ade1f7
type: log
title: "The gate moved rather than being rewritten: warn_orphaned_citations' walk is now citations_of, one"
created: 2026-08-26T00:30:41Z
author: claude-code/opus-5+gate
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-c90651901f22
seq: 1
schema: 4
version: 1
---

 function over one file list, and both callers ask it. accept calls refuse_stale_citations between succession() and promote(), inside the 'everything that can refuse, refuses before anything is written' block, and the post-commit warning is gone -- so --quiet cannot touch it (a refusal travels the error path) and there is no second pass to repair. check calls check_stale_citations from inspect, over the file list already walked for the scope verdicts, with the needles being exactly the entities reading superseded: a proposal is never a needle, which is how 'reported by neither' falls out of the shape rather than out of a special case. The note names chain_head, not the first hop, because the repair is to write what binds today. Declared in verbs.rs as a second Prerequisite refusal beside the branch one; every_declared_refusal_is_printed_on_the_page_that_declares_it carries it onto ank help accept with no rendering change.
