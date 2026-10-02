// Turns the engine's buckets into what the pages show, for one time zone and one date range.
// The engine counts in UTC hours/days/months; everything local happens here, so changing the
// time zone or the range never needs the package again.

import { countryName, deviceName, price } from "./names.js";

const HOUR = 3_600_000;
const DAY = 86_400_000;

const nf = new Intl.NumberFormat("en-US");
const compact = new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 2 });
export const fmt = (n) => nf.format(Math.round(n));
export const fmtCompact = (n) => compact.format(n);

/** "Thursday 1 August 2024" for a "2024-08-01" day key. */
export const longDate = (key) =>
  new Date(`${key}T12:00:00Z`).toLocaleDateString("en-GB", { timeZone: "UTC", weekday: "long", day: "numeric", month: "long", year: "numeric" });

export const shortMonth = (ms) => new Date(ms).toLocaleDateString("en-GB", { timeZone: "UTC", month: "short", year: "numeric" });
export const monthYear = (ms) => new Date(ms).toLocaleDateString("en-GB", { timeZone: "UTC", month: "long", year: "numeric" });
export const shortDate = (ms) => new Date(ms).toLocaleDateString("en-GB", { timeZone: "UTC", day: "numeric", month: "short", year: "numeric" });

/** Month index (months since Jan 2015, as the engine writes them) to a year. */
const yearOfMonth = (m) => 2015 + Math.floor(m / 12);
const yearOfDay = (d) => new Date(d * DAY).getUTCFullYear();

/**
 * Converts UTC hour buckets to local time in `tz`. Returns Dates whose UTC fields are the local
 * wall-clock time. Offsets are looked up once per UTC day (at noon), which is exact except for
 * the hour or two around a daylight-saving switch.
 */
function localizer(tz) {
  const f = new Intl.DateTimeFormat("en-US", {
    timeZone: tz, hourCycle: "h23", year: "numeric", month: "numeric", day: "numeric", hour: "numeric",
  });
  const cache = new Map();
  const offset = (day) => {
    let o = cache.get(day);
    if (o === undefined) {
      const ms = day * DAY + 12 * HOUR;
      const p = Object.fromEntries(f.formatToParts(ms).map((x) => [x.type, +x.value]));
      o = Date.UTC(p.year, p.month - 1, p.day, p.hour) - ms;
      cache.set(day, o);
    }
    return o;
  };
  return (hour) => {
    const ms = hour * HOUR;
    return new Date(ms + offset(Math.floor(ms / DAY)));
  };
}

/**
 * A 24-hour clock and an hour × weekday grid in `tz`, from `[UTC hour index, amount]` buckets,
 * for the years `inRange` accepts.
 */
export function hourClock(tz, buckets, inRange = () => true) {
  const local = localizer(tz);
  const clock = new Array(24).fill(0);
  const grid = Array.from({ length: 7 }, () => Array(24).fill(0));
  for (const [hour, n] of buckets) {
    const d = local(hour);
    if (!inRange(d.getUTCFullYear())) continue;
    clock[d.getUTCHours()] += n;
    grid[(d.getUTCDay() + 6) % 7][d.getUTCHours()] += n;
  }
  return { clock, grid };
}

/** Everything that depends on the time zone but not the range. */
export function buildView(facts, tz) {
  const local = localizer(tz);
  const conv = (list) =>
    list.map(([hour, n]) => {
      const d = local(hour);
      const y = d.getUTCFullYear(), m = d.getUTCMonth();
      return { key: d.toISOString().slice(0, 10), y, m, t: Date.UTC(y, m, d.getUTCDate()), h: d.getUTCHours(), dow: (d.getUTCDay() + 6) % 7, n };
    });
  const alive = conv(facts.alive_hours);
  const daily = new Map();
  for (const b of alive) daily.set(b.key, (daily.get(b.key) ?? 0) + b.n);

  // One colour scale for every year, so years are comparable: breaks at quantiles of active days.
  const vals = [...daily.values()].sort((a, b) => a - b);
  const q = (p) => vals[Math.floor(p * (vals.length - 1))] ?? 0;

  const voiceHours = (facts.voice?.hours_by_hour ?? []).map(([hour, n]) => {
    const d = local(hour);
    return { y: d.getUTCFullYear(), h: d.getUTCHours(), dow: (d.getUTCDay() + 6) % 7, key: d.toISOString().slice(0, 10), n };
  });

  return {
    tz,
    voiceHours,
    alive,
    deleted: conv(facts.deleted_hours),
    lost: conv(facts.lost_hours),
    daily,
    years: [...new Set(alive.map((b) => b.y))].sort((a, b) => a - b),
    breaks: [q(0.3), q(0.55), q(0.78), q(0.93)],
    people: new Map(facts.people.map((p) => [p.id, p])),
  };
}

/** Heat level 0..5 for a count. */
export const level = (n, breaks) => (n <= 0 ? 0 : 1 + breaks.filter((b) => n > b).length);

/** Numbers for the Overview, for `range` ("all" or a year). */
export function overview(view, facts, range) {
  const inRange = (y) => range === "all" || y === range;
  const sum = (list) => list.reduce((s, b) => (inRange(b.y) ? s + b.n : s), 0);
  const alive = sum(view.alive);
  const deleted = sum(view.deleted);
  const lost = sum(view.lost);

  let daysActive = 0;
  for (const [k, n] of view.daily) if (n && inRange(+k.slice(0, 4))) daysActive++;

  let voiceHours = 0, voiceSessions = 0;
  for (const [d, h, s] of facts.voice?.days ?? []) {
    if (inRange(yearOfDay(d))) { voiceHours += h; voiceSessions += s; }
  }

  const hours = Array.from({ length: 7 }, () => Array(24).fill(0));
  for (const b of view.alive) if (inRange(b.y)) hours[b.dow][b.h] += b.n;

  const perPerson = new Map();
  for (const c of facts.channels) {
    if (c.kind !== "dm" || !c.person) continue;
    let n = 0;
    for (const [m, k] of c.months) if (inRange(yearOfMonth(m))) n += k;
    if (n) perPerson.set(c.person, (perPerson.get(c.person) ?? 0) + n);
  }
  const people = [...perPerson]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
    .map(([id, messages]) => ({ ...(view.people.get(id) ?? { id, display_name: "Unknown user" }), messages }));

  return { alive, deleted, lost, sent: alive + deleted + lost, daysActive, voiceHours, voiceSessions, hours, people };
}

