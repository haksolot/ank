---
id: TASK-5325eab5fce2
type: task
slug: the-readme-stops-offering-ank-as-a-first-class-r
title: The README stops offering .ank/ as a first-class read
created: 2026-08-01T02:28:25Z
status: done
scope:
  - README.md
blocked_by: []
done_criteria: |
  The README no longer presents reading .ank/ directly as a first-class use, and no longer says cat is an agent's show. It names ank show, ank find and ank context as the way in, and keeps the distinction it already draws correctly: what the CLI adds over the files is not display but the claim refs, the verifiers and the freeze. The verb count in the layout section matches the surface as ADR-3859eb46bdc3 leaves it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/c6b9838736e1@30d6996
    tree: scope/a7a9d0ce42ad
    criteria: 4cc884d81e10
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@30d6996
    tree: scope/a7a9d0ce42ad
    criteria: 4cc884d81e10
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/01a34aa2be7f@30d6996
    tree: scope/a7a9d0ce42ad
    criteria: 4cc884d81e10
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 4
---

ADR-01b6dd05f0db closes .ank/ to direct reads and writes by an agent. The README still says the opposite, in a paragraph that reads as an invitation:

    Reading .ank/ directly is still a first-class use -- the format is the
    specification, and cat is an agent's show.

Same sentence as the one SKILL.md carried, in the file a newcomer reads first. TASK-3109a736c255 removed it from SKILL.md and TASK-9e9f815e7ef6 removed it from the specification; the README was in neither scope, and widening one to reach it is the move the working loop forbids.

The paragraph is not simply wrong and should not simply be deleted. Its second half draws the right line -- direct access is not a way around claim and done, which write git refs and run verifiers -- and that half survives ADR-01b6dd05f0db intact. What changes is the first half: the reason to prefer the CLI is no longer only that it enforces the transitions, it is that ank show, ank find and ank context know what the files do not, and the reading side is now closed to an agent too.

Worth checking the layout section in the same pass: it calls the binary twelve verbs, which the succession did not change -- show moved between audiences without altering the total -- so the number is right and this is a confirmation rather than an edit.

## Log
- 2026-08-01T02:42:43Z seanl@sean-laptop — I was wrong in this task's own body. It predicted the layout line reading twelve verbs was still correct -- show moved between audiences without altering the total -- and said the pass would be a confirmation rather than an edit. It is thirteen: attest landed on the human surface with TASK-1f4f7b57039b, after that line was written, and the count had been stale since before the succession touched anything. Counted through ank help --json rather than by reading the table, which is what should have been done when the task was filed.

Two more stale counts in the status section, which the task body had not spotted at all: seven of the agent surface listing context claim log done new find release, and five of the human one listing show among them. Now eight with show, and the human five with attest in show's place -- attest had never been listed there either.

The paragraph kept the shape the body argued for. Its second half survives ADR-01b6dd05f0db intact and is the sharper reason: claim and done write git refs and run verifiers, so editing by hand skips both. That was already the rule for writing, and reading joined it only once there was nothing left the CLI could not serve -- which is the honest chronology and reads better than asserting the rule flat.
- 2026-08-01T02:43:17Z seanl@sean-laptop — done, proof test:local/c6b9838736e1@30d6996 test:local/e3b0c44298fc@30d6996 test:local/01a34aa2be7f@30d6996
