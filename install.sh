#!/bin/sh
#
# Install ank from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh -s -- --version v0.2.0
#
# One executable lands: `ank`. The protocol surface and the watcher are verbs of
# it (ADR-1ea31c2f3c5a), so there is no second file for this script to place and
# no way for one to fail to arrive.
#
# `--version` reaches releases published before that was true, whose archive
# carries a second executable beside `ank`. Nothing here asks: the archive is
# unpacked, `ank` is taken out of it, and whatever else the directory holds goes
# with the temporary directory. So an old release installs the one file this
# script promises, by the same code path a new one does -- an installer that
# only worked on releases that do not exist yet would not be a working
# installer.
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
# script draws for a human and everything it asks one, and leaves only the
# lines a machine reads. It is absent from the list above on purpose: the
# welcome is drawn before the first request goes out and the offer comes after
# the binary is on disk and verified, so neither is on the path to any of them
# -- a flag able to change one of these five codes would be a flag that made
# the install depend on it.
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

# Colour and the marker, behind the gate the logo is drawn behind. Empty until
# ui_enable runs, which is what keeps ADR-5fbd99bf6fd5's promise to a caller
# with nobody at the end of it: a pipe reads the bytes it read before this file
# learned about either.
#
# Every call site below spells the sequences out rather than asking a function
# for them, so a line that is half plain and half painted is one string and not
# a concatenation of three -- and the plain reading of it, with every variable
# empty, is the line that used to be there.
ui_off="" ui_bold="" ui_dim="" ui_cyan="" ui_green=""
ui_pad="" ui_tick=""

# `ok <line>`: a step that finished. The marker and the indent are drawn for a
# person and for nobody else -- what a pipe reads is the sentence alone, which
# is why the two live in variables rather than in the format string.
ok() {
  say "${ui_pad}${ui_green}${ui_tick}${ui_off}$*"
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

# The logo, from assets/ank.svg read as pixels: twenty-four columns by ten
# rows, one cell per pixel column and one row per pair of pixel rows, which is
# the ratio that makes a terminal cell square. That file is the reference for
# the shape and nothing here reads it: an installer that fetches a logo before
# it fetches the binary is an installer with a second way to fail before doing
# anything useful, so the shape is bytes in this file.
#
# U+2588 where the locale says UTF-8, `#` where it does not, decided once in
# logo_charset. The hedge is the one the shebang makes -- this runs on busybox
# under a locale nobody chose, and a logo that arrives as question marks is
# worse than no logo at all -- but a locale that announces UTF-8 is a locale
# that can draw a block, and the block is the whole difference between a logo
# and a fence of hashes.
#
# \033 and not \e: the octal escape is the one POSIX printf guarantees, and \e
# is a bashism in a file that has none.
logo_esc=$(printf '\033')

# Eleven spaces: columns 0 to 10, which is what stands to the left of the stem
# on a row whose leg has not grown yet. Every such row is the same width, so
# the padding is a constant rather than something measured -- ${#s} counts
# bytes, and the moment the cell is U+2588 that is three times the answer.
logo_pad11="           "

# The cell, and the three runs the shape is made of, built once. A frame is
# drawn seventeen times and must not repeat this work inside the loop.
logo_charset() {
  case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *UTF-8* | *utf-8* | *UTF8* | *utf8*) logo_cell='█' ;;
    *) logo_cell='#' ;;
  esac
  logo_b2=$(logo_run 2)
  logo_b4=$(logo_run 4)
  logo_b14=$(logo_run 14)
}

logo_run() {
  logo_run_n=$1
  logo_run_s=""
  while [ "$logo_run_n" -gt 0 ]; do
    logo_run_s="${logo_run_s}${logo_cell}"
    logo_run_n=$((logo_run_n - 1))
  done
  printf '%s' "$logo_run_s"
}

