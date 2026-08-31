---
id: TASK-c5c93cc8e5f8
type: task
slug: release-and-close-warn-when-the-claim-deletion-d
title: release and close warn when the claim deletion does not reach the remote
created: 2026-08-31T07:29:51Z
author: claude-code/opus-5+degrade
status: done
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Against a corpus whose origin is configured and unreachable, "ank release --reason <r>" and "ank close <id> --reason <r>" each exit 0 and emit a warning naming the claim the remote still holds; a test in crates/ank-cli/tests/cli.rs drives the binary for both verbs, asserts exit 0 and the warning text on the combined output, and asserts that a repository with no remote at all emits no such warning.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/2c9efa235677@c48e002
    tree: scope/8459ebd2b571
    criteria: a8ce54e5859e
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@c48e002
    tree: scope/8459ebd2b571
    criteria: a8ce54e5859e
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

ADR-af533e7a3e03 says a verb whose write survives a refused push "degrades, warns and exits zero", and verbs.rs declares PUSH_DEGRADES for both release and close, so their help already says which class they are in.

Measured on ank built from c48e002, against a scratch corpus whose origin is a bare repository moved out of the way between the claim and the release:

    claim   (origin reachable)    ref reaches origin              exit 0
    release (origin gone)         stdout "released ... -> open"   exit 0
                                  stderr 0 bytes, --json warnings []
    close   (origin gone)         stdout "closed ... -> closed"   exit 0
                                  stderr 0 bytes
    ls-remote origin refs/ank/*   after the release: the claim is still there

So the local ref is deleted, the remote keeps a claim nobody holds until its
lease runs out, and neither verb says a word. claim, on the same corpus, prints
"warning: claim not pushed: ...". The divergence is the "warns" clause and
nothing else: the exit code is right and the help is honest.

The repair is the warning, not an edit to the ADR. The risk the ADR asks to be
displayed is real and measured above -- another clone reads the task as held --
and a ratified constraint is superseded by a proposal a human signs, never
edited in place.

claim::delete_at discards the push result outright (let _ = push(...)), so the
verbs cannot see it. It has to carry the Sync out, and the sentence is the
mirror of Sync::warning rather than that sentence reused: an acquisition that
did not travel can be taken twice, a revocation that did not travel is read as
held. done is the precedent for where it goes -- standard error, both modes,
because stdout under --json is a parser input (ADR-6fd69efb629c).
