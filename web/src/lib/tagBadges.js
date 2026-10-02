// Discord's server tag badges, redrawn. They're 16×16 pixel art in a few tones made from the
// tag's two colours, so a badge can be drawn from the shape name and colours that tag events
// log (they don't say which image Discord made). Traced from real badges: in all of them the
// darker tone is the primary colour and the main fill the secondary, with lighter tints of it.
//
// Each template is 16 rows of 16 digits ("." is empty) and what each digit is drawn in:
//   "o" outline, "p" primary, "s" secondary, "w" white, or [base, toward, amount]: the primary
//   ("p") or secondary ("s") moved `amount` of the way toward another colour.

const WHITE = [255, 255, 255], BLACK = [0, 0, 0], GREY = [128, 128, 128];

const TEMPLATES = {
  HEART: {
    rows: [
      "................", "...1111..1111...", "..144441144441..", ".14000044000041.",
      "1404000000000031", "1400022002200031", "1000222222220031", "1000222222220031",
      "1000222222220031", ".13002222220031.", ".13000022000031.", "..133000000331..",
      "...1133003311...", ".....113311.....", ".......11.......", "................",
    ],
    digits: ["s", "o", ["s", WHITE, 0.6], "p", ["s", WHITE, 0.84]],
  },
  FIRE: {
    rows: [
      "...........1....", "..........131...", ".....1...1301...", "....131.13001...",
      "...13031300241..", "..130000000041..", ".13000000020041.", ".13002000220041.",
      "130032222222041.", "1302223333222041", "1002233333322041", "1000233333320041",
      ".14022333322041.", "..140222222041..", "...1444444441...", "....11111111....",
    ],
    digits: ["s", "o", ["s", WHITE, 0.5], ["s", WHITE, 0.94], "p"],
  },
  WATER_DROP: {
    rows: [
      ".......22.......", "......2442......", ".....240002.....", "....24000002....",
      "...2404000002...", "..240001100032..", ".24000111100032.", ".24001111110032.",
      "2400111111110032", "2400111111110032", "2000111111110032", "2000011111100032",
      "2000000000000332", ".23300000000332.", "..223333333322..", "....22222222....",
    ],
    digits: ["s", ["s", WHITE, 0.5], "o", "p", ["s", WHITE, 0.92]],
  },
  SKULL: {
    rows: [
      "....11111111....", "...1333333331...", "..130000000031..", ".13000000000031.",
      "1303000000000021", "1300000000000021", "1300000000000021", "1300222002220021",
      "1300244002440021", "1300244002440021", ".12000000000021.", "..120000000021..",
      "...1000000001...", "...1204004021...", "....12422421....", ".....111111.....",
    ],
    digits: ["s", "o", "p", "w", ["p", BLACK, 0.59]],
  },
  // The sword's blade and the crown's jewel have tones of their own; these are close.
  SWORD: {
    rows: [
      "...........00000", "..........033320", ".........0311120", "........03332120",
      "..00...031121120", ".0460.033321120.", ".0540.01121120..", "..04603121120...",
      "..0546001120....", "...05460200.....", "...000460.......", "..064054600.....",
      ".00550054460....", "06400..00540....", "0550.....00.....", ".00.............",
    ],
    digits: ["o", ["s", WHITE, 0.72], ["s", GREY, 0.35], ["s", WHITE, 0.92], "s", "p", ["s", WHITE, 0.43]],
  },
  CROWN: {
    rows: [
      "0......00......0", "00....0330....00", "030..033330..010", "0130031331300110",
      "0311111331111210", "0332211441112210", "0332214554112210", "0332245335412210",
      "0332205335012210", "0332204444012210", "0332204554012210", "0332210550112210",
      "0332211001112210", "0132211331112210", ".01221133111220.", "..000000000000..",
    ],
    digits: ["o", "s", "p", "w", ["p", BLACK, 0.4], ["p", BLACK, 0.2]],
  },
};

const rgb = (c) => {
  const m = /^#?([0-9a-f]{6})$/i.exec(c ?? "");
  return m ? [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16)) : null;
};
const css = (c) => `rgb(${c.map((v) => Math.round(v)).join(" ")})`;

/**
 * A badge as runs of same-coloured pixels on a 16×16 grid, `[{ x, y, w, fill }]`, or null for a
 * shape this doesn't know.
 */
export function badgePixels(name, primary, secondary) {
  const t = TEMPLATES[(name ?? "").toUpperCase()];
  if (!t) return null;
  const p = rgb(primary) ?? [138, 67, 255];
  const s = rgb(secondary) ?? p.map((v) => v + (255 - v) * 0.4);
  const named = { o: "#000", p: css(p), s: css(s), w: "#fff" };
  const fills = t.digits.map((d) => {
    if (typeof d === "string") return named[d];
    const [base, toward, amount] = d;
    return css((base === "p" ? p : s).map((v, i) => v + (toward[i] - v) * amount));
  });
  const runs = [];
  t.rows.forEach((row, y) => {
    for (let x = 0; x < 16; ) {
      let w = 1;
      while (x + w < 16 && row[x + w] === row[x]) w++;
      if (row[x] !== ".") runs.push({ x, y, w, fill: fills[+row[x]] });
      x += w;
    }
  });
  return runs;
}