# The mark carries no colour of its own. assets/ank.svg is #1f2328 and
# assets/ank-dark.svg is #e6edf3 -- the same shape, black on a light ground and
# white on a dark one -- and \033[39m is how a terminal spells that: the
# foreground the reader already chose. A hue picked here would be a third logo
# neither asset agrees with, and would be the one thing on the screen that does
# not belong to the person running the installer.
#
# So the animation is built on the one axis that survives not knowing the
# background: dim against normal. Brighter does not survive it -- bold on a
# glyph that is already a solid block does nothing at all in a terminal that
# does not render bold as bright, and white on a light ground is invisible. A
# part therefore arrives dim and settles, and the shape blinks by going dim,
# which reads the same on either ground.
#
# NO_COLOR empties all four rather than branching at each of the places they are
# used, so a run without colour draws the same shape through the same code, on
# the growth alone.
logo_palette() {
  if [ -n "${NO_COLOR:-}" ]; then
    logo_lit="" logo_soft="" logo_name="" logo_off=""
    return
  fi
  logo_lit="${logo_esc}[39m"
  logo_soft="${logo_esc}[2;39m"
  logo_name="${logo_esc}[1m"
  logo_off="${logo_esc}[0m"
}

# The lines the install prints after the logo, which are not the mark and may
# therefore carry a hue: bold for what landed, dim for what is incidental, cyan
# for a command the reader is meant to run, green for a step that finished. Five
# of the eight the binary allows itself, and the mark still uses none of them.
#
# The marker is U+2713 where the locale says
# UTF-8 and `- ` where it does not, on the reasoning logo_charset is written
# out for: a tick that arrives as a question mark is worse than no tick.
#
# The marker survives NO_COLOR, and that is the point of it being a character.
# Nothing here is carried by colour alone -- take every sequence away and the
# steps are still marked, and still say what they say.
ui_enable() {
  ui_pad="  "
  ui_tick="- "
  case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *UTF-8* | *utf-8* | *UTF8* | *utf8*) ui_tick="✓ " ;;
  esac
  [ -z "${NO_COLOR:-}" ] || return 0
  ui_off="${logo_esc}[0m"
  ui_bold="${logo_esc}[1m"
  ui_dim="${logo_esc}[2m"
  ui_cyan="${logo_esc}[36m"
  ui_green="${logo_esc}[32m"
}

# One of rows 6 to 9: `<pad><leg><gap><stem><gap><leg>`, where a part that has
# not grown yet gives up its columns to the eleven-space pad, so the parts that
# have stay where they are. The trailing pad is never drawn -- \033[K ends the
# line, and spaces that reach the right edge of a narrow window wrap it.
logo_band() {
  lb_legs=$1 lb_legs_at=$2 lb_stem=$3 lb_stem_at=$4
  lb_run=$5 lb_gap=$6 lb_legs_c=$7 lb_stem_c=$8

  if [ "$lb_legs" -ge "$lb_legs_at" ]; then
    lb_left="   ${lb_legs_c}${lb_run}${logo_off}${lb_gap}"
    lb_right="${lb_gap}${lb_legs_c}${lb_run}${logo_off}"
  else
    lb_left=$logo_pad11
    lb_right=""
  fi

  if [ "$lb_stem" -ge "$lb_stem_at" ]; then
    lb_mid="${lb_stem_c}${logo_b2}${logo_off}"
  elif [ -n "$lb_right" ]; then
    lb_mid="  "
  else
    lb_left="" lb_mid=""
  fi

  printf '%s%s%s' "$lb_left" "$lb_mid" "$lb_right"
}

