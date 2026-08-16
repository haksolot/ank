#!/usr/bin/env bash
#
# Builds the whole apt repository from this project's GitHub releases.
#
#   apt-repo.sh <output-dir>
#
# The repository is hosted on this project's own GitHub Pages, so the index
# lives where every other channel lives and no satellite appears
# (ADR-782a3556cf2d, carrying ADR-e3cb36646d77 forward).
#
# **The pool is a function of the releases, and nothing else.** Every run
# rebuilds it entire, from every published release that is not a prerelease and
# carries a musl archive. Nothing is carried over from the previous deployment
# and nothing is edited in place, so there is no state between runs to drift:
# a release yanked on GitHub disappears from apt at the next publish, and a
# pool nobody could reconstruct never exists. The cost is one download per
# release per run, which is a handful today and is the number to watch if this
# project ever has hundreds.
#
# Prereleases are excluded because the suite is `stable` and the specification
# already says a prerelease is marked as one on the release page. A channel that
# quietly served them would be a second reading of the same tag.
#
# **The binary is the released one, verified.** The .deb is built around the
# archive the release published, checked against the .sha256 published beside
# it, and never around a fresh `cargo build`: a package built from a second
# compilation is a second artefact, and the point of a channel is to carry the
# one that was released.
#
# **Which key signs is decided by the caller, and a release may not fall back.**
# APT_GPG_PRIVATE_KEY is the distribution key held in Actions secrets. When it
# is absent -- a pull request from a fork, where secrets are not exposed -- this
# script generates a throwaway key instead, so the whole path stays testable:
# what is under test on those runs is the mechanism, an apt client fetching a
# key and refusing an index it cannot verify, and a throwaway key exercises it
# exactly. What must never happen is a release publishing a tree signed by a key
# no user trusts, so APT_REQUIRE_KEY=1 refuses the fallback outright, and
# publish-apt.yml sets it on the release path.
#
# The distribution key is never the ratification key. Section 8's authority
# model rests on the ratification key meaning one thing -- this decision was
# ratified -- and signing packages with it would put the same signature on a
# claim of an entirely different kind.
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: apt-repo.sh <output-dir>

  <output-dir>   where to write the site (the apt tree lands in <dir>/deb)

Environment:
  APT_GPG_PRIVATE_KEY   armoured private key to sign the index with.
                        Absent, a throwaway key is generated instead.
  APT_REQUIRE_KEY=1     refuse the throwaway fallback and fail instead.
  GH_REPO               which repository to read releases from.

Needs `gh` authenticated, plus dpkg-dev, apt-utils and gpg.
USAGE
}

for arg in "$@"; do
  case "$arg" in
    -h | --help)
      usage
      exit 0
      ;;
  esac
done

site="${1:-}"
if [ -z "$site" ]; then
  echo "apt-repo.sh: no output directory given." >&2
  usage >&2
  exit 2
fi

repo="${GH_REPO:-haksolot/ank}"
server="${GITHUB_SERVER_URL:-https://github.com}"
repo_url="${server}/${repo}"
pages_url="${APT_PAGES_URL:-https://haksolot.github.io/ank}"

# The one architecture there is. release.yml builds no aarch64 Linux target, so
# a repository claiming arm64 would claim a package it cannot serve.
arch="amd64"
suite="stable"
component="main"

for tool in gh gpg dpkg-deb dpkg-scanpackages apt-ftparchive; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "apt-repo.sh: ${tool} is not installed." >&2
    echo "  apt-get install -y dpkg-dev apt-utils gnupg" >&2
    exit 9
  fi
done

work=$(mktemp -d)
export GNUPGHOME="${work}/gnupg"
mkdir -p "$GNUPGHOME"
chmod 700 "$GNUPGHOME"
# The private key exists in this process and in nothing that outlives it.
trap 'rm -rf "$work"' EXIT

deb="${site}/deb"
pool="${deb}/pool/${component}/a/ank"
dist="${deb}/dists/${suite}/${component}/binary-${arch}"
rm -rf "$site"
mkdir -p "$pool" "$dist"

# ---------------------------------------------------------------- the key

if [ -n "${APT_GPG_PRIVATE_KEY:-}" ]; then
  printf '%s\n' "$APT_GPG_PRIVATE_KEY" | gpg --batch --quiet --import
  key_kind="the distribution key from Actions secrets"
else
  if [ "${APT_REQUIRE_KEY:-0}" = "1" ]; then
    echo "apt-repo.sh: APT_GPG_PRIVATE_KEY is empty and APT_REQUIRE_KEY is set." >&2
    echo "  a published tree signed by a throwaway key is a tree no user can install from." >&2
    echo "" >&2
    echo "  gh secret set APT_GPG_PRIVATE_KEY --repo ${repo}" >&2
    exit 1
  fi
  echo "no APT_GPG_PRIVATE_KEY: generating a throwaway key for this run only" >&2
  gpg --batch --quiet --passphrase '' \
    --quick-generate-key "ank throwaway signing key <nobody@example.invalid>" \
    rsa3072 sign never
  key_kind="a throwaway key generated for this run"
