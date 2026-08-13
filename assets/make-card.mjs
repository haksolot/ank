// Generates assets/ank-card.png, the still the pi gallery renders on ank's card
// (TASK-72baa24eef8f).
//
// pi.image takes PNG, JPEG, GIF or WebP and not SVG, and SVG is all this
// repository draws. There is no rasteriser on the maintainer's machine and one
// asset does not justify pulling one in, so this reads the mark straight out of
// assets/ank-dark.svg and writes the PNG with node's built-in zlib. Nothing is
// added to the tree but this file.
//
// It is committed because a binary asset nobody can regenerate is a mystery in
// a repository that otherwise explains itself. It lives beside its output
// rather than under .github/scripts/ for two reasons: assets/** is this task's
// declared scope, and the generator reads the SVG next to it, so the mark and
// the card that derives from it stay in one place.
//
//   node assets/make-card.mjs           write assets/ank-card.png
//   node assets/make-card.mjs --check   regenerate and compare, exit 1 on drift
//
// The SVG is `shape-rendering="crispEdges"` over a 24 by 24 integer grid, and
// every subpath in it is an axis-aligned rectangle. Nearest-neighbour scaling of
// that is not an approximation of the mark, it is the mark: each source pixel
// becomes an exact square of output pixels. The wordmark and the tagline are
// drawn from a bitmap font in the same idiom, so the whole card is one grid.

import { deflateSync } from "node:zlib";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..");

// ---------------------------------------------------------------------------
// Reading the mark out of the SVG
// ---------------------------------------------------------------------------

/// Every axis-aligned rectangle a path's `d` attribute draws.
///
/// A general SVG path parser this is not: it walks M/m/L/l/H/h/V/v/Z/z, keeps
/// the bounding box of each subpath, and emits that box. That is exact only
/// because every subpath here is a rectangle, so `assertRectangle` checks the
/// assumption rather than trusting it -- a curve or a diagonal appearing in the
/// mark must stop this script, not be silently squared off.
function rectsOf(d) {
  const rects = [];
  let i = 0;
  let cmd = "";
  let cx = 0;
  let cy = 0;
  let sx = 0;
  let sy = 0;
  let points = [];

  const touch = (x, y) => points.push([x, y]);

  const flush = () => {
    if (points.length === 0) return;
    const xs = points.map((p) => p[0]);
    const ys = points.map((p) => p[1]);
    const box = {
      x: Math.min(...xs),
      y: Math.min(...ys),
      w: Math.max(...xs) - Math.min(...xs),
      h: Math.max(...ys) - Math.min(...ys),
    };
    assertRectangle(points, box);
    if (box.w > 0 && box.h > 0) rects.push(box);
    points = [];
  };

  const num = () => {
    while (i < d.length && /[\s,]/.test(d[i])) i++;
    let j = i;
    if (d[j] === "-" || d[j] === "+") j++;
    while (j < d.length && /[0-9.]/.test(d[j])) j++;
    if (j === i) throw new Error(`expected a number at offset ${i} of "${d}"`);
    const v = Number(d.slice(i, j));
    i = j;
    return v;
  };

  while (i < d.length) {
    const c = d[i];
    if (/[\s,]/.test(c)) {
      i++;
      continue;
    }
    if (/[A-Za-z]/.test(c)) {
      cmd = c;
      i++;
    }
    switch (cmd) {
      case "M":
        flush();
        cx = num();
        cy = num();
        sx = cx;
        sy = cy;
        touch(cx, cy);
        cmd = "L";
        break;
      case "m":
        flush();
        cx += num();
        cy += num();
        sx = cx;
        sy = cy;
        touch(cx, cy);
        cmd = "l";
        break;
      case "L":
        cx = num();
        cy = num();
        touch(cx, cy);
        break;
      case "l":
        cx += num();
        cy += num();
        touch(cx, cy);
        break;
      case "H":
        cx = num();
        touch(cx, cy);
        break;
      case "h":
        cx += num();
        touch(cx, cy);
        break;
      case "V":
        cy = num();
        touch(cx, cy);
        break;
      case "v":
        cy += num();
        touch(cx, cy);
        break;
      case "Z":
      case "z":
        cx = sx;
        cy = sy;
        flush();
        break;
      default:
        throw new Error(`unsupported path command "${cmd}" in "${d}"`);
    }
  }
  flush();
  return rects;
}

/// Every point of the subpath sits on a corner of its own bounding box.
///
/// True of an axis-aligned rectangle and of nothing else a path can draw, so it
/// is what makes the bounding box a faithful reading rather than a guess.
function assertRectangle(points, box) {
  for (const [x, y] of points) {
    const onX = x === box.x || x === box.x + box.w;
    const onY = y === box.y || y === box.y + box.h;
    if (!onX || !onY) {
      throw new Error(
        `the mark is no longer rectangles on a grid: (${x}, ${y}) is not a ` +
          `corner of ${JSON.stringify(box)}. Rasterise it with a real ` +
          `rasteriser rather than with this script.`,
      );
    }
  }
}

