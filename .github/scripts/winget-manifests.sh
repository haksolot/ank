#!/usr/bin/env bash
#
# Renders packaging/winget/ from a published GitHub release.
#
#   winget-manifests.sh <version> [output-dir]
#
# winget's registry is `microsoft/winget-pkgs`, a central index that accepts a
# manifest and holds no authority over it -- the shape ADR-782a3556cf2d names as
# a registry rather than a satellite, and the same shape npm already has. So
# this repository derives the manifest and opens a pull request; the registry
# reviews it, and merging is theirs.
#
# The three files are winget's own requirement and not a layout chosen here. A
# single-file manifest is no longer accepted for a new submission: a version
# manifest names the package and the default locale, an installer manifest names
# the archive and its hash, and a locale manifest carries everything a human
# reads. All three repeat `PackageIdentifier` and `PackageVersion`, which is
# three chances to disagree, which is why one script writes all three.
#
# They are therefore derived files that happen to be committed, exactly like
# Formula/ank.rb and bucket/ank.json, and publish-winget.yml follows from that:
# a release runs this script and commits the result, and every other run runs it
# and asserts the committed files are exactly what it produced. A hand edit
# turns that job red rather than failing on somebody's machine.
#
# **The hash is read, never typed.** The .sha256 asset beside the archive is the
# only source of the number below. A constant written here is a constant that
# goes stale one tag later, and `winget install` verifies the hash itself, so a
# wrong one is a failed install reported by a user rather than a red job.
#
# There is no `--require-all` here, and the asymmetry with brew-formula.sh is
# the decision. That script covers three archives and tolerates a release
# missing one, because a formula narrowed to what exists is still a formula
# somebody can install. This one covers exactly one archive -- winget installs
# on Windows and nowhere else -- so a release without it leaves nothing to
# render at all, and the honest report is a failure rather than a manifest with
# no installer in it.
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: winget-manifests.sh <version> [output-dir]

  <version>       the released version, without the leading v (e.g. 0.2.0)
  [output-dir]    where to write the three manifests (default: packaging/winget)

Reads the sha256 from the .sha256 asset the release publishes. Needs `gh`
authenticated against the repository; GH_REPO overrides which one.
USAGE
}

positional=()
for arg in "$@"; do
  case "$arg" in
    -h | --help)
      usage
      exit 0
      ;;
    -*)
      echo "winget-manifests.sh: unknown flag '$arg'" >&2
      usage >&2
      exit 2
      ;;
    *) positional+=("$arg") ;;
  esac
done

# `${positional[0]:-}` and not `"${positional[@]}"`: expanding an empty array
# under `set -u` is an error on the bash 3.2 the macOS runners still carry, and
# this script has no reason to be the one that finds out.
version="${positional[0]:-}"
outdir="${positional[1]:-packaging/winget}"

if [ -z "$version" ]; then
  echo "winget-manifests.sh: no version given." >&2
  usage >&2
  exit 2
fi
version="${version#v}"

repo="${GH_REPO:-haksolot/ank}"
server="${GITHUB_SERVER_URL:-https://github.com}"
repo_url="${server}/${repo}"

# The identifier winget knows this package by. It is `Publisher.Package`, it is
# the name of every file below, and it is the path the submission lands on in
# microsoft/winget-pkgs. Changing it after a release is published is a new
# package rather than a rename, so it is written once, here.
identifier="Haksolot.Ank"

# The one archive winget installs from, and the directory it wraps its contents
# in. release.yml packages a zip around a directory named after the archive, so
# the nested path below is that name and not `ank.exe` alone -- the same trap
# `extract_dir` is on the Scoop side, and it fails at install time on a user's
# machine rather than at validation.
archive="ank-${version}-x86_64-pc-windows-msvc"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The directory handed to `gh` is not always the directory handed to `cut`.
# Under Git Bash on Windows, gh.exe is a native binary, so MSYS rewrites the
# POSIX path it is given -- and gets it wrong here: `gh release download`
# reports success having written the asset somewhere this script never looks,
# which reads exactly like an asset the release did not carry. cygpath exists
# only where the question does.
gh_tmp="$tmp"
if command -v cygpath >/dev/null 2>&1; then
  gh_tmp=$(cygpath -w "$tmp")
fi

echo "deriving the winget manifests for ${version} from ${repo_url}" >&2

if ! gh release download "v${version}" \
  --repo "$repo" \
  --pattern "${archive}.zip.sha256" \
  --dir "$gh_tmp" \
  --clobber >/dev/null 2>&1; then
  # A release that is not there at all is not the same fact as a release that
  # carried no Windows archive, and the command to run next differs.
  if ! gh release view "v${version}" --repo "$repo" >/dev/null 2>&1; then
    echo "winget-manifests.sh: ${repo} has no release v${version}." >&2
    echo "  a manifest is derived from a tag: cut it, or pass a version that has one." >&2
    exit 1
  fi
  echo "winget-manifests.sh: v${version} carries no ${archive}.zip." >&2
  echo "  winget installs on Windows and nowhere else, so there is nothing to render." >&2
  echo "  read the build jobs of the release run at ${repo_url}/releases/tag/v${version}" >&2
  exit 1
