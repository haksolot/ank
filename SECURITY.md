# Security policy

## 1. What ank protects against

Ank protects against **drift, not against an adversary**. The specification
says so in section 1 and the README repeats it: ank is **not a security
boundary**. Its guardrails exist so that an agent going off the rails leaves a
trace a reader can find, not so that a malicious actor is stopped.

This is a position, not an omission, and the rest of this file follows from it.
Three consequences are worth naming out loud, because each one looks like a
hole to anyone who assumed otherwise.

**The roles in `.ank/config.yml` are read, and the CLI never refuses on
one.** `permissions.enforce` sets how they apply, and an identity absent from
`identities:`, including `$ANK_AGENT` being absent, gets the role `agent`:

- **`off`**, or the key absent, applies roles to nothing: no verb warns and
  `check` reports no act against them.
- **`warn`**, which `ank init` writes, lets a verb performing an act the
  identity's role forbids perform it, name on stderr the role and the entry
  that forbid it, and exit as it would have. `check` reports each recorded act
  of that kind (an accept, a close, a constraint edit, a new entity, a write to
  the policy keys of `config.yml`) as a signal of kind `role`.
- **`deny`** does all of that, and the harness hook refuses the call: `ank
  hooks run <agent> pre-tool-use`, matched to the shell tool, refuses a shell
  command that runs an ank verb the session's role forbids, naming the entry
  and telling the agent the command is a human's to run. The refusal is the
  hook's, outside the process it guards; the verb itself, run anywhere else,
  still only warns.

`ANK_PERMISSIONS_ENFORCE`, set by whoever launches the agent, may tighten the
declared level and never loosen it. A refusal the CLI derived from `$ANK_AGENT`
would be one the caller lifts by exporting a different string, which is why
the only refusal sits in a hook the agent did not start. Declaring yourself
`human` in the config confers no authority; the signature is what carries it.

**`check.severity` can only raise.** `check.severity.<kind>: fault` makes
`check` report every finding of a kind it reports as a signal by default as a
fault, naming the repository as what raised it. No value lowers a kind: `ank
config` refuses, and `check` faults on `config.yml`, a kind that is unknown,
already a fault, or given any value but `fault`. A repository can make `check`
stricter about itself and never quieter. `docs/check.md` lists the kinds that
may be raised.

**The CLI is a reference implementation, not a gatekeeper.** The format is the
specification, so any tool can read and write `.ank/`. Immutability is
therefore *verifiable, not defended*: every freeze is anchored by a hash in an
artifact the file's editor does not control (the claim record, the
ratification tag, the proof entry) and `ank check` is what compares. A
direct edit is not prevented. It is noticed.

**Enforcement, where it is real, lives outside ank.** A harness `PreToolUse`
hook that refuses a tool call cannot be talked out of it by the agent it
guards, because that agent does not start the harness and does not get to set
its environment. That is the layer to reach for when you need a refusal that
holds, and ank ships one for Claude Code.

`ank hooks install claude` adds two entries to the user-level `settings.json`
of the harness, both running the binary that installed them: one at session
start, which serves `ank status`, and one before a file or shell tool, which
applies the permissions guard above to a shell call under `deny` and two file
guards declared in `.ank/config.yml` and set with `ank config`:

- **`guard.corpus`**, `deny` by default, refuses a `Read`, `Write`, `Edit`,
  `MultiEdit`, `NotebookEdit`, `Glob` or `Grep` aimed under the corpus
  directory, naming the verbs that read it instead. `off` lets it through.
- **`guard.edit`**, `warn` by default, looks at an `Edit`, `MultiEdit`, `Write`
  or `NotebookEdit` that no live claim of the identity covers: `claim` refuses
  it, `warn` lets it through with a message naming the path, `off` says
  nothing. A tool that only reads is never an edit.