/// The mark: its grid size, its rectangles and the colour the SVG fills them.
function readMark(file) {
  const svg = readFileSync(join(repo, file), "utf8");
  const view = svg.match(/viewBox="0 0 (\d+) (\d+)"/);
  if (!view) throw new Error(`${file} declares no viewBox starting at 0 0`);
  const fill = svg.match(/fill="(#[0-9a-fA-F]{6})"/);
  if (!fill) throw new Error(`${file} declares no six-digit hex fill`);
  const rects = [...svg.matchAll(/\sd="([^"]+)"/g)].flatMap((m) =>
    rectsOf(m[1]),
  );
  if (rects.length === 0) throw new Error(`${file} draws no rectangle`);
  return {
    grid: { w: Number(view[1]), h: Number(view[2]) },
    rects,
    fill: fill[1],
  };
}

// ---------------------------------------------------------------------------
// The bitmap font
// ---------------------------------------------------------------------------

// 5 by 7, drawn on the same integer grid as the mark, so scaling it by a whole
// number lands on whole pixels exactly as the mark does. Uppercase because the
// tagline is set in caps; `ank` is lowercase because the wordmark is, and those
// three glyphs are the only lowercase the card needs.
const FONT = {
  A: [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
  B: ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
  C: [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."],
  D: ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
  E: ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
  F: ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
  G: [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".###."],
  H: ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
  I: ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"],
  J: ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
  K: ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
  L: ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
  M: ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"],
  N: ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
  O: [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
  P: ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
  Q: [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
  R: ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
  S: [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
  T: ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
  U: ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
  V: ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
  W: ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
  X: ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
  Y: ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
  Z: ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
  a: [".....", ".....", ".###.", "....#", ".####", "#...#", ".####"],
  n: [".....", ".....", "####.", "#...#", "#...#", "#...#", "#...#"],
  k: ["#....", "#....", "#..#.", "#.#..", "##...", "#.#..", "#..#."],
  ",": [".....", ".....", ".....", ".....", ".....", "..#..", ".#..."],
  ".": [".....", ".....", ".....", ".....", ".....", ".....", "..#.."],
  " ": [".....", ".....", ".....", ".....", ".....", ".....", "....."],
};

const GLYPH_W = 5;
const GLYPH_H = 7;

/// Width in pixels of `text` set at `scale`, one blank column between glyphs.
function textWidth(text, scale) {
  if (text.length === 0) return 0;
  return (text.length * (GLYPH_W + 1) - 1) * scale;
}

// ---------------------------------------------------------------------------
// The canvas
// ---------------------------------------------------------------------------

function rgb(hex) {
  return [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
}

class Canvas {
  constructor(w, h, background) {
    this.w = w;
    this.h = h;
    this.px = Buffer.alloc(w * h * 3);
    this.fill(0, 0, w, h, background);
  }

  fill(x, y, w, h, hex) {
    const [r, g, b] = rgb(hex);
    const x0 = Math.max(0, x);
    const y0 = Math.max(0, y);
    const x1 = Math.min(this.w, x + w);
    const y1 = Math.min(this.h, y + h);
    for (let py = y0; py < y1; py++) {
      let o = (py * this.w + x0) * 3;
      for (let px = x0; px < x1; px++) {
        this.px[o++] = r;
        this.px[o++] = g;
        this.px[o++] = b;
      }
    }
  }

  /// A hollow rectangle `t` pixels thick, drawn inward from the given box.
  frame(x, y, w, h, t, hex) {
    this.fill(x, y, w, t, hex);
    this.fill(x, y + h - t, w, t, hex);
    this.fill(x, y, t, h, hex);
    this.fill(x + w - t, y, t, h, hex);
  }

  /// The mark, each grid unit becoming a `scale` by `scale` square.
  mark(m, x, y, scale, hex) {
    for (const r of m.rects) {
      this.fill(x + r.x * scale, y + r.y * scale, r.w * scale, r.h * scale, hex);
    }
  }

  text(s, x, y, scale, hex) {
    let cx = x;
    for (const ch of s) {
      const glyph = FONT[ch];
      if (!glyph) throw new Error(`the font has no glyph for "${ch}"`);
      for (let row = 0; row < GLYPH_H; row++) {
        for (let col = 0; col < GLYPH_W; col++) {
          if (glyph[row][col] === "#") {
            this.fill(cx + col * scale, y + row * scale, scale, scale, hex);
          }
        }
      }
      cx += (GLYPH_W + 1) * scale;
    }
  }
}

// ---------------------------------------------------------------------------
// PNG
// ---------------------------------------------------------------------------

const CRC_TABLE = (() => {
  const t = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c;
  }
  return t;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

/// Truecolour, 8 bits a channel, no alpha: the card is opaque by design, so
/// whatever the gallery paints behind it never reaches the pixels.
function encodePng(canvas) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(canvas.w, 0);
  ihdr.writeUInt32BE(canvas.h, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // colour type: truecolour
  ihdr[10] = 0; // deflate
  ihdr[11] = 0; // adaptive filtering
  ihdr[12] = 0; // no interlace

  // Filter type 0 on every scanline. The image is flat blocks of colour, so
  // deflate already finds the runs; a filter would only get in the way.
  const stride = canvas.w * 3;
  const raw = Buffer.alloc((stride + 1) * canvas.h);
  for (let y = 0; y < canvas.h; y++) {
    raw[y * (stride + 1)] = 0;
    canvas.px.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// ---------------------------------------------------------------------------
// The card
// ---------------------------------------------------------------------------

const OUTPUT = "assets/ank-card.png";
const SOURCE = "assets/ank-dark.svg";

// The card carries its own ground rather than trusting the gallery's. The two
// SVG variants exist precisely because that background is not knowable, and an
// opaque card with a lit frame settles the question in one image instead of
// betting on a light or a dark host.
const GROUND = "#0d1117";
const EDGE = "#30363d";
const ACCENT = "#58a6ff";
const MUTED = "#8b949e";

const WIDTH = 1200;
const HEIGHT = 600;

function draw() {
  const mark = readMark(SOURCE);
  const c = new Canvas(WIDTH, HEIGHT, GROUND);

  // The frame is what keeps a dark card from bleeding into a dark gallery.
  c.frame(0, 0, WIDTH, HEIGHT, 5, EDGE);

  const markScale = 15; // 24 grid units -> 360 px
  const markW = mark.grid.w * markScale;
  const markH = mark.grid.h * markScale;

  const wordScale = 22;
  const wordW = textWidth("ank", wordScale);
  const wordH = GLYPH_H * wordScale;

  const leadScale = 4;
  const lead = "THE STUPID COORDINATION TOOL";
  const bodyScale = 3;
  // Broken by hand rather than wrapped: the widest line is what sets the text
  // column, and the accent rule is drawn to it.
  const body = [
    "TASKS AND ARCHITECTURE DECISIONS",
    "IN YOUR REPO, BEHIND ONE CLI",
    "ANY CODING AGENT CAN CALL.",
  ];

  const ruleGap = 26;
  const ruleH = 4;
  const leadGap = 26;
  const bodyGap = 24;
  const lineGap = 12;
  const bodyLineH = GLYPH_H * bodyScale;

  const textW = Math.max(
    wordW,
    textWidth(lead, leadScale),
    ...body.map((l) => textWidth(l, bodyScale)),
  );
  const textH =
    wordH +
    ruleGap +
    ruleH +
    leadGap +
    GLYPH_H * leadScale +
    bodyGap +
    body.length * bodyLineH +
    (body.length - 1) * lineGap;

  // The mark and the text block are one group, centred as a whole.
  const gutter = 68;
  const groupW = markW + gutter + textW;
  if (groupW > WIDTH) {
    throw new Error(
      `the group is ${groupW} px wide and the card is ${WIDTH}: shorten a ` +
        `line or drop a scale`,
    );
  }
  const groupX = Math.round((WIDTH - groupW) / 2);
  const textX = groupX + markW + gutter;
  const textY = Math.round((HEIGHT - textH) / 2);

  c.mark(mark, groupX, Math.round((HEIGHT - markH) / 2), markScale, mark.fill);

  let y = textY;
  c.text("ank", textX, y, wordScale, mark.fill);
  y += wordH + ruleGap;
  c.fill(textX, y, textW, ruleH, ACCENT);
  y += ruleH + leadGap;
  c.text(lead, textX, y, leadScale, ACCENT);
  y += GLYPH_H * leadScale + bodyGap;
  for (const line of body) {
    c.text(line, textX, y, bodyScale, MUTED);
    y += bodyLineH + lineGap;
  }

  return encodePng(c);
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// The manifest names an image this repository actually carries, in a format pi
/// accepts. The URL is remote and only the merge can prove it resolves, but the
/// basename it ends in is checkable here and now, and a rename that forgets the
/// manifest is exactly how the card goes blank.
function checkManifest(png) {
  const manifest = JSON.parse(
    readFileSync(join(repo, "npm/ank/package.json"), "utf8"),
  );
  const url = manifest.pi?.image;
  if (!url) throw new Error("npm/ank/package.json declares no pi.image");
  if (!url.startsWith("https://")) {
    throw new Error(`pi.image is not an HTTPS URL: ${url}`);
  }
  const name = url.split("/").pop();
  if (name !== OUTPUT.split("/").pop()) {
    throw new Error(`pi.image names ${name}, this script writes ${OUTPUT}`);
  }
  if (/\.svg$/i.test(name)) throw new Error("pi does not accept SVG");
  const magic = Buffer.from([0x89, 0x50, 0x4e, 0x47]);
  if (!png.subarray(0, 4).equals(magic)) {
    throw new Error("the generated bytes are not a PNG");
  }
}

const png = draw();
checkManifest(png);
const target = join(repo, OUTPUT);

if (process.argv.includes("--check")) {
  const on_disk = readFileSync(target);
  if (!on_disk.equals(png)) {
    console.error(
      `${OUTPUT} is not what this script produces: run ` +
        `\`node assets/make-card.mjs\` and commit the result`,
    );
    process.exit(1);
  }
  console.log(`${OUTPUT} ok (${png.length} bytes)`);
} else {
  writeFileSync(target, png);
  console.log(`${OUTPUT} written (${png.length} bytes)`);
}