fi

# `sha256sum` wrote the line, so it is "<hash>  <name>" and the hash is the
# first space-separated field in either of its two output modes.
hash=$(cut -d' ' -f1 <"${tmp}/${archive}.zip.sha256")

# A truncated download, an HTML error page saved to disk, or a checksum file
# that changed shape would each yield something that is not a hash, and winget
# would report it as a mismatched download rather than as a broken manifest.
# Named here instead.
if ! printf '%s' "$hash" | grep -Eq '^[0-9a-f]{64}$'; then
  echo "winget-manifests.sh: the .sha256 asset of ${archive}.zip did not yield a sha256: '${hash}'" >&2
  echo "  check the asset at ${repo_url}/releases/tag/v${version}" >&2
  exit 1
fi

# Upper case because that is what the registry's own tooling writes, and a
# manifest that differs from wingetcreate's output only in the case of a hex
# string is a diff a reviewer has to read before dismissing.
hash=$(printf '%s' "$hash" | tr 'a-f' 'A-F')

# winget shows the release date in `winget show`, and it is a fact about the
# tag rather than about this run: read from the release, never from `date`.
published=$(gh release view "v${version}" --repo "$repo" --json publishedAt --jq .publishedAt) || exit 1
release_date="${published%%T*}"
if ! printf '%s' "$release_date" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
  echo "winget-manifests.sh: the release did not yield a publication date: '${published}'" >&2
  exit 1
fi

mkdir -p "$outdir"

# The note every one of the three files carries. `winget show` and a reviewer's
# eye both land on these files, and the first question either arrives with is
# where the numbers came from.
header="# Derived from the GitHub release by .github/scripts/winget-manifests.sh, which
# .github/workflows/publish-winget.yml runs on every pull request and asserts
# this file is exactly what deriving it produces. A hand edit turns that job red
# instead of failing on somebody's machine -- in particular the sha256, which is
# read from the .sha256 asset the release publishes and is never typed."

# Written at column 0 because a heredoc keeps every byte it is given, and the
# indentation below is YAML's rather than the shell's.

cat >"${outdir}/${identifier}.yaml" <<YAML
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json

${header}
PackageIdentifier: ${identifier}
PackageVersion: ${version}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
YAML

cat >"${outdir}/${identifier}.installer.yaml" <<YAML
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json

${header}
PackageIdentifier: ${identifier}
PackageVersion: ${version}
MinimumOSVersion: 10.0.0.0
ReleaseDate: ${release_date}
# The release publishes a zip and not an installer, so winget unpacks it and
# links the executable inside: \`zip\` with a \`portable\` nested type is that
# shape. The relative path is inside the directory the archive wraps its
# contents in, which release.yml names after the archive itself.
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: ${archive}\ank.exe
    PortableCommandAlias: ank
Installers:
  - Architecture: x64
    InstallerUrl: ${repo_url}/releases/download/v${version}/${archive}.zip
    InstallerSha256: ${hash}
ManifestType: installer
ManifestVersion: 1.6.0
YAML

cat >"${outdir}/${identifier}.locale.en-US.yaml" <<YAML
# yaml-language-server: \$schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json

${header}
PackageIdentifier: ${identifier}
PackageVersion: ${version}
PackageLocale: en-US
Publisher: haksolot
PublisherUrl: ${server}/haksolot
PublisherSupportUrl: ${repo_url}/issues
PackageName: ank
PackageUrl: ${repo_url}
License: GPL-3.0-only
LicenseUrl: ${repo_url}/blob/main/LICENSE
Copyright: Copyright (C) 2026 haksolot
ShortDescription: Tasks and architecture decisions in your repo, behind one CLI agents can call
Description: |-
  ank keeps a project's tasks and architecture decisions in the repository
  itself, as files an agent reads through one CLI. A decision carries a scope,
  so it binds the work that matches it rather than the work that remembers it,
  and a task's completion criterion is frozen by hash the moment it is claimed.
Moniker: ank
Tags:
  - agent
  - architecture
  - cli
  - decision-records
  - developer-tools
  - project-management
  - rust
  - tasks
ReleaseNotesUrl: ${repo_url}/releases/tag/v${version}
ManifestType: defaultLocale
ManifestVersion: 1.6.0
YAML

echo "wrote ${outdir}/${identifier}.yaml" >&2
echo "wrote ${outdir}/${identifier}.installer.yaml" >&2
echo "wrote ${outdir}/${identifier}.locale.en-US.yaml" >&2
