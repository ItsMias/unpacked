// Emoji the way Discord draws them: Twemoji, the open emoji set Discord uses, from jsDelivr.

const BASE = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@17.0.3/assets/svg/";

/** File name Twemoji uses for an emoji: code points in hex, joined by "-". */
export function twemojiCode(emoji) {
  const cps = [...emoji].map((c) => c.codePointAt(0).toString(16));
  // The variation selector is dropped unless the emoji is a joined sequence (like 🏳️‍🌈).
  return (cps.includes("200d") ? cps : cps.filter((c) => c !== "fe0f")).join("-");
}

export const twemojiUrl = (emoji) => `${BASE}${twemojiCode(emoji)}.svg`;

const PICTO = /\p{Extended_Pictographic}|\p{Regional_Indicator}/u;
const segmenter = typeof Intl.Segmenter === "function" ? new Intl.Segmenter("en", { granularity: "grapheme" }) : null;

/** Splits plain text into `{ text }` and `{ emoji }` parts. */
export function splitEmoji(text) {
  if (!segmenter || !PICTO.test(text)) return [{ text }];
  const out = [];
  let buf = "";
  for (const { segment } of segmenter.segment(text)) {
    // Digits and # have emoji forms too; only count them when they're a keycap (1️⃣).
    const keycap = /^[0-9#*]️?⃣$/.test(segment);
    if (keycap || (PICTO.test(segment) && !/^[0-9#*©®™]$/.test(segment))) {
      if (buf) { out.push({ text: buf }); buf = ""; }
      out.push({ emoji: segment });
    } else buf += segment;
  }
  if (buf) out.push({ text: buf });
  return out;
}