`ANK_GUARD_CORPUS` and `ANK_GUARD_EDIT`, set by whoever launches the agent, may
tighten either level and never loosen it; a looser value is ignored and the
hook says so.

What the guards do not refuse is as much a part of the model as what they do:

- **A shell command, except an ank verb under `deny`.** The file guards never
  read a `Bash` call, so `cat .ank/config.yml` or a redirect into a file no
  claim covers goes through at every level. The permissions guard reads only
  the ank invocations a command line names directly: ank run through `sh -c`,
  `xargs`, `$(...)`, an alias or a script is let through, and the verb still
  warns when it runs.
- **An identity.** "The identity" is `$ANK_AGENT`, which the launcher sets, so
  `guard.edit` measures an edit against the claims of whatever identity the
  session was started with, and the permissions guard judges a shell call as
  that identity, never as an `ANK_AGENT=...` prefix the command itself sets.
- **Its own failure.** The hook fails open: outside a corpus it is silent, and a
  payload it cannot read or an error of its own lets the tool call through
  rather than blocking the harness.
- **A machine where nobody installed it.** The entries live in the user's own
  settings, not in the repository, and only for Claude Code. Nothing in this
  repository installs them or depends on them being installed, and their
  absence is not a vulnerability: the corpus rules they guard stay readable
  through `ank context`, and `ank check` still reports a file written behind
  the CLI's back.

## 2. The one hard line, and what it does not prove

The single hard line of authority is the **signed ratification tag**
written by `ank accept`: an annotated tag at `refs/ank/ratify/<id>`, pointing at
the blob of the entity file as `accept` wrote it, whose message carries the
anchor (the hash of an ADR's `constraint` and `scope`, or of a spec's body and
`scope`), the actor and any supersession. It is signed where the repository
signs. `accept` runs on any branch, commits nothing, and pushes the ref with
compare-and-swap; when the ref does not reach the remote it exits 9 and keeps
nothing.

`ank check` verifies that signature against the keys declared under `signers:`
in `.ank/config.yml`, and reads no other list. The block is written by `ank
signers add`, `retire` and `remove`, which write files and commit nothing, so
declaring a key is a diff landed by pull request, and a ratification is judged
against the `signers:` the default branch holds. A ratification made before
tags existed is a signed commit with the subject `ratify <id>`; `check` still
finds it by walking history, and `ank accept --reanchor` moves its anchor into
a tag.

**A repository may require the signature.** `ratify.require_signature: true` in
`.ank/config.yml` makes `ank accept` refuse at 7, before it writes a ref or a
file, when git names no signing key, when `signers:` does not declare the key
git would sign with, or when that key's `valid_before` has passed; with a
declared key it signs whatever `commit.gpgsign` says. `check` then faults an
unsigned ratification, or one signed by a key `signers:` does not declare. `ank
config` refuses to set it `true` while `signers:` declares no key. Absent or
`false`, `accept` signs where the repository signs and writes an unsigned tag
otherwise, the regime the rest of this section describes.

**What the signature proves is access to an authorised key, not human intent.**
An agent running on a developer machine whose git signing is configured and
unlocked can produce a valid signed tag. The defence against that case is
operational rather than cryptographic: keep the ratification key behind a
passphrase or hardware touch-to-sign, distinct from the everyday commit key if
needed. That limitation is consistent with section 1: drift, not an adversary.

**With no signing configured at all, the hard line is gone and nothing replaces
it.** Roles never made the CLI refuse, and the ratification anchor becomes a
hash in an unsigned tag message anyone can write. `ank check` displays that
rather than hiding it, and it distinguishes a fault from a signal: a signature
present with no local public key is reported as *not verified, and not
refused*, because a clone without the key is a correct repository on an
incomplete machine. The rule underneath is the one to remember when reading any ank output: **a
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
`done`, in the same commit or another, is detectable. And `check` reports the
pattern directly: a verifier modified inside the task's activity window, or a
proof hash diverging from the definition in force at the `done` commit.

