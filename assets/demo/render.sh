#!/usr/bin/env bash
#
# Renders assets/demo.cast to the light and dark SVGs the README shows.
#
#   render.sh <cast> <out-dir>
#
# svg-term-cli takes `--term iterm2 --profile <file>` and, in the version this
# runs against, ignores it: a profile path that does not exist produces the same
# bytes as one that does, and two renders asked for opposite themes came back
# byte-identical. Measured, not assumed. So the palette is applied here instead,
# by substituting the eight colours svg-term actually emits.
#
# Eight is the whole palette because the recording uses eight: the background,
# the foreground, the three ANSI colours ank's output carries, the two the
# prompt uses, and the dim one. A ninth appearing in a future recording shows up
# as an unmapped colour rather than as a wrong one -- the check at the end is
# what makes that true.
set -euo pipefail

cast="${1:?usage: render.sh <cast> <out-dir>}"
out="${2:?usage: render.sh <cast> <out-dir>}"

# What svg-term emits, and what each one is.
declare -a FROM=(
  "#282d35" # background
  "#b9c0cb" # foreground
  "#dbab79" # yellow
  "#a8cc8c" # green
  "#71bef2" # blue
  "#d7afff" # mauve, the prompt
  "#8a8a8a" # grey, the prompt's $
  "#6f7683" # dim
)
declare -a MOCHA=(
  "#1e1e2e" "#cdd6f4" "#f9e2af" "#a6e3a1" "#89b4fa" "#cba6f7" "#7f849c" "#585b70"
)
declare -a LATTE=(
  "#eff1f5" "#4c4f69" "#df8e1d" "#40a02b" "#1e66f5" "#8839ef" "#9ca0b0" "#acb0be"
)

mkdir -p "$out"
base="${out}/.demo-base.svg"

npx --yes svg-term-cli --in "$cast" --out "$base" \
  --width 100 --height 34 --padding 14 >/dev/null 2>&1

# Every colour the base carries has to be one this script knows how to map. An
# unmapped one would survive into both themes unchanged, which is a colour that
# is right in one and wrong in the other, and nothing would say so.
mapfile -t seen < <(grep -o '#[0-9a-fA-F]\{6\}' "$base" | tr 'A-F' 'a-f' | sort -u)
for c in "${seen[@]}"; do
  known=no
  for f in "${FROM[@]}"; do [ "$c" = "$f" ] && known=yes; done
  if [ "$known" = no ]; then
    echo "render.sh: the recording carries an unmapped colour: ${c}" >&2
    echo "  add it to FROM, MOCHA and LATTE, or it is right in one theme only." >&2
    exit 1
  fi
done

emit() {
  local -n palette=$1
  local dest="$2" i
  cp "$base" "$dest"
  for i in "${!FROM[@]}"; do
    # A placeholder pass first, so a colour mapped onto another source colour is
    # not then remapped by the substitution that follows it.
    sed -i "s/${FROM[$i]}/@@${i}@@/gI" "$dest"
  done
  for i in "${!FROM[@]}"; do
    sed -i "s/@@${i}@@/${palette[$i]}/g" "$dest"
  done
  echo "wrote ${dest}" >&2
}

emit LATTE "${out}/demo.svg"
emit MOCHA "${out}/demo-dark.svg"
rm -f "$base"
