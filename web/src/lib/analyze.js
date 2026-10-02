// Reads a data package. Three jobs, run in three workers at once so the page can show up before
// the slow parts are done: `analyze` (account, messages, activity log), `analyzeGames` (the
// optional analytics folder: game history and server tags, by far the biggest file) and
// `analyzeTns` (what only the trust & safety log has: streams, edits, two-factor…).
// The WASM module must already be initialised.
//
// Both report progress through `onProgress(update)`. Updates carry a `type`:
//   start    { packageDate, bytes, channels, hasAnalytics }
//   account  { name, username, created, avatar }      (as soon as the account file is read)
//   progress { stage, done, total, counts, days?, activity? }
//     days:     Uint32Array [utcDay, messages, …], or [utcDay, minutes played, …] for games
//     activity: Uint32Array [utcDay, minutes in voice, deleted messages, …]
import { WasmEngine, wanted } from "./pkg/engine.js";
import { listEntries, openEntry, readEntry } from "./zip.js";

// Must match KINDS in engine/src/wasm.rs.
const K = { User: 0, MessagesIndex: 1, ServersIndex: 2, Quests: 3, ChannelMeta: 4, ChannelMessages: 5, Events: 6, EventsFallback: 7, Analytics: 8, Traits: 9, Poker: 10 };