Verifiers execute through `sh -c` on all three supported platforms. On Windows,
`sh` is resolved from Git for Windows; a missing `sh` is an explicit error, never
a silent fallback to `cmd`.

## 4. Supported versions

Ank is pre-1.0. Only the **latest published release** is supported, and fixes
ship in the next release rather than in a backport. Every earlier release is
unsupported: upgrade.

The latest release is the one this page links rather than names, because a
number written here would go stale at the next one:
<https://github.com/haksolot/ank/releases/latest>. `ank update --check` reports
the running version against the latest published one and installs nothing, and
`ank --version` prints the running build.

Ank requires **git 2.34 or newer**, checked by each verb that needs git and
never at startup.

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
makes `check` report success where the record is not intact.

In the corpus:

- execution of a command that is not a declared verifier;
- a corpus alteration that leaves `check` green;
- a hash anchor that can be forged rather than detected;
- a write outside what the verb run documents as its own: a path where `ank`
  touches the working tree, a branch, a tag or a ref that its `ank help <verb>`
  entry does not name, or a commit made by any verb, since none of them commits;
- secrets leaking into output, into a proof or into an entity.

Each verb's documented perimeter is what that fourth item measures against, so
read `ank help <verb>` before reporting one. `ank init` writing `.gitattributes`,
`.gitignore` and a pointer line in `AGENTS.md` at the repository root is the
obvious case: those writes are what the verb is for, they are printed as they
happen, and they are not a vulnerability. A verb that wrote them without saying
so would be.

In the distribution, which is the larger surface and the one an attacker would
actually reach for:

- `install.sh` and `install.ps1`, the scripts the documentation tells people to
  pipe into a shell. Both download a release archive and verify it against the
  `.sha256` published beside it, so what is in scope is a path around that: an
  archive accepted when the hash did not match or could not be computed, a
  redirect followed somewhere the script did not intend, an archive that unpacks
  outside the directory asked for, or an argument that reaches a command
  unquoted;
- the npm packages: `@haksolot/ank`, whose `bin/wrapper.js` resolves and spawns
  a binary out of one of the four platform packages
  (`@haksolot/ank-linux-x64-musl`, `@haksolot/ank-linux-arm64-musl`,
  `@haksolot/ank-darwin-arm64`, `@haksolot/ank-win32-x64`) selected through
  `optionalDependencies`. The
  wrapper downloads nothing and runs no install script, so what is in scope is
  the resolution: a path where it can be made to spawn something other than the
  platform package npm installed, or to pass arguments to it other than the ones
  it was given;
- `ank update`, which runs one of those routes as a child process and is the
  only verb that reaches the network for this question. A path where it installs
  a build the running route did not authenticate, where `$ANK_UPDATE_REPOSITORY`
  redirects a user who set nothing, or where the Windows rename-aside leaves the
  old binary unrecoverable;
- `ank mcp`, which runs `ank <verb> --repo <corpus> --json` as a child for a
  client that has no shell: a tool call whose arguments escape into that command
  line, or a call answering about a corpus other than the one `--repo` addressed
  at startup;
- `ank watch`, which fetches `refs/ank/*` from remotes you declared and mirrors
  claims under `refs/ank/watch/<remote>/claims/`: a fetched ref that can move
  anything else, a declaration file that makes it read or write outside the
  corpora it names, or a crafted ref that reaches `events.jsonl` as something
  other than data.

**Out of scope**, because section 1 already answers it: an agent editing
`.ank/` directly instead of going through the CLI; a falsified `$ANK_AGENT`; a
verb that warns on a role and runs, a role nothing refuses under `off` or
`warn`, or an ank call the `deny` hook does not read; a ratification signature
produced on a machine whose signing key is unlocked; a shell command the
`PreToolUse` hook never sees, or a machine where the hook is not installed.
These are documented properties. If you think one of them should change, that
is an ADR, not an advisory.
