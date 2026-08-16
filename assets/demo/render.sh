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

# agg already asks for JetBrains Mono first and falls back silently through
# DejaVu and Liberation when it is not installed -- which is what the first
# render did, and it showed. The fallback is a worse-looking image and nothing
# says so, so this refuses instead of producing one quietly.
FONTS="${ANK_DEMO_FONT_DIR:-$HOME/fonts}"
if [ ! -d "$FONTS" ] || ! ls "$FONTS"/JetBrainsMono-*.ttf >/dev/null 2>&1; then
  echo "render.sh: JetBrains Mono is not in ${FONTS}." >&2
  echo "  agg would fall back to another monospace font without saying so." >&2
  echo "" >&2
  echo "  mkdir -p ${FONTS} && cd ${FONTS} \\" >&2
  echo "    && curl -fsSLO https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip \\" >&2
  echo "    && unzip -oj JetBrainsMono-2.304.zip 'fonts/ttf/*.ttf' && rm JetBrainsMono-2.304.zip" >&2
  exit 9
fi

# 22px rather than the terminal's own size: the README scales the image down to
# its column width, and a raster larger than it is displayed at survives that
# and a HiDPI screen. Rendered at 15 it was soft on both.
#
# --idle-time-limit above the longest pause play.sh takes, or the beat the whole
# recording exists for gets cut to five seconds by a default.
render() {
  "$agg" --theme "$1" --font-dir "$FONTS" --idle-time-limit 10 --fps-cap 10 \
    --font-size 22 "$cast" "$2" 2>/dev/null
  printf '%s  %s\n' "$(du -h "$2" | cut -f1)" "$2" >&2
}

mkdir -p "$out"
render "$LATTE" "${out}/demo.gif"
render "$MOCHA" "${out}/demo-dark.gif"
