#!/bin/sh
#
# Install ank from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh -s -- --version v0.2.0
#
# Two executables land and not one: `ank`, and `ank-mcp` beside it, the protocol
# surface a client speaks to. They come out of one archive and go into one
# directory, because ADR-e39a44f80e0e says no route carries one without the
# other. An archive published before ank-mcp existed carries only `ank`, and
# --version points this script at exactly those: such a release installs `ank`
# and is told what is missing, rather than becoming a route that stopped
# working.
#
# POSIX sh and no bashisms, deliberately. This is the channel for the Linux
# distribution that will never have a native package, and the smallest of those
# ship busybox ash with no bash at all. The binary is a static musl build, so
# one archive covers every distribution -- but only if the script that fetches
# it runs there too.
#
# Exit codes, so a caller can branch on the failure rather than on the message:
#
#   1  usage, or a directory that cannot be written
#   2  unsupported platform
#   3  the download failed, or the release does not carry this archive
#   4  the checksum did not match the one the release published
#   5  a tool this script needs is missing
#
# --no-welcome, or ANK_NO_WELCOME in the environment, turns off everything this
# script draws for a human and leaves only the lines a machine reads. It is
# absent from the list above on purpose: the welcome is drawn before the first
# request goes out, nothing in it can fail, and a flag that could change one of
# these five codes would be a flag that made the install depend on it.
#
set -eu

repo="haksolot/ank"
raw_url="https://raw.githubusercontent.com/${repo}/main/install.sh"
releases_url="https://github.com/${repo}/releases"
default_base_url="${releases_url}/download"

# --------------------------------------------------------------------------
# Output
# --------------------------------------------------------------------------

# Everything informational goes to stderr, because the interesting use of this
# script is `curl ... | sh` and the caller may well be reading stdout.
say() {
  printf '%s\n' "$*" >&2
}

# `die <code> <line>...`: the code carries the kind of failure, the lines carry
# what to do next. Nothing here ever ends in silence -- that is the one thing
# an install script cannot do, since the caller is otherwise left with no
# binary and no idea why.
die() {
  die_code=$1
  shift
  say "ank: $1"
  shift
  for die_line in "$@"; do
    say "$die_line"
  done
  exit "$die_code"
}

# --------------------------------------------------------------------------
# Welcome
# --------------------------------------------------------------------------

# The logo, at half the resolution of assets/ank.svg and drawn in ASCII. That
# file is the reference for the shape and nothing here reads it: an installer
# that fetches a logo before it fetches the binary is an installer with a
# second way to fail before doing anything useful, so the frames are bytes in
# this file. ASCII and not U+2588 for the reason the shebang is /bin/sh -- this
# runs on busybox, under a locale nobody chose, and a logo that arrives as
# question marks is worse than no logo at all.
#
# Twelve lines, always, including the empty ones. The animation redraws the
# same block in place, so a frame that were shorter would leave the tail of the
# one before it on the screen.
#
# \033 and not \e: the octal escape is the one POSIX printf guarantees, and \e
# is a bashism in a file that has none. \033[K clears what the previous frame
# left to the right of this line.
logo_line() {
  case $1 in
    1) logo_art='          ####' ;;
    2) logo_art='        ##    ##' ;;
    3) logo_art='        ##    ##' ;;
    4) logo_art='          ####' ;;
    5) logo_art='          ####' ;;
    6) logo_art='  ######  ####  ######' ;;
    7) logo_art='  ####    ####    ####' ;;
    8) logo_art='  ####    ####    ####' ;;
    9) logo_art='  ####    ####    ####' ;;
    10) logo_art='    ################' ;;
    12) logo_art='          ank' ;;
    *) logo_art='' ;;
  esac
  printf '%s\033[K\n' "$logo_art" >&2
}