/** Busiest day, active days and longest streak within one calendar year. */
export function yearStats(view, year) {
  let best = null, days = 0, streak = 0, longest = 0, total = 0;
  for (let t = Date.UTC(year, 0, 1); new Date(t).getUTCFullYear() === year; t += DAY) {
    const key = new Date(t).toISOString().slice(0, 10);
    const n = view.daily.get(key) ?? 0;
    total += n;
    if (n && (!best || n > best.n)) best = { key, n };
    if (n) { days++; streak++; longest = Math.max(longest, streak); } else streak = 0;
  }
  return { best, days, longest, total };
}

/** Per calendar year: still there, deleted and lost, oldest first. */
export function stillThereByYear(view) {
  const rows = new Map();
  const add = (list, k) => {
    for (const b of list) {
      const r = rows.get(b.y) ?? rows.set(b.y, { y: b.y, alive: 0, deleted: 0, lost: 0 }).get(b.y);
      r[k] += b.n;
    }
  };
  add(view.alive, "alive");
  add(view.deleted, "deleted");
  add(view.lost, "lost");
  return [...rows.values()].sort((a, b) => a.y - b.y);
}

/** Calendar-year totals, for the year tabs. */
export function yearTotals(view) {
  const t = new Map();
  for (const b of view.alive) t.set(b.y, (t.get(b.y) ?? 0) + b.n);
  return t;
}

/** Profile badges, in the order Discord shows them: account flag -> [label, badge icon hash]. */
const FLAG_BADGES = [
  ["STAFF", "Discord Staff", "5e74e9b61934fc1f67c65515d1f7e60d"],
  ["PARTNER", "Partnered Server Owner", "3f9748e53446a137a052f3454e2de41e"],
  ["CERTIFIED_MODERATOR", "Moderator Programs Alumni", "fee1624003e2fee35cb398e125dc479b"],
  ["HYPESQUAD", "HypeSquad Events", "bf01d1073931f921909045f3a39fd264"],
  ["HYPESQUAD_ONLINE_HOUSE_1", "HypeSquad Bravery", "8a88d63823d8a71cd5e390baa45efa02"],
  ["HYPESQUAD_ONLINE_HOUSE_2", "HypeSquad Brilliance", "011940fd013da3f7fb926e4a1cd2e618"],
  ["HYPESQUAD_ONLINE_HOUSE_3", "HypeSquad Balance", "3aa41de486fa12454c3761e8e223442e"],
  ["BUG_HUNTER_LEVEL_1", "Discord Bug Hunter", "2717692c7dca7289b35297368a940dd0"],
  ["BUG_HUNTER_LEVEL_2", "Discord Bug Hunter", "848f79194d4be5ff5f81505cbd0ce1e6"],
  ["ACTIVE_DEVELOPER", "Active Developer", "6bdc42827a38498929a4920da12695d9"],
  ["VERIFIED_DEVELOPER", "Early Verified Bot Developer", "6df5892e0f35b051f8b61eace34f4967"],
  ["PREMIUM_EARLY_SUPPORTER", "Early Supporter", "7060786766c9c840eb3019e725d2b358"],
];
// Server booster badge by months boosted: 1, 2, 3, 6, 9, 12, 15, 18, 24.
const BOOSTER = [
  [1, "51040c70d4f20a921ad6674ff86fc95c"], [2, "0e4080d1d333bc7ad29ef6528b6f2fb7"], [3, "72bed924410c304dbe3d00a6e593ff59"],
  [6, "df199d2050d3ed4ebf84d64ae83989f8"], [9, "996b3e870e8a22ce519b3a50e6bdd52f"], [12, "991c9f39ee33d7537d9f408c3e53141e"],
  [15, "cb3ae83c15e970e8f3d410bc62cb8b99"], [18, "7142225d31238f6387d9f09efaa02759"], [24, "ec92202290b48d0879b7413d2dde3bab"],
];
const NITRO = "2ba85e8026a8614b640c2837bcdfe21b";
const LEGACY = "6de6d34650760ba5551a79732e98ed60";

export const badgeUrl = (hash) => `https://cdn.discordapp.com/badge-icons/${hash}.png`;

/** [{ label, icon }] for the profile, as of the package date. */
export function badges(profile, packageDate) {
  const out = FLAG_BADGES.filter(([f]) => profile.flags.includes(f)).map(([, label, icon]) => ({ label, icon }));
  if (profile.premium_until && Date.parse(profile.premium_until) > packageDate) {
    const since = profile.premium_since ? `Subscriber since ${shortDate(Date.parse(profile.premium_since))}` : "Nitro subscriber";
    out.push({ label: since, icon: NITRO });
  }
  if (profile.boosting_since) {
    const start = Date.parse(profile.boosting_since);
    const months = (packageDate - start) / (30.44 * DAY);
    const tier = BOOSTER.filter(([m]) => months >= m).at(-1) ?? BOOSTER[0];
    out.push({ label: `Server boosting since ${shortDate(start)}`, icon: tier[1] });
  }
  if (profile.legacy_username) out.push({ label: `Originally known as ${profile.legacy_username}`, icon: LEGACY });
  return out;
}

/**
 * Messages and voice over time, for the activity chart: per month on "All time", per week
 * inside a single year.
 */
