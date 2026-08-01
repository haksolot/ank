---
id: TASK-5325eab5fce2
type: task
slug: the-readme-stops-offering-ank-as-a-first-class-r
title: The README stops offering .ank/ as a first-class read
created: 2026-08-01T02:28:25Z
status: open
scope:
  - README.md
blocked_by: []
done_criteria: |
  The README no longer presents reading .ank/ directly as a first-class use, and no longer says cat is an agent's show. It names ank show, ank find and ank context as the way in, and keeps the distinction it already draws correctly: what the CLI adds over the files is not display but the claim refs, the verifiers and the freeze. The verb count in the layout section matches the surface as ADR-3859eb46bdc3 leaves it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 1
version: 1
---

ADR-01b6dd05f0db closes .ank/ to direct reads and writes by an agent. The README still says the opposite, in a paragraph that reads as an invitation:

    Reading .ank/ directly is still a first-class use -- the format is the
    specification, and cat is an agent's show.

Same sentence as the one SKILL.md carried, in the file a newcomer reads first. TASK-3109a736c255 removed it from SKILL.md and TASK-9e9f815e7ef6 removed it from the specification; the README was in neither scope, and widening one to reach it is the move the working loop forbids.

The paragraph is not simply wrong and should not simply be deleted. Its second half draws the right line -- direct access is not a way around claim and done, which write git refs and run verifiers -- and that half survives ADR-01b6dd05f0db intact. What changes is the first half: the reason to prefer the CLI is no longer only that it enforces the transitions, it is that ank show, ank find and ank context know what the files do not, and the reading side is now closed to an agent too.

Worth checking the layout section in the same pass: it calls the binary twelve verbs, which the succession did not change -- show moved between audiences without altering the total -- so the number is right and this is a confirmation rather than an edit.
