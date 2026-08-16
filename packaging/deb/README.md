# The apt channel

`ank` installs from an apt repository this project hosts on its own GitHub
Pages, at <https://haksolot.github.io/ank/deb>. The index lives where every
other channel lives, and no satellite repository exists to keep in step
(ADR-782a3556cf2d, carrying ADR-e3cb36646d77 forward).

```sh
sudo apt-get install -y curl gnupg
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://haksolot.github.io/ank/deb/ank-archive-keyring.asc |
  sudo tee /etc/apt/keyrings/ank-archive-keyring.asc > /dev/null
echo "deb [signed-by=/etc/apt/keyrings/ank-archive-keyring.asc] https://haksolot.github.io/ank/deb stable main" |
  sudo tee /etc/apt/sources.list.d/ank.list > /dev/null
sudo apt-get update
sudo apt-get install -y ank
```

## What is in this directory

`ank-archive-keyring.asc` is the public half of the key the index is signed
with, and it is the only file here that is not generated. Its fingerprint:

```
307CF7403388668AC8C18C9EAD44831D7AB3E0C1
ank distribution key (apt) <83018259+haksolot@users.noreply.github.com>
```

**It is a distribution key and never this project's ratification key.** Section
8's authority model rests on the ratification key meaning exactly one thing --
this decision was ratified, and `ank check` verifies it against
`.ank/allowed_signers`. Signing packages with it would put the same signature on
a claim of an entirely different kind, and the whole value of the line is that
it means one thing. The two are different keys, generated in different keyrings,
and `.github/scripts/apt-repo.sh` fails if the key it signs with is not the one
committed here.

## What is generated, and when

Everything else. `.github/scripts/apt-repo.sh` builds the entire site --
`pool/`, `dists/`, the signed `Release`, `InRelease` and `Release.gpg`, the
exported keyring and the landing page -- from this project's GitHub releases,
and `.github/workflows/publish-apt.yml` runs it. Nothing is carried over
between runs and nothing is edited in place, so there is no state to drift: the
pool is a function of the releases, and a release yanked on GitHub disappears
from apt at the next publish.

The `.deb` is built around the archive the release published, verified against
the `.sha256` published beside it, and never around a fresh `cargo build`. A
package built from a second compilation is a second artefact, and the point of
a channel is to carry the one that was released.

`amd64` only: `release.yml` builds no aarch64 Linux target, so a repository
claiming `arm64` would claim a package it cannot serve.
