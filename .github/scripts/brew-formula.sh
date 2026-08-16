#!/usr/bin/env bash
#
# Renders Formula/ank.rb from a published GitHub release.
#
#   brew-formula.sh <version> [output] [--require-all]
#
# This repository is its own Homebrew tap: `brew tap <user>/<name> <url>` takes
# an arbitrary URL, so Formula/ank.rb in this tree is a working tap and no
# satellite repository exists to keep in step (ADR-782a3556cf2d, carrying
# ADR-e3cb36646d77 forward).
#
# Formula/ank.rb is therefore a derived file that happens to be committed, and
# the design of publish-brew.yml follows from that: the derivation lives here,
# a release runs it and commits the result, and every other run runs it and
# asserts the committed file is exactly what it produced. A hand edit turns
# that job red rather than failing on somebody's machine.
#
# **The hash is read, never typed.** Every archive on the release page carries a
# `.sha256` beside it, and that asset is the only source of the numbers below. A
# constant written here is a constant that goes stale one tag later, and it is
# the one defect in a formula that shows up nowhere except on the machine of
# whoever runs the install: `brew install` verifies the checksum itself, so a
# wrong number is a failed download reported by a user rather than a red job.
#
# `--require-all` refuses a release missing any archive the formula covers, and
# publish-brew.yml passes it on a release event and nowhere else. The asymmetry
# is the decision. A tag cut from today's matrix builds all four targets, so at
# release time a missing archive is a broken release and never a fact about the
# formula. Between releases the last release is whatever it was, and it is not
# this script's business to wish otherwise: v0.2.0 was cut on 2026-08-11, the
# Intel macOS row landed in release.yml on 2026-08-15, and a formula that
# pointed at an x86_64 archive v0.2.0 never published would 404 on the user's
# machine. A branch omitted is a branch a user can read; a branch pointing at
# nothing is the failure this formula was written to avoid.
set -euo pipefail

usage() {
  cat <<'USAGE'
usage: brew-formula.sh <version> [output] [--require-all]

  <version>       the released version, without the leading v (e.g. 0.2.0)
  [output]        where to write the formula (default: Formula/ank.rb)
  --require-all   fail if the release is missing any archive the formula covers

Reads each sha256 from the .sha256 asset the release publishes. Needs `gh`
authenticated against the repository; GH_REPO overrides which one.
USAGE
}

require_all=0
positional=()
for arg in "$@"; do
  case "$arg" in
    --require-all) require_all=1 ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*)
      echo "brew-formula.sh: unknown flag '$arg'" >&2
      usage >&2
      exit 2
      ;;
    *) positional+=("$arg") ;;
  esac
done

# `${positional[0]:-}` and not `"${positional[@]}"`: this also runs on the macOS
# runner's bash 3.2, where expanding an empty array under `set -u` is an error.
version="${positional[0]:-}"
output="${positional[1]:-Formula/ank.rb}"

if [ -z "$version" ]; then
  echo "brew-formula.sh: no version given." >&2
  usage >&2
  exit 2
fi
version="${version#v}"

repo="${GH_REPO:-haksolot/ank}"
server="${GITHUB_SERVER_URL:-https://github.com}"
repo_url="${server}/${repo}"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The directory handed to `gh` is not always the directory handed to `cut`.
# Under Git Bash on Windows, gh.exe is a native binary, so MSYS rewrites the
# POSIX path it is given -- and gets it wrong here: `gh release download`
# reports success having written the asset somewhere this script never looks,
# which reads exactly like an asset the release did not carry. That is the one
# failure this script must never confuse with a fact about the release, so the
# spelling is asked for rather than assumed. cygpath exists only where the
# question does.
gh_tmp="$tmp"
if command -v cygpath >/dev/null 2>&1; then
  gh_tmp=$(cygpath -w "$tmp")
fi