export function activity(view, facts, range, until) {
  const points = new Map();
  const weekly = range !== "all";
  const keyOf = (t) => {
    const d = new Date(t);
    if (!weekly) return Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), 1);
    const start = Date.UTC(range, 0, 1);
    return start + Math.floor((t - start) / (7 * DAY)) * 7 * DAY;
  };
  const inRange = (t) => range === "all" || new Date(t).getUTCFullYear() === range;
  const slot = (t) => {
    const k = keyOf(t);
    let p = points.get(k);
    if (!p) points.set(k, (p = { t: k, alive: 0, deleted: 0, lost: 0, voice: 0 }));
    return p;
  };
  for (const [list, field] of [[view.alive, "alive"], [view.deleted, "deleted"], [view.lost, "lost"]]) {
    for (const b of list) if (inRange(b.t)) slot(b.t)[field] += b.n;
  }
  for (const [d, h] of facts.voice?.days ?? []) if (inRange(d * DAY)) slot(d * DAY).voice += h;

  // Fill gaps so quiet stretches show as zero, from the first point to the package date.
  const keys = [...points.keys()].sort((a, b) => a - b);
  if (!keys.length) return { points: [], weekly };
  const out = [];
  const end = Math.min(keyOf(until), weekly ? Date.UTC(range, 11, 31) : Infinity);
  for (let k = keys[0]; k <= end; ) {
    out.push(points.get(k) ?? { t: k, alive: 0, deleted: 0, lost: 0, voice: 0 });
    if (weekly) k += 7 * DAY;
    else { const d = new Date(k); k = Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 1); }
  }
  return { points: out, weekly };
}

const sumYears = (years, inRange) => years.reduce((s, n, i) => (inRange(2015 + i) ? s + n : s), 0);

/** Everything on the Messages page, for `range`. */
export function messageStats(view, facts, range) {
  const inRange = (y) => range === "all" || y === range;
  const t = facts.text;
  const messages = view.alive.reduce((s, b) => (inRange(b.y) ? s + b.n : s), 0);

  const lengths = Array(8).fill(0);
  t.lengths.forEach((row, i) => { if (inRange(2015 + i)) row.forEach((n, j) => (lengths[j] += n)); });

  const rank = (list, n) =>
    list
      .map((w) => ({ text: w.text, n: sumYears(w.years, inRange) }))
      .filter((w) => w.n > 0)
      .sort((a, b) => b.n - a.n)
      .slice(0, n);

  // Where: DMs, group chats, servers, from per-channel months.
  const where = { dm: 0, group: 0, guild: 0, other: 0 };
  const channels = [];
  const serverName = new Map(facts.servers.map((s) => [s.id, s.name]));
  for (const c of facts.channels) {
    let n = 0;
    for (const [m, k] of c.months) if (inRange(2015 + Math.floor(m / 12))) n += k;
    if (!n) continue;
    where[c.kind] += n;
    if (c.kind === "guild") channels.push({ id: c.id, n, ...channelName(c, serverName.get(c.guild)) });
  }
  channels.sort((a, b) => b.n - a.n);

  return {
    messages,
    words: sumYears(t.words, inRange),
    chars: sumYears(t.chars, inRange),
    attachments: sumYears(t.with_attachments, inRange),
    links: sumYears(t.with_links, inRange),
    lengths,
    words_top: rank(t.top_words, 40),
    fillers: rank(t.filler_words ?? [], 40),
    domains: rank(t.top_domains, 10),
    where,
    channels: channels.slice(0, 10),
  };
}

/** "general" and "Server name" from Messages/index.json's "general, Server name". */
export function channelName(c, server) {
  let name = c.name ?? "unknown channel";
  if (server && name.endsWith(`, ${server}`)) name = name.slice(0, -server.length - 2);
  return { channel: name, server: server ?? "Server no longer available" };
}

// ---------- shared ----------

const yearOfMs = (ms) => new Date(ms).getUTCFullYear();
const monthsIn = (c, inRange) => c.months.reduce((s, [m, k]) => (inRange(yearOfMonth(m)) ? s + k : s), 0);
export const inRangeFn = (range) => (y) => range === "all" || y === range;

/** Every server-tag event, from the main facts and the analytics worker, oldest first. */
export function allTags(facts, games) {
  return [...(facts.tags ?? []), ...(games?.tags ?? [])].sort((a, b) => a.ms - b.ms);
}

/** Tags you wore, with consecutive repeats folded together: [{ ...event, until }]. */
export function tagHistory(tags) {
  const out = [];
  for (const t of tags.filter((x) => x.kind === "adopted" && x.tag)) {
    const last = out.at(-1);
    if (last && last.tag === t.tag && last.guild === t.guild) { last.badge = t.badge; last.colours = t.colours; continue; }
    if (last) last.until = t.ms;
    out.push({ ...t });
  }
  return out;
}

/** Display name for a channel id: "DM with X", a group's name or "#channel". */
export function placeName(facts, view, channelId, servers = new Map(facts.servers.map((s) => [s.id, s.name]))) {
  const c = facts.channels.find((x) => x.id === channelId);
  if (!c) return "a call";
  if (c.kind === "dm") return view.people.get(c.person)?.display_name ?? "a DM";
  if (c.kind === "group") return c.name && c.name !== "None" ? c.name : "a group chat";
  const n = channelName(c, servers.get(c.guild));
  return `#${n.channel}`;
}

// ---------- people ----------

const COLOURS = ["var(--cat-1)", "var(--cat-2)", "var(--cat-3)", "var(--cat-4)"];