# One frame, whole, in one write. Twelve separate writes to a terminal are
# twelve chances for a person to catch the logo half-drawn, and that flicker is
# what makes an animation look cheap.
#
# The arguments are the state of the build and not a frame number: base is on or
# off, legs and stem are how many of their rows have grown, loop is on or off,
# soft names the part still arriving this frame, name is the wordmark. A
# timeline written as states can be re-timed without recomputing which row is
# which.
#
# Every sequence is closed on the run that opened it. A frame that ends with one
# still open is a terminal somebody has to repair with `reset`.
logo_frame() {
  lf_base=$1 lf_legs=$2 lf_stem=$3 lf_loop=$4 lf_soft=$5 lf_name=$6

  lf_loop_c=$logo_lit lf_stem_c=$logo_lit lf_legs_c=$logo_lit lf_base_c=$logo_lit
  case $lf_soft in
    loop) lf_loop_c=$logo_soft ;;
    stem) lf_stem_c=$logo_soft ;;
    legs) lf_legs_c=$logo_soft ;;
    base) lf_base_c=$logo_soft ;;
    all)
      lf_loop_c=$logo_soft lf_stem_c=$logo_soft
      lf_legs_c=$logo_soft lf_base_c=$logo_soft
      ;;
  esac

  # Rows 1 and 4, then 2 and 3: the loop is columns 10-13, then 8-9 with 14-15.
  if [ "$lf_loop" = 1 ]; then
    lf_r1="          ${lf_loop_c}${logo_b4}${logo_off}"
    lf_r2="        ${lf_loop_c}${logo_b2}${logo_off}    ${lf_loop_c}${logo_b2}${logo_off}"
  else
    lf_r1="" lf_r2=""
  fi

  # Row 5, the stem alone, the last of its five rows to grow.
  if [ "$lf_stem" -ge 5 ]; then
    lf_r5="           ${lf_stem_c}${logo_b2}${logo_off}"
  else
    lf_r5=""
  fi

  # Rows 6-9 carry a leg on each side and the stem between them, and the three
  # arrive at different moments.
  lf_r6=$(logo_band "$lf_legs" 4 "$lf_stem" 4 "$logo_b4" "    " "$lf_legs_c" "$lf_stem_c")
  lf_r7=$(logo_band "$lf_legs" 3 "$lf_stem" 3 "$logo_b2" "      " "$lf_legs_c" "$lf_stem_c")
  lf_r8=$(logo_band "$lf_legs" 2 "$lf_stem" 2 "$logo_b2" "      " "$lf_legs_c" "$lf_stem_c")
  lf_r9=$(logo_band "$lf_legs" 1 "$lf_stem" 1 "$logo_b2" "      " "$lf_legs_c" "$lf_stem_c")

  # Row 10, the base, columns 5-18.
  if [ "$lf_base" = 1 ]; then
    lf_r10="     ${lf_base_c}${logo_b14}${logo_off}"
  else
    lf_r10=""
  fi

  if [ "$lf_name" = 1 ]; then
    lf_r12="          ${logo_name}ank${logo_off}"
  else
    lf_r12=""
  fi

  # \033[K clears what the previous frame left to the right of each line, which
  # is what lets a row shrink. Twelve lines every time, including the empty
  # ones: the redraw moves back over a fixed block, and a frame that were
  # shorter would leave the tail of the one before it on the screen.
  printf '%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n%s\033[K\n\033[K\n%s\033[K\n' \
    "$lf_r1" "$lf_r2" "$lf_r2" "$lf_r1" "$lf_r5" \
    "$lf_r6" "$lf_r7" "$lf_r8" "$lf_r9" "$lf_r10" "$lf_r12" >&2
}

# A frame, its delay, and the move back over the block it drew, so the caller's
# timeline reads as states and never as cursor arithmetic. The last frame is
# drawn by the caller instead, because it is the one that has to stay.
logo_step() {
  logo_frame "$1" "$2" "$3" "$4" "$5" 0
  [ -z "$logo_delay" ] || sleep "$logo_delay"
  printf '\033[12A' >&2
}

# Bottom up: the base arrives dim and settles, the legs and then the stem grow
# out of it a row at a time, the loop crowns them, and the whole shape blinks
# twice before the name appears. That last beat is what the animation exists for
# -- it costs the time it takes to read the name of the tool and not a second
# more.
draw_logo() {
  logo_charset
  logo_palette

  # POSIX sleep takes an integer, and every sleep this script is likely to meet
  # -- coreutils, BSD, busybox built with the fancy option -- takes a fraction.
  # The ones that do not are told apart by asking rather than by guessing:
  # without a delay the frames still draw, in the order they draw, as fast as
  # the terminal will take them.
  logo_fast=0.045 logo_beat=0.10 logo_hold=0.45
  sleep 0.01 2>/dev/null || { logo_fast="" logo_beat="" logo_hold=""; }

  # The cursor would otherwise blink in the middle of the drawing. Restored on
  # the way out and on the two signals a human sends, because a terminal left
  # with an invisible cursor is a terminal somebody has to repair with `reset`.
  # The reset travels with it: a ^C between the opening of a colour and its
  # close would otherwise leave the shell prompt dim.
  trap 'printf "%s[0m%s[?25h\n" "$logo_esc" "$logo_esc" >&2; exit 130' INT
  trap 'printf "%s[0m%s[?25h\n" "$logo_esc" "$logo_esc" >&2; exit 143' TERM
  printf '\033[?25l' >&2

  # The two parts that appear whole -- the base and the loop -- come in dim and
  # settle. The two that grow do not: growing a row at a time is already motion,
  # and dimming it as well would say the same thing twice.
  #
  #                base legs stem loop soft
  logo_delay=$logo_beat
  logo_step         1    0    0    0   base
  logo_step         1    0    0    0   none
  logo_delay=$logo_fast
  logo_step         1    1    0    0   none
  logo_step         1    2    0    0   none
  logo_step         1    3    0    0   none
  logo_step         1    4    0    0   none
  logo_step         1    4    1    0   none
  logo_step         1    4    2    0   none
  logo_step         1    4    3    0   none
  logo_step         1    4    4    0   none
  logo_step         1    4    5    0   none
  logo_delay=$logo_beat
  logo_step         1    4    5    1   loop
  logo_step         1    4    5    1   none
  logo_delay=$logo_fast
  logo_step         1    4    5    1   all
  logo_step         1    4    5    1   none
  logo_step         1    4    5    1   all

  # The name, on the frame that stays. No move back after it, so what the
  # install says next begins under the logo rather than over it.
  logo_frame 1 4 5 1 none 1
  [ -z "$logo_hold" ] || sleep "$logo_hold"

  # The cursor comes back first and the blank line after it, so the last byte
  # this function writes is a newline: whatever the install says next starts on
  # a line of its own rather than behind an escape sequence.
  printf '\033[?25h\n' >&2
  trap - INT TERM
}

