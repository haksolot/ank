#!/usr/bin/env bash
#
# check-version.sh, run against fixture trees: one where every manifest agrees,
# and three where something is out of step.
#
#   check-version-fixtures.sh
#
# A check that gates a release is a check nobody watches until the day it is
# wrong, and the one failure mode that costs a tag is a check that passes when
# it should not. So the fixtures are built from the manifests actually in the
# tree, copied and then broken: what is under test is the parsing of the shapes
# this repository really writes, not of a simplified copy that would drift away
# from them without either one turning red.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

check="$root/.github/scripts/check-version.sh"

manifests=(
  crates/ank-cli/Cargo.toml
  crates/ank-core/Cargo.toml
  crates/ank-mcp/Cargo.toml
  npm/ank/package.json
  npm/ank-linux-x64-musl/package.json
  npm/ank-darwin-arm64/package.json
  npm/ank-win32-x64/package.json
  .claude-plugin/plugin.json
)

# Read independently of the script under test, so the fixture's expectation is
# not derived from the code that has to meet it.
version="$(sed -n '/^\[package\]/,/^\[[^p]/ s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' \
  crates/ank-cli/Cargo.toml | head -n 1)"
if [ -z "$version" ]; then
  echo "cannot read the version out of crates/ank-cli/Cargo.toml" >&2
  exit 9
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# plant <dir> -- the eight manifests, at their real paths, byte for byte.
plant() {
  local dir="$1" m
  for m in "${manifests[@]}"; do
    mkdir -p "$dir/$(dirname "$m")"
    cp "$m" "$dir/$m"
  done
}

# rewrite <file> <sed script> -- sed -i is not portable to the macOS runner.
rewrite() {
  local f="$1" script="$2"
  sed "$script" "$f" > "$f.new"
  mv "$f.new" "$f"
}

failures=0

# expect_pass <name> <tree>
expect_pass() {
  local name="$1" tree="$2" out
  if out="$(bash "$check" "$version" "$tree" 2>&1)"; then
    echo "ok    $name"
    return
  fi
  echo "FAIL  $name: expected agreement, got"
  echo "$out" | sed 's/^/      /'
  failures=$((failures + 1))
}

# expect_fail <name> <tree> <expected count> <substring ...>
expect_fail() {
  local name="$1" tree="$2" want="$3" out s
  shift 3
  if out="$(bash "$check" "$version" "$tree" 2>&1)"; then
    echo "FAIL  $name: expected a refusal, the check passed"
    echo "$out" | sed 's/^/      /'
    failures=$((failures + 1))
    return
  fi
  if ! printf '%s\n' "$out" | grep -q "disagrees with $want of "; then
    echo "FAIL  $name: expected $want disagreements, got"
    echo "$out" | sed 's/^/      /'
    failures=$((failures + 1))
    return
  fi
  for s in "$@"; do
    if ! printf '%s\n' "$out" | grep -qF "$s"; then
      echo "FAIL  $name: the report never names $s"
      echo "$out" | sed 's/^/      /'
      failures=$((failures + 1))
      return
    fi
  done
  echo "ok    $name"
}

# Every manifest as the tree carries it. This one also asserts the tree agrees
# with itself between releases, which is the state a reader sees.
plant "$tmp/agree"
expect_pass "the tree agrees with itself at $version" "$tmp/agree"

# One manifest left behind, and only that one reported.
plant "$tmp/one"
rewrite "$tmp/one/npm/ank-win32-x64/package.json" \
  's/^  "version": ".*"/  "version": "0.0.0-stale"/'
expect_fail "one stale manifest is named" "$tmp/one" 1 \
  "npm/ank-win32-x64/package.json" \
  "0.0.0-stale" \
  "npm pkg set version=$version --prefix npm/ank-win32-x64"

# Two, in two files and of two shapes -- a Cargo table and a pinned dependency.
# Both are named: a report that stops at the first sends the maintainer round
# the loop once per stale file.
plant "$tmp/two"
rewrite "$tmp/two/crates/ank-core/Cargo.toml" \
  '0,/^version = ".*"/s/^version = ".*"/version = "0.0.0-stale"/'
rewrite "$tmp/two/npm/ank/package.json" \
  's|^    "@haksolot/ank-darwin-arm64": ".*"|    "@haksolot/ank-darwin-arm64": "0.0.0-stale"|'
expect_fail "every disagreement is named, not the first" "$tmp/two" 2 \
  "crates/ank-core/Cargo.toml" \
  "npm/ank/package.json @haksolot/ank-darwin-arm64"

# A literal the parser can no longer find. This is the failure that matters:
# a shape that moves must turn the check red rather than quietly stop checking
# that file.
plant "$tmp/shape"
rewrite "$tmp/shape/.claude-plugin/plugin.json" '/^  "version":/d'
expect_fail "a version literal that moved is a failure, not a pass" "$tmp/shape" 1 \
  ".claude-plugin/plugin.json" \
  "the file's shape moved"

# A file that is not there at all.
plant "$tmp/missing"
rm "$tmp/missing/npm/ank-linux-x64-musl/package.json"
expect_fail "a manifest that is gone is a failure" "$tmp/missing" 1 \
  "npm/ank-linux-x64-musl/package.json" \
  "file missing"

if [ "$failures" -ne 0 ]; then
  echo
  echo "$failures fixture(s) failed"
  exit 1
fi

echo
echo "check-version.sh behaves on every fixture"
