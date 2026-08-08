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

exe_of() {
  case "$1" in
    ank-win32-x64) echo ank.exe ;;
    *) echo ank ;;
  esac
}

for p in "${packages[@]}"; do
  target="$(target_of "$p")"
  exe="$(exe_of "$p")"
  # The loose binary the build job uploads beside the archives. Reaching for it
  # rather than unpacking a .zip keeps this script free of unzip, which the
  # Windows runner's bash does not have.
  src="$(find dist -type f -path "*${target}*" -name "$exe" | head -n 1)"
  if [ -z "$src" ]; then
    echo "no $exe built for $target under dist/" >&2
    exit 1
  fi
  mkdir -p "npm/$p/bin"
  cp "$src" "npm/$p/bin/$exe"
  chmod +x "npm/$p/bin/$exe"
  cp LICENSE "npm/$p/LICENSE"
  (
    cd "npm/$p"
    npm pkg set version="$version"
    npm pack --silent > /dev/null
  )
  echo "assembled npm/$p from $src"
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
# skills/<name>/SKILL.md. The source is skill/SKILL.md at the repository root,
# where the freeze lives: build.rs hashes it into `ank --version` and
# tests/skill.rs holds the file to that hash. A second copy committed beside it
# would have no such anchor and would drift with nothing turning red, which is
# what ADR-e3cb36646d77 refuses. So the copy is made here, from the one file, on
# every run, and .gitignore keeps it out of the tree -- the same arrangement as
# LICENSE on the line above. The release smoke job is what checks it arrived.
mkdir -p skills/ank
cp ../../skill/SKILL.md skills/ank/SKILL.md

npm pack --silent > /dev/null
echo "assembled npm/ank at $version"
