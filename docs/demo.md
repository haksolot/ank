# Re-recording the demo

The README opens on a recording of the loop: `ank context`, `ank graph`, a claim,
the code change, and `ank done`. It is a function of three scripts in
`assets/demo/`, so a retake is a re-run and not a performance, and it goes stale
whenever the output it shows changes.

    assets/demo/setup.sh    builds the demo repository the recording is made in
    assets/demo/play.sh     what the recording plays, command by command
    assets/demo/render.sh   renders the cast to the two images the README shows
    assets/demo/demo.cast   the last recording, as asciicast v2

## What the scripts expect

They were written for one machine and say so in their first lines, rather than
taking arguments:

- the repository checked out at `~/ank`, and built with `cargo build --release`,
  because both `setup.sh` and `play.sh` run `~/ank/target/release/ank`;
- `~/demo` free: `setup.sh` deletes and rebuilds it, and writes a throwaway SSH
  signing key to `~/demo-sign` so the ratification in the recording is signed;
- [agg](https://github.com/asciinema/agg) on the `PATH` or in `~/.cargo/bin`,
  and JetBrains Mono in `~/fonts` or in the directory `ANK_DEMO_FONT_DIR` names.
  `render.sh` refuses at exit 9 and prints the install command when either is
  missing, rather than falling back to another font without saying so.

## Retaking it

    cargo build --release
    bash assets/demo/setup.sh
    asciinema rec --cols 100 --rows 34 -c "bash assets/demo/play.sh" assets/demo/demo.cast
    bash assets/demo/render.sh assets/demo/demo.cast assets

`setup.sh` ends by printing `ank graph` and `ank context` on the fresh demo
repository; read them before recording, because they are the state the
recording starts from. The cast is recorded at 100 by 34, the size the current
one declares. `render.sh` writes `assets/demo.gif` in Catppuccin Latte and
`assets/demo-dark.gif` in Mocha, which the README picks between with
`prefers-color-scheme`.

**GIF and not SVG, and that was measured.** An SVG rendering of a cast animates
when the SVG is inlined into a page and not when it is loaded through an
`<img>`, which is the only way a README can embed it, so it showed an empty
terminal. `play.sh` keeps to the eight ANSI colours for the same kind of
reason: a 256-colour escape would come out of the terminal's palette instead of
the theme `render.sh` hands agg.

Commit the cast and both images together, and update the image's `alt` text in
the README if what the recording shows changed.