# Bottom up: the base, then the stem, then the loop, which is the order the
# shape is built in and the order that leaves the whole of it standing at the
# end. Frame 11 adds the name, and that is the beat the animation exists for:
# it costs the time it takes to read the name of the tool and not a second
# more.
draw_logo() {
  # POSIX sleep takes an integer, and every sleep this script is likely to meet
  # -- coreutils, BSD, busybox built with the fancy option -- takes a fraction.
  # The ones that do not are told apart by asking rather than by guessing:
  # without a delay the frames still draw, in the order they draw, as fast as
  # the terminal will take them.
  frame_delay=0.06
  sleep 0.01 2>/dev/null || frame_delay=""

  # The cursor would otherwise blink in the middle of the drawing. Restored on
  # the way out and on the two signals a human sends, because a terminal left
  # with an invisible cursor is a terminal somebody has to repair with `reset`.
  trap 'printf "\033[?25h" >&2; exit 130' INT
  trap 'printf "\033[?25h" >&2; exit 143' TERM
  printf '\033[?25l' >&2

  frame=1
  while [ "$frame" -le 11 ]; do
    line=1
    while [ "$line" -le 12 ]; do
      if [ "$line" -le 10 ] && [ "$line" -ge $((11 - frame)) ]; then
        logo_line "$line"
      elif [ "$line" -eq 12 ] && [ "$frame" -eq 11 ]; then
        logo_line 12
      else
        logo_line 0
      fi
      line=$((line + 1))
    done
    if [ "$frame" -lt 11 ]; then
      [ -z "$frame_delay" ] || sleep "$frame_delay"
      printf '\033[12A' >&2
    fi
    frame=$((frame + 1))
  done

  # The cursor comes back first and the blank line after it, so the last byte
  # this function writes is a newline: whatever the install says next starts on
  # a line of its own rather than behind an escape sequence.
  printf '\033[?25h\n' >&2
  trap - INT TERM
}

# ADR-5fbd99bf6fd5 read as an absence: where no human is looking, this script
# draws nothing at all.
#
# Both streams are tested and not only one. Under `curl ... | sh` stdin is the
# script and both of the others are the terminal, which is the case that must
# animate; under `sh install.sh > install.log` stderr is still a terminal, and
# a log file full of cursor movements is exactly the noise this must never
# produce. /dev/tty is the third test and the one the decision names: no
# controlling terminal means a provisioning script, a Dockerfile or a runner,
# whatever the streams happen to say.
welcome_wanted() {
  [ "$no_welcome" = no ] || return 1
  [ -t 1 ] || return 1
  [ -t 2 ] || return 1
  # A runner sets this, and some runner images hand their steps a pty.
  [ -z "${CI:-}" ] || return 1
  # A terminal that says it cannot move a cursor is taken at its word.
  case "${TERM:-}" in
    "" | dumb) return 1 ;;
  esac
  # A subshell: a redirection that fails on a special builtin is fatal to the
  # shell itself, and this test exists to be answered no.
  ( : > /dev/tty ) 2>/dev/null || return 1
  # A window narrower than the art wraps every line, and a block that is taller
  # than twelve rows is a block the redraw moves back over the middle of. Asked
  # of tput, and only believed when it answers a number: a machine without it
  # is a machine this refuses to guess about in the direction of drawing.
  logo_cols=$(tput cols 2>/dev/null) || logo_cols=""
  case "$logo_cols" in
    "" | *[!0-9]*) : ;;
    *) [ "$logo_cols" -ge 24 ] || return 1 ;;
  esac
}

# --------------------------------------------------------------------------
# Platforms
# --------------------------------------------------------------------------

# The targets release.yml builds, minus Windows, which ships a .zip that this
# script has no business unpacking. Written once and read by both the refusal
# and the help text: a list that says one thing when it refuses and another
# when it is asked is worse than no list at all.
supported_lines() {
  cat <<'EOF'
  Linux  x86_64        x86_64-unknown-linux-musl
  Darwin arm64         aarch64-apple-darwin
  Darwin x86_64        x86_64-apple-darwin
EOF
}

usage() {
  cat >&2 <<EOF
install ank from a GitHub release

Two executables land in the install directory: ank, and ank-mcp beside it,
the protocol surface a client speaks to. A release published before ank-mcp
existed carries only ank; this installs it and says what is missing.

usage:
  install.sh [--version <version>] [--dir <path>]

  curl -fsSL ${raw_url} | sh
  curl -fsSL ${raw_url} | sh -s -- --version v0.2.0

options:
  --version <version>  install this release instead of the latest one;
                       "v0.2.0" and "0.2.0" both work
  --dir <path>         install into <path> instead of \$HOME/.local/bin
  --no-welcome         draw nothing; install exactly what an install with the
                       animation installs, and say the same things about it
  -h, --help           print this and exit

environment:
  ANK_VERSION          same as --version
  ANK_INSTALL_DIR      same as --dir
  ANK_NO_WELCOME       same as --no-welcome, for a caller that pipes this
                       script into sh and cannot pass an argument to it
  ANK_BASE_URL         where the archives are fetched from, for a mirror or a
                       staged release; requires --version, since only GitHub
                       can be asked which release is the latest

platforms:
$(supported_lines)

  Windows is published as a .zip and this script does not install it:
    ${releases_url}

exit codes:
  1 usage   2 unsupported platform   3 download   4 checksum   5 missing tool
EOF
}