# ADR-5fbd99bf6fd5 read as an absence: where no human is looking, this script
# draws nothing at all and asks nothing at all.
#
# Both streams are tested and not only one. Under `curl ... | sh` stdin is the
# script and both of the others are the terminal, which is the case that must
# animate; under `sh install.sh > install.log` stderr is still a terminal, and
# a log file full of cursor movements is exactly the noise this must never
# produce. /dev/tty is the third test and the one the decision names: no
# controlling terminal means a provisioning script, a Dockerfile or a runner,
# whatever the streams happen to say.
#
# The logo and the offer read this same answer, which is what makes
# --no-welcome and an interactive run that declined everything leave the same
# machine behind: one predicate, so there is no second gate to disagree with
# this one.
human_at_terminal() {
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
}

# The width is about the block of art and about nothing else, so it is here and
# not above: a window too narrow to hold the logo is still a window with a
# person in front of it, and a question fits in twenty columns.
welcome_wanted() {
  human_at_terminal || return 1
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

One executable lands in the install directory: ank. The protocol surface and
the watcher are verbs of it -- ank mcp, ank watch -- so there is nothing
further to fetch and nothing further to configure a client against.

usage:
  install.sh [--version <version>] [--dir <path>]

  curl -fsSL ${raw_url} | sh
  curl -fsSL ${raw_url} | sh -s -- --version v0.2.0

options:
  --version <version>  install this release instead of the latest one;
                       "v0.2.0" and "0.2.0" both work
  --dir <path>         install into <path> instead of \$HOME/.local/bin
  --no-welcome         draw nothing and ask nothing; install exactly what an
                       interactive run that declined every offer installs
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

the two questions:
  With a terminal attached, once ank is installed and verified, this asks
  two things and nothing else. The first offers to run:
    npx skills add ${repo}
  which teaches an agent how to use ank. The second offers to print three
  prompts that adopt ank in a repository that already has code and no
  .ank; they are in docs/getting-started.md too, and printing them writes
  nothing anywhere.
  Enter accepts each, declining either does nothing at all, and nothing
  either does can change any of the codes below.

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
# The colour is turned on by the presence of a human and not by the width of
# their window: a terminal too narrow for the logo is still a terminal, and the
# sentence it is too narrow for is the one nobody wrote yet.
if human_at_terminal; then
  ui_enable
fi
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

say "${ui_pad}${ui_bold}ank ${tag}${ui_off}  ${ui_dim}${target}${ui_off}"

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

ok "checksum ok  ${ui_dim}${actual}${ui_off}"

tar xzf "${tmp}/${archive}" -C "$tmp" ||
  die 3 "could not unpack ${archive}." "" "Nothing was installed."

# The layout release.yml packages: one directory named after the archive,
# carrying the executable beside README.md, LICENSE and SKILL.md.
#
# `ank` is required and the rest of the directory is not read at all. An
# archive published before ADR-1ea31c2f3c5a carries a second executable there,
# and the answer to it is the same as the answer to README.md: it is not what
# was asked for, so it is not moved anywhere. A check that refused an archive
# holding more than this would refuse every release published so far.
unpacked="${tmp}/ank-${bare}-${target}"
binary="${unpacked}/ank"
if [ ! -f "$binary" ]; then
  die 3 "${archive} does not contain ank where this script expected it." \
    "" \
    "  looked for:  ank-${bare}-${target}/ank" \
    "" \
    "Nothing was installed."
fi

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

installed_version=$("${install_dir}/ank" --version 2>/dev/null | head -1) ||
  installed_version=""

say ""
ok "installed  ${ui_bold}${install_dir}/ank${ui_off}"
if [ -n "$installed_version" ]; then
  say "${ui_pad}${ui_pad}           ${ui_dim}${installed_version}${ui_off}"
fi

# The last way left to leave a caller without a working `ank`: a binary in a
# directory nothing looks in. Naming the line to add is the difference between
# an install that worked and an install that appears not to have run.
case ":${PATH}:" in
  *":${install_dir}:"*)
    say ""
    say "${ui_pad}${install_dir} is on your PATH. Run: ${ui_cyan}ank help${ui_off}"
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
    say "${ui_pad}${install_dir} is not on your PATH, so \`ank\` is not a command yet."
    say "${ui_pad}Add this line to ${ui_bold}${path_file}${ui_off}:"
    say ""
    say "${ui_pad}  ${ui_cyan}${path_line}${ui_off}"
    say ""
    say "${ui_pad}then open a new shell, or run that line now in this one."
    ;;
