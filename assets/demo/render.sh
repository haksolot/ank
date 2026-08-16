#!/usr/bin/env bash
#
# Renders assets/demo/demo.cast to the light and dark images the README shows.
#
#   render.sh <cast> <out-dir>
#
# **GIF and not SVG, and that was measured rather than preferred.** svg-term
# renders a cast as every frame side by side in one long strip, slid by a CSS
# animation. That animation runs when the SVG is inlined into a page and does
# not run when the same file is loaded through an `<img>` -- verified both ways
# in Chrome, with `prefers-reduced-motion` off. A README can only embed an
# image, so the SVG showed frame zero for ever: an empty terminal. A GIF
# animates everywhere an image does.
#
# agg takes the palette as sixteen hex values, so Catppuccin is exact here
# rather than substituted after the fact, and play.sh keeps to the eight ANSI
# colours so the theme reaches every pixel on screen.
set -euo pipefail

cast="${1:?usage: render.sh <cast> <out-dir>}"
out="${2:?usage: render.sh <cast> <out-dir>}"

agg="$(command -v agg || echo "$HOME/.cargo/bin/agg")"
if [ ! -x "$agg" ]; then
  echo "render.sh: agg is not installed." >&2
  echo "  cargo install --locked --git https://github.com/asciinema/agg" >&2
  exit 9
fi

# bg, fg, then the eight ANSI colours. Catppuccin Latte and Mocha.
LATTE="eff1f5,4c4f69,5c5f77,d20f39,40a02b,df8e1d,1e66f5,ea76cb,179299,acb0be"
MOCHA="1e1e2e,cdd6f4,45475a,f38ba8,a6e3a1,f9e2af,89b4fa,f5c2e7,94e2d5,bac2de"

# --idle-time-limit above the longest pause play.sh takes, or the beat the whole
# recording exists for gets cut to five seconds by a default.
render() {
  "$agg" --theme "$1" --idle-time-limit 10 --fps-cap 10 --font-size 15 \
    "$cast" "$2" 2>/dev/null
  printf '%s  %s\n' "$(du -h "$2" | cut -f1)" "$2" >&2
}

mkdir -p "$out"
render "$LATTE" "${out}/demo.gif"
render "$MOCHA" "${out}/demo-dark.gif"