fi

fingerprint=$(gpg --list-secret-keys --with-colons | sed -n 's/^fpr:::::::::\(.*\):/\1/p' | head -1)
if [ -z "$fingerprint" ]; then
  echo "apt-repo.sh: no secret key to sign with after import." >&2
  echo "  APT_GPG_PRIVATE_KEY is set but gpg imported nothing from it: check it is the" >&2
  echo "  armoured private key and not the public one." >&2
  exit 1
fi
echo "signing with ${key_kind}: ${fingerprint}" >&2

gpg --armor --export "$fingerprint" > "${deb}/ank-archive-keyring.asc"

# The committed copy is what a reader is pointed at, and what the instructions
# in packaging/deb/README.md name. If the secret is ever rotated and the tree is
# not, an apt client would fetch a key that verifies nothing -- a failure that
# shows up on the user's machine and nowhere else. Compared here instead.
committed="packaging/deb/ank-archive-keyring.asc"
if [ -n "${APT_GPG_PRIVATE_KEY:-}" ] && [ -f "$committed" ]; then
  have=$(gpg --show-keys --with-colons "$committed" | sed -n 's/^fpr:::::::::\(.*\):/\1/p' | head -1)
  if [ "$have" != "$fingerprint" ]; then
    echo "apt-repo.sh: the signing key is not the one committed to the tree." >&2
    echo "  APT_GPG_PRIVATE_KEY:  ${fingerprint}" >&2
    echo "  ${committed}: ${have}" >&2
    echo "" >&2
    echo "  the secret was rotated and the public half was not. export it and commit it:" >&2
    echo "  gpg --armor --export ${fingerprint} > ${committed}" >&2
    exit 1
  fi
  echo "the signing key matches ${committed}" >&2
fi

# ---------------------------------------------------------------- the pool

# Drafts are invisible to a reader and prereleases are marked as such on the
# release page, so neither belongs in a suite called stable.
mapfile -t versions < <(
  gh release list --repo "$repo" --limit 100 \
    --json tagName,isPrerelease,isDraft \
    --jq '.[] | select(.isPrerelease == false and .isDraft == false) | .tagName' |
    sed 's/^v//'
)

if [ "${#versions[@]}" -eq 0 ]; then
  echo "apt-repo.sh: ${repo} has published no release to build a pool from." >&2
  exit 1
fi

built=0
skipped=()
for version in "${versions[@]}"; do
  name="ank-${version}-x86_64-unknown-linux-musl"

  if ! gh release download "v${version}" --repo "$repo" \
    --pattern "${name}.tar.gz" --pattern "${name}.tar.gz.sha256" \
    --dir "${work}/dl" --clobber >/dev/null 2>&1; then
    # A release with no Linux archive is a fact about that release and not a
    # failure here: v0.1.0 predates rows that were added later, and a pool that
    # refused to build because of it would serve nothing at all.
    skipped+=("$version")
    continue
  fi

  ( cd "${work}/dl" && sha256sum -c "${name}.tar.gz.sha256" >/dev/null ) || {
    echo "apt-repo.sh: ${name}.tar.gz does not match the sha256 the release published." >&2
    echo "  check the assets at ${repo_url}/releases/tag/v${version}" >&2
    exit 1
  }

  root="${work}/pkg/${version}"
  rm -rf "$root"
  mkdir -p "${root}/DEBIAN" "${root}/usr/bin" "${root}/usr/share/doc/ank"
  tar xzf "${work}/dl/${name}.tar.gz" -C "${work}/dl"

  install -m 0755 "${work}/dl/${name}/ank" "${root}/usr/bin/ank"
  # Debian reads a package's licence from this path and from no other, so the
  # release's LICENSE arrives under the name policy gives it.
  install -m 0644 "${work}/dl/${name}/LICENSE" "${root}/usr/share/doc/ank/copyright"
  install -m 0644 "${work}/dl/${name}/README.md" "${root}/usr/share/doc/ank/README.md"
  # SKILL.md travels with the binary because a channel shipping ank and not what
  # it teaches has shipped half of it.
  install -m 0644 "${work}/dl/${name}/SKILL.md" "${root}/usr/share/doc/ank/SKILL.md"

  # What the package puts on the filesystem, which is not what builds it: the
  # control directory never lands on the target and Installed-Size is a promise
  # about disk after unpacking.
  size=$(du -ks --exclude=DEBIAN "$root" | cut -f1)

  # No Depends field, and that is the decision rather than an omission: the
  # binary is a static musl build, so it links no libc and needs nothing from
  # the archive. This is the rare .deb that is genuinely architecture-bound and
  # dependency-free.
  cat > "${root}/DEBIAN/control" <<CONTROL