export function peopleStats(view, facts, range) {
  const inRange = inRangeFn(range);
  const dmPerson = new Map();
  const per = new Map(); // person -> { messages, first, last, channels: [] }
  const groups = [];
  for (const c of facts.channels) {
    if (c.kind === "dm" && c.person) {
      dmPerson.set(c.id, c.person);
      const p = per.get(c.person) ?? per.set(c.person, { messages: 0, first: Infinity, last: 0, channels: [], voice: 0 }).get(c.person);
      p.messages += monthsIn(c, inRange);
      p.channels.push(c);
      if (c.first_ms) p.first = Math.min(p.first, c.first_ms);
      p.last = Math.max(p.last, c.last_ms);
    } else if (c.kind === "group") {
      const n = monthsIn(c, inRange);
      if (n) groups.push({ id: c.id, name: c.name && c.name !== "None" ? c.name : "Unnamed group", n, first: c.first_ms, last: c.last_ms });
    }
  }
  for (const v of facts.voice?.places ?? []) {
    const person = v.kind === "channel" && dmPerson.get(v.id);
    if (!person) continue;
    per.get(person).voice += v.months.reduce((s, [m, h]) => (inRange(yearOfMonth(m)) ? s + h : s), 0);
  }
  const ranked = [...per]
    .filter(([, p]) => p.messages > 0 || p.voice > 0)
    .map(([id, p]) => ({ ...(view.people.get(id) ?? { id, display_name: "Unknown user", username: "" }), ...p }))
    .sort((a, b) => b.messages + b.voice * 60 - (a.messages + a.voice * 60));
  groups.sort((a, b) => b.n - a.n);

  // The top four people over time, plus everyone else, per month (or week inside a year).
  const top = ranked.slice(0, 4);
  const series = top.map((p, i) => ({ key: p.id, label: p.display_name, colour: COLOURS[i] }));
  series.push({ key: "rest", label: "Everyone else", colour: "var(--t-faint)" });
  const pts = new Map();
  const topIds = new Set(top.map((p) => p.id));
  for (const c of facts.channels) {
    if (c.kind !== "dm" || !c.person) continue;
    const key = topIds.has(c.person) ? c.person : "rest";
    for (const [m, k] of c.months) {
      const y = yearOfMonth(m);
      if (!inRange(y)) continue;
      const t = Date.UTC(y, m % 12, 1);
      const pt = pts.get(t) ?? pts.set(t, { t }).get(t);
      pt[key] = (pt[key] ?? 0) + k;
    }
  }
  const points = fillMonths([...pts.values()].sort((a, b) => a.t - b.t), series.map((s) => s.key));

  // Friend list changes per year (the log doesn't say who).
  const years = new Map();
  for (const [ms, type] of facts.friend_events) {
    const y = yearOfMs(ms);
    if (!inRange(y)) continue;
    const r = years.get(y) ?? years.set(y, { label: String(y), up: 0, down: 0 }).get(y);
    if (type === 1) r.up++;
    else if (type === 0) r.down++;
  }
  const friendYears = [...years.values()].sort((a, b) => a.label - b.label);
  const added = friendYears.reduce((s, r) => s + r.up, 0);
  const removed = friendYears.reduce((s, r) => s + r.down, 0);

  return { ranked, groups, series, points, friendYears, added, removed, dms: ranked.filter((p) => p.messages > 0).length };
}

/** Monthly points from the first to the last, with empty months filled in as zero. */
function fillMonths(points, keys) {
  if (!points.length) return [];
  const out = [];
  const last = points.at(-1).t;
  const byT = new Map(points.map((p) => [p.t, p]));
  for (let t = points[0].t; t <= last; ) {
    const p = byT.get(t) ?? { t };
    for (const k of keys) p[k] ??= 0;
    out.push(p);
    const d = new Date(t);
    t = Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 1);
  }
  return out;
}

// ---------- servers ----------

export function serverStats(view, facts, range, sources) {
  const inRange = inRangeFn(range);
  const names = new Map(facts.servers.map((s) => [s.id, s]));
  const current = new Set(sources?.serverIds ?? []);
  const per = new Map();
  const get = (id) => per.get(id) ?? per.set(id, { id, messages: 0, voice: 0, joined: null, left: null, mod: 0, boosts: 0 }).get(id);
  for (const c of facts.channels) if (c.kind === "guild" && c.guild) get(c.guild).messages += monthsIn(c, inRange);
  for (const v of facts.voice?.places ?? []) {
    if (v.kind === "guild") get(v.id).voice += v.months.reduce((s, [m, h]) => (inRange(yearOfMonth(m)) ? s + h : s), 0);
  }

  const kinds = { join: new Set(), leave: new Set(), create: new Set(), delete: new Set() };
  const mods = new Map(), boosts = [], bots = new Set();
  const years = new Map();
  for (const e of facts.guild_events) {
    const g = get(e.guild);
    if (e.kind === "join") g.joined ??= e.ms;
    if (e.kind === "leave") g.left = e.ms;
    if (!inRange(yearOfMs(e.ms))) continue;
    if (kinds[e.kind]) kinds[e.kind].add(e.guild);
    if (e.kind === "join" || e.kind === "leave") {
      const y = yearOfMs(e.ms);
      const r = years.get(y) ?? years.set(y, { label: String(y), up: 0, down: 0 }).get(y);
      r[e.kind === "join" ? "up" : "down"]++;
    }
    if (e.kind === "mod") { mods.set(e.detail ?? "other", (mods.get(e.detail ?? "other") ?? 0) + 1); g.mod++; }
    if (e.kind === "boost") { boosts.push(e); g.boosts++; }
    if (e.kind === "bot") bots.add(`${e.guild}/${e.detail}`);
  }

  const ranked = [...per.values()]
    .filter((s) => s.messages > 0 || s.voice > 0)
    .map((s) => ({ ...s, name: names.get(s.id)?.name ?? "Server no longer available", info: names.get(s.id), member: current.has(s.id) }))
    .sort((a, b) => b.messages + b.voice * 60 - (a.messages + a.voice * 60));

  return {
    ranked,
    joined: kinds.join.size,
    left: kinds.leave.size,
    created: kinds.create.size,
    deleted: kinds.delete.size,
    boosts: boosts.length,
    bots: bots.size,
    mods: [...mods].sort((a, b) => b[1] - a[1]),
    modTotal: [...mods.values()].reduce((a, b) => a + b, 0),
    joinYears: [...years.values()].sort((a, b) => a.label - b.label),
    counts: serverCounts(facts, range),
  };
}

