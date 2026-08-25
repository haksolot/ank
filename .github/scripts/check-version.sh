#!/usr/bin/env bash
#
# Every version literal in the tree, compared with the version the release is
# about to publish.
#
#   check-version.sh <version> [root]
#
# The release derives what it publishes from the tag and from nothing else, so
# a tag pushed without bumping the manifests produces an archive, an npm package
# and a release page that all agree with each other and all disagree with the
# binary they contain. The smoke job cannot catch it: it compares the artefact
# with itself, which is a correct test of the wrapper and no test at all of the
# version.
#
# Ten literals across seven files carry the version today, which is nine more
# chances to be careful than anyone gets right forever. This makes the
# disagreement fail instead.
#
# Every disagreement is named, not the first one: a check that stops at the
# first sends the maintainer round the loop once per stale file, and the loop
# here costs a tag.
#
# The parsing is sed and awk. jq is on the runner but not on every machine a
# maintainer would run this from, and the shapes read here are seven files this
# repository writes itself. A literal the parser cannot find is a failure and
# never a pass -- a shape that moved must turn this red, or the check quietly
# stops checking.
set -uo pipefail

version="${1:?usage: check-version.sh <version> [root]}"
root="${2:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

cd "$root" || exit 9

# Parallel arrays rather than one associative array: this also runs on the
# macOS runner's bash 3.2, where associative arrays do not exist.
bad_what=()
bad_found=()
bad_fix=()
total=0

# The version in the [package] table, which is rust-version's neighbour. Scoped
# to that table on purpose: a `version = "..."` under [dependencies] is a
# dependency requirement and has nothing to do with what this release ships.
cargo_version() {
  awk '
    /^\[/ { in_package = ($0 == "[package]") }
    in_package && /^version[[:space:]]*=/ {
      if (match($0, /"[^"]*"/)) {
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
      }
    }
  ' "$1"
}

# The top-level "version" key, anchored on its two-space indentation. A nested
# "version" lives deeper and is not what a release publishes.
json_version() {
  sed -n 's/^  "version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -n 1
}

# One pinned entry of optionalDependencies, four-space indented.
json_pinned_dep() {
  sed -n "s|^    \"$2\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*|\1|p" "$1" | head -n 1
}

# compare <label> <file> <found> <fix>
#
# An absent file and an unreadable literal are both failures, and both say which
# it was: "no version found" means the file's shape moved and this script is the
# thing to repair, not the manifest.
compare() {
  local what="$1" file="$2" found="$3" fix="$4"
  total=$((total + 1))
  if [ ! -f "$file" ]; then
    bad_what+=("$what")
    bad_found+=("file missing")
    bad_fix+=("$fix")
    return
  fi
  if [ -z "$found" ]; then
    bad_what+=("$what")
    bad_found+=("no version found -- the file's shape moved")
    bad_fix+=("teach .github/scripts/check-version.sh the new shape of $file")
    return
  fi
  [ "$found" = "$version" ] && return
  bad_what+=("$what")
  bad_found+=("$found")
  bad_fix+=("$fix")
}

# The two crates whose version a release is about: `ank-cli` is the executable
# every route carries, and `ank-core` is what it is. The libraries linked into
# it -- the verb table, the protocol surface, the watcher, the terminal reader
# -- are not gated here, for the reason ADR-1ea31c2f3c5a removed the gate that
# used to be: nothing published carries any of them as a file of its own, so a
# version they disagree on is not a version anybody downloads.
#
# **That reason is about files, and one of those libraries was also telling
# people a number** (TASK-ae64d1c5678d). `ank-mcp` used to be gated here and was
# dropped with the rest of the second executable's machinery, while its
# `CARGO_PKG_VERSION` went on reaching a client in `serverInfo` and landing in
# the `ank-mcp/<version>` identity of every claim taken through the surface. The
# repair is not a gate back: the surface reports the version the dispatch hands
# it, which is `ank-cli`'s, so the number a client reads is a number already
# compared above. Gating the library too would have put a second literal here
# whose only job was to be equal to the first -- one more of the ten, and the
# whole argument of this script is that nobody keeps ten right forever.
#
# So the rule for this loop is what a release publishes as a file, and nothing
# else. A library that starts telling a reader its own number belongs in one of
# the two lists: this one, or `Address::version`'s.
for crate in ank-cli ank-core; do
  f="crates/$crate/Cargo.toml"
  compare "$f" "$f" "$(cargo_version "$f" 2>/dev/null)" \
    "$f: version = \"$version\""
done

# npm-assemble.sh stamps the npm manifests at release time with `npm pkg set`,
# so the run overwrites these rather than reading them. They are compared all
# the same: the tree is what a reader sees between releases, and a tree that
# says 0.1.3 while `latest` is 0.2.0 is the same lie told to a different
# audience.
compare "npm/ank/package.json" "npm/ank/package.json" \
  "$(json_version npm/ank/package.json 2>/dev/null)" \
  "npm pkg set version=$version --prefix npm/ank"

for p in ank-linux-x64-musl ank-darwin-arm64 ank-win32-x64; do
  # The wrapper pins its platform packages exactly, so a pin left behind
  # resolves an install to a binary from a different build.
  compare "npm/ank/package.json @haksolot/$p" "npm/ank/package.json" \
    "$(json_pinned_dep npm/ank/package.json "@haksolot/$p" 2>/dev/null)" \
    "npm pkg set \"optionalDependencies.@haksolot/$p=$version\" --prefix npm/ank"

  compare "npm/$p/package.json" "npm/$p/package.json" \
    "$(json_version "npm/$p/package.json" 2>/dev/null)" \
    "npm pkg set version=$version --prefix npm/$p"
done

compare ".claude-plugin/plugin.json" ".claude-plugin/plugin.json" \
  "$(json_version .claude-plugin/plugin.json 2>/dev/null)" \
  ".claude-plugin/plugin.json: \"version\": \"$version\""

count=${#bad_what[@]}

if [ "$count" -eq 0 ]; then
  echo "version $version agrees with all $total version literals"
  exit 0
fi

echo "version $version disagrees with $count of $total version literals"
echo
i=0
while [ "$i" -lt "$count" ]; do
  printf '  %-50s %s\n' "${bad_what[$i]}" "${bad_found[$i]}"
  i=$((i + 1))
done
echo
echo "repair every line above, then move the tag:"
i=0
while [ "$i" -lt "$count" ]; do
  echo "  ${bad_fix[$i]}"
  i=$((i + 1))
done
exit 1