esac

# --------------------------------------------------------------------------
# The skills
# --------------------------------------------------------------------------

# ADR-5fbd99bf6fd5's offer, and the last thing this script does. Everything
# above has already happened: the binary is on disk, verified, reported, and
# the PATH advice given. That ordering is the decision rather than a layout --
# an installation that stops to ask something is an installation that can be
# abandoned half-done, and half-done is the worst state for a tool whose next
# action is `ank context`.
#
# `npx skills add <owner>/ank` is what skill/SKILL.md already teaches, and it
# serves every agent the skills CLI knows about rather than one. An installer
# that learned where each of them keeps its skills is an installer that goes
# stale silently, so this one hands that work to the tool whose job it is.
offer_skills() {
  human_at_terminal || return 0

  say ""
  say "${ui_pad}The skills teach an agent how to use ank: the contract, and one policy"
  say "${ui_pad}per activity. They install through the skills CLI, which puts them where"
  say "${ui_pad}each agent looks."
  say ""
  say "${ui_pad}  ${ui_cyan}npx skills add ${repo}${ui_off}"
  say ""

  # /dev/tty and nowhere else, which is the trap ADR-5fbd99bf6fd5 exists to
  # name. Under `curl ... | sh` standard input is this script: a plain `read`
  # would consume the rest of the file and execute none of it, and it would do
  # so only on the route people actually use, having worked in every local test
  # where the script was run from a file.
  printf '%s%sInstall them now?%s [Y/n] ' "$ui_pad" "$ui_bold" "$ui_off" >&2
  if ! IFS= read -r offer_answer < /dev/tty; then
    # End of input rather than an answer. Nothing was typed, so nothing is
    # assumed, and the newline is ours because no Enter was pressed to echo
    # one.
    say ""
    return 0
  fi

  # Enter is yes and everything unrecognised is no, in that order: a default
  # the criterion names, and a decline for anything else because asking twice
  # is asking twice.
  case "$offer_answer" in
    "" | y | Y | yes | Yes | YES) : ;;
    *) return 0 ;;
  esac

  if ! have node; then
    say ""
    say "${ui_pad}  ${ui_cyan}npx skills add ${repo}${ui_off}"
    say "node is not on PATH, so that was not run."
    return 0
  fi

  say ""

  # Two redirections, each doing something this cannot work without.
  #
  # `< /dev/tty` for the reason the prompt above reads from there: standard
  # input is still this script, and npx asks its own question on a cold cache
  # -- "Ok to proceed?" -- which it would answer with the next lines of this
  # file. npm_config_yes is `npx --yes` spelled as the environment, so that
  # question is not asked at all: the person already answered it, once, above.
  # It is set for the child and not exported, which is what keeps it out of
  # every other command here.
  #
  # `>&2` for the reason `say` writes there: the interesting use of this script
  # is `curl ... | sh`, and npx's output on stdout would land in whatever the
  # caller was reading. A redirection and not a pipe, so nothing is buffered
  # and the terminal shows npx working while it works.
  offer_code=0
  npm_config_yes=1 npx skills add "$repo" < /dev/tty >&2 || offer_code=$?

  if [ "$offer_code" -eq 0 ]; then
    say ""
    ok "the skills are installed"
  else
    say ""
    say "npx skills add ${repo} exited ${offer_code}, so the skills are not"
    say "installed. ank is, and it is exactly the ank this script installs when"
    say "nobody is asked anything at all."
    say ""
    say "Run that line again when you want them:"
    say "${ui_pad}  ${ui_cyan}npx skills add ${repo}${ui_off}"
  fi
}