# Sets \$target, or refuses naming the pair it saw. uname -m is normalised
# first, because one machine answers differently depending on the distribution:
# amd64 and x86_64 are one target, aarch64 and arm64 are another.
detect_target() {
  detect_os=$(uname -s 2>/dev/null) || detect_os=unknown
  detect_raw=$(uname -m 2>/dev/null) || detect_raw=unknown
  detect_arch=$detect_raw

  case "$detect_arch" in
    x86_64 | amd64) detect_arch=x86_64 ;;
    aarch64 | arm64) detect_arch=arm64 ;;
  esac

  case "${detect_os}/${detect_arch}" in
    Linux/x86_64) target=x86_64-unknown-linux-musl ;;
    Darwin/arm64) target=aarch64-apple-darwin ;;
    Darwin/x86_64) target=x86_64-apple-darwin ;;
    *)
      say "ank: no release archive for ${detect_os}/${detect_arch}."
      say ""
      say "uname reported: ${detect_os} ${detect_raw}"
      say ""
      say "The release carries these:"
      supported_lines >&2
      say ""
      say "Windows is published as a .zip and this script does not install it."
      say "Everything the release carries is listed at:"
      say "  ${releases_url}"
      say ""
      say "Nothing was installed."
      exit 2
      ;;
  esac
}

# --------------------------------------------------------------------------
# Tools
# --------------------------------------------------------------------------

have() {
  command -v "$1" >/dev/null 2>&1
}

# curl or wget, whichever is there. Both are missing often enough on a minimal
# container image that naming the two that would work is worth the lines.
pick_downloader() {
  if have curl; then
    downloader=curl
  elif have wget; then
    downloader=wget
  else
    die 5 "neither curl nor wget is installed, so nothing can be downloaded." \
      "" \
      "Install one of them and run this again:" \
      "  apt-get install -y curl     # debian, ubuntu" \
      "  apk add curl                # alpine" \
      "  dnf install -y curl         # fedora, rhel"
  fi
}

# sha256sum on Linux, shasum on macOS, openssl as the fallback that is on
# nearly everything else. If none of the three is present the script stops
# here rather than installing unverified: skipping the check is the failure
# this whole file is shaped to avoid, so it is not allowed to be the thing
# that degrades quietly.
pick_sha_tool() {
  if have sha256sum; then
    sha_tool=sha256sum
  elif have shasum; then
    sha_tool=shasum
  elif have openssl; then
    sha_tool=openssl
  else
    die 5 "no way to compute a SHA-256 on this machine." \
      "" \
      "The release publishes a .sha256 beside every archive and this script" \
      "refuses to unpack one it could not verify. Install any of these:" \
      "  sha256sum (coreutils), shasum (perl), openssl" \
      "" \
      "Nothing was installed."
  fi
}

sha256_of() {
  case "$sha_tool" in
    sha256sum) sha256sum "$1" | awk '{print $1}' ;;
    shasum) shasum -a 256 "$1" | awk '{print $1}' ;;
    openssl) openssl dgst -sha256 "$1" | awk '{print $NF}' ;;
  esac
}

lower() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

# --------------------------------------------------------------------------
# Downloading
# --------------------------------------------------------------------------

