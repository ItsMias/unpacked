// Discord image URLs and the few public lookups the page makes. Everything here asks Discord's
// CDN or public API for pictures and names; nothing from the package is sent except ids.

const API = "https://discord.com/api/v9";
const CDN = "https://cdn.discordapp.com";

export const avatarUrl = (id, hash, size = 64) =>
  hash ? `${CDN}/avatars/${id}/${hash}.${hash.startsWith("a_") ? "gif" : "png"}?size=${size}` : null;
export const decorationUrl = (asset) => (asset ? `${CDN}/avatar-decoration-presets/${asset}.png?size=96&passthrough=true` : null);
export const tagBadgeUrl = (guild, badge) => (badge ? `${CDN}/guild-tag-badges/${guild}/${badge}.png?size=32` : null);
export const guildIconUrl = (id, hash) => (hash ? `${CDN}/icons/${id}/${hash}.${hash.startsWith("a_") ? "gif" : "webp"}?size=96` : null);
export const emojiUrl = (id, animated) => `${CDN}/emojis/${id}.${animated ? "gif" : "png"}?size=48`;
export const appIconUrl = (id, hash) => (hash ? `${CDN}/app-icons/${id}/${hash}.png?size=128` : null);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function getJson(url, retries = 2) {
  const r = await fetch(url, { credentials: "omit", referrerPolicy: "no-referrer" });
  if (r.status === 429 && retries > 0) {
    // Rate limited: wait as long as Discord asks (capped), then try again.
    const wait = Number((await r.json().catch(() => ({}))).retry_after ?? r.headers.get("retry-after") ?? 1);
    await sleep(Math.min(5000, Math.max(250, wait * 1000)));
    return getJson(url, retries - 1);
  }
  if (!r.ok) throw new Error(`${r.status}`);
  return r.json();
}

// ---------- games and apps ----------

const apps = new Map(); // id -> Promise<{ name, icon, cover } | null>

// At most four lookups at a time, to stay friendly with Discord's rate limits.
let running = 0;
const waiting = [];
async function limited(fn) {
  if (running >= 4) await new Promise((r) => waiting.push(r));
  running++;
  try { return await fn(); } finally { running--; waiting.shift()?.(); }
}

/**
 * Name, icon and (for Steam games) portrait cover art of a Discord application, cached for the
 * session. Bots and activities are applications too.
 */
export function appInfo(id) {
  if (!/^\d+$/.test(id ?? "")) return Promise.resolve(null);
  if (!apps.has(id)) {
    apps.set(id, limited(() => getJson(`${API}/applications/${encodeURIComponent(id)}/rpc`))
      .then((a) => {
        const steam = a.third_party_skus?.find((x) => x.distributor === "steam" && /^\d+$/.test(x.id ?? ""));
        return {
          name: a.name,
          icon: appIconUrl(a.id, a.icon),
          cover: steam ? `https://cdn.cloudflare.steamstatic.com/steam/apps/${steam.id}/library_600x900.jpg` : null,
        };
      })
      .catch(() => null));
  }
  return apps.get(id);
}

export const stickerUrl = (id, ext = "png") => `https://media.discordapp.net/stickers/${id}.${ext}?size=160`;
export const soundUrl = (id) => `${CDN}/soundboard-sounds/${id}`;

// ---------- servers ----------

const MAX_INVITE_LOOKUPS = 6;
const servers = new Map(); // guild id -> Promise<{ icon?, name? }>
const resolved = new Map(); // any guild an invite resolved to -> { icon, name }
let queue = Promise.resolve();

/**
 * Icon (and name, when the package doesn't have it) for a server. Servers you manage have their
 * icon in the package (`local`); others are looked up through invites you joined with, sent or
 * posted, one server at a time to stay under Discord's rate limit.
 */
export function serverInfo(server, local) {
  if (servers.has(server.id)) return servers.get(server.id);
  let p;
  if (local) p = Promise.resolve({ icon: URL.createObjectURL(local) });
  else if (!server.invites?.length) p = Promise.resolve({});
  else {
    p = queue = queue.then(async () => {
      for (const code of server.invites.slice(0, MAX_INVITE_LOOKUPS)) {
        if (resolved.has(server.id)) break;
        try {
          const inv = await getJson(`${API}/invites/${encodeURIComponent(code)}?with_counts=false`);
          if (inv.guild?.id) resolved.set(inv.guild.id, { icon: guildIconUrl(inv.guild.id, inv.guild.icon), name: inv.guild.name });
        } catch { /* expired, or rate limited */ }
      }
      return resolved.get(server.id) ?? {};
    }).catch(() => ({}));
  }
  servers.set(server.id, p);
  return p;
}