/**
 * How many servers you were in, month by month, from the counts Discord logged with joins,
 * invites and friend list changes (the last count of each month; quiet months keep the last).
 */
export const serverCounts = (facts, range) => monthEnd(facts.guild_counts ?? [], "servers", range);

/**
 * How many friends you had, month by month, from the counts Discord logged when you opened the
 * friends list (only since mid-2022): the last count of each month; quiet months keep the last.
 */
export const friendCounts = (facts, range) => monthEnd(facts.friend_counts ?? [], "friends", range);

function monthEnd(readings, key, range) {
  const byMonth = new Map();
  for (const [ms, n] of readings) {
    const d = new Date(ms);
    byMonth.set(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), 1), n);
  }
  const keys = [...byMonth.keys()].sort((a, b) => a - b);
  if (!keys.length) return [];
  const out = [];
  let last = 0;
  for (let t = keys[0]; t <= keys.at(-1); ) {
    last = byMonth.get(t) ?? last;
    if (range === "all" || new Date(t).getUTCFullYear() === range) out.push({ t, [key]: last });
    const d = new Date(t);
    t = Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 1);
  }
  return out;
}

// ---------- voice ----------

export function voiceStats(view, facts, range) {
  const inRange = inRangeFn(range);
  const v = facts.voice;
  if (!v) return null;
  let hours = 0, sessions = 0;
  const days = new Set();
  for (const [d, h, s] of v.days) {
    if (!inRange(new Date(d * DAY).getUTCFullYear())) continue;
    hours += h; sessions += s;
    if (h > 0) days.add(d);
  }
  const clock = new Array(24).fill(0);
  const grid = Array.from({ length: 7 }, () => Array(24).fill(0));
  for (const b of view.voiceHours) if (inRange(b.y)) { clock[b.h] += b.n; grid[b.dow][b.h] += b.n; }

  const kindOf = new Map(facts.channels.map((c) => [c.id, c.kind]));
  const servers = new Map(facts.servers.map((s) => [s.id, s.name]));
  const pts = new Map();
  const places = [];
  for (const p of v.places) {
    const kind = p.kind === "guild" ? "servers" : kindOf.get(p.id) === "group" ? "groups" : "dms";
    let total = 0;
    for (const [m, h] of p.months) {
      const y = yearOfMonth(m);
      if (!inRange(y)) continue;
      total += h;
      const t = Date.UTC(y, m % 12, 1);
      const pt = pts.get(t) ?? pts.set(t, { t, servers: 0, dms: 0, groups: 0 }).get(t);
      pt[kind] += h;
    }
    if (total > 0) {
      places.push({
        id: p.id, kind, hours: total,
        name: p.kind === "guild" ? servers.get(p.id) ?? "Server no longer available" : placeName(facts, view, p.id, servers),
        person: kind === "dms" ? view.people.get(facts.channels.find((c) => c.id === p.id)?.person) : null,
      });
    }
  }
  places.sort((a, b) => b.hours - a.hours);
  const longest = v.longest
    .filter((s) => inRange(yearOfMs(s.start_ms)))
    .map((s) => ({ ...s, hours: (s.end_ms - s.start_ms) / HOUR, name: s.kind === "guild" ? servers.get(s.id) ?? "a server" : placeName(facts, view, s.id, servers) }));

  return {
    hours, sessions, days: days.size, avg: sessions ? hours / sessions : 0,
    clock, grid,
    points: fillMonths([...pts.values()].sort((a, b) => a.t - b.t), ["servers", "dms", "groups"]),
    places,
    longest,
    split: ["servers", "dms", "groups"].map((k) => places.filter((p) => p.kind === k).reduce((s, p) => s + p.hours, 0)),
  };
}

// ---------- timeline ----------