# hash_for <archive-with-extension>
#
# Prints the sha256 and returns 0. Prints nothing and returns 0 for an archive
# the release did not carry, so the caller decides whether absence is a fact or
# a failure. Returns 1 for everything else, and a caller that ignores that is a
# caller writing a formula out of an error message.
#
# Anything that came back and is not a sha256 is always a failure: a truncated
# download, an HTML error page saved to disk, or a checksum file that changed
# shape would each yield a formula Homebrew reports as a mismatched download
# rather than as a broken formula.
hash_for() {
  local archive="$1" hash

  if ! gh release download "v${version}" \
    --repo "$repo" \
    --pattern "${archive}.sha256" \
    --dir "$gh_tmp" \
    --clobber >/dev/null 2>&1; then
    # A release that is not there at all is not the same fact as an archive a
    # release did not carry, and only the second one is ever tolerated.
    if ! gh release view "v${version}" --repo "$repo" >/dev/null 2>&1; then
      echo "brew-formula.sh: ${repo} has no release v${version}." >&2
      echo "  a formula is derived from a tag: cut it, or pass a version that has one." >&2
      return 1
    fi
    return 0
  fi

  # `sha256sum` wrote the line, so it is "<hash>  <name>" and the hash is the
  # first space-separated field in either of its two output modes.
  hash=$(cut -d' ' -f1 <"${tmp}/${archive}.sha256")
  if ! printf '%s' "$hash" | grep -Eq '^[0-9a-f]{64}$'; then
    echo "brew-formula.sh: the .sha256 asset of ${archive} did not yield a sha256: '${hash}'" >&2
    echo "  check the asset at ${repo_url}/releases/tag/v${version}" >&2
    return 1
  fi
  printf '%s' "$hash"
}

# The three archives the formula covers. Windows is a release target too and is
# deliberately not one of them: Homebrew does not install there, and the Scoop
# bucket is that channel.
darwin_arm_archive="ank-${version}-aarch64-apple-darwin.tar.gz"
darwin_intel_archive="ank-${version}-x86_64-apple-darwin.tar.gz"
linux_intel_archive="ank-${version}-x86_64-unknown-linux-musl.tar.gz"

echo "deriving the formula for ${version} from ${repo_url}" >&2

# `|| exit 1` rather than leaning on `set -e`: a failing command substitution
# does trip it, but the mechanism is subtle enough that a later edit could
# quietly turn a hard failure into an empty string, which is the one value this
# script treats as normal.
darwin_arm_sha=$(hash_for "$darwin_arm_archive") || exit 1
darwin_intel_sha=$(hash_for "$darwin_intel_archive") || exit 1
linux_intel_sha=$(hash_for "$linux_intel_archive") || exit 1

missing=()
if [ -z "$darwin_arm_sha" ]; then missing+=("$darwin_arm_archive"); fi
if [ -z "$darwin_intel_sha" ]; then missing+=("$darwin_intel_archive"); fi
if [ -z "$linux_intel_sha" ]; then missing+=("$linux_intel_archive"); fi

if [ "${#missing[@]}" -gt 0 ]; then
  echo "v${version} carries no archive for:" >&2
  for m in "${missing[@]}"; do echo "  $m" >&2; done
  if [ "$require_all" -eq 1 ]; then
    echo >&2
    echo "a release builds every row of the matrix in .github/workflows/release.yml," >&2
    echo "so an archive missing from a release is a broken release and not a formula" >&2
    echo "this script may narrow. read the build jobs of the release run for v${version}." >&2
    exit 1
  fi
  echo "  omitted from the formula rather than pointed at: a URL that 404s fails" >&2
  echo "  on the user's machine, where nothing here can see it." >&2
fi

# A formula with no url at all is not a formula, and the diagnostic Homebrew
# prints for one is worse than saying so here.
if [ "${#missing[@]}" -eq 3 ]; then
  echo "brew-formula.sh: v${version} carries none of the archives the formula covers." >&2
  echo "  there is nothing to install from: read ${repo_url}/releases/tag/v${version}" >&2
  exit 1
fi

# The note is part of the derived file on purpose. `brew cat` is where a user
# lands after an install that found nothing for their machine, and a sentence
# naming the release is the answer to the question they arrived with.
coverage_note=""
if [ "${#missing[@]}" -gt 0 ]; then
  coverage_note="
#
# v${version} published no archive for:"
  for m in "${missing[@]}"; do
    coverage_note="${coverage_note}
