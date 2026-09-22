# Signing keys

A ratification is a commit, and a signature is what makes it answerable to
somebody. Signing is a regime the corpus is in, not a precondition of ratifying
(ADR-964be4d940b2): `ank accept` produces a signed ratification commit where the
repository is configured to sign, and an unsigned one where it is not, and it
never refuses for want of a key. What changes is what `ank check` can say about
the result. The rules it applies, and the four outcomes it keeps apart, are
section 8 of the specification, in [Proof, anchoring and
authority](specification.md); this page is how to set a repository up so they
apply.

## Until a key is declared

A corpus that declares no key says so, once, as a signal on `allowed_signers`:
`no ratification key declared: permissions are advisory, not enforced (§8)`.
That is one of the lines [the quickstart](quickstart.md) ends with. It is honest rather
than wrong: every ratification is still anchored by its hash, and nothing
pretends a signature was checked. Declaring a key is what turns it into a check.

## Sign your commits

The set-up is git's. With an SSH key, which git has signed with since 2.34:

    git config gpg.format ssh
    git config user.signingkey ~/.ssh/id_ed25519.pub

With OpenPGP, `gpg.format` stays at its default and `user.signingkey` names the
key id. `accept` signs whenever the repository can: when `user.signingkey` is
set, or `commit.gpgsign` is `true`. It does not wait for `commit.gpgsign`,
because a maintainer who signs selectively still means a ratification to be
signed, so whether your ordinary commits are signed is yours to decide; `check`
only ever judges the ratification commits.

## Declare the key in .ank/allowed_signers

`.ank/allowed_signers` lists the keys allowed to ratify, one per line, in git's
allowed-signers layout: a principal, then the key type, then the key.

    marie@example.com  ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI...
    sean@example.com   gpg 0123456789ABCDEF0123456789ABCDEF01234567

The key type names the format. An `ssh-*` entry is the public key itself; a
`gpg` entry is the full fingerprint or the long key id. Lines starting with `#`
are comments, and this repository's own file uses them to record why each key
was added.

The file is versioned like any other, so adding a key is a diff in review, and
it lands on `main` by pull request before the key it declares can ratify
anything there.

**Who reads the file depends on the format.** `ank check` reads it itself and
judges whichever key signed against it, for both formats. git's own `git
verify-commit` reads it only under `gpg.format = ssh` with
`gpg.ssh.allowedSignersFile` pointed at it; under OpenPGP it resolves the
signature through the keyring and never opens this file. So the file is
load-bearing for ank either way, and for git only if you point git at it.

## What check says afterwards

Once a key is declared, `check` judges every ratification commit against the
file and keeps four outcomes apart, which section 8 of the specification lists
with the reason for each. The one to expect on a machine that is not yours is
the signal and not the fault: a CI runner without your public key is a correct
repository on an incomplete machine, and `check` says it could not verify rather
than either failing or passing it.

A ratification merged by squash or rebase loses what this page set up; the merge
method that keeps it is [Ratifying and archiving](ratifying.md#ratifying-a-decision).