const ordinal = (n) => `${fmt(n)}${[11, 12, 13].includes(n % 100) ? "th" : ["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
const duration = (h) => (h >= 1 ? `${Math.floor(h)} h ${Math.round((h % 1) * 60)} min` : `${Math.round(h * 60)} min`);

// Discord's tiered profile badges: Account Age by years, Streaming by hours streamed to others.
const AGE_BADGES = ["Seed", "Sprout", "Bud", "Sapling", "Blossom", "Redwood", "Sequoia", "Bristlecone", "Stromatolite", "Primordial"];
const STREAM_BADGES = [
  [1, "Newcomer"], [5, "Fledgling"], [20, "Breakout"], [75, "Standout"], [150, "Trendsetter"],
  [300, "Headliner"], [500, "Star"], [1000, "Sensation"], [2000, "Visionary"], [5000, "Phenomenon"],
];
const dayMonth = (ms) => new Date(ms).toLocaleDateString("en-GB", { timeZone: "UTC", day: "numeric", month: "short" });
const messagesIn = (c) => c.months.reduce((s, [, k]) => s + k, 0);

/**
 * Milestones from everything the package says, oldest first:
 * [{ ms, icon, title, sub?, quote?, tag?, image?, person?, flag?, tier?, app? }]
 *   person: their profile, drawn as their avatar; flag: a country code; app: an application id
 *   the page looks the name up for; tier: { kind: "age" | "stream", n } for Discord's badges.
 */
export function timeline(view, facts, games, result) {
  const items = [];
  const add = (ms, icon, title, extra = {}) => { if (ms) items.push({ ms, icon, title, ...extra }); };
  const p = facts.profile;
  const packageDate = result.sources?.packageDate;
  const until = packageDate || Date.now();
  // Only servers the package names; the rest are worded around.
  const servers = new Map(facts.servers.filter((s) => s.name_known).map((s) => [s.id, s.name]));

  // The account, and the Account Age badge on each anniversary.
  if (p) {
    add(p.created_ms, "star", "Created your Discord account", { sub: `@${p.username}` });
    const c = new Date(p.created_ms);
    AGE_BADGES.forEach((name, i) => {
      const ms = Date.UTC(c.getUTCFullYear() + i + 1, c.getUTCMonth(), c.getUTCDate(), c.getUTCHours());
      if (ms <= until) add(ms, "age", `${i + 1} year${i ? "s" : ""} on Discord`, { sub: `Account Age badge: ${name} (tier ${i + 1})`, tier: { kind: "age", n: i + 1 } });
    });
  }

  const first = facts.text.first;
  if (first) add(first.ms, "chat", "Oldest message still in your package", { sub: placeName(facts, view, first.channel, servers), quote: first.text });

  // Message milestones, from the per-hour counts in your time zone.
  const marks = [1_000, 10_000, 50_000, 100_000, 250_000, 500_000, 750_000, 1_000_000];
  let total = 0, i = 0;
  for (const b of [...view.alive].sort((a, c) => a.t - c.t || a.h - c.h)) {
    total += b.n;
    while (i < marks.length && total >= marks[i]) add(b.t + b.h * HOUR, "flag", `Your ${ordinal(marks[i++])} message`);
  }
  let best = null;
  for (const [k, n] of view.daily) if (!best || n > best.n) best = { k, n };
  if (best) add(Date.parse(`${best.k}T12:00:00Z`), "fire", "Your busiest day", { sub: `${fmt(best.n)} messages` });

  // The first DM with each of your 25 most messaged people (the oldest message still there).
  const dms = new Map(); // person -> { messages, first }
  for (const c of facts.channels) {
    if (c.kind !== "dm" || !c.person || !c.first_ms) continue;
    const d = dms.get(c.person) ?? dms.set(c.person, { messages: 0, first: Infinity }).get(c.person);
    d.messages += messagesIn(c);
    d.first = Math.min(d.first, c.first_ms);
  }
  [...dms].sort((a, b) => b[1].messages - a[1].messages).slice(0, 25).forEach(([id, d], k) => {
    const person = view.people.get(id);
    if (person) add(d.first, "chat", `First DM with ${person.display_name}`, { sub: `#${k + 1} in your DMs · ${fmt(d.messages)} messages`, person });
  });

  // Servers: the first join, ones you made, and leaving ones you were active in (100+
  // messages) unless you're back in them.
  const firstJoin = facts.guild_events.find((e) => e.kind === "join");
  if (firstJoin) add(firstJoin.ms, "door", "First server join in the log", { sub: servers.get(firstJoin.guild) ?? "A server that's gone now" });
  for (const e of facts.guild_events) {
    if (e.kind === "create" && servers.has(e.guild)) add(e.ms, "plus", `Created ${servers.get(e.guild)}`);
  }
  const sentIn = new Map();
  for (const c of facts.channels) if (c.kind === "guild" && c.guild) sentIn.set(c.guild, (sentIn.get(c.guild) ?? 0) + messagesIn(c));
  const current = new Set(result.sources?.serverIds ?? []);
  const lastLeave = new Map();
  for (const e of facts.guild_events) if (e.kind === "leave") lastLeave.set(e.guild, e.ms);
  for (const [g, ms] of lastLeave) {
    const n = sentIn.get(g) ?? 0;
    if (n >= 100 && !current.has(g)) add(ms, "leave", `Left ${servers.get(g) ?? "a server"}`, { sub: `${fmt(n)} messages there` });
  }

  // Boosting: the first boost, then each server's first.
  const boosted = new Set();
  for (const e of facts.guild_events) {
    if (e.kind !== "boost" || boosted.has(e.guild)) continue;
    const where = servers.get(e.guild) ?? "a server";
    add(e.ms, "gem", boosted.size ? `Boosted ${where}` : "Started boosting", boosted.size ? {} : { sub: where });
    boosted.add(e.guild);
  }
  if (!boosted.size && p?.boosting_since) add(Date.parse(p.boosting_since), "gem", "Started boosting");

  // Money: getting Nitro, the first purchase, and every 100, 250, 500… spent (per currency).
  const buys = [...(facts.money?.purchases ?? [])].filter((x) => !x.refunded).sort((a, b) => a.ms - b.ms);
  const nitro = buys.find((x) => !x.gift && /nitro/i.test(x.title ?? ""));
  const since = p?.premium_since ? Date.parse(p.premium_since) : 0;
  if (since && (!nitro || since < nitro.ms)) add(since, "nitro", "Got Nitro");
  else if (nitro) add(nitro.ms, "nitro", `Got ${nitro.title}`, { sub: nitro === buys[0] ? `Your first purchase · ${price(nitro.cents, nitro.currency)}` : price(nitro.cents, nitro.currency) });
  if (buys[0] && buys[0] !== nitro) add(buys[0].ms, "coin", "First purchase on Discord", { sub: `${buys[0].title ?? "Shop item"} · ${price(buys[0].cents, buys[0].currency)}` });
  const spent = new Map();
  for (const b of buys) {
    const before = spent.get(b.currency) ?? 0, after = before + b.cents;
    spent.set(b.currency, after);
    for (const m of [100, 250, 500, 1000, 2500, 5000, 10000]) {
      if (before < m * 100 && after >= m * 100) add(b.ms, "coin", `${price(m * 100, b.currency).replace(/[.,]00$/, "")} spent on Discord`);
    }
  }

  // Games and Activities.
  const played = (games?.games ?? facts.games)?.top ?? [];
  const firstGame = played.filter((x) => x.first_ms).sort((a, b) => a.first_ms - b.first_ms)[0];
  if (firstGame) add(firstGame.first_ms, "games", "First game Discord saw you play", { sub: firstGame.name });
  const firstActivity = [...(facts.activities ?? [])].sort((a, b) => a[2] - b[2])[0];
  if (firstActivity) add(firstActivity[2], "games", "First Activity you joined", { app: firstActivity[0] });

  // The trust & safety log: the Streaming badge as streamed hours add up, and two-factor.
  const tns = facts.tns;
  if (tns?.streamed.length) {
    let hours = 0, k = 0;
    for (const [s, e] of tns.streamed) {
      const h = (e - s) / HOUR;
      while (k < STREAM_BADGES.length && hours + h >= STREAM_BADGES[k][0]) {
        const [at, name] = STREAM_BADGES[k];
        const from = k ? "" : ` · counted from ${monthYear(tns.since_ms)}`;
        add(s + (at - hours) * HOUR, "stream", `${fmt(at)} hour${at > 1 ? "s" : ""} streamed`, { sub: `Streaming badge: ${name} (tier ${k + 1})${from}`, tier: { kind: "stream", n: k + 1 } });
        k++;
      }
      hours += h;
    }
  }
  for (const [ms, on] of tns?.two_factor ?? []) add(ms, "shield", `Turned ${on ? "on" : "off"} two-factor authentication`);

  // Devices: new phones and tablets (one per model, however its code was logged) and desktop systems.
  const phones = new Map();
  for (const c of facts.devices?.clients ?? []) {
    if ((c.platform === "android" || c.platform === "ios") && c.device) {
      const name = deviceName(c.device);
      const kind = /^iPad|\bTab\b/.test(name) ? "tablet" : /Chromebook/.test(name) ? "Chromebook" : "phone";
      const x = phones.get(name) ?? phones.set(name, { first: c.first_ms, count: 0, kind }).get(name);
      x.first = Math.min(x.first, c.first_ms);
      x.count += c.count;
    } else if (c.platform === "desktop" && c.count >= 50) {
      add(c.first_ms, "desktop", `Started using the desktop app${c.os ? ` on ${c.os}` : ""}`, { sub: `${fmt(c.count)} sessions` });
    }
  }
  for (const [name, x] of phones) {
    if (x.count >= 20) add(x.first, "phone", x.kind === "Chromebook" ? "New Chromebook" : `New ${x.kind}: ${name}`, { sub: `${fmt(x.count)} sessions` });
  }

  // Trips: days mostly spent in another country. Other countries with a few sessions get a first visit.
  const countries = facts.devices?.countries ?? [];
  const tripped = new Set();
  for (const t of facts.devices?.trips ?? []) {
    tripped.add(t.country);
    add(t.start_ms, "plane", `${t.days > 1 ? "Trip" : "Day"} ${t.days > 1 ? "to" : "in"} ${countryWith(t.country)}`, {
      sub: t.days > 1 ? `${dayMonth(t.start_ms)} – ${shortDate(t.end_ms)} · ${t.days} days` : `${fmt(t.sessions)} sessions`,
      flag: t.country,
    });
  }
  for (const c of countries.slice(1)) {
    if (!tripped.has(c.name) && c.count >= 20) add(c.first_ms, "plane", `First time on Discord in ${countryWith(c.name)}`, { sub: `${fmt(c.count)} sessions`, flag: c.name });
  }

  // Server tags: ones you set up for your servers, and wearing one.
  const setUp = new Set();
  for (const t of facts.tags) {
    if (t.kind !== "created" || !t.tag || setUp.has(`${t.guild}/${t.tag}`)) continue;
    setUp.add(`${t.guild}/${t.tag}`);
    add(t.ms, "tag", `Set up the ${t.tag} tag`, { tag: t, sub: servers.get(t.guild) });
  }
  for (const t of tagHistory(allTags(facts, games))) add(t.ms, "tag", `Started wearing the ${t.tag} tag`, { tag: t, sub: servers.get(t.guild) });
  for (const a of result.avatars ?? []) add(a.ms, "avatar", "Changed your avatar", { image: a.blob });

  const v = facts.voice;
  if (v?.days.length) add(v.days[0][0] * DAY, "voice", "First voice session in the log");
  if (v?.longest[0]) {
    const l = v.longest[0];
    add(l.start_ms, "voice", `Longest call: ${duration((l.end_ms - l.start_ms) / HOUR)}`, {
      sub: l.kind === "guild" ? servers.get(l.id) ?? "a server" : placeName(facts, view, l.id, servers),
    });
  }

  let friends = 0, j = 0;
  const friendMarks = [1, 100, 250, 500, 1000];
  for (const [ms, type] of facts.friend_events) {
    if (type !== 1) continue;
    friends++;
    while (j < friendMarks.length && friends >= friendMarks[j]) {
      const n = friendMarks[j++];
      add(ms, "friend", n === 1 ? "First new friend in the log" : `Your ${ordinal(n)} new friend`);
    }
  }

  // Data requests: the first, this package (the last request before it was made), and the rest.
  const requests = facts.data_requests;
  const thisOne = requests.filter((ms) => !packageDate || ms <= packageDate).at(-1);
  requests.forEach((ms, k) => {
    const nth = requests.length > 1 ? `Your ${ordinal(k + 1)} request` : "";
    if (ms === thisOne) add(ms, "box", "Requested this package", { sub: nth });
    else add(ms, "box", k ? "Requested your data" : "First time you requested your data", { sub: k ? nth : "" });
  });

  return items.sort((a, b) => a.ms - b.ms);
}

