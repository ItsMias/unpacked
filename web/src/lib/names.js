// Friendlier names for things the package only has codes for.

// Phone and tablet model codes seen in Discord's logs -> what people call them.
const DEVICES = {
  beyond0: "Samsung Galaxy S10e", beyond1: "Samsung Galaxy S10", beyond2: "Samsung Galaxy S10+", beyondx: "Samsung Galaxy S10 5G",
  starlte: "Samsung Galaxy S9", star2lte: "Samsung Galaxy S9+", dreamlte: "Samsung Galaxy S8", dream2lte: "Samsung Galaxy S8+",
  x1s: "Samsung Galaxy S20", y2s: "Samsung Galaxy S20+", z3s: "Samsung Galaxy S20 Ultra", o1s: "Samsung Galaxy S21",
  t2s: "Samsung Galaxy S21+", p3s: "Samsung Galaxy S21 Ultra", r0s: "Samsung Galaxy S22", g0s: "Samsung Galaxy S22+",
  b0s: "Samsung Galaxy S22 Ultra", dm1q: "Samsung Galaxy S23", dm2q: "Samsung Galaxy S23+", dm3q: "Samsung Galaxy S23 Ultra",
  e1s: "Samsung Galaxy S24", e2s: "Samsung Galaxy S24+", e3q: "Samsung Galaxy S24 Ultra", a52q: "Samsung Galaxy A52",
  a53x: "Samsung Galaxy A53", a54x: "Samsung Galaxy A54", oriole: "Google Pixel 6", raven: "Google Pixel 6 Pro",
  panther: "Google Pixel 7", cheetah: "Google Pixel 7 Pro", shiba: "Google Pixel 8", husky: "Google Pixel 8 Pro",
  tokay: "Google Pixel 9", caiman: "Google Pixel 9 Pro", komodo: "Google Pixel 9 Pro XL",
};

/** "a52q" or "SM-G991B, o1s" -> a model name, or the code itself. */
export function deviceName(code) {
  if (DEVICES[code]) return DEVICES[code];
  // Android logs "model, codename"; Apple model ids have a comma of their own ("iPad13,8").
  if (/^(iPhone|iPad)\d+,\d+$/.test(code)) return code.startsWith("iPad") ? `iPad (${code})` : `iPhone (${code})`;
  const parts = code.split(",").map((s) => s.trim()).filter(Boolean);
  for (const p of parts) if (DEVICES[p]) return DEVICES[p];
  if (/^iPhone\d/.test(parts[0] ?? "")) return `iPhone (${parts[0]})`;
  if (/^iPad\d/.test(parts[0] ?? "")) return `iPad (${parts[0]})`;
  return parts[0] ?? code;
}

const regions = new Intl.DisplayNames(["en"], { type: "region" });
/** "NL" -> "Netherlands". */
export const countryName = (code) => { try { return regions.of(code) ?? code; } catch { return code; } };
/** "NL" -> 🇳🇱 */
export const flag = (code) => (/^[A-Z]{2}$/.test(code) ? String.fromCodePoint(...[...code].map((c) => 0x1f1a5 + c.charCodeAt(0))) : "");

const money = new Map();
/** 1499, "EUR" -> "€14.99". */
export function price(cents, currency) {
  const key = currency;
  if (!money.has(key)) {
    try { money.set(key, new Intl.NumberFormat("en-GB", { style: "currency", currency, currencyDisplay: "narrowSymbol" })); } catch { money.set(key, null); }
  }
  const f = money.get(key);
  return f ? f.format(cents / 100) : `${(cents / 100).toFixed(2)} ${currency}`;
}

export const PLATFORM_LABELS = { android: "Android app", ios: "iOS app", desktop: "Desktop app", web: "Browser", other: "Other" };
