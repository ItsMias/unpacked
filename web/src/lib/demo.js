// Made-up data for the "Try the demo" button: a fictional account, shaped exactly like the engine's
// output so every page works. Generated from a fixed seed, so it's the same every time. The only
// real ids are public ones (games, Discord Activities, built-in sounds), for their names and art.

const HOUR = 3_600_000;
const DAY = 86_400_000;
const EPOCH = 1_420_070_400_000;

function random(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const snowflake = (ms, n = 0) => ((BigInt(ms - EPOCH) << 22n) + BigInt(n)).toString();
const monthIndex = (ms) => { const d = new Date(ms); return (d.getUTCFullYear() - 2015) * 12 + d.getUTCMonth(); };
const yearSlot = (y) => Math.min(15, Math.max(0, y - 2015));
// Madrid is UTC+2 in summer, UTC+1 otherwise (close enough for made-up data).
const offset = (ms) => { const m = new Date(ms).getUTCMonth(); return m >= 3 && m <= 9 ? 2 : 1; };

const PEOPLE = [
  ["Moth", "mothlight", 9], ["Juniper", "juniper.wav", 7], ["Kit", "kitkat", 5], ["Ren", "renderer", 4],
  ["Pip", "pipsqueak", 3], ["Nova", "novaa", 3], ["Sable", "sable_", 2], ["Ash", "ashfall", 2],
  ["Wren", "wrenegade", 1.5], ["Lumen", "lumen", 1.2], ["Clover", "cloverfield", 1], ["Echo", "echo.echo", 0.8],
];
const SERVERS = [
  ["Midnight Snack Club", ["general", "food-pics", "late-night"], 6], ["Pixel Pals", ["general", "screenshots", "lfg"], 5],
  ["Study Hall", ["general", "focus-room", "homework"], 3], ["Cozy Corner", ["chat", "plants", "music"], 2.5],
  ["Retro Arcade", ["general", "speedruns"], 2], ["Art Dump", ["art", "critique"], 1.6],
  ["Speedrun Society", ["general", "runs"], 1.2], ["Late Night Lo-fi", ["chat", "now-playing"], 1],
  ["Homework Help", ["maths", "science"], 0.6], ["Plant Parents", ["general"], 0.5],
];
const GROUPS = [["the squad", 4], ["movie night", 1.5], ["valheim vikings", 1]];
const GAMES = [
  ["Deep Rock Galactic", "428054228511227914", 380], ["Valheim", "1124358970618953818", 160], ["Celeste", "1402416901551816837", 72],
  ["Rocket League", "356877880938070016", 64], ["Lethal Company", "1167674267748540516", 48], ["Slay the Spire", "1402418606364688549", 36],
  ["Among Us", "1402418440685486130", 14], ["Portal 2", "359508941782122496", 9],
];
const EMOJI = [["😭", 2840], ["💀", 2210], ["😂", 1630], ["❤️", 1320], ["🔥", 890], ["👀", 760], ["🙏", 610], ["✨", 540], ["😔", 480], ["👍", 450], ["🥺", 410], ["💜", 380], ["🎉", 300], ["😎", 240], ["🫠", 210], ["😴", 190], ["🤝", 150], ["🍕", 120], ["🌙", 110], ["🎮", 95]];
const WORDS = ["lol", "yeah", "like", "game", "tonight", "omg", "fr", "same", "wait", "bro", "valheim", "server", "vc", "sleep", "time", "good", "really", "think", "know", "need", "play", "people", "tomorrow", "lmao", "okay", "maybe", "actually", "love", "food", "school", "build", "music", "night", "rn", "idk", "art", "stream", "update", "cute", "help"];
const FILLER = ["the", "i", "you", "to", "it", "and", "is", "a", "that", "of", "in", "my", "for", "this", "on", "me", "but", "so", "just", "with"];
const LINES = [
  "okay so here's the plan for saturday", "we start at the spawn and head east until we hit the big river",
  "someone needs to bring enough food because last time we starved halfway there", "i'll set up the farm while you two go mining",
  "honestly the new update changed everything about how the boss fights work", "if we finish the castle before sunday we should stream it",
  "i made a list of every biome we still need to find", "the portal is on the left side of the base, not the right",
  "don't forget we promised to help with the art server event too", "also the music bot keeps leaving the call for some reason",
  "i rewrote the rules channel so it actually makes sense now", "anyway that's everything, tell me if i missed something",
  "the speedrun route is faster if you skip the second village", "we should really back up the world before trying anything risky",
];

const pick = (r, list) => list[Math.floor(r() * list.length)];

// Made-up counts for the Statistics page, sized for an account of this age.
const EVENTS = {
  send_message: 487_000, message_edited: 2_900, message_deleted: 610, reply_message_started: 6_200, pin_message: 140,
  add_reaction: 4_500, remove_reaction: 380, attachment_upload_finished: 5_200, message_sent_with_gif: 900, sticker_attached: 120,
  message_link_copied: 310, application_command_used: 1_400,
  join_voice_channel: 3_100, start_call: 420, join_call: 380, ring_call: 450, screenshare_finished: 260, input_mute_toggled: 5_400,
  self_deafen_toggled: 310, voice_channel_effect_sent: 45,
  guild_viewed: 61_000, guild_joined: 140, leave_guild: 96, create_guild: 6, delete_guild: 2, create_channel: 85, channel_deleted: 40,
  channel_updated: 160, guild_role_updated: 420, guild_settings_updated: 75, moderation_action: 40, create_instant_invite: 120,
  copy_instant_invite: 64, invite_sent: 210, accepted_instant_invite: 150, guild_bot_added: 12, webhook_created: 4, create_emoji: 38,
  delete_emoji: 9, create_sticker: 5, soundboard_sound_uploaded: 6, thread_creation_started: 70, member_list_viewed: 24_000,
  view_as_roles_selected: 30,
  friends_list_viewed: 2_600, dm_list_viewed: 15_800, friend_suggestion_skipped: 9_400, message_request_action: 22,
  add_channel_recipient: 14, remove_channel_recipient: 3, block_user_confirmed: 3,
  app_opened: 22_590, app_background: 19_800, app_native_crash: 46, search_started: 6_100, keyboard_shortcut_used: 8_400,
  keyboard_mode_toggled: 900, link_clicked: 3_900, notification_clicked: 4_100, notification_settings_updated: 160, open_popout: 31_000,
  open_modal: 7_800, change_log_opened: 30, premium_upsell_viewed: 1_900,
  login_successful: 48, login_attempted: 52, captcha_served: 18, update_user_settings: 1_300, custom_status_updated: 64,
  user_avatar_updated: 11, oauth2_authorize_accepted: 85, application_created: 2, data_request_initiated: 2, email_sent: 60,
  payment_flow_started: 40, payment_flow_canceled: 18, gift_code_created: 2, gift_code_copied: 2,
};

function paragraph(r, words) {
  let s = "";
  while (s.length < words) s += `${pick(r, LINES)}${r() < 0.35 ? "\n\n" : ". "}`;
  return s.slice(0, words).trim();
}

let cached = null;

/** `{ result, games }` as App.svelte gets them from the workers, plus `demo: true`. */
export function demoResult() {
  return (cached ??= build());
}

function build() {
  const r = random(424242);
  const created = Date.UTC(2019, 10, 3, 15, 20);
  const start = Date.UTC(2020, 2, 14);
  const end = Date.UTC(2026, 8, 20);
  const me = snowflake(created, 7);

  const people = PEOPLE.map(([name, username], i) => ({
    id: snowflake(Date.UTC(2017 + (i % 5), i, 3 + i), i + 1),
    username,
    display_name: name,
    avatar: null,
    friend: i < 10,
    relationship: i < 10 ? 1 : 0,
    nickname: i === 0 ? "mothy" : null,
    tag: i % 3 === 0 ? { guild: "0", tag: ["MOTH", "PXL", "LATE", "ARC"][i / 3], badge: null } : null,
    decoration: null,
  }));
  const servers = SERVERS.map(([name], i) => ({ id: snowflake(Date.UTC(2018, i, 10), 30 + i), name, name_known: true, invites: [] }));

  // Channels, each with a weight and a stretch of time it's active in.
  const channels = [];
  people.forEach((p, i) => {
    const from = start + Math.floor(r() * 900) * DAY;
    channels.push({ id: snowflake(from, 100 + i), kind: "dm", guild: null, person: p.id, name: `Direct Message with ${p.username}#0`, w: PEOPLE[i][2], from, to: end - Math.floor(r() * 200) * DAY });
  });
  GROUPS.forEach(([name, w], i) => channels.push({ id: snowflake(start + i * 200 * DAY, 200 + i), kind: "group", guild: null, person: null, name, w, from: start + i * 200 * DAY, to: end }));
  SERVERS.forEach(([sname, chans, w], si) => {
    chans.forEach((c, ci) => {
      const from = start + Math.floor(r() * 1200) * DAY;
      channels.push({ id: snowflake(from, 300 + si * 10 + ci), kind: "guild", guild: servers[si].id, person: null, name: `${c}, ${sname}`, w: w / chans.length, from, to: si === 5 ? from + 500 * DAY : end });
    });
  });

  // Messages per day: a slow rise, a dip, a busy 2024, with weekends and summers busier. Each day's
  // messages are spread over the hours (Madrid time) and over the chats active that day.
  const hourly = [0.2, 0.1, 0.05, 0.03, 0.02, 0.02, 0.05, 0.2, 0.35, 0.45, 0.55, 0.6, 0.7, 0.75, 0.8, 0.9, 1, 1.1, 1.2, 1.35, 1.5, 1.6, 1.3, 0.7];
  const hourSum = hourly.reduce((a, b) => a + b, 0);
  const alive = new Map(), deleted = new Map(), lost = new Map();
  const months = new Map(); // channel id -> Map(month -> n)
  const first = new Map(), last = new Map();
  const yearly = new Map();
  for (let t = start; t <= end; t += DAY) {
    if (r() < 0.1) continue;
    const yf = (t - start) / (365 * DAY);
    const trend = 35 + 150 * Math.min(1, yf / 2.2) - (yf > 2.7 && yf < 3.6 ? 55 : 0) + (yf > 3.8 && yf < 4.9 ? 70 : 0);
    const d = new Date(t);
    const weekend = d.getUTCDay() === 0 || d.getUTCDay() === 6 ? 1.3 : 1;
    const summer = d.getUTCMonth() >= 5 && d.getUTCMonth() <= 7 ? 1.2 : 1;
    const want = trend * weekend * summer * Math.exp((r() - 0.5) * 1.1);
    let n = 0;
    for (let h = 0; h < 24; h++) {
      const k = Math.round((want * hourly[h]) / hourSum + (r() - 0.5));
      if (k <= 0) continue;
      n += k;
      const hi = Math.floor((t + (h - offset(t)) * HOUR) / HOUR);
      alive.set(hi, (alive.get(hi) ?? 0) + k);
      if (r() < 0.12) deleted.set(hi, (deleted.get(hi) ?? 0) + 1 + Math.floor(r() * 2));
      // A server that was deleted later took these with it.
      if (yf > 0.8 && yf < 2.6 && r() < 0.6) lost.set(hi, (lost.get(hi) ?? 0) + Math.round(k * 0.45));
    }
    if (!n) continue;
    yearly.set(d.getUTCFullYear(), (yearly.get(d.getUTCFullYear()) ?? 0) + n);
    // The day's messages over the chats active that day, by weight with some noise; the rounding
    // leftovers go to the busiest one so the chats add up to the day.
    const active = channels.filter((c) => t >= c.from && t <= c.to).map((c) => [c, c.w * (0.4 + r() * 1.2)]);
    const wsum = active.reduce((s, a) => s + a[1], 0);
    const takes = active.map(([, w]) => Math.floor((n * w) / wsum));
    takes[takes.indexOf(Math.max(...takes))] += n - takes.reduce((a, b) => a + b, 0);
    const mi = monthIndex(t), ms = t + 15 * HOUR;
    active.forEach(([c], i) => {
      if (!takes[i]) return;
      const m = months.get(c.id) ?? months.set(c.id, new Map()).get(c.id);
      m.set(mi, (m.get(mi) ?? 0) + takes[i]);
      if (!first.has(c.id)) first.set(c.id, ms);
      last.set(c.id, ms);
    });
  }
  const sorted = (m) => [...m].sort((a, b) => a[0] - b[0]);
  const outChannels = channels.map((c) => ({
    id: c.id, kind: c.kind, guild: c.guild, person: c.person, name: c.name,
    months: sorted(months.get(c.id) ?? new Map()), first_ms: first.get(c.id) ?? 0, last_ms: last.get(c.id) ?? 0,
  }));
  const dmChannels = outChannels.filter((c) => c.kind === "dm");

  // Voice: evenings, more often on weekends.
  const vdays = [], vhours = new Map(), vplaces = new Map(), longest = [];
  const callPlaces = [...servers.slice(0, 4).map((s) => ["guild", s.id, 3]), ...dmChannels.slice(0, 4).map((c, i) => ["channel", c.id, 3 - i * 0.5])];
  for (let t = start + 60 * DAY; t <= end; t += DAY) {
    if (r() > 0.42) continue;
    const hours = Math.min(11, 0.3 + r() * r() * 7);
    const sessions = 1 + Math.floor(r() * 3);
    vdays.push([Math.floor(t / DAY), hours, sessions]);
    const x = r();
    const local = x < 0.6 ? 19 + Math.floor(r() * 3) : x < 0.8 ? 14 + Math.floor(r() * 4) : x < 0.95 ? 22 + Math.floor(r() * 2) : 10 + Math.floor(r() * 3);
    const startHour = local - offset(t);
    let left = hours, h = startHour;
    while (left > 0) { const part = Math.min(1, left); const hi = Math.floor(t / HOUR) + h; vhours.set(hi, (vhours.get(hi) ?? 0) + part); left -= part; h++; }
    const [kind, id] = pick(r, callPlaces);
    const pm = vplaces.get(id) ?? vplaces.set(id, { kind, id, months: new Map() }).get(id);
    const mi = monthIndex(t);
    pm.months.set(mi, (pm.months.get(mi) ?? 0) + hours);
    if (hours > 6) longest.push({ start_ms: t + startHour * HOUR, end_ms: t + startHour * HOUR + hours * HOUR, kind, id });
  }
  longest.sort((a, b) => b.end_ms - b.start_ms - (a.end_ms - a.start_ms));
  const voiceHours = vdays.reduce((s, d) => s + d[1], 0);

  // What the messages say, per year slot.
  const zero = () => new Array(16).fill(0);
  const words = zero(), chars = zero(), att = zero(), links = zero(), lengths = Array.from({ length: 16 }, () => [0, 0, 0, 0, 0, 0, 0, 0]);
  const shares = [0.03, 0.22, 0.12, 0.27, 0.23, 0.09, 0.03, 0.01];
  for (const [y, n] of yearly) {
    const s = yearSlot(y);
    words[s] = Math.round(n * 5.9); chars[s] = Math.round(n * 31); att[s] = Math.round(n * 0.03); links[s] = Math.round(n * 0.02);
    lengths[s] = shares.map((f) => Math.round(n * f));
  }
  // Word counts per year: `rate` uses of the top word per message, tailing off down the list.
  const counted = (list, rate) => list.map((w, i) => {
    const years = zero();
    for (const [y, n] of yearly) years[yearSlot(y)] = Math.round(((n * rate) / (i + 1.5)) * (0.75 + r() * 0.5));
    return { text: w, years };
  });

  const firstDm = dmChannels[1];
  const longMsgs = [];
  for (let i = 0; i < 20; i++) {
    const c = pick(r, outChannels.filter((x) => x.months.length));
    const ms = start + Math.floor(r() * (end - start));
    const text = paragraph(r, 600 + Math.floor(r() * 1400));
    longMsgs.push({ ms, channel: c.id, text, chars: [...text].length });
  }
  longMsgs.sort((a, b) => b.chars - a.chars);
  const walls = [0, 1, 2].map((i) => {
    const c = outChannels.find((x) => x.kind === "group") ?? outChannels[0];
    const t0 = Date.UTC(2023 + i, 4 + i, 12, 20, 15);
    const parts = Array.from({ length: 5 + i * 2 }, (_, j) => [t0 + j * 140_000, paragraph(r, 250 + Math.floor(r() * 300))]);
    return { channel: c.id, start_ms: parts[0][0], end_ms: parts.at(-1)[0], count: parts.length, chars: parts.reduce((s, p) => s + p[1].length, 0), parts };
  }).sort((a, b) => b.chars - a.chars);

  // Games, with playtime per month.
  const gameTop = GAMES.map(([name, id, hours], i) => {
    const from = monthIndex(start) + Math.floor(r() * 30), to = Math.min(monthIndex(end), from + 20 + Math.floor(r() * 40));
    const ms = [];
    for (let m = from; m <= to; m++) ms.push([m, (hours / (to - from + 1)) * (0.4 + r() * 1.2)]);
    return {
      name, id, sessions: Math.round(hours * 1.6), hours,
      months: ms, first_ms: Date.UTC(2015 + Math.floor(from / 12), from % 12, 3), last_ms: Date.UTC(2015 + Math.floor(to / 12), to % 12, 20),
    };
  });
  // Playtime by hour: afternoons on weekends, evenings and the odd late night, a few hours at a time.
  const gameHours = new Map();
  for (const [d, h] of vdays) {
    const x = r();
    const local = x < 0.55 ? 19 + Math.floor(r() * 3) : x < 0.85 ? 13 + Math.floor(r() * 4) : 22 + Math.floor(r() * 3);
    let left = h * (0.6 + r() * 0.5);
    for (let hi = d * 24 + local - 2; left > 0; hi++) {
      const part = Math.min(1, left);
      gameHours.set(hi, (gameHours.get(hi) ?? 0) + part);
      left -= part;
    }
  }

  const joinEvents = servers.map((s, i) => ({ ms: start + i * 70 * DAY, kind: "join", guild: s.id, detail: i % 3 ? "invite" : "invite - vanity" }));
  const otherEvents = [];
  for (let i = 0; i < 90; i++) {
    const ms = start + Math.floor(r() * (end - start));
    otherEvents.push({ ms, kind: r() < 0.5 ? "join" : "leave", guild: snowflake(ms, 900 + i), detail: null });
  }
  for (let i = 0; i < 40; i++) otherEvents.push({ ms: start + Math.floor(r() * (end - start)), kind: "mod", guild: servers[1].id, detail: pick(r, ["add_role", "add_role", "kick", "ban", "timeout", "change_nickname"]) });
  [2, 0, 3].forEach((si, i) => otherEvents.push({ ms: Date.UTC(2024 + (i % 2), 2 + i * 3, 9), kind: "boost", guild: servers[si].id, detail: null }));
  otherEvents.push({ ms: Date.UTC(2021, 6, 1), kind: "create", guild: servers[3].id, detail: null });

  const friendEvents = [];
  for (let i = 0; i < 240; i++) friendEvents.push([start + Math.floor(r() * (end - start)), pick(r, [1, 1, 1, 0, 0, 3, 4]), r() < 0.5]);
  friendEvents.sort((a, b) => a[0] - b[0]);

  const guildCounts = [];
  for (let t = start; t <= end; t += 9 * DAY) {
    const yf = (t - start) / (365 * DAY);
    guildCounts.push([t, Math.round(4 + Math.min(1, yf / 4) * 70 - (yf > 5 ? 18 : 0) + (r() - 0.5) * 4)]);
  }

  const platforms = [];
  for (let m = monthIndex(start); m <= monthIndex(end); m++) {
    const f = (m - monthIndex(start)) / 78;
    platforms.push([m, [Math.round(800 + 600 * f + r() * 200), 0, Math.round(120 + 260 * f + r() * 60), Math.round(200 * (1 - f) + r() * 40), 0]]);
  }
  const openHours = [...alive].filter(() => r() < 0.5).map(([h, n]) => [h, Math.max(1, Math.round(n / 6))]);
  const totalAlive = [...alive.values()].reduce((a, b) => a + b, 0);

  // Friend counts, as Discord logs them when the friends list is opened (from mid-2022).
  const friendCounts = [];
  let friendsNow = 52;
  for (let t = Date.UTC(2022, 7, 20); t <= end; t += (2 + Math.floor(r() * 6)) * DAY) {
    friendsNow = Math.max(20, friendsNow + Math.round((r() - 0.35) * 3) - (t > Date.UTC(2025, 9, 1) && t < Date.UTC(2025, 9, 9) ? 6 : 0));
    friendCounts.push([t, friendsNow]);
  }

  // The trust & safety log's two years: Go Live streams, watching others, edits.
  const tnsSince = Date.UTC(2024, 8, 27);
  const streamed = [], watchedMonths = new Map(), editMonths = new Map();
  for (let t = tnsSince; t <= end; t += DAY) {
    const mi = monthIndex(t);
    if (r() < 0.22) {
      const start = t + (19 + Math.floor(r() * 3) - offset(t)) * HOUR + Math.floor(r() * 50) * 60_000;
      streamed.push([start, start + Math.round((0.4 + r() * r() * 3) * HOUR)]);
    }
    if (r() < 0.5) watchedMonths.set(mi, (watchedMonths.get(mi) ?? 0) + 0.3 + r() * 2.2);
    editMonths.set(mi, (editMonths.get(mi) ?? 0) + Math.floor(r() * 9));
  }

  const year = (y, m, d) => Date.UTC(y, m, d, 18);
  const facts = {
    profile: {
      id: me, username: "you", display_name: "You", avatar: null, created_ms: created,
      flags: ["HYPESQUAD_ONLINE_HOUSE_1"], premium_until: "2026-12-01T00:00:00+00:00", premium_since: null,
      boosting_since: "2025-02-11T10:00:00+00:00", legacy_username: "You#4242", orbs: 680, friends: 84, blocked: 3,
      widgets: [{ kind: "played_games", games: GAMES.slice(0, 4).map(([, id]) => ({ id, tags: [], comment: null })) }],
    },
    time_zone: "Europe/Madrid",
    alive_hours: sorted(alive), deleted_hours: sorted(deleted), lost_hours: sorted(lost), has_send_log: true,
    channels: outChannels,
    people, servers,
    voice: {
      hours: voiceHours, sessions: vdays.reduce((s, d) => s + d[2], 0), days: vdays, hours_by_hour: sorted(vhours),
      places: [...vplaces.values()].map((p) => ({ kind: p.kind, id: p.id, months: sorted(p.months) })), longest: longest.slice(0, 10),
    },
    emojis: { total: 16_900, in_messages: 12_400, reactions: 4_500, top: EMOJI.map(([text, count]) => ({ text, name: null, id: null, animated: false, count })) },
    games: null,
    quests_completed: 2,
    text: {
      words, chars, with_attachments: att, with_links: links, lengths,
      top_words: counted(WORDS, 0.1), filler_words: counted(FILLER, 0.5),
      top_domains: counted(["tenor.com", "youtube.com", "discord.com", "twitter.com", "github.com", "reddit.com", "twitch.tv", "spotify.com"], 0.008),
      first: { ms: (firstDm.first_ms || start) - DAY * 3, channel: firstDm.id, text: "heyyy 👋 is this the right server for the valheim build?", chars: 57 },
      longest: longMsgs, walls,
    },
    tags: [
      { ms: year(2024, 4, 2), kind: "adopted", guild: servers[1].id, tag: "PXL", badge: "HEART", colours: ["#ff5fa2", "#ffc0dc"] },
      { ms: year(2025, 1, 14), kind: "adopted", guild: servers[0].id, tag: "SNAK", badge: "FIRE", colours: ["#ff7a45", "#ffc9a8"] },
      { ms: year(2025, 9, 30), kind: "adopted", guild: servers[3].id, tag: "COZY", badge: "WATER_DROP", colours: ["#6d5dfc", "#c9c2ff"] },
      { ms: year(2025, 1, 2), kind: "created", guild: servers[3].id, tag: "COZY", badge: "WATER_DROP", colours: ["#6d5dfc", "#c9c2ff"] },
    ],
    guild_events: [...joinEvents, ...otherEvents].sort((a, b) => a.ms - b.ms),
    friend_events: friendEvents,
    data_requests: [year(2023, 4, 2), year(2026, 8, 18)],
    guild_counts: guildCounts,
    friend_counts: friendCounts,
    devices: {
      clients: [
        { platform: "android", device: "beyond1", os: "Android", browser: "Discord Android", count: 31_200, first_ms: Date.UTC(2021, 3, 2), last_ms: end },
        { platform: "desktop", device: "", os: "Windows", browser: "Discord Client", count: 9_400, first_ms: start, last_ms: end },
        { platform: "android", device: "a52q", os: "Android", browser: "Discord Android", count: 6_100, first_ms: start, last_ms: Date.UTC(2021, 3, 1) },
        { platform: "web", device: "", os: "Windows", browser: "Chrome", count: 2_300, first_ms: start, last_ms: Date.UTC(2024, 1, 1) },
        { platform: "web", device: "", os: "Mac OS X", browser: "Safari", count: 180, first_ms: Date.UTC(2023, 5, 1), last_ms: Date.UTC(2023, 7, 1) },
      ],
      platforms,
      trips: [
        ["HR", Date.UTC(2021, 7, 14), 1, 38], ["PL", Date.UTC(2022, 6, 9), 6, 210], ["HU", Date.UTC(2023, 3, 7), 3, 96],
        ["DK", Date.UTC(2024, 6, 20), 12, 640], ["HR", Date.UTC(2025, 7, 2), 9, 380],
      ].map(([country, start, days, sessions]) => ({ country, start_ms: start, end_ms: start + days * DAY - 1, days, sessions })),
      countries: [["ES", 41_000], ["HR", 820], ["PL", 410], ["HU", 260], ["DK", 140]].map(([name, count], i) => ({ name, count, first_ms: start + i * 200 * DAY, last_ms: end - i * 90 * DAY })),
      cities: [["Valencia, ES", 30_000], ["Zaragoza, ES", 9_000], ["Zagreb, HR", 800], ["Kraków, PL", 400]].map(([name, count], i) => ({ name, count, first_ms: start + i * 150 * DAY, last_ms: end - i * 120 * DAY })),
      isps: [{ name: "Example Fibre", count: 30_000, first_ms: start, last_ms: end }, { name: "Example Mobile", count: 9_000, first_ms: start, last_ms: end }],
      time_zones: [{ name: "Europe/Madrid", count: 41_000, first_ms: start, last_ms: end }],
      opened_from: [["launcher", 18_400], ["notification", 4_100], ["deeplink", 90]],
      open_hours: openHours, opens: openHours.reduce((s, x) => s + x[1], 0), sessions: 41_000,
    },
    money: {
      purchases: [
        ...[2023, 2024, 2025, 2026].map((y) => ({ ms: Date.UTC(y, 2, 6), sku: "nitro", title: "Nitro (yearly)", currency: "EUR", cents: 9999, refunded: false, gift: false, kind: "subscription" })),
        { ms: Date.UTC(2024, 9, 31), sku: "shop", title: "Midnight Bundle", currency: "EUR", cents: 499, refunded: false, gift: false, kind: "sku" },
        { ms: Date.UTC(2024, 11, 24), sku: "nitro", title: "Nitro (1 month)", currency: "EUR", cents: 999, refunded: false, gift: true, kind: "subscription" },
        { ms: Date.UTC(2025, 1, 14), sku: "nitro", title: "Nitro (1 month)", currency: "EUR", cents: 999, refunded: false, gift: true, kind: "subscription" },
        { ms: Date.UTC(2025, 6, 2), sku: "shop", title: "Pixel Cat Decoration", currency: "EUR", cents: 399, refunded: true, gift: false, kind: "sku" },
        { ms: Date.UTC(2026, 4, 19), sku: "shop", title: "Starfall Profile Effect", currency: "EUR", cents: 599, refunded: false, gift: false, kind: "sku" },
      ],
      gifts_made: [
        { ms: Date.UTC(2024, 11, 24), title: "Nitro", sent_ms: Date.UTC(2024, 11, 24, 1), channel: dmChannels[0].id, guild: null },
        { ms: Date.UTC(2025, 1, 14), title: "Nitro", sent_ms: Date.UTC(2025, 1, 14, 1), channel: dmChannels[1].id, guild: null },
      ],
      gifts_received: [{ ms: Date.UTC(2022, 11, 25), title: "Nitro" }, { ms: Date.UTC(2023, 7, 9), title: "Nitro Basic" }],
      entitlements: [{ ms: Date.UTC(2021, 4, 2), title: "Free game weekend" }],
    },
    // Blazing 8s, SpellCast, Ask Away, Color Together.
    activities: [["832025144389533716", 48, Date.UTC(2024, 0, 3), end], ["852509694341283871", 22, Date.UTC(2022, 1, 1), Date.UTC(2025, 1, 1)], ["976052223358406656", 15, Date.UTC(2022, 9, 1), Date.UTC(2026, 3, 1)], ["1039835161136746497", 9, Date.UTC(2023, 1, 1), Date.UTC(2026, 6, 1)]],
    account: {
      email: "you@example.com", phone: null, ip: "203.0.113.42", verified: true, has_mobile: true, date_of_birth: null,
      age_assurance: { inferred_age_group: "adult", method: null }, predicted_age: null, predicted_gender: null, temp_banned_until: null,
      sessions: [
        { created_ms: Date.UTC(2026, 4, 3), last_used_ms: end, os: "Android", platform: "Discord Android", ip: "203.0.113.42", mfa: true },
        { created_ms: Date.UTC(2025, 10, 9), last_used_ms: end - 4 * DAY, os: "Windows", platform: "Discord Client", ip: "198.51.100.7", mfa: true },
      ],
      connections: [{ kind: "spotify", name: "you", visible: true, verified: true }, { kind: "steam", name: "you_plays", visible: true, verified: true }, { kind: "github", name: "you-codes", visible: false, verified: true }],
      notes: [[people[0].id, "met in the valheim server, likes moths obviously"], [people[3].id, "owes me a pizza"]],
      server_settings: 64, muted_servers: 21,
      privacy: { detectPlatformAccounts: true, contactSyncEnabled: false, passwordless: true, allowAccessibilityDetection: false },
      app_stats: GAMES.slice(0, 6).map(([, id, h]) => ({ id, seconds: Math.round(h * 3600 * 1.4), first_ms: start, last_ms: end })),
      library: [],
    },
    expressions: {
      favorite_gifs: [], stickers: [],
      sounds: [["1", 12], ["2", 9], ["6", 5], ["4", 3], ["5", 2], ["3", 1]], sounds_span: [Date.UTC(2026, 6, 10), Date.UTC(2026, 8, 18)],
      favorite_sounds: ["1", "2", "6"], commands: [], apps: [],
    },
    ad_profile: {
      age_group: "18-24", reg_country_code: "ES", reg_region: "Southern Europe", primary_platform_l30: "mobile", has_active_subscription: true,
      maid_state: "gaid_available", theme_names_l90: ["RACING", "SPORTS"], mobile_genre_names: ["Mobile Gamer > Word", "Mobile Gamer > Card"],
      movie_and_tv_genre_names: ["Movie and TV > Documentary", "Movie and TV > Drama"], music_and_audio_genre_names: ["Music and Audio > Jazz", "Music and Audio > K-Pop"],
      custom_audiences: ["mobile_install_puzzle", "1234567890"], game_ids_l730: GAMES.slice(0, 5).map(([, id]) => id),
    },
    poker: { games_played: 40, games_won: 17, all_ins: 9, biggest_pot: 640, most_chips: 520, level: 5 },
    tns: {
      streamed,
      watched: [...watchedMonths].sort((a, b) => a[0] - b[0]),
      edits: [...editMonths].sort((a, b) => a[0] - b[0]),
      two_factor: [[Date.UTC(2024, 10, 3, 20), true]],
      calls_started: streamed.slice(0, 40).map(([a]) => a - 600_000),
      emoji_created: [Date.UTC(2024, 11, 2), Date.UTC(2025, 3, 19), Date.UTC(2025, 9, 30)],
      since_ms: tnsSince,
    },
  };

  const games = {
    state: "done", tags: [], events: 4_210_000,
    types: [["channel_opened", 380_000], ["app_ui_viewed", 210_000], ["push_notification_received", 96_000], ["running_game_heartbeat", 18_000], ...Object.entries(EVENTS)],
    games: { source: "analytics", distinct: GAMES.length, hours: GAMES.reduce((s, g) => s + g[2], 0), top: gameTop, by_hour: sorted(gameHours) },
  };

  return {
    demo: true,
    games,
    result: {
      facts,
      avatar: null,
      avatars: [],
      icons: {},
      hasAnalytics: true,
      sources: {
        channels: outChannels.length,
        serversNow: 64,
        serverIds: servers.map((s) => s.id),
        packageDate: end,
        folders: {
          reporting: { size: 2.3 * 2 ** 30, events: 1_900_000, read: true, types: [["send_message", Math.round(totalAlive * 1.08)], ["channel_opened", 210_000], ["session_start", 41_000], ["add_reaction", 4_500]] },
          tns: { size: 1.1 * 2 ** 30, events: 640_000, read: true, types: [["message_edited", 2_900], ["video_stream_ended", 1_100], ["experiment_user_triggered", 9_800]] },
          analytics: { size: 5.2 * 2 ** 30, events: 0, read: false },
          modeling: { size: 1.9 * 2 ** 30, events: 0, read: false },
        },
      },
    },
  };
}
