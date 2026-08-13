---
id: ADR-af533e7a3e03
type: adr
slug: a-write-whose-only-product-is-a-ref-fails-when-t
title: A write whose only product is a ref fails when the ref does not reach the remote
created: 2026-08-13T16:20:32Z
author: claude-code/2.1.229
status: accepted
scope:
  - crates/ank-cli/src/claim.rs
  - .github/workflows/ci.yml
  - docs/ank-spec-v1.1.md
constraint: |
  A verb whose whole product is a ref exits non-zero when the push of that ref is refused. A verb that also wrote to disk degrades, warns and exits zero. Which of the two a verb is, its help says; a caller never has to infer it from what the verb happens to touch.
ratified: 2840cf51dcd9
schema: 3
version: 2
---

Section 2 says degrade rather than fail, and that principle is right for a claim:
the claim holds in this clone, the work goes on, and the risk of a concurrent
claim is displayed rather than hidden. `attest --detached` inherited the same
treatment and it is the wrong treatment, because the two are not the same object.

Measured on a scratch corpus against a real `file://` remote:

    remote reachable    "pushed":true    --            exit 0
    remote unreachable  "pushed":false   warning: ...  exit 0

**A proof ref exists to be readable by somebody else.** That is the whole of what
`--detached` is for (ADR-493471d64ba0): a pipeline anchors a run without
producing a commit. When the push fails, the ref is readable by nobody, the
attestation did not happen, and the verb reports success. There is no degraded
mode here to fall back to -- unlike a claim, nothing was written to disk that
still means something.

**The proof this is not theoretical is in the tree.** `.github/workflows/ci.yml`
carries a comment that reads, in full: *read the flag, not the exit code. A proof
is a ref, so it is worth something only once it reaches the remote -- and when
the push is refused, `attest` still exits 0 and warns on standard error. Trusting
the code would go green having lost the attestation, which is the exact silence
this job was written to remove.* The repository that ships the verb already works
around the verb, in a comment, in its own pipeline. That is the strongest
argument available that the default contract is wrong: the first integration
written against it had to be written against the flag instead.

The information is in `--json` and an integration written carefully reads it. The
default is what an integration written in a hurry uses, and an automated caller
reads the exit code -- that is what an exit code is.

**Stated per verb, never inferred.** The line between the two kinds is not
"writes a ref" versus "writes a file": `claim` writes a ref too and belongs on
the degrade side, because the claim it recorded still governs this clone. The
distinguishing question is whether anything survives the failed push that is
worth having. A caller cannot derive that from the outside, so each verb's help
says which it is, and the rule stops being something to reason about.
