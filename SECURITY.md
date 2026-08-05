# Security policy

## 1. What ank protects against

Ank protects against **drift, not against an adversary**. The specification
says so in section 1 and the README repeats it: ank is **not a security
boundary**. Its guardrails exist so that an agent going off the rails leaves a
trace a reader can find, not so that a malicious actor is stopped.

This is a position, not an omission, and the rest of this file follows from it.
Three consequences are worth naming out loud, because each one looks like a
hole to anyone who assumed otherwise.

**The roles in `.ank/config.yml` are advisory.** They declare intent where a
reader can see it, and an unknown identity — including `$ANK_AGENT` being
absent — falls back to the least-privileged role. What they never do is make
the CLI refuse. A refusal derived from `$ANK_AGENT` would be a refusal the
caller lifts by exporting a different string, and shipping that as a guarantee
is worse than shipping no guarantee at all. Declaring yourself `human` in the
config confers no authority; the signature is what carries it.

**The CLI is a reference implementation, not a gatekeeper.** The format is the
specification, so any tool can read and write `.ank/`. Immutability is
therefore *verifiable, not defended*: every freeze is anchored by a hash in an
artifact the file's editor does not control — the claim record, the
ratification commit, the proof entry — and `ank check` is what compares. A
direct edit is not prevented. It is noticed.

**Enforcement, where it is real, lives outside ank.** A harness `PreToolUse`
hook that refuses a tool call cannot be talked out of it by an environment
variable, because the process it guards does not get to set one. That is the
layer to reach for when you need a refusal that holds; this repository's own
hook, refusing direct reads of `.ank/`, is the working example.

## 2. The one hard line, and what it does not prove

The single hard line of authority is the **signed ratification commit**
produced by `ank accept`. It records the SHA and the hash of the accepted ADR's
`constraint` and `scope`, `ank check` verifies that signature against the keys
declared in `.ank/allowed_signers`, and `accept` refuses to run anywhere but
the default branch. The key file is versioned, so adding a key is a diff in
review.

**What the signature proves is access to an authorised key, not human intent.**
An agent running on a developer machine whose git signing is configured and
unlocked can produce a valid signed commit. The defence against that case is
operational rather than cryptographic: keep the ratification key behind a
passphrase or hardware touch-to-sign, distinct from the everyday commit key if
needed. That limitation is consistent with section 1 — drift, not an adversary.

**With no signing configured at all, the hard line is gone and nothing replaces
it.** Roles were already advisory, and the ratification anchor becomes a hash
in a commit message anyone can write. `ank check` displays that rather than
hiding it, and it distinguishes a fault from a signal: a signature present with
no local public key is reported as *not verified, and not refused*, because a
clone without the key is a correct repository on an incomplete machine. The
rule underneath is the one to remember when reading any ank output — **a
verification that degrades to success is not a verification**.

## 3. Verifiers run only what the repository accepted

A task's `verify` field references verifiers declared in `config.yml`, **never
an inline shell command**. This is the property that matters most here, and it
is deliberate: a task may arrive through a pull request from a fork, and an
inline command would be arbitrary code execution triggered by `ank done`. Git
had exactly this problem with hooks and solved it the same way. Because
verifiers live in a file the repository controls, changing one goes through
code review like any other change.

Editing `config.yml` to replace a verifier with `true` remains possible, and
two mechanisms make it visible rather than impossible. The proof records a hash
of the definition that actually ran, so a verifier weakened before or after the
`done` — in the same commit or another — is detectable. And `check` reports the
pattern directly: a verifier modified inside the task's activity window, or a
proof hash diverging from the definition in force at the `done` commit.

Verifiers execute through `sh -c` on all three supported platforms. On Windows,
`sh` is resolved from Git for Windows; a missing `sh` is an explicit error, never
a silent fallback to `cmd`.

## 4. Supported versions

Ank is pre-1.0. Only the **latest published release** is supported, and fixes
ship in the next release rather than in a backport.

| Version | Supported |
|---|---|
| 0.1.2 (latest) | yes |
| earlier 0.1.x | no — upgrade |

Ank requires **git 2.34 or newer** and checks at startup.

## 5. Reporting a vulnerability

Report privately through **GitHub private vulnerability reporting**, which is
enabled on this repository:

<https://github.com/haksolot/ank/security/advisories/new>

Do not open a public issue for a suspected vulnerability. Include the exact
command, its exit code, the output of `ank --version` and `git --version`, and
the operating system. If a report needs a corpus to reproduce, prefer a minimal
throwaway repository over anything real.

Expect an acknowledgement within seven days. This is a small project with no
on-call rotation, so that is a commitment to answer, not to a fix window.

**In scope.** Anything that makes ank do what the repository did not accept, or
makes `check` report success where the record is not intact. For example:
execution of a command that is not a declared verifier; a path where `ank`
writes outside `.ank/` and the refs under `refs/ank/`, or commits when only
`accept` may; a corpus alteration that leaves `check` green; a hash anchor that
can be forged rather than detected; secrets leaking into output or into a proof.

**Out of scope**, because section 1 already answers it: an agent editing
`.ank/` directly instead of going through the CLI; a falsified `$ANK_AGENT`; a
role in `config.yml` not being enforced; a ratification signature produced on a
machine whose signing key is unlocked. These are documented properties. If you
think one of them should change, that is an ADR, not an advisory.
