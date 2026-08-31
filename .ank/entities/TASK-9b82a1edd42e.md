---
id: TASK-9b82a1edd42e
type: task
slug: the-relicensing-notice-names-0-2-0-where-0-3-0-w
title: The relicensing notice names 0.2.0 where 0.3.0 was the last GPL release
created: 2026-08-31T07:51:45Z
author: claude-code/opus-5+drift2
status: done
scope:
  - NOTICE
  - npm/ank/README.md
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  Every tracked statement outside .ank/ of when the GPL-to-Apache change took effect names 0.3.0 as the last release distributed under GPL-3.0-only and 0.4.0 as the first under Apache-2.0; NOTICE and npm/ank/README.md agree on that boundary; and a test in crates/ank-cli/tests/skill.rs fails when a tracked file states a different one.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/ca53baefe30c@bbdf494
    tree: scope/54b2d675a14e
    criteria: bac966a2fa4a
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@bbdf494
    tree: scope/54b2d675a14e
    criteria: bac966a2fa4a
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Measured on 2026-08-31 against the tags themselves, not against the prose.

`git show v0.3.0:crates/ank-cli/Cargo.toml` declares `license = "GPL-3.0-only"`,
and `git show v0.3.0:LICENSE` is the GNU General Public License version 3. So do
`npm/ank/package.json`, the root `package.json` and `.claude-plugin/plugin.json`
at that tag. v0.3.0 is a published release: `gh release list` dates it
2026-08-17. v0.4.0 (2026-08-18) is the first tag whose LICENSE is the Apache
License and whose manifests declare Apache-2.0 -- with the single exception
`crates/ank-cli/tests/skill.rs:918` already records, `.claude-plugin/plugin.json`,
which went on declaring GPL-3.0-only at v0.4.0 and for thirteen days after.

So the last GPL-3.0-only release is 0.3.0 and the first Apache-2.0 release is
0.4.0. Two tracked documents say otherwise:

  NOTICE:18                "distributed under GPL-3.0-only up to and including 0.2.0"
  npm/ank/README.md:33     "Ank was GPL-3.0-only until 0.3.0"

Both exclude 0.3.0 from the GPL era. ADR-534c7a3e6cf8 (successor of
ADR-9f03438f5422, which said the same) requires that the relicensing be
prospective and say so -- "a release already made under GPL-3.0 stays available
under GPL-3.0 to whoever received it" -- and that the documentation state the
licence "in one place, with no second answer left anywhere else to contradict
it". Somebody who took 0.3.0 is told by NOTICE, the licence notice at the root
of the repository, that their release was not one of the GPL ones. It was.

Nothing catches it today. `declared_licences()` in crates/ank-cli/tests/skill.rs
walks every manifest for its `license` field, which is exactly the drift the
list missed before; the prose boundary is a different claim and no test reads it.

The two documents also disagree with each other on where the line falls, which
is the second answer that ADR forbids on its own terms.