# `fetch <url> <path>`. Returns non-zero on failure and leaves the HTTP status
# in \$fetch_status when it has one, because "this release does not carry that
# archive" and "the network is down" are different messages and the 404 is
# what tells them apart.
fetch() {
  fetch_url=$1
  fetch_out=$2
  fetch_status=""

  if [ "$downloader" = curl ]; then
    # No -f: with it curl returns non-zero and the status is harder to read
    # back out. The status is checked here instead, which is the same
    # guarantee with a better message.
    fetch_status=$(curl -sSL --retry 3 --retry-delay 1 \
      -o "$fetch_out" -w '%{http_code}' "$fetch_url" 2>/dev/null) || fetch_status=""
    case "$fetch_status" in
      2??) return 0 ;;
      *) return 1 ;;
    esac
  else
    # wget does not hand the status back, so the caller falls through to the
    # general message. Parsing wget's stderr for it is not more reliable than
    # saying less.
    wget -q -O "$fetch_out" "$fetch_url" 2>/dev/null || return 1
    return 0
  fi
}

# The tag of the latest release, read off the redirect rather than through the
# API: /releases/latest redirects to /releases/tag/<tag>, which costs no rate
# limit, where api.github.com allows sixty unauthenticated calls an hour per
# address and a shared CI runner or an office NAT can be out of them before
# anyone types this.
resolve_latest() {
  if [ "$downloader" = curl ]; then
    resolved=$(curl -sSL -o /dev/null -w '%{url_effective}' \
      "${releases_url}/latest" 2>/dev/null) || resolved=""
    tag=${resolved##*/}
  else
    # wget does not report the URL it landed on, so the one place the API is
    # called is here, on the machine that has no curl.
    tag=$(wget -q -O - "https://api.github.com/repos/${repo}/releases/latest" 2>/dev/null |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      head -1) || tag=""
  fi

  case "$tag" in
    v*) : ;;
    *)
      die 3 "could not work out which release is the latest." \
        "" \
        "Name one instead:" \
        "  install.sh --version v0.2.0" \
        "" \
        "The releases are listed at:" \
        "  ${releases_url}"
      ;;
  esac
}

# --------------------------------------------------------------------------
# Arguments
# --------------------------------------------------------------------------

version=${ANK_VERSION:-}
install_dir=${ANK_INSTALL_DIR:-}
base_url=${ANK_BASE_URL:-}
no_welcome=no
[ -z "${ANK_NO_WELCOME:-}" ] || no_welcome=yes

while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      [ $# -ge 2 ] ||
        die 1 "--version needs a value." "" "  install.sh --version v0.2.0"
      version=$2
      shift 2
      ;;
    --version=*)
      version=${1#--version=}
      shift
      ;;
    --dir)
      [ $# -ge 2 ] ||
        die 1 "--dir needs a value." "" "  install.sh --dir \$HOME/.local/bin"
      install_dir=$2
      shift 2
      ;;
    --dir=*)
      install_dir=${1#--dir=}
      shift
      ;;
    --no-welcome)
      no_welcome=yes
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      die 1 "unknown option: $1" \
        "" \
        "Run it with --help to see what it takes:" \
        "  install.sh --help"
      ;;
  esac
done

if [ -n "$base_url" ]; then
  case "$base_url" in
    http://* | https://*) : ;;
    *)
      die 1 "ANK_BASE_URL must start with http:// or https://, got: ${base_url}" \
        "" \
        "Unset it to fetch from the GitHub release:" \
        "  unset ANK_BASE_URL"
      ;;
  esac
  if [ -z "$version" ]; then
    die 1 "ANK_BASE_URL is set, so the version has to be named." \
      "" \
      "Only GitHub can be asked which release is the latest; a mirror cannot." \
      "  install.sh --version v0.2.0"
  fi
fi

# --------------------------------------------------------------------------
# Install
# --------------------------------------------------------------------------

# Before anything is asked of the network, which is what "before the download
# starts" has to mean here: the first request this script makes is the redirect
# that resolves the latest tag, and it is below.
if welcome_wanted; then
  draw_logo
fi

pick_downloader
pick_sha_tool
detect_target

# The second form is BSD mktemp's, which wants a template where GNU's is happy
# without one. Both spellings rather than the one that works on the machine
# this was written on.
tmp=$(mktemp -d 2>/dev/null) || tmp=$(mktemp -d -t ank 2>/dev/null) || tmp=""
[ -n "$tmp" ] || die 5 "could not create a temporary directory."
# On the ordinary exit as well, so a refusal below leaves nothing behind.
trap 'rm -rf "$tmp"' EXIT
trap 'rm -rf "$tmp"; exit 130' INT
trap 'rm -rf "$tmp"; exit 143' TERM