/** A country's name as it reads after "to" or "in": "the Netherlands", "Italy". */
function countryWith(code) {
  const name = countryName(code);
  return /^(United|Netherlands|Czech|Philippines|Bahamas|Gambia|Maldives|Dominican|Central African|Comoros|Marshall|Solomon)/.test(name) ? `the ${name}` : name;
}

/**
 * Streams and edits for `range`, from the trust & safety log (null without one):
 * { streamed, watched (hours), edits, since (when the log starts) }.
 */
export function tnsStats(facts, range) {
  const t = facts.tns;
  if (!t) return null;
  const inRange = inRangeFn(range);
  let streamed = 0;
  for (const [s, e] of t.streamed) {
    // Split at midnight UTC so a stream over New Year counts in the right years.
    for (let a = s; a < e; ) {
      const b = Math.min(e, (Math.floor(a / DAY) + 1) * DAY);
      if (inRange(yearOfMs(a))) streamed += (b - a) / HOUR;
      a = b;
    }
  }
  const sum = (list) => list.reduce((n, [m, x]) => (inRange(yearOfMonth(m)) ? n + x : n), 0);
  return { streamed, watched: sum(t.watched), edits: sum(t.edits), since: t.since_ms };
}

// ---------- games ----------

/** Games for `range`: from the analytics worker when it has them, otherwise from the main facts. */
export function gameStats(view, facts, games, range) {
  const inRange = inRangeFn(range);
  const g = games?.games ?? facts.games;
  if (!g) return null;
  const analytics = g.source === "analytics";
  const list = g.top
    .map((x) => {
      const hours = analytics ? x.months.reduce((s, [m, h]) => (inRange(yearOfMonth(m)) ? s + h : s), 0) : null;
      return { ...x, rangeHours: hours };
    })
    .filter((x) => range === "all" || (analytics ? x.rangeHours > 0 : inRange(yearOfMs(x.last_ms ?? 0)) || inRange(yearOfMs(x.first_ms ?? 0))))
    .sort((a, b) => (analytics ? b.rangeHours - a.rangeHours : b.sessions - a.sessions));
  const total = analytics ? list.reduce((s, x) => s + x.rangeHours, 0) : null;

  // Playtime per month for the top five games, plus everything else.
  const top = list.slice(0, 5);
  const series = top.map((x, i) => ({ key: x.name, label: x.name, colour: ["var(--cat-1)", "var(--cat-2)", "var(--cat-3)", "var(--cat-4)", "var(--acc-text)"][i] }));
  series.push({ key: "rest", label: "Other games", colour: "var(--t-faint)" });
  const pts = new Map();
  if (analytics) {
    const topNames = new Set(top.map((x) => x.name));
    for (const x of g.top) {
      for (const [m, h] of x.months) {
        const y = yearOfMonth(m);
        if (!inRange(y)) continue;
        const t = Date.UTC(y, m % 12, 1);
        const p = pts.get(t) ?? pts.set(t, { t }).get(t);
        const k = topNames.has(x.name) ? x.name : "rest";
        p[k] = (p[k] ?? 0) + h;
      }
    }
  }
  const points = fillMonths([...pts.values()].sort((a, b) => a.t - b.t), series.map((s) => s.key));
  const clock = analytics ? hourClock(view.tz, g.by_hour ?? [], inRange) : null;
  return { analytics, list, total, series, points, clock, distinct: list.length };
}