# --------------------------------------------------------------------------
# Adopting ank where there is already code
# --------------------------------------------------------------------------

# ADR-5fbd99bf6fd5's second offer, and the last question this script asks.
# Installing ank is the easy half. The half nobody had written down is what to
# say to an agent so that a repository with two years of history acquires a
# corpus worth having, and the moment after an install is the one moment the
# person is certainly reading.
#
# Three prompts, because the adoption has three moments: state as ADRs what the
# code already decided, so the constraints that exist implicitly become
# readable; turn a list of intentions into tasks carrying a scope and a
# criterion; and check what came out. The first one is the one the reader
# judges the tool on, which is why it is first.
#
# The same three prose blocks live in install.ps1 and in docs/getting-started.md,
# and a test holds the three copies character for character
# (crates/ank-cli/tests/adopt.rs). Prose duplicated in three files diverges, and
# this is the prose where divergence is worst: an installer teaching a prompt
# the documentation has since corrected. The markers below are what the test
# reads; the block between them is the one to edit, and the other two follow.
# adopt-prompts:begin
adopt_walkthrough() {
  cat >&2 <<'ADOPT_EOF'
In a repository that already has code and no .ank, start with:

  ank init

Then paste these three into your agent, one at a time, and read what each
one produces before you send the next.

1. What the code already decided:

    Read this repository and write, as ank ADRs, the decisions its code
    has already made: the ones a newcomer would break without knowing
    they existed. One ADR per decision, each with a scope glob covering
    the files it binds and a constraint stated as a rule. Leave them
    proposed; I ratify them myself.

2. What is still owed:

    Read the TODOs, the open issues and the README of this repository,
    and turn what they promise into ank tasks. Give each one a scope
    glob and a done_criteria a test could settle, and use blocked_by
    only where a task genuinely waits on another.

3. What you now have:

    Run ank check and ank review here, then read every ADR back against
    the code its scope matches. Tell me which constraints the code
    already breaks and which scopes match no file, and change nothing
    until I have read your answer.

The same three are in docs/getting-started.md, which says what to expect
from each:

  https://github.com/haksolot/ank/blob/main/docs/getting-started.md
ADOPT_EOF
}
# adopt-prompts:end

# The second question, asked on the same terms as the first: only with a human
# at a terminal, from /dev/tty and nowhere else, with a default Enter accepts.
# Declining prints nothing -- not a shortened version, not a pointer to one.
# An offer that answers a no with half of a yes is an offer that was not really
# asked.
offer_adoption() {
  human_at_terminal || return 0

  say ""
  printf '%s%sPrint the three prompts that adopt ank in a repository you already have?%s [Y/n] ' \
    "$ui_pad" "$ui_bold" "$ui_off" >&2
  if ! IFS= read -r adopt_answer < /dev/tty; then
    # End of input rather than an answer. The newline is ours because no Enter
    # was pressed to echo one.
    say ""
    return 0
  fi

  case "$adopt_answer" in
    "" | y | Y | yes | Yes | YES) : ;;
    *) return 0 ;;
  esac

  say ""
  # To stderr, for the reason `say` writes there: the interesting use of this
  # script is `curl ... | sh`, and stdout belongs to whoever is reading it.
  adopt_walkthrough
  say ""
}

# `|| :` on each and not a bare call, and it is the whole guarantee in two
# lines. These are the last commands in the file, so the script's status is the
# status of the last of them: with `set -e` in force a single failure anywhere
# inside either would become the exit code of an install that has already
# succeeded. Called this way, the failure is tested rather than fatal, `set -e`
# is suspended for the duration, and the status this script leaves with is the
# status it had before the first question was asked.
offer_skills || :
offer_adoption || :