if [ -n "$version" ]; then
  # "v0.2.0" and "0.2.0" are the same request. The tag carries the v and the
  # archive name does not, and making the caller know which is which is the
  # kind of detail a released script gets to absorb.
  tag=v${version#v}
else
  resolve_latest
fi
bare=${tag#v}

archive="ank-${bare}-${target}.tar.gz"
url="${base_url:-$default_base_url}/${tag}/${archive}"

say "ank ${tag}  ${target}"

# The checksum first: it is the smaller of the two files, so a release that
# does not carry this target says so before megabytes move. It is also the
# request that answers "is this platform in this release", which is a question
# the caller deserves answered rather than left as a stalled download.
if ! fetch "${url}.sha256" "${tmp}/sha256"; then
  if [ "$fetch_status" = 404 ]; then
    die 3 "${tag} does not carry an archive for this platform." \
      "" \
      "  looked for:  ${archive}" \
      "  at:          ${url}" \
      "" \
      "The platform is one this project builds for, so this is about the" \
      "release and not about the machine. What ${tag} carries is listed at:" \
      "  ${releases_url}/tag/${tag}" \
      "" \
      "Pick a release that carries it:" \
      "  install.sh --version <version>" \
      "" \
      "Nothing was installed."
  fi
  die 3 "could not download the checksum for ${archive}${fetch_status:+ (HTTP ${fetch_status})}." \
    "" \
    "  ${url}.sha256" \
    "" \
    "Nothing was installed."
fi

expected=$(lower "$(awk 'NR==1 {print $1}' "${tmp}/sha256")")

# A captive portal answering every request with a login page is a real way to
# get a .sha256 that parses into something. Sixty-four hex characters, or this
# was not the release answering.
case "$expected" in
  *[!0-9a-f]* | "") expected_ok=no ;;
  *) [ "${#expected}" -eq 64 ] && expected_ok=yes || expected_ok=no ;;
esac
if [ "$expected_ok" = no ]; then
  die 4 "the published checksum for ${archive} is not a SHA-256." \
    "" \
    "  ${url}.sha256" \
    "  read back:  ${expected:-<empty>}" \
    "" \
    "Something other than the release answered that request. Nothing was" \
    "unpacked and nothing was installed."
fi

if ! fetch "$url" "${tmp}/${archive}"; then
  die 3 "could not download ${archive}${fetch_status:+ (HTTP ${fetch_status})}." \
    "" \
    "  ${url}" \
    "" \
    "Nothing was installed."
fi

actual=$(lower "$(sha256_of "${tmp}/${archive}")")

# Before unpacking, and this is what the file is for. A script that downloads
# an executable over the network and runs it without checking the hash
# published beside it is a supply chain with a hole in the middle.
#
# What the check buys, stated honestly: it catches a truncated or corrupted
# download, a mirror serving the wrong file, and an archive that is not the
# one the release recorded. It is not a signature -- the hash comes from the
# same host as the archive -- which is why the default host is GitHub over TLS
# and why ANK_BASE_URL is documented as a mirror rather than as a default.
if [ "$expected" != "$actual" ]; then
  die 4 "checksum mismatch, refusing to unpack ${archive}." \
    "" \
    "  expected:   ${expected}" \
    "  actual:     ${actual}" \
    "  published:  ${url}.sha256" \
    "" \
    "The download does not match the hash the release published beside it." \
    "Nothing was unpacked and nothing was installed." \
    "" \
    "Retry once, in case the transfer was truncated. If it happens again, do" \
    "not install this archive, and say so:" \
    "  https://github.com/${repo}/security"
fi

say "checksum ok  ${actual}"

tar xzf "${tmp}/${archive}" -C "$tmp" ||
  die 3 "could not unpack ${archive}." "" "Nothing was installed."

# The layout release.yml packages: one directory named after the archive,
# carrying the two executables beside README.md, LICENSE and SKILL.md.
unpacked="${tmp}/ank-${bare}-${target}"
binary="${unpacked}/ank"
if [ ! -f "$binary" ]; then
  die 3 "${archive} does not contain ank where this script expected it." \
    "" \
    "  looked for:  ank-${bare}-${target}/ank" \
    "" \
    "Nothing was installed."
fi