const AVATAR = /^Account\/avatar\.(png|jpe?g|gif|webp)$/i;
// Past avatars, named by a snowflake id that carries the date they were replaced.
const OLD_AVATAR = /^Account\/recent_avatars\/(\d+)\.(png|jpe?g|gif|webp)$/i;
// Server icons are only in the package for servers you manage.
const SERVER_ICON = /^Servers\/(\d+)\/icon\.(png|jpe?g|gif|webp)$/i;
const SERVER = /^Servers\/(\d+)\/guild\.json$/i;
const ACTIVITY = /^Activity\/([a-z]+)\//i;
const IMAGE_TYPES = { png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg", gif: "image/gif", webp: "image/webp" };

const imageType = (name) => IMAGE_TYPES[name.split(".").pop().toLowerCase()];
const snowflakeMs = (id) => Number(BigInt(id) >> 22n) + 1_420_070_400_000;

/** Calls `fn` at most every `ms`, plus whenever `force` is set. */
function throttle(ms, fn) {
  let last = 0;
  return (force, ...args) => {
    const now = Date.now();
    if (force || now - last >= ms) { last = now; fn(...args); }
  };
}

export async function analyze(blob, onProgress = () => {}) {
  const t0 = Date.now();
  const entries = await listEntries(blob);

  const singles = [], events = [], fallback = [], analytics = [];
  const channels = new Map(); // folder -> { meta, messages }
  const iconEntries = new Map();
  const folders = {}; // activity folder -> { size, events, read }
  let avatarEntry = null, packageDate = 0;
  const serverIds = [];
  const oldAvatars = [];

  for (const e of entries) {
    packageDate = Math.max(packageDate, e.modified);
    const act = ACTIVITY.exec(e.name);
    if (act && e.size) {
      const f = (folders[act[1].toLowerCase()] ??= { size: 0, events: 0, read: false });
      f.size += e.size;
    }
    if (AVATAR.test(e.name)) { avatarEntry = e; continue; }
    const old = OLD_AVATAR.exec(e.name);
    if (old) { oldAvatars.push({ id: old[1], e }); continue; }
    const srv = SERVER.exec(e.name);
    if (srv) serverIds.push(srv[1]);
    const icon = SERVER_ICON.exec(e.name);
    if (icon) { iconEntries.set(icon[1], e); continue; }
    const kind = wanted(e.name);
    if (kind < 0) continue;
    if (kind === K.Events) events.push(e);
    else if (kind === K.EventsFallback) fallback.push(e);
    else if (kind === K.Analytics) analytics.push(e);
    else if (kind === K.ChannelMeta || kind === K.ChannelMessages) {
      const folder = e.name.slice(0, e.name.lastIndexOf("/"));
      const slot = channels.get(folder) ?? {};
      slot[kind === K.ChannelMeta ? "meta" : "messages"] = e;
      channels.set(folder, slot);
    } else singles.push({ e, kind });
  }
  if (!singles.some((s) => s.kind === K.User) && channels.size === 0) {
    throw new Error("This zip doesn't look like a Discord data package. It has no Account or Messages folder.");
  }
  const pairs = [...channels.values()].filter((c) => c.meta && c.messages);
  onProgress({ type: "start", packageDate, bytes: blob.size, channels: pairs.length, hasAnalytics: analytics.length > 0 });

  const read = async (e) => new Blob([await readEntry(blob, e)], { type: imageType(e.name) });
  const avatar = avatarEntry ? await read(avatarEntry).catch(() => null) : null;

  const smallBytes = singles.reduce((n, s) => n + s.e.compressedSize, 0) +
    pairs.reduce((n, c) => n + c.meta.compressedSize + c.messages.compressedSize, 0);
  const total = smallBytes + events.reduce((n, e) => n + e.compressedSize, 0);
  let done = 0, stage = "account", channelsDone = 0;

  const engine = new WasmEngine();
  const counts = () => ({ ...JSON.parse(engine.progress()), channelsDone, channelsTotal: pairs.length });
  const report = throttle(120, () => onProgress({ type: "progress", stage, done, total, counts: counts() }));
  const reportDays = throttle(350, () => onProgress({ type: "progress", stage, done, total, counts: counts(), days: engine.day_counts() }));
  const reportActivity = throttle(350, () => onProgress({ type: "progress", stage, done, total, counts: counts(), activity: engine.activity_days() }));

  // 1. Account, indexes, quests. The user file goes first so the rest can refer to it.
  singles.sort((a, b) => (a.kind === K.User ? -1 : b.kind === K.User ? 1 : 0));
  for (const { e, kind } of singles) {
    const bytes = await readEntry(blob, e);
    engine.feed_file(kind, bytes);
    done += e.compressedSize;
    if (kind === K.User) {
      try {
        const u = JSON.parse(new TextDecoder().decode(bytes));
        onProgress({ type: "account", name: u.global_name || u.username, username: u.username, created: snowflakeMs(u.id), avatar });
      } catch { /* the engine reports a broken file */ }
    }
  }

  // 2. Message folders, a few at a time. These must all be in before the activity log.
  stage = "messages";
  report(true);
  let next = 0;
  const reader = async () => {
    while (next < pairs.length) {
      const c = pairs[next++];
      const [meta, msgs] = await Promise.all([readEntry(blob, c.meta), readEntry(blob, c.messages)]);
      try { engine.feed_channel(meta, msgs); } catch { /* skip a malformed folder */ }
      done += c.meta.compressedSize + c.messages.compressedSize;
      channelsDone++;
      reportDays(false);
    }
  };
  await Promise.all(Array.from({ length: 8 }, reader));
  reportDays(true);

  // 3. Activity logs, streamed.
  stage = "activity";
  const stream = async (list, folder, feed) => {
    for (const e of list) {
      const r = (await openEntry(blob, e, (n) => { done += n; report(false); reportActivity(false); })).getReader();
      for (;;) {
        const { value, done: end } = await r.read();
        if (end) break;
        feed(value);
      }
      folders[folder].events += engine.finish_events();
      const types = new Map(folders[folder].types ?? []);
      for (const [t, n] of JSON.parse(engine.take_event_types())) types.set(t, (types.get(t) ?? 0) + n);
      folders[folder].types = [...types].sort((a, b) => b[1] - a[1]);
      folders[folder].read = true;
    }
  };
  await stream(events, "reporting", (c) => engine.feed_events(c));
  if (!engine.has_voice() && fallback.length) await stream(fallback, "tns", (c) => engine.feed_events(c));
  reportActivity(true);

  stage = "done";
  report(true);
  const facts = JSON.parse(engine.finish());

  const avatars = [];
  for (const { id, e } of oldAvatars) {
    try { avatars.push({ id, ms: snowflakeMs(id), blob: await read(e) }); } catch { /* skip */ }
  }
  const icons = {};
  for (const [id, e] of iconEntries) {
    try { icons[id] = await read(e); } catch { /* the page falls back to initials */ }
  }

  return {
    facts,
    avatar,
    avatars,
    icons,
    hasAnalytics: analytics.length > 0,
    sources: { channels: pairs.length, folders, serversNow: serverIds.length, serverIds, packageDate },
    seconds: (Date.now() - t0) / 1000,
  };
}

/**
 * Game history and server tags from the analytics folder.
 * Resolves to `null` when the package doesn't have one.
 */
export async function analyzeGames(blob, onProgress = () => {}) {
  const analytics = (await listEntries(blob)).filter((e) => wanted(e.name) === K.Analytics);
  if (!analytics.length) return null;
  const total = analytics.reduce((n, e) => n + e.compressedSize, 0);
  let done = 0, events = 0;
  const engine = new WasmEngine();
  const report = throttle(250, () => onProgress({ type: "progress", stage: "games", done, total, counts: JSON.parse(engine.progress()) }));
  const reportDays = throttle(500, () => onProgress({ type: "progress", stage: "games", done, total, counts: JSON.parse(engine.progress()), days: engine.game_days() }));
  for (const e of analytics) {
    const r = (await openEntry(blob, e, (n) => { done += n; report(false); reportDays(false); })).getReader();
    for (;;) {
      const { value, done: end } = await r.read();
      if (end) break;
      engine.feed_analytics(value);
    }
    events += engine.finish_events();
  }
  const types = JSON.parse(engine.take_event_types());
  const facts = JSON.parse(engine.finish());
  return { games: facts.games, tags: facts.tags, events, types };
}

/**
 * What only the trust & safety log has: Go Live streams, edited messages, two-factor changes,
 * calls you started, emoji you uploaded. The rest of that log repeats the reporting one.
 * Resolves to `null` when the package doesn't have one.
 */
export async function analyzeTns(blob, onProgress = () => {}) {
  const files = (await listEntries(blob)).filter((e) => wanted(e.name) === K.EventsFallback);
  if (!files.length) return null;
  const total = files.reduce((n, e) => n + e.compressedSize, 0);
  let done = 0, events = 0;
  const engine = new WasmEngine();
  const report = throttle(250, () => onProgress({ type: "progress", stage: "tns", done, total, counts: JSON.parse(engine.progress()) }));
  for (const e of files) {
    const r = (await openEntry(blob, e, (n) => { done += n; report(false); })).getReader();
    for (;;) {
      const { value, done: end } = await r.read();
      if (end) break;
      engine.feed_events(value);
    }
    events += engine.finish_events();
  }
  const types = JSON.parse(engine.take_event_types());
  const facts = JSON.parse(engine.finish());
  return { tns: facts.tns, events, types };
}
