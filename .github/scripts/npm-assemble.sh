#!/usr/bin/env bash
#
# Fills the npm packages with the binaries the build job produced, stamps one
# version across all four, and packs them.
#
# The binaries come from the same artefacts the GitHub release publishes: one
# build, two channels, and no second compilation that could disagree with the
# first. Nothing here downloads anything, which is the property the whole
# channel exists for (npm/README.md).
#
# Two executables per platform package, never one: ADR-e39a44f80e0e makes
# ank-mcp freight of every route that carries ank, so the pair travels in one
# package under one version and an install cannot end up holding half of it.
#
#   npm-assemble.sh <version> [package ...]
#
# With no package named, all three are assembled -- which is what the publish
# job wants. The smoke job names the one platform whose artefact it downloaded,
# because it only has that one.

set -euo pipefail

version="${1:?usage: npm-assemble.sh <version> [package ...]}"
shift
packages=("$@")
if [ "${#packages[@]}" -eq 0 ]; then
  packages=(ank-linux-x64-musl ank-darwin-arm64 ank-win32-x64)
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

target_of() {
  case "$1" in
    ank-linux-x64-musl) echo x86_64-unknown-linux-musl ;;
    ank-darwin-arm64) echo aarch64-apple-darwin ;;
    ank-win32-x64) echo x86_64-pc-windows-msvc ;;
    *)
      echo "unknown package: $1" >&2
      return 1
      ;;
  esac
}

# The extension, applied to both names rather than each name written out twice
# per platform. Windows is the only row that has one.
suffix_of() {
  case "$1" in
    ank-win32-x64) echo .exe ;;
    *) echo "" ;;
  esac
}

# The two executables a platform package carries, in the order the wrapper
# declares them. A name added here is a name the release must have built, and a
# missing one below stops the assembly rather than shipping a package that is
# short a binary.
executables=(ank ank-mcp)

for p in "${packages[@]}"; do
  target="$(target_of "$p")"
  suffix="$(suffix_of "$p")"
  mkdir -p "npm/$p/bin"
  for name in "${executables[@]}"; do
    exe="${name}${suffix}"
    # The loose binaries the build job uploads beside the archives. Reaching for
    # them rather than unpacking a .zip keeps this script free of unzip, which
    # the Windows runner's bash does not have.
    src="$(find dist -type f -path "*${target}*" -name "$exe" | head -n 1)"
    if [ -z "$src" ]; then
      echo "no $exe built for $target under dist/" >&2
      exit 1
    fi
    cp "$src" "npm/$p/bin/$exe"
    chmod +x "npm/$p/bin/$exe"
    echo "  npm/$p/bin/$exe from $src"
  done
  cp LICENSE "npm/$p/LICENSE"
  (
    cd "npm/$p"
    npm pkg set version="$version"
    npm pack --silent > /dev/null
  )
  echo "assembled npm/$p at $version"
done

# The wrapper pins the platform packages exactly. A range would let an install
# resolve a binary from a different build than the wrapper it came with, and
# the version is the only thing tying the two together.
cd npm/ank
npm pkg set version="$version"
for p in ank-linux-x64-musl ank-darwin-arm64 ank-win32-x64; do
  npm pkg set "optionalDependencies.@haksolot/$p=$version"
done
cp ../../LICENSE LICENSE

# pi reads a package's skills from its pi.skills path, and its convention is
# skills/<name>/SKILL.md. The sources are skill/SKILL.md at the repository root
# and every skill/<dir>/SKILL.md beside it: the contract and one policy per
# activity (ADR-91b77f036884). Each is anchored where it lives -- build.rs
# hashes the contract into `ank --version`, tests/skill.rs holds every skill to
# its declared revision -- and a copy committed beside them would have no such
# anchor and would drift with nothing turning red, which is what
# ADR-e3cb36646d77 refuses. So the copies are made here, from the one file per
# skill, on every run, and .gitignore keeps them out of the tree, the same
# arrangement as LICENSE on the line above. The release smoke job is what checks
# they arrived.
#
# The destination is the name the skill declares in its own frontmatter, not the
# directory it sits in: `ank-plan` lives in `skill/plan/`, and pi installs what
# the frontmatter calls it. Deriving it from the file means a skill added later
# is packaged by existing, with nothing here to remember.
skill_name() {
  sed -n 's/^name:[[:space:]]*//p' "$1" | head -n 1
}

for src in ../../skill/SKILL.md ../../skill/*/SKILL.md; do
  [ -f "$src" ] || continue
  name="$(skill_name "$src")"
  if [ -z "$name" ]; then
    echo "$src declares no name: in its frontmatter" >&2
    exit 1
  fi
  mkdir -p "skills/$name"
  cp "$src" "skills/$name/SKILL.md"
  echo "packaged skill $name from ${src#../../}"
done

npm pack --silent > /dev/null
echo "assembled npm/ank at $version"