# The other half of ADR-e39a44f80e0e: where a route places ank, it places
# ank-mcp next to it. Missing only from an archive older than the protocol
# surface itself, which is a release --version still reaches, so its absence is
# reported at the end rather than refused here.
mcp_binary="${unpacked}/ank-mcp"

if [ -z "$install_dir" ]; then
  [ -n "${HOME:-}" ] ||
    die 1 "HOME is not set, so there is no default install directory." \
      "" \
      "Name one:" \
      "  install.sh --dir /usr/local/bin"
  install_dir="${HOME}/.local/bin"
fi

mkdir -p "$install_dir" 2>/dev/null ||
  die 1 "could not create ${install_dir}." \
    "" \
    "Install somewhere writable, or run this with the rights to write there:" \
    "  install.sh --dir \$HOME/.local/bin"

[ -w "$install_dir" ] ||
  die 1 "${install_dir} is not writable." \
    "" \
    "Install somewhere writable, or run this with the rights to write there:" \
    "  install.sh --dir \$HOME/.local/bin"

chmod +x "$binary"
# A rename into place rather than a copy over the file: on Linux, replacing a
# binary that is currently running is fine, writing into it is not.
mv -f "$binary" "${install_dir}/ank" ||
  die 1 "could not write ${install_dir}/ank." "" "Nothing was installed."

# The same two operations in the same order, which is what gives the pair the
# same permissions rather than a second mechanism that hopes to: tar restored
# both files out of one archive under one umask, so their modes are equal
# before this line, and `chmod +x` adds to the mode it is applied to instead of
# imposing one. Reading ank's mode back and copying it onto ank-mcp would need
# two spellings of stat, GNU's and BSD's, to buy a guarantee already held.
if [ -f "$mcp_binary" ]; then
  chmod +x "$mcp_binary"
  mv -f "$mcp_binary" "${install_dir}/ank-mcp" ||
    die 1 "could not write ${install_dir}/ank-mcp." \
      "" \
      "ank is installed at ${install_dir}/ank. ank-mcp is not, so a client" \
      "has no protocol surface to reach until this is run again."
  mcp_installed=yes
else
  mcp_installed=no
fi

installed_version=$("${install_dir}/ank" --version 2>/dev/null | head -1) ||
  installed_version=""

mcp_version=""
if [ "$mcp_installed" = yes ]; then
  mcp_version=$("${install_dir}/ank-mcp" --version 2>/dev/null | head -1) ||
    mcp_version=""
fi

say ""
say "installed  ${install_dir}/ank"
if [ -n "$installed_version" ]; then
  say "           ${installed_version}"
fi
if [ "$mcp_installed" = yes ]; then
  say "           ${install_dir}/ank-mcp"
  if [ -n "$mcp_version" ]; then
    say "           ${mcp_version}"
  fi
else
  # Said rather than swallowed, and on stderr like everything else here. The
  # caller asked for ank and has ank, so this is not a failure; but a client
  # pointed at this directory will find no ank-mcp in it, and this is the only
  # place able to say why.
  say ""
  say "${archive} carries no ank-mcp, so only ank was installed."
  say "That archive predates the protocol surface. A release that carries both"
  say "installs both:"
  say "  curl -fsSL ${raw_url} | sh"
fi

# The last way left to leave a caller without a working `ank`: a binary in a
# directory nothing looks in. Naming the line to add is the difference between
# an install that worked and an install that appears not to have run.
case ":${PATH}:" in
  *":${install_dir}:"*)
    say ""
    say "${install_dir} is on your PATH. Run: ank help"
    ;;
  *)
    case "${SHELL:-}" in
      */fish)
        path_line="fish_add_path ${install_dir}"
        path_file="~/.config/fish/config.fish"
        ;;
      */zsh)
        path_line="export PATH=\"${install_dir}:\$PATH\""
        path_file="~/.zshrc"
        ;;
      */bash)
        path_line="export PATH=\"${install_dir}:\$PATH\""
        path_file="~/.bashrc"
        ;;
      *)
        path_line="export PATH=\"${install_dir}:\$PATH\""
        path_file="your shell's startup file"
        ;;
    esac
    say ""
    say "${install_dir} is not on your PATH, so \`ank\` is not a command yet."
    say "Add this line to ${path_file}:"
    say ""
    say "  ${path_line}"
    say ""
    say "then open a new shell, or run that line now in this one."
    ;;
esac