Package: ank
Version: ${version}
Architecture: ${arch}
Maintainer: haksolot <83018259+haksolot@users.noreply.github.com>
Installed-Size: ${size}
Section: devel
Priority: optional
Homepage: ${repo_url}
Description: Tasks and architecture decisions in your repo, behind one CLI
 ank keeps a project's tasks and architecture decisions in the repository
 itself, as files an agent reads through one CLI. A decision carries a scope,
 so it binds the work that matches it rather than the work that remembers it,
 and a task's completion criterion is frozen by hash the moment it is claimed.
 .
 The binary is statically linked against musl and depends on nothing in the
 archive.
CONTROL

  # --root-owner-group so the package does not carry whatever uid built it.
  dpkg-deb --build --root-owner-group "$root" "${pool}/ank_${version}_${arch}.deb" >/dev/null
  echo "built ank_${version}_${arch}.deb" >&2
  built=$((built + 1))
done

if [ "${#skipped[@]}" -gt 0 ]; then
  echo "no ${arch} archive, omitted from the pool: ${skipped[*]}" >&2
fi
if [ "$built" -eq 0 ]; then
  echo "apt-repo.sh: no release carried a Linux archive, so the pool is empty." >&2
  echo "  an empty index is worse than none: apt reports it as a repository with" >&2
  echo "  nothing in it rather than as a broken publish." >&2
  exit 1
fi

# --------------------------------------------------------------- the index

# From inside the tree, because the paths dpkg-scanpackages writes into
# Packages are relative to where it ran and apt resolves them against the
# repository root.
(
  cd "$deb"
  dpkg-scanpackages --multiversion "pool" /dev/null > "dists/${suite}/${component}/binary-${arch}/Packages"
  gzip -9kf "dists/${suite}/${component}/binary-${arch}/Packages"

  # Written elsewhere and moved into place, which is not fussiness: the
  # redirection would create an empty Release before apt-ftparchive scanned the
  # directory, and it would then list Release among the files it checksums --
  # with the size and hash of the empty file it is about to become. apt reports
  # that as a hash mismatch on the index it just downloaded, and the cause is
  # nowhere in the message.
  apt-ftparchive \
    -o "APT::FTPArchive::Release::Origin=ank" \
    -o "APT::FTPArchive::Release::Label=ank" \
    -o "APT::FTPArchive::Release::Suite=${suite}" \
    -o "APT::FTPArchive::Release::Codename=${suite}" \
    -o "APT::FTPArchive::Release::Architectures=${arch}" \
    -o "APT::FTPArchive::Release::Components=${component}" \
    -o "APT::FTPArchive::Release::Description=ank, built from its GitHub releases" \
    release "dists/${suite}" > "${work}/Release"
  mv "${work}/Release" "dists/${suite}/Release"

  # Both forms, because both are still in use: apt prefers InRelease and falls
  # back to Release plus Release.gpg, and a repository serving only one of them
  # works until it meets a client that wanted the other.
  gpg --batch --yes --local-user "$fingerprint" \
    --clearsign -o "dists/${suite}/InRelease" "dists/${suite}/Release"
  gpg --batch --yes --local-user "$fingerprint" \
    -abs -o "dists/${suite}/Release.gpg" "dists/${suite}/Release"
)

# ---------------------------------------------------------------- the page

# The root of the site is where a user lands after reading `apt-get install ank`
# somewhere, so it answers the question they arrive with rather than 404ing.
cat > "${site}/index.html" <<HTML
<!doctype html>
<meta charset="utf-8">
<title>ank &mdash; apt repository</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
  body { font: 16px/1.6 system-ui, sans-serif; max-width: 42rem; margin: 4rem auto; padding: 0 1rem; }
  pre { background: #f4f4f4; padding: 1rem; overflow-x: auto; }
  code { font-family: ui-monospace, monospace; }
  footer { margin-top: 3rem; font-size: .875rem; color: #555; }
</style>
<h1>ank</h1>
<p>Tasks and architecture decisions in your repo, behind one CLI agents can
call. <a href="${repo_url}">Source</a>.</p>

<h2>Install on Debian or Ubuntu</h2>
<pre><code>sudo apt-get install -y curl gnupg
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL ${pages_url}/deb/ank-archive-keyring.asc |
  sudo tee /etc/apt/keyrings/ank-archive-keyring.asc &gt; /dev/null
echo "deb [signed-by=/etc/apt/keyrings/ank-archive-keyring.asc] ${pages_url}/deb ${suite} ${component}" |
  sudo tee /etc/apt/sources.list.d/ank.list &gt; /dev/null
sudo apt-get update
sudo apt-get install -y ank</code></pre>

<p>The signing key is a distribution key and never this project's ratification
key. Its fingerprint is:</p>
<pre><code>${fingerprint}</code></pre>

<p>${arch} only: the release matrix builds no aarch64 Linux target, so a
repository claiming arm64 would claim a package it cannot serve.</p>

<footer>This page and the repository below it are rebuilt from
${repo_url}/releases on every publish. Nothing here is edited by hand.</footer>
HTML

echo "" >&2
echo "wrote ${built} package(s) to ${deb}" >&2
echo "signed with ${fingerprint}" >&2
