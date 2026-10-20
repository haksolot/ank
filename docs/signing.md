# Signing keys

A ratification is a tag object at `refs/ank/ratify/<id>` (a commit, for one
made before ADR-5e47007a75be), and a signature is what makes it answerable to
somebody. Signing is a regime the corpus is in, not a precondition of ratifying
(ADR-964be4d940b2): `ank accept` writes a signed tag where the repository is
configured to sign, and an unsigned one where it is not, and it never refuses
for want of a key. What changes is what `ank check` can say about
the result. The rules it applies, and the four outcomes it keeps apart, are
section 8 of the specification, in [Proof, anchoring and
authority](specification.md); this page is how to set a repository up so they
apply.

## Until a key is declared

A corpus that declares no key says so, once, as a signal on `signers`:
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
only ever judges the ratifications.

## Declare the key under signers:

The keys allowed to ratify are declared in `.ank/config.yml`, under `signers:`,
keyed by principal (ADR-5c6db7a161b8). Each key carries git's `<keytype> <key>`
form, an optional `note`, and an optional `valid_before` date:

```yaml
signers:
  marie@example.com:
    keys:
      - key: ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI...
        note: laptop
  sean@example.com:
    keys:
      - key: gpg 0123456789ABCDEF0123456789ABCDEF01234567
        note: release key
      - key: ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAJ...
        note: lost on reinstall
        valid_before: 2026-10-11
```

The key type names the format. An `ssh-*` entry is the public key itself, its
trailing comment left out; a `gpg` entry is the full fingerprint or the long key
id. `config.yml` is versioned like any other file, so declaring a key is a diff
in review, and it lands on `main` by pull request before the key it declares can
ratify anything there.

**The block has one writer, and it is not `ank config`.** Declaring who may
ratify is a trust act the skills reserve to a human, as they reserve `accept`:
`ank config signers.*` is refused and names `ank signers`, which writes only the
lines of its change and leaves every comment, order and quoting elsewhere in
`config.yml` as it was.

## A new machine

A key that is not declared ratifies nothing, so a new machine is declared before
it ratifies, and the declaration lands on `main` the way any change does. With
git set up to sign on that machine, as above, from a branch cut from `main`:

    git switch -c signers/laptop main
    ank signers add --note "laptop"

`add` declares the key `user.signingkey` names, under the principal
`user.email` names. `--key` takes a public key file or `'<keytype> <key>'`
instead, `--principal` another principal, and `--note` what a reviewer should
know about the key. A key already declared, under any principal, is refused at
6 and nothing is written. `add` commits nothing: review the diff, then land it
by pull request:

    git diff .ank/config.yml
    git commit -am "Declare the laptop key"
    git push -u origin signers/laptop
    gh pr create --fill --base main --head signers/laptop

Until that pull request is merged, a ratification signed on the new machine is
judged against the `signers:` that `main` holds, and the key is not in it.

## Retiring and removing a key

    ank signers retire 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAJ...'
    ank signers retire 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAJ...' --date 2026-10-11

`retire` sets `valid_before`, today in UTC or the date `--date` names. `remove`
deletes the key outright:

    ank signers remove 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAJ...'

Both take the key as `signers:` declares it, `<keytype> <key>` or the key alone,
and both land by pull request, as `add` does.

**A key is retired by date rather than removed.** With `valid_before` set, the
key ratifies nothing committed from that day on (midnight UTC), and what it
signed before stays verified. Removing a key instead turns every ratification it
signed into a fault, which is the act for a compromised key and the faults it
raises are the point: the committer date is chosen by whoever signs, so a stolen
key can still sign a backdated commit that a date would accept.

**`check` reads `signers:` and nothing else, for both formats.** Under
`gpg.format = ssh` it hands git an allowed-signers file it generates from the
block, each `valid_before` written as git's `valid-before` option, and judges
the date itself as well, so a git that does not honour the option still refuses
a retired key. Under OpenPGP git never reads an allowed-signers file at all, so
`check` matches the fingerprint and judges `valid_before` against the committer
date itself. Your repository's own git configuration is never changed: `git
verify-commit` goes on resolving signatures the way you set it up.

### Coming from .ank/allowed_signers

Before `signers:`, the keys were declared in `.ank/allowed_signers`, in git's
allowed-signers layout. `check` no longer reads that file, and reports one that
is still present as a fault naming the command that moves it:

    ank signers migrate

It writes every key of the file under `signers:`, keyed by its principal, keeps
the comment lines above a key and the comment after it as that key's `note`,
carries a `valid-before` option as `valid_before`, appends the block without
touching a byte above it, and deletes the file. It commits nothing: review the
diff and land it by pull request like any other change to `config.yml`.

## What check says afterwards

Once a key is declared, `check` judges every ratification against `signers:`,
a tag through `git verify-tag` and an older commit through its own signature,
and keeps four outcomes apart, which section 8 of the specification lists with
the reason for each. The one to expect on a machine that is not yours is
the signal and not the fault: a CI runner without your public key is a correct
repository on an incomplete machine, and `check` says it could not verify rather
than either failing or passing it.

A ratification made before ADR-5e47007a75be is a commit, and merged by squash or
rebase it loses what this page set up; the merge method that keeps it is
[Ratifying and archiving](ratifying.md#anchors-made-before-the-ref). A tag
travels as a ref and no merge method touches it.