#   ${m}"
  done
  coverage_note="${coverage_note}
# so this formula carries no block for it. That is a row of the release matrix
# and not a decision here: the first tag cut with that row restores the block,
# derived, with no edit to this file or to the script that writes it."
fi

# platform_block <indent> <archive> <sha>
#
# Empty for an archive the release did not carry, which leaves the omission
# visible in the diff rather than buried inside a conditional.
platform_block() {
  local indent="$1" archive="$2" sha="$3"
  if [ -z "$sha" ]; then
    return 0
  fi
  printf '%surl "%s/releases/download/v%s/%s"\n' "$indent" "$repo_url" "$version" "$archive"
  printf '%ssha256 "%s"' "$indent" "$sha"
}

macos_arm=$(platform_block '      ' "$darwin_arm_archive" "$darwin_arm_sha")
macos_intel=$(platform_block '      ' "$darwin_intel_archive" "$darwin_intel_sha")
linux_intel=$(platform_block '      ' "$linux_intel_archive" "$linux_intel_sha")

macos_section=""
if [ -n "$macos_arm" ]; then
  macos_section="    on_arm do
${macos_arm}
    end"
fi
if [ -n "$macos_intel" ]; then
  if [ -n "$macos_section" ]; then
    macos_section="${macos_section}

"
  fi
  macos_section="${macos_section}    on_intel do
${macos_intel}
    end"
fi

os_blocks=""
if [ -n "$macos_section" ]; then
  os_blocks="  on_macos do
${macos_section}
  end"
fi
if [ -n "$linux_intel" ]; then
  if [ -n "$os_blocks" ]; then
    os_blocks="${os_blocks}

"
  fi
  os_blocks="${os_blocks}  on_linux do
    on_intel do
${linux_intel}
    end
  end"
fi

mkdir -p "$(dirname "$output")"

# Written at column 0 because a heredoc keeps every byte it is given, and the
# indentation below is Ruby's rather than the shell's.
cat >"$output" <<RUBY
# typed: false
# frozen_string_literal: true

# Derived from the GitHub release by .github/scripts/brew-formula.sh, which
# .github/workflows/publish-brew.yml runs on every pull request and asserts this
# file is exactly what deriving it produces. A hand edit turns that job red
# instead of failing on somebody's machine -- in particular a sha256, which is
# read from the .sha256 asset the release publishes and is never typed.
#
# This repository is its own tap, and there is no satellite to keep in step
# (ADR-782a3556cf2d):
#
#   brew tap ${repo} ${repo_url}
#   brew install ${repo}/ank
#
# It is a tap and not homebrew-core because core's gate is notability -- of the
# order of 75 stars or 30 forks, with a track record -- and this repository has
# 1 star, 0 forks and three weeks of history. That is their documented door and
# not a judgement about the tool. The move costs a change of source and not a
# rewrite: the platform blocks below become one url and sha256 on the source
# tarball plus \`system "cargo", "install", *std_cargo_args\`, and the rest of
# this file stands.${coverage_note}
class Ank < Formula
  desc "Tasks and architecture decisions in your repo, behind one CLI agents can call"
  homepage "${repo_url}"
  version "${version}"
  license "GPL-3.0-only"

${os_blocks}

  # The archive wraps its contents in a directory named after itself and
  # Homebrew stages inside it, so these are the names as packaged by the
  # \`package\` step of release.yml. SKILL.md travels with the binary because a
  # channel shipping ank and not what it teaches has shipped half of it.
  def install
    bin.install "ank"
    doc.install "README.md"
    pkgshare.install "SKILL.md"
  end

  # \`ank --version\` prints "ank <version> (<commit>, skill <hash>)", so the
  # assertion is on the version token: pinning the whole line would make every
  # commit of a release a change to this file.
  test do
    assert_match(/\A\s*ank\s+#{Regexp.escape(version.to_s)}(\s|\z)/,
                 shell_output("#{bin}/ank --version"))
  end

  # For the move to core, and for anyone watching the tap: the newest tag is
  # what this formula tracks.
  livecheck do
    url :stable
    strategy :github_latest
  end
end
RUBY

echo "wrote ${output}" >&2