// ---------- devices ----------

export function deviceStats(view, facts, range) {
  const d = facts.devices;
  if (!d) return null;
  const inRange = inRangeFn(range);
  const active = (x) => range === "all" || (yearOfMs(x.first_ms) <= range && yearOfMs(x.last_ms) >= range);
  const clients = d.clients.filter(active);
  const keys = ["android", "ios", "desktop", "web", "other"];
  const points = [];
  for (const [m, counts] of d.platforms) {
    const y = yearOfMonth(m);
    if (!inRange(y)) continue;
    const p = { t: Date.UTC(y, m % 12, 1) };
    keys.forEach((k, i) => (p[k] = counts[i]));
    points.push(p);
  }
  const clock = hourClock(view.tz, d.open_hours, inRange);
  const opens = clock.clock.reduce((a, b) => a + b, 0);
  return { clients, points: fillMonths(points, keys), clock, opens, opened_from: d.opened_from };
}

// ---------- purchases ----------

export function purchaseStats(facts, range, until) {
  const m = facts.money;
  const inRange = (ms) => range === "all" || yearOfMs(ms) === range;
  const purchases = m.purchases.filter((p) => inRange(p.ms));
  const spent = new Map(); // currency -> cents
  const perMonth = new Map(); // currency -> Map(month start -> cents)
  for (const p of purchases) {
    if (p.refunded) continue;
    spent.set(p.currency, (spent.get(p.currency) ?? 0) + p.cents);
    const d = new Date(p.ms);
    const t = Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), 1);
    const pm = perMonth.get(p.currency) ?? perMonth.set(p.currency, new Map()).get(p.currency);
    pm.set(t, (pm.get(t) ?? 0) + p.cents);
  }
  // Every month from the first purchase in any currency (or January of the chosen year) to the
  // last, or to the package (or December of the chosen year), so quiet months show as zero and
  // the currencies share one timeline.
  const keys = [...perMonth.values()].flatMap((m) => [...m.keys()]).sort((a, b) => a - b);
  const months = (byMonth) => {
    const end = new Date(range === "all" ? Math.max(keys.at(-1), until ?? 0) : Math.min(Date.UTC(range, 11, 1), until ?? Infinity));
    const out = [];
    for (let t = range === "all" ? keys[0] : Date.UTC(range, 0, 1); t <= end.getTime(); ) {
      out.push({ t, spent: (byMonth.get(t) ?? 0) / 100 });
      const d = new Date(t);
      t = Date.UTC(d.getUTCFullYear(), d.getUTCMonth() + 1, 1);
    }
    return out;
  };
  const boosts = facts.guild_events.filter((e) => e.kind === "boost" && inRange(e.ms)).length;
  return {
    purchases,
    spent: [...spent].sort((a, b) => b[1] - a[1]),
    perMonth: [...perMonth].map(([currency, byMonth]) => ({ currency, points: months(byMonth) })),
    refunded: purchases.filter((p) => p.refunded).length,
    made: m.gifts_made.filter((g) => inRange(g.ms)),
    received: m.gifts_received.filter((g) => inRange(g.ms)),
    entitlements: m.entitlements.filter((g) => inRange(g.ms)),
    boosts,
  };
}
