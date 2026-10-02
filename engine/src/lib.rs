//! Unpacked engine: turns a Discord data package into all-time, time-bucketed facts.
//!
//! The caller (CLI or browser worker) owns the zip. It asks [`wanted`] which entries to read,
//! passes small files to [`Engine::feed_file`] / [`Engine::feed_channel`] and streams the big
//! events files through [`Engine::feed_events`] / [`Engine::feed_analytics`] in chunks of any
//! size. Nothing here does I/O.
//!
//! Output is bucketed (messages per hour, channels per month, voice per day) rather than summed,
//! so the page can filter by year or time zone without reading the package again.

pub mod account;
pub mod devices;
pub mod emoji;
pub mod games;
pub mod money;
pub mod text;
pub mod time;
pub mod tns;
pub mod voice;
#[cfg(target_arch = "wasm32")]
mod wasm;

use std::collections::{BTreeMap, HashMap, HashSet};

use memchr::memmem::Finder;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use emoji::Emoji;
use voice::VoiceKey;

/// What a zip entry is used for.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Entry {
    User,
    MessagesIndex,
    ServersIndex,
    Quests,
    ChannelMeta,
    ChannelMessages,
    /// `Activity/reporting/events-*.json`: always in the package, streamed.
    Events,
    /// `Activity/tns/events-*.json`: only read when `reporting` is missing.
    EventsFallback,
    /// `Activity/analytics/events-*.json`: only in packages of people who opted in to analytics.
    Analytics,
    /// `Ads/traits.json`: what Discord's ad system has inferred about you.
    Traits,
    /// `Activities/*/poker/poker.json`: Poker Night's own stats.
    Poker,
}

/// Classifies a zip entry path. Anything returning `None` is never opened, which includes
/// the `modeling` folder.
pub fn wanted(path: &str) -> Option<Entry> {
    let p = path.to_ascii_lowercase();
    let file = p.rsplit('/').next().unwrap_or("");
    match p.as_str() {
        "account/user.json" => return Some(Entry::User),
        "messages/index.json" => return Some(Entry::MessagesIndex),
        "servers/index.json" => return Some(Entry::ServersIndex),
        "ads/quests_user_status.json" => return Some(Entry::Quests),
        "ads/traits.json" => return Some(Entry::Traits),
        _ => {}
    }
    if p.starts_with("activities/") && p.ends_with("/poker/poker.json") {
        return Some(Entry::Poker);
    }
    let is_events = file.starts_with("events-") && file.ends_with(".json");
    if p.starts_with("activity/reporting/") && is_events {
        return Some(Entry::Events);
    }
    if p.starts_with("activity/tns/") && is_events {
        return Some(Entry::EventsFallback);
    }
    if p.starts_with("activity/analytics/") && is_events {
        return Some(Entry::Analytics);
    }
    if p.starts_with("messages/") && p.matches('/').count() == 2 {
        return match file {
            "channel.json" => Some(Entry::ChannelMeta),
            "messages.json" => Some(Entry::ChannelMessages),
            _ => None,
        };
    }
    None
}

// ---------- output ----------

#[derive(Serialize, Debug)]
pub struct Facts {
    pub profile: Option<Profile>,
    /// Most common `time_zone` on your sent messages: the default for day and hour charts.
    pub time_zone: Option<String>,
    /// Messages still in the package, as `[UTC hour index, count]`, sorted.
    pub alive_hours: Vec<(u32, u32)>,
    /// Sends logged in the activity log whose message is gone but whose channel is still in the
    /// package: deleted by you or a moderator.
    pub deleted_hours: Vec<(u32, u32)>,
    /// Logged sends whose whole channel is gone (deleted server, channel or DM).
    pub lost_hours: Vec<(u32, u32)>,
    /// Whether the activity log had any `send_message` events (if not, deleted/lost are unknown).
    pub has_send_log: bool,
    pub channels: Vec<ChannelFacts>,
    /// DM partners and friends, for names and avatars.
    pub people: Vec<PersonInfo>,
    pub servers: Vec<ServerInfo>,
    pub voice: Option<VoiceFacts>,
    pub emojis: EmojiStats,
    pub games: Option<GameStats>,
    pub quests_completed: u64,
    pub text: text::TextStats,
    /// Server tags: ones you wore (analytics only) and ones you set up for servers you run.
    pub tags: Vec<TagEvent>,
    /// Server joins, leaves, creations, boosts, moderation and bots, oldest first.
    pub guild_events: Vec<GuildEvent>,
    /// Friend list changes: `[ms, type, you started it]`. Type 1 = became friends, 0 = removed,
    /// 2 = blocked, 3 = request received, 4 = request sent. The log doesn't say who.
    pub friend_events: Vec<(i64, u8, bool)>,
    /// When you asked Discord for your data.
    pub data_requests: Vec<i64>,
    /// How many servers you were in, as Discord logged it on joins, invites and friend changes:
    /// `[ms, servers]`, oldest first.
    pub guild_counts: Vec<(i64, u16)>,
    /// How many friends you had, as Discord logged it when you opened the friends list:
    /// `[ms, friends]`, oldest first (only since mid-2022).
    pub friend_counts: Vec<(i64, u32)>,
    pub devices: Option<devices::DeviceFacts>,
    pub money: money::MoneyFacts,
    /// Discord Activities you joined: `[application id, sessions, first ms, last ms]`.
    pub activities: Vec<(String, u32, i64, i64)>,
    pub account: Option<account::AccountInfo>,
    pub expressions: Option<account::Expressions>,
    /// `Ads/traits.json` as is.
    pub ad_profile: Option<Value>,
    /// Poker Night's `global_stats`.
    pub poker: Option<Value>,
    /// What only the trust & safety log has (streams, edits, two-factor…), when it was read.
    pub tns: Option<tns::TnsFacts>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TagEvent {
    pub ms: i64,
    /// `adopted` (you wore it) or `created` (you set it up for a server).
    pub kind: &'static str,
    pub guild: String,
    pub tag: Option<String>,
    /// Badge shape, like `HEART` or `SKULL`.
    pub badge: Option<String>,
    pub colours: [Option<String>; 2],
}

#[derive(Serialize, Debug)]
pub struct GuildEvent {
    pub ms: i64,
    /// `join`, `leave`, `create`, `delete`, `boost`, `unboost`, `mod` or `bot`.
    pub kind: &'static str,
    pub guild: String,
    /// join: how (`invite`, `invite - vanity`, `direct - discovery`…); mod: the action; bot: its id.
    pub detail: Option<String>,
}

/// Live counters while the package is being read, for the loading screen.
#[derive(Serialize, Debug)]
pub struct Progress {
    pub channels: usize,
    pub messages: u64,
    pub events: u64,
    pub sends: usize,
    pub deleted: u64,
    pub lost: u64,
    pub voice: usize,
}

#[derive(Serialize, Debug, Default)]
pub struct Profile {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar: Option<String>,
    /// Account creation, from the user id.
    pub created_ms: i64,
    pub flags: Vec<String>,
    pub premium_until: Option<String>,
    pub premium_since: Option<String>,
    pub boosting_since: Option<String>,
    pub legacy_username: Option<String>,
    pub orbs: Option<i64>,
    pub friends: u32,
    pub blocked: u32,
    /// Profile widgets, like the game collection.
    pub widgets: Vec<Widget>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Widget {
    /// `played_games`, `current_games`, `want_to_play_games`, `favorite_games`…
    pub kind: String,
    pub games: Vec<WidgetGame>,
}

#[derive(Serialize, Debug, Clone)]
pub struct WidgetGame {
    pub id: String,
    pub tags: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ChannelFacts {
    pub id: String,
    /// `dm`, `group`, `guild` or `other`.
    pub kind: &'static str,
    pub guild: Option<String>,
    /// The other person in a DM.
    pub person: Option<String>,
    /// From `Messages/index.json` (`#general` style names are not included there for DMs).
    pub name: Option<String>,
    /// `[month index, messages]`, see [`time::month_index`].
    pub months: Vec<(u16, u32)>,
    /// First and last message still in the package, Unix ms.
    pub first_ms: i64,
    pub last_ms: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct PersonInfo {
    pub id: String,
    pub username: String,
    pub display_name: String,
    /// Avatar hash for `cdn.discordapp.com/avatars/{id}/{hash}`.
    pub avatar: Option<String>,
    pub friend: bool,
    /// 1 friend, 2 blocked, 3 incoming request, 4 outgoing request, 0 none.
    pub relationship: u8,
    /// The nickname you gave them.
    pub nickname: Option<String>,
    /// Their server tag.
    pub tag: Option<PersonTag>,
    /// Avatar decoration asset, for `cdn.discordapp.com/avatar-decoration-presets/{asset}.png`.
    pub decoration: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PersonTag {
    pub guild: String,
    pub tag: String,
    /// Badge image hash, for `cdn.discordapp.com/guild-tag-badges/{guild}/{badge}.png`.
    pub badge: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    /// False when the package doesn't name the server (deleted, or left without messages).
    pub name_known: bool,
    /// Invite codes, best first, for looking up the icon.
    pub invites: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct VoiceFacts {
    pub hours: f64,
    pub sessions: usize,
    /// `[UTC day number, hours, sessions started]`.
    pub days: Vec<(u32, f32, u16)>,
    /// `[UTC hour index, hours]`, for time-of-day charts.
    pub hours_by_hour: Vec<(u32, f32)>,
    pub places: Vec<VoicePlace>,
    /// Longest single sessions in one place, longest first.
    pub longest: Vec<VoiceSession>,
}

#[derive(Serialize, Debug)]
pub struct VoiceSession {
    pub start_ms: i64,
    pub end_ms: i64,
    pub kind: &'static str,
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct VoicePlace {
    /// `guild` or `channel` (DM and group calls).
    pub kind: &'static str,
    pub id: String,
    /// `[month index, hours]`.
    pub months: Vec<(u16, f32)>,
}

#[derive(Serialize, Debug)]
pub struct EmojiStats {
    pub total: u64,
    pub in_messages: u64,
    pub reactions: u64,
    pub top: Vec<EmojiCount>,
}

#[derive(Serialize, Debug)]
pub struct EmojiCount {
    /// Unicode text, or `None` for custom emoji.
    pub text: Option<String>,
    pub name: Option<String>,
    pub id: Option<String>,
    pub animated: bool,
    pub count: u64,
}

#[derive(Serialize, Debug)]
pub struct GameStats {
    /// `"analytics"` (full history) or `"voice"` (only games running while in voice).
    pub source: &'static str,
    pub distinct: usize,
    pub hours: Option<f64>,
    pub top: Vec<GameCount>,
    /// `[UTC hour index, hours played]`, analytics only.
    pub by_hour: Vec<(u32, f32)>,
}

#[derive(Serialize, Debug)]
pub struct GameCount {
    pub name: String,
    pub id: Option<String>,
    pub sessions: usize,
    pub hours: Option<f64>,
    pub months: Vec<(u16, f32)>,
    pub first_ms: Option<i64>,
    pub last_ms: Option<i64>,
}

// ---------- input shapes ----------

#[derive(Deserialize)]
struct RawUser {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar_hash: Option<String>,
    #[serde(default)]
    relationships: Vec<RawRelationship>,
    #[serde(default)]
    flags: Value,
    premium_until: Option<String>,
    user_profile_metadata: Option<RawProfileMeta>,
    current_orbs_balance: Option<i64>,
}

#[derive(Deserialize)]
struct RawProfileMeta {
    boosting_started_at: Option<String>,
    premium_started_at: Option<String>,
    legacy_username: Option<String>,
    #[serde(default)]
    widgets: Value,
}

#[derive(Deserialize)]
struct RawRelationship {
    #[serde(rename = "type", default)]
    kind: Value,
    nickname: Option<String>,
    user: RawFriend,
}

#[derive(Deserialize)]
struct RawFriend {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
    #[serde(default)]
    primary_guild: Value,
    #[serde(default)]
    avatar_decoration_data: Value,
}

#[derive(Deserialize)]
struct RawChannel {
    id: String,
    #[serde(rename = "type")]
    kind: Value,
    guild: Option<RawGuild>,
    #[serde(default)]
    recipients: Vec<String>,
}

#[derive(Deserialize)]
struct RawGuild {
    id: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct RawMessage {
    /// A number in current packages, a string in some older ones.
    #[serde(rename = "ID", default, deserialize_with = "id_any")]
    id: Option<u64>,
    #[serde(rename = "Timestamp")]
    timestamp: String,
    #[serde(rename = "Contents", default)]
    contents: String,
    /// A space-separated string of URLs in current packages.
    #[serde(rename = "Attachments", default)]
    attachments: Value,
}

fn id_any<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

/// Invites kept per server.
const INVITES_PER_SERVER: usize = 8;

#[derive(Deserialize)]
struct RawEvent {
    event_type: String,
    timestamp: Option<String>,
    event_id: Option<String>,
    guild_id: Option<Value>,
    channel_id: Option<Value>,
    duration: Option<Value>,
    duration_connected_ms: Option<Value>,
    game_name: Option<String>,
    emoji_name: Option<Value>,
    guild: Option<Value>,
    invite: Option<Value>,
    invite_guild_id: Option<Value>,
    invite_code: Option<Value>,
    game: Option<String>,
    game_id: Option<Value>,
    application_id: Option<Value>,
    application_name: Option<String>,
    game_session_id: Option<String>,
    duration_tracked_ms: Option<Value>,
    activity_duration_s: Option<Value>,
    emoji_id: Option<Value>,
    emoji_animated: Option<Value>,
    join_method: Option<String>,
    join_type: Option<String>,
    action_type: Option<String>,
    #[serde(rename = "type")]
    rel_type: Option<Value>,
    is_initiator: Option<Value>,
    tag: Option<String>,
    new_tag: Option<String>,
    badge_name: Option<String>,
    badge_color_primary: Option<String>,
    badge_color_secondary: Option<String>,
    payment_id: Option<Value>,
    amount: Option<Value>,
    presentment_amount: Option<Value>,
    currency: Option<String>,
    presentment_currency: Option<String>,
    sku_id: Option<Value>,
    store_title: Option<String>,
    is_gift: Option<Value>,
    payment_type: Option<String>,
    gift_code: Option<String>,
    to_step: Option<String>,
    created_at: Option<String>,
    price: Option<Value>,
    user_guilds: Option<Value>,
    num_friends: Option<Value>,
    participant_type: Option<String>,
}

#[derive(Deserialize)]
struct RawQuest {
    completed_at: Option<String>,
}

fn as_str(v: &Option<Value>) -> Option<String> {
    match v.as_ref()? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn as_i64(v: &Option<Value>) -> Option<i64> {
    match v.as_ref()? {
        Value::String(s) => s.parse::<f64>().ok().map(|f| f as i64),
        Value::Number(n) => n.as_f64().map(|f| f as i64),
        _ => None,
    }
}

/// The string value of `"key":"value"` in a raw event line, found by its needle (`"key":"`).
/// Much cheaper than parsing the whole line, which matters for the ~500k `send_message` events.
fn raw_field<'a>(line: &'a [u8], needle: &Finder) -> Option<&'a [u8]> {
    let start = needle.find(line)? + needle.needle().len();
    let rest = &line[start..];
    // Timestamps are double-quoted: "timestamp":"\"2025-01-01T00:00:00Z\"".
    let rest = rest.strip_prefix(b"\\\"").unwrap_or(rest);
    let end = memchr::memchr2(b'"', b'\\', rest)?;
    Some(&rest[..end])
}

/// Event type of a line, read from its first key (where Discord always puts it).
fn event_type(line: &[u8]) -> Option<&[u8]> {
    let rest = line.strip_prefix(b"{\"event_type\":\"")?;
    Some(&rest[..memchr::memchr(b'"', rest)?])
}

/// Badge names for the numeric `flags` some packages use instead of a list of names.
const FLAG_BITS: &[(u32, &str)] = &[
    (0, "STAFF"),
    (1, "PARTNER"),
    (2, "HYPESQUAD"),
    (3, "BUG_HUNTER_LEVEL_1"),
    (6, "HYPESQUAD_ONLINE_HOUSE_1"),
    (7, "HYPESQUAD_ONLINE_HOUSE_2"),
    (8, "HYPESQUAD_ONLINE_HOUSE_3"),
    (9, "PREMIUM_EARLY_SUPPORTER"),
    (14, "BUG_HUNTER_LEVEL_2"),
    (17, "VERIFIED_DEVELOPER"),
    (18, "CERTIFIED_MODERATOR"),
    (22, "ACTIVE_DEVELOPER"),
];

fn flag_names(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect(),
        Value::Number(n) => {
            let bits = n.as_u64().unwrap_or(0);
            FLAG_BITS.iter().filter(|(b, _)| bits & (1 << b) != 0).map(|(_, n)| (*n).to_owned()).collect()
        }
        _ => vec![],
    }
}

fn widgets(v: &Value) -> Vec<Widget> {
    let Some(list) = v.as_array() else { return vec![] };
    list.iter()
        .filter_map(|w| {
            let data = w.get("data")?;
            let games = data.get("games")?.as_array()?;
            Some(Widget {
                kind: data.get("type")?.as_str()?.to_owned(),
                games: games
                    .iter()
                    .filter_map(|g| {
                        Some(WidgetGame {
                            id: g.get("game_id")?.as_str()?.to_owned(),
                            tags: g.get("tags").and_then(Value::as_array).map_or(vec![], |t| {
                                t.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()
                            }),
                            comment: g.get("comment").and_then(Value::as_str).map(str::to_owned),
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn person_tag(v: &Value) -> Option<PersonTag> {
    if v.get("identity_enabled").and_then(Value::as_bool) == Some(false) {
        return None;
    }
    Some(PersonTag {
        guild: v.get("identity_guild_id")?.as_str()?.to_owned(),
        tag: v.get("tag")?.as_str()?.to_owned(),
        badge: v.get("badge").and_then(Value::as_str).map(str::to_owned),
    })
}

/// Relationship type: 1 friend, 2 blocked (named in some packages, numbered in others).
fn relationship(v: &Value) -> u8 {
    match v {
        Value::String(s) if s == "FRIEND" => 1,
        Value::String(s) if s == "BLOCKED" => 2,
        Value::Number(n) => n.as_u64().unwrap_or(0) as u8,
        _ => 0,
    }
}

// ---------- engine ----------

enum ChannelKind {
    Dm(Vec<String>),
    Group,
    Guild(String),
    /// A server channel or thread whose server the package doesn't say (left or deleted).
    UnknownGuild,
    Other,
}

struct Channel {
    kind: ChannelKind,
    months: BTreeMap<u16, u32>,
    first: i64,
    last: i64,
}

pub struct Engine {
    user: Option<RawUser>,
    channel_names: HashMap<String, String>,
    server_names: HashMap<String, String>,
    channels: HashMap<String, Channel>,

    alive_ids: HashSet<u64>,
    alive_hours: HashMap<u32, u32>,
    sent_ids: HashSet<u64>,
    deleted_hours: HashMap<u32, u32>,
    lost_hours: HashMap<u32, u32>,
    time_zones: HashMap<String, u32>,

    /// Guild id -> (from the activity log?, timestamp, invite code).
    invites: HashMap<String, Vec<(bool, i64, String)>>,
    emoji_counts: HashMap<Emoji, u64>,
    emojis_in_messages: u64,
    reactions: u64,

    voice: voice::Voice,
    games: games::Games,
    quests_completed: u64,
    text: text::Text,
    tags: Vec<TagEvent>,
    guild_events: Vec<GuildEvent>,
    friend_events: Vec<(i64, u8, bool)>,
    data_requests: Vec<i64>,
    guild_counts: Vec<(i64, u16)>,
    friend_counts: Vec<(i64, u32)>,
    devices: devices::Devices,
    money: money::Money,
    activities: HashMap<String, (u32, i64, i64)>,
    account: Option<account::AccountInfo>,
    expressions: Option<account::Expressions>,
    ad_profile: Option<Value>,
    poker: Option<Value>,
    tns: tns::Tns,
    /// Events per type in the file being read, for [`Self::take_event_types`].
    type_counts: HashMap<Vec<u8>, u64>,
    /// Running totals for [`Self::progress`].
    msgs_alive: u64,
    deleted_total: u64,
    lost_total: u64,
    events_done: u64,

    carry: Vec<u8>,
    carry_is_analytics: bool,
    lines: u64,
    finders: Vec<Finder<'static>>,
    analytics_finders: Vec<Finder<'static>>,
    invite_finder: Finder<'static>,
    f_message_id: Finder<'static>,
    f_channel: Finder<'static>,
    f_timestamp: Finder<'static>,
    f_time_zone: Finder<'static>,
    f_browser: Finder<'static>,
    f_os: Finder<'static>,
    f_device: Finder<'static>,
    f_country: Finder<'static>,
    f_city: Finder<'static>,
    f_isp: Finder<'static>,
    f_opened_from: Finder<'static>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let needles: [&'static [u8]; 6] = [
            b"leave_voice_channel",
            b"voice_disconnect",
            b"\"game_name\"",
            b"add_reaction",
            b"accepted_instant_invite",
            b"\"invite_sent\"",
        ];
        let analytics_needles: [&'static [u8]; 4] = [
            b"\"launch_game\"",
            b"\"running_game_heartbeat\"",
            b"\"application_closed\"",
            b"\"user_adopted_guild_identity\"",
        ];
        Self {
            user: None,
            channel_names: HashMap::new(),
            server_names: HashMap::new(),
            channels: HashMap::new(),
            alive_ids: HashSet::new(),
            alive_hours: HashMap::new(),
            sent_ids: HashSet::new(),
            deleted_hours: HashMap::new(),
            lost_hours: HashMap::new(),
            time_zones: HashMap::new(),
            invites: HashMap::new(),
            emoji_counts: HashMap::new(),
            emojis_in_messages: 0,
            reactions: 0,
            voice: voice::Voice::default(),
            games: games::Games::default(),
            quests_completed: 0,
            text: text::Text::default(),
            tags: Vec::new(),
            guild_events: Vec::new(),
            friend_events: Vec::new(),
            data_requests: Vec::new(),
            guild_counts: Vec::new(),
            friend_counts: Vec::new(),
            devices: devices::Devices::default(),
            money: money::Money::default(),
            activities: HashMap::new(),
            account: None,
            expressions: None,
            ad_profile: None,
            poker: None,
            tns: tns::Tns::default(),
            type_counts: HashMap::new(),
            msgs_alive: 0,
            deleted_total: 0,
            lost_total: 0,
            events_done: 0,
            carry: Vec::new(),
            carry_is_analytics: false,
            lines: 0,
            finders: needles.iter().map(|n| Finder::new(*n)).collect(),
            analytics_finders: analytics_needles.iter().map(|n| Finder::new(*n)).collect(),
            invite_finder: Finder::new(b"discord"),
            f_message_id: Finder::new(b"\"message_id\":\""),
            f_channel: Finder::new(b"\"channel\":\""),
            f_timestamp: Finder::new(b"\"timestamp\":\""),
            f_time_zone: Finder::new(b"\"time_zone\":\""),
            f_browser: Finder::new(b"\"browser\":\""),
            f_os: Finder::new(b"\"os\":\""),
            f_device: Finder::new(b"\"device\":\""),
            f_country: Finder::new(b"\"country_code\":\""),
            f_city: Finder::new(b"\"city\":\""),
            f_isp: Finder::new(b"\"isp\":\""),
            f_opened_from: Finder::new(b"\"opened_from\":\""),
        }
    }

    /// Feeds a single small file (user, indexes, quests). Channel files go through [`Self::feed_channel`].
    pub fn feed_file(&mut self, kind: Entry, bytes: &[u8]) -> Result<(), String> {
        let err = |e: serde_json::Error| format!("{kind:?}: {e}");
        match kind {
            Entry::User => {
                self.user = Some(serde_json::from_slice(bytes).map_err(err)?);
                let raw: Value = serde_json::from_slice(bytes).map_err(err)?;
                self.account = Some(account::account(&raw));
                self.expressions = Some(account::expressions(&raw));
            }
            Entry::Traits => self.ad_profile = Some(serde_json::from_slice(bytes).map_err(err)?),
            Entry::Poker => {
                let v: Value = serde_json::from_slice(bytes).map_err(err)?;
                self.poker = v.get("global_stats").cloned().or(Some(v));
            }
            Entry::MessagesIndex => {
                let m: HashMap<String, Option<String>> = serde_json::from_slice(bytes).map_err(err)?;
                self.channel_names = m.into_iter().filter_map(|(k, v)| Some((k, v?))).collect();
            }
            Entry::ServersIndex => {
                self.server_names = serde_json::from_slice(bytes).map_err(err)?;
            }
            Entry::Quests => {
                let q: Vec<RawQuest> = serde_json::from_slice(bytes).map_err(err)?;
                self.quests_completed = q.iter().filter(|q| q.completed_at.is_some()).count() as u64;
            }
            _ => return Err(format!("{kind:?} is not a single file")),
        }
        Ok(())
    }

    /// Feeds one `Messages/c<id>/` folder: its `channel.json` and `messages.json`.
    pub fn feed_channel(&mut self, channel_json: &[u8], messages_json: &[u8]) -> Result<(), String> {
        let ch: RawChannel = serde_json::from_slice(channel_json).map_err(|e| format!("channel.json: {e}"))?;
        let msgs: Vec<RawMessage> =
            serde_json::from_slice(messages_json).map_err(|e| format!("messages.json in {}: {e}", ch.id))?;

        let type_is = |name: &str, n: i64| ch.kind.as_str() == Some(name) || ch.kind.as_i64() == Some(n);
        let kind = if let Some(g) = &ch.guild {
            if let Some(name) = &g.name {
                self.server_names.entry(g.id.clone()).or_insert_with(|| name.clone());
            }
            ChannelKind::Guild(g.id.clone())
        } else if type_is("DM", 1) {
            ChannelKind::Dm(ch.recipients.clone())
        } else if type_is("GROUP_DM", 3) {
            ChannelKind::Group
        } else if is_server_channel(&ch.kind) {
            ChannelKind::UnknownGuild
        } else {
            ChannelKind::Other
        };

        let mut months: BTreeMap<u16, u32> = BTreeMap::new();
        let (mut first, mut last) = (i64::MAX, i64::MIN);
        for m in &msgs {
            // The id is a snowflake, so it carries the exact send time; the Timestamp string is the fallback.
            let t = match m.id {
                Some(id) => time::snowflake_ms(id),
                None => match time::parse_ms(&m.timestamp) {
                    Some(t) => t,
                    None => continue,
                },
            };
            if let Some(id) = m.id {
                self.alive_ids.insert(id);
            }
            *self.alive_hours.entry(time::hour_index(t)).or_default() += 1;
            *months.entry(time::month_index(t)).or_default() += 1;
            self.msgs_alive += 1;
            (first, last) = (first.min(t), last.max(t));
            let has_attachment = match &m.attachments {
                Value::String(s) => !s.trim().is_empty(),
                Value::Array(a) => !a.is_empty(),
                _ => false,
            };
            let year = time::civil_from_days(t.div_euclid(time::DAY_MS)).0;
            self.text.add(year, t, &ch.id, &m.contents, has_attachment);

            if let ChannelKind::Guild(g) = &kind {
                if self.invite_finder.find(m.contents.as_bytes()).is_some() {
                    let list = self.invites.entry(g.clone()).or_default();
                    for code in invite_codes(&m.contents) {
                        list.push((false, t, code.to_owned()));
                    }
                }
            }
            // Each emoji counts once per message, so "<:heart:><:heart:><:heart:>" spam doesn't dominate.
            let mut in_this: Vec<Emoji> = vec![];
            emoji::for_each(&m.contents, |e| {
                if !in_this.contains(&e) {
                    in_this.push(e);
                }
            });
            self.emojis_in_messages += in_this.len() as u64;
            for e in in_this {
                *self.emoji_counts.entry(e).or_default() += 1;
            }
        }
        self.text.end_channel(&ch.id);
        self.channels.insert(ch.id, Channel { kind, months, first, last });
        Ok(())
    }

    /// Streams part of a `reporting` / `tns` events file. Chunks may split lines anywhere.
    /// Must come after every message folder, so sends can be matched against them.
    pub fn feed_events(&mut self, chunk: &[u8]) {
        self.stream(chunk, false);
    }

    /// Streams part of an `analytics` events file (game history only).
    pub fn feed_analytics(&mut self, chunk: &[u8]) {
        self.stream(chunk, true);
    }

    fn stream(&mut self, chunk: &[u8], analytics: bool) {
        let mut buf = std::mem::take(&mut self.carry);
        buf.extend_from_slice(chunk);
        let mut start = 0;
        for nl in memchr::memchr_iter(b'\n', &buf) {
            self.line(&buf[start..nl], analytics);
            start = nl + 1;
        }
        buf.drain(..start);
        self.carry = buf;
        self.carry_is_analytics = analytics;
    }

    fn line(&mut self, line: &[u8], analytics: bool) {
        if line.is_empty() {
            return;
        }
        self.lines += 1;
        if let Some(t) = event_type(line) {
            match self.type_counts.get_mut(t) {
                Some(n) => *n += 1,
                None => {
                    self.type_counts.insert(t.to_vec(), 1);
                }
            }
        }
        if analytics { self.analytics_line(line) } else { self.event_line(line) }
    }

    /// Call after the last chunk of each events file. Returns how many events the file had.
    pub fn finish_events(&mut self) -> u64 {
        let rest = std::mem::take(&mut self.carry);
        self.line(&rest, self.carry_is_analytics);
        let n = std::mem::take(&mut self.lines);
        self.events_done += n;
        n
    }

    /// Events per type since the last call (call it after each file), most common first.
    pub fn take_event_types(&mut self) -> Vec<(String, u64)> {
        let mut v: Vec<(String, u64)> = std::mem::take(&mut self.type_counts)
            .into_iter()
            .map(|(k, n)| (String::from_utf8_lossy(&k).into_owned(), n))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// Live counters for the loading screen.
    pub fn progress(&self) -> Progress {
        Progress {
            channels: self.channels.len(),
            messages: self.msgs_alive,
            events: self.events_done + self.lines,
            sends: self.sent_ids.len(),
            deleted: self.deleted_total,
            lost: self.lost_total,
            voice: self.voice.len(),
        }
    }

    /// Messages per UTC day so far, flattened as `[day, count, day, count, …]`, for the loading
    /// screen's live calendar.
    pub fn day_counts(&self) -> Vec<u32> {
        let mut days: HashMap<u32, u32> = HashMap::new();
        for (&h, &n) in &self.alive_hours {
            *days.entry(h / 24).or_default() += n;
        }
        days.into_iter().flat_map(|(d, n)| [d, n]).collect()
    }

    /// Voice minutes and deleted messages (by you, a mod, or with their channel) per UTC day so
    /// far, flattened as `[day, minutes, deleted, …]`, for the loading screen while the activity
    /// log is read.
    pub fn activity_days(&self) -> Vec<u32> {
        let mut days: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
        for (d, h) in self.voice.day_hours() {
            days.entry(d).or_default().0 = (h * 60.0).round() as u32;
        }
        for (&h, &n) in self.deleted_hours.iter().chain(&self.lost_hours) {
            days.entry(h / 24).or_default().1 += n;
        }
        days.into_iter().flat_map(|(d, (m, g))| [d, m, g]).collect()
    }

    /// Playtime minutes per UTC day so far, flattened as `[day, minutes, …]`, for the loading
    /// screen while the analytics log is read.
    pub fn game_days(&self) -> Vec<u32> {
        self.games.day_hours().into_iter().flat_map(|(d, h)| [d, (h * 60.0).round() as u32]).collect()
    }

    fn analytics_line(&mut self, line: &[u8]) {
        if !self.analytics_finders.iter().any(|f| f.find(line).is_some()) {
            return;
        }
        let Ok(ev) = serde_json::from_slice::<RawEvent>(line) else { return };
        if ev.event_type == "user_adopted_guild_identity" {
            if let (Some(ms), Some(guild)) = (ev.timestamp.as_deref().and_then(time::parse_ms), as_str(&ev.guild_id)) {
                self.tags.push(TagEvent {
                    ms,
                    kind: "adopted",
                    guild,
                    tag: ev.tag.clone(),
                    badge: ev.badge_name.clone(),
                    colours: [ev.badge_color_primary.clone(), ev.badge_color_secondary.clone()],
                });
            }
            return;
        }
        let ts = ev.timestamp.as_deref().and_then(time::parse_ms).unwrap_or(0);
        let (name, id, what) = match ev.event_type.as_str() {
            "launch_game" => (ev.game.as_deref().or(ev.game_name.as_deref()), as_str(&ev.game_id), games::Analytics::Launch),
            "running_game_heartbeat" => (
                ev.game_name.as_deref().or(ev.game.as_deref()),
                as_str(&ev.game_id),
                games::Analytics::Heartbeat {
                    session: ev.game_session_id.as_deref(),
                    ms: as_i64(&ev.duration_tracked_ms).unwrap_or(0),
                },
            ),
            "application_closed" => (
                ev.application_name.as_deref(),
                as_str(&ev.application_id),
                games::Analytics::Closed { ms: as_i64(&ev.activity_duration_s).unwrap_or(0) * 1000 },
            ),
            _ => return,
        };
        if let Some(name) = name {
            self.games.add_analytics(name, id, ev.event_id.as_deref(), ts, what);
        }
    }

    /// Whether any voice events were seen (decides if the `tns` fallback is needed).
    pub fn has_voice(&self) -> bool {
        !self.voice.is_empty()
    }

    /// One logged send: is the message still in the package, deleted, or gone with its channel?
    fn sent_message(&mut self, line: &[u8]) {
        let field = |f: &Finder| raw_field(line, f).and_then(|b| std::str::from_utf8(b).ok());
        let Some(id) = field(&self.f_message_id).and_then(|s| s.parse::<u64>().ok()) else { return };
        if !self.sent_ids.insert(id) {
            return;
        }
        if let Some(tz) = field(&self.f_time_zone) {
            if let Some(n) = self.time_zones.get_mut(tz) {
                *n += 1;
            } else {
                self.time_zones.insert(tz.to_owned(), 1);
            }
        }
        if self.alive_ids.contains(&id) {
            return;
        }
        let t = field(&self.f_timestamp).and_then(time::parse_ms).unwrap_or_else(|| time::snowflake_ms(id));
        let channel_known = field(&self.f_channel).is_some_and(|c| self.channels.contains_key(c));
        let bucket = if channel_known { &mut self.deleted_hours } else { &mut self.lost_hours };
        *bucket.entry(time::hour_index(t)).or_default() += 1;
        if channel_known { self.deleted_total += 1 } else { self.lost_total += 1 }
    }

    /// A session start or app open: where and on what, read field by field.
    fn sighting(&mut self, line: &[u8], kind: &[u8]) {
        let field = |f: &Finder| raw_field(line, f).and_then(|b| std::str::from_utf8(b).ok()).unwrap_or("");
        let Some(ms) = time::parse_ms(field(&self.f_timestamp)) else { return };
        let s = devices::Sighting {
            ms,
            kind: std::str::from_utf8(kind).unwrap_or(""),
            browser: field(&self.f_browser),
            os: field(&self.f_os),
            device: field(&self.f_device),
            country: field(&self.f_country),
            city: field(&self.f_city),
            isp: field(&self.f_isp),
            time_zone: field(&self.f_time_zone),
            opened_from: field(&self.f_opened_from),
        };
        self.devices.add(s);
    }

    fn event_line(&mut self, line: &[u8]) {
        // Discord puts the event type first, so most lines are skipped without parsing them.
        // Lines laid out differently are parsed in full.
        match event_type(line) {
            Some(b"send_message") => return self.sent_message(line),
            Some(t @ (b"session_start" | b"session_start_success" | b"app_opened")) => return self.sighting(line, t),
            Some(t) if !WANTED_EVENTS.iter().any(|w| w.as_bytes() == t) => return,
            None if !self.finders.iter().any(|f| f.find(line).is_some()) => return,
            _ => {}
        }
        let Ok(ev) = serde_json::from_slice::<RawEvent>(line) else { return };
        if ev.event_type == "send_message" {
            return self.sent_message(line);
        }
        let Some(ts) = ev.timestamp.as_deref().and_then(time::parse_ms) else { return };
        if let Some(n) = as_i64(&ev.user_guilds) {
            self.guild_counts.push((ts, n.clamp(0, u16::MAX as i64) as u16));
        }
        let guild = as_str(&ev.guild_id);
        let mut server = |kind: &'static str, detail: Option<String>| {
            if let Some(g) = &guild {
                self.guild_events.push(GuildEvent { ms: ts, kind, guild: g.clone(), detail });
            }
        };

        match ev.event_type.as_str() {
            "leave_voice_channel" | "voice_disconnect" => {
                let dur = if ev.event_type == "voice_disconnect" {
                    as_i64(&ev.duration_connected_ms)
                } else {
                    as_i64(&ev.duration)
                };
                let key = match (as_str(&ev.guild_id), as_str(&ev.channel_id)) {
                    (Some(g), _) => VoiceKey::Guild(g),
                    (None, Some(c)) => VoiceKey::Channel(c),
                    (None, None) => VoiceKey::Channel(String::new()),
                };
                if let Some(d) = dur {
                    self.voice.add(ev.event_id.as_deref(), ts, d, key);
                }
            }
            "accepted_instant_invite" | "invite_sent" => {
                let (guild, code) = if ev.event_type == "invite_sent" {
                    (as_str(&ev.invite_guild_id), as_str(&ev.invite_code))
                } else {
                    (as_str(&ev.guild), as_str(&ev.invite))
                };
                if let (Some(g), Some(c)) = (guild, code) {
                    self.invites.entry(g).or_default().push((true, ts, c));
                }
            }
            // Membership screening and role grants also log a "join"; only real joins count.
            "guild_joined" if !matches!(ev.join_method.as_deref(), Some("MEMBERSHIP GATING" | "ROLE GRANTED")) => {
                server("join", ev.join_type.clone().or(ev.join_method.clone()))
            }
            "leave_guild" => server("leave", None),
            "create_guild" => server("create", None),
            "delete_guild" => server("delete", None),
            "premium_guild_subscription_created" => server("boost", None),
            "premium_guild_subscription_removed" => server("unboost", None),
            "moderation_action" => server("mod", ev.action_type.clone()),
            "guild_bot_added" => server("bot", as_str(&ev.application_id)),
            "guild_tag_updated" => {
                if let Some(g) = guild.clone() {
                    self.tags.push(TagEvent {
                        ms: ts,
                        kind: "created",
                        guild: g,
                        tag: ev.new_tag.clone(),
                        badge: ev.badge_name.clone(),
                        colours: [ev.badge_color_primary.clone(), ev.badge_color_secondary.clone()],
                    });
                }
            }
            "update_relationship" => {
                if let Some(t) = as_i64(&ev.rel_type) {
                    let mine = matches!(&ev.is_initiator, Some(Value::Bool(true))) || as_str(&ev.is_initiator).as_deref() == Some("true");
                    self.friend_events.push((ts, t.clamp(0, 9) as u8, mine));
                }
            }
            "data_request_initiated" => self.data_requests.push(ts),
            // 0 means the list hadn't loaded yet.
            "friends_list_viewed" | "friends_list_clicked" => {
                if let Some(n) = as_i64(&ev.num_friends).filter(|&n| n > 0) {
                    self.friend_counts.push((ts, n.min(u32::MAX as i64) as u32));
                }
            }
            "video_stream_ended" => {
                if let (Some(secs), Some(role)) = (as_i64(&ev.duration), ev.participant_type.as_deref()) {
                    self.tns.stream_ended(ts, secs, role);
                }
            }
            "message_edited" => self.tns.edited(ts),
            "enable_totp" | "disable_totp" => self.tns.two_factor(ts, ev.event_type == "enable_totp"),
            "start_call" => self.tns.call_started(ts),
            "create_emoji" => self.tns.emoji_created(ts),
            "activity_session_joined" => {
                if let Some(app) = as_str(&ev.application_id) {
                    let a = self.activities.entry(app).or_insert((0, ts, ts));
                    a.0 += 1;
                    a.1 = a.1.min(ts);
                    a.2 = a.2.max(ts);
                }
            }
            "payment_succeeded" | "external_payment_succeeded" | "transaction_completed" | "gift_code_created" | "gift_code_sent"
            | "gift_code_resolved" | "gift_accept_step" | "sku_entitlement_created" => {
                let tx = ev.event_type == "transaction_completed";
                let amount = if tx { as_i64(&ev.presentment_amount) } else { as_i64(&ev.amount).or(as_i64(&ev.price)) };
                self.money.add(money::MoneyEvent {
                    kind: ev.event_type.as_str(),
                    // A transaction row's own date (settlements are back-dated); otherwise the event time.
                    ms: if tx { ev.created_at.as_deref().and_then(time::parse_ms).unwrap_or(ts) } else { ts },
                    event_id: ev.event_id.as_deref(),
                    payment_id: as_str(&ev.payment_id),
                    amount,
                    currency: if tx { ev.presentment_currency.clone() } else { ev.currency.clone() },
                    sku: as_str(&ev.sku_id),
                    title: ev.store_title.clone(),
                    gift: match &ev.is_gift {
                        Some(Value::Bool(b)) => Some(*b),
                        Some(Value::String(s)) => Some(s == "true"),
                        _ => None,
                    },
                    payment_type: ev.payment_type.clone(),
                    code: ev.gift_code.clone(),
                    channel: as_str(&ev.channel_id),
                    guild: guild.clone(),
                    step: ev.to_step.clone(),
                });
            }
            "add_reaction" => {
                self.reactions += 1;
                let name = as_str(&ev.emoji_name);
                let emoji = match (as_str(&ev.emoji_id), name) {
                    (Some(id), Some(name)) => Some(Emoji::Custom {
                        name,
                        id,
                        animated: matches!(ev.emoji_animated, Some(Value::Bool(true))),
                    }),
                    (None, Some(text)) => Some(Emoji::Unicode(text.trim_end_matches('\u{FE0F}').to_owned())),
                    _ => None,
                };
                if let Some(e) = emoji {
                    *self.emoji_counts.entry(e).or_default() += 1;
                }
            }
            _ => {}
        }
        // Only voice events: quest adverts also carry a game_name for games never played.
        if let Some(g) = ev.game_name.as_deref() {
            if VOICE_EVENTS_WITH_GAME.contains(&ev.event_type.as_str()) {
                self.games.add(g, ts);
            }
        }
    }

    /// Every game seen, for debugging the not-a-game filter.
    pub fn all_games(&self) -> Vec<games::GameTotal> {
        self.games.totals(i64::MIN, i64::MAX)
    }

    pub fn finish(self) -> Facts {
        let self_id = self.user.as_ref().map(|u| u.id.clone()).unwrap_or_default();

        // People: friends from the account file, then DM partners the package only names.
        let mut people: HashMap<String, PersonInfo> = HashMap::new();
        let (mut friends, mut blocked) = (0, 0);
        for r in self.user.iter().flat_map(|u| &u.relationships) {
            match relationship(&r.kind) {
                1 => friends += 1,
                2 => blocked += 1,
                _ => {}
            }
            people.insert(
                r.user.id.clone(),
                PersonInfo {
                    id: r.user.id.clone(),
                    username: r.user.username.clone(),
                    display_name: r.user.global_name.clone().unwrap_or_else(|| r.user.username.clone()),
                    avatar: r.user.avatar.clone(),
                    friend: relationship(&r.kind) == 1,
                    relationship: relationship(&r.kind),
                    nickname: r.nickname.clone(),
                    tag: person_tag(&r.user.primary_guild),
                    decoration: r.user.avatar_decoration_data.get("asset").and_then(Value::as_str).map(str::to_owned),
                },
            );
        }

        let mut channels: Vec<ChannelFacts> = Vec::with_capacity(self.channels.len());
        let mut guild_ids: HashSet<String> = HashSet::new();
        for (id, ch) in self.channels {
            let name = self.channel_names.get(&id).cloned();
            let (kind, guild, person) = match ch.kind {
                ChannelKind::Dm(recipients) => {
                    let other = recipients.into_iter().find(|r| *r != self_id);
                    if let Some(o) = &other {
                        people.entry(o.clone()).or_insert_with(|| {
                            let n = name
                                .as_deref()
                                .map(|n| n.trim_start_matches("Direct Message with ").trim_end_matches("#0").to_owned())
                                .unwrap_or_else(|| "Unknown user".into());
                            PersonInfo {
                                id: o.clone(),
                                username: n.clone(),
                                display_name: n,
                                avatar: None,
                                friend: false,
                                relationship: 0,
                                nickname: None,
                                tag: None,
                                decoration: None,
                            }
                        });
                    }
                    ("dm", None, other)
                }
                ChannelKind::Group => ("group", None, None),
                ChannelKind::Guild(g) => {
                    guild_ids.insert(g.clone());
                    ("guild", Some(g), None)
                }
                ChannelKind::UnknownGuild => ("guild", None, None),
                ChannelKind::Other => ("other", None, None),
            };
            let (first_ms, last_ms) = if ch.months.is_empty() { (0, 0) } else { (ch.first, ch.last) };
            channels.push(ChannelFacts { id, kind, guild, person, name, months: ch.months.into_iter().collect(), first_ms, last_ms });
        }
        channels.sort_by(|a, b| a.id.cmp(&b.id));

        let voice = (!self.voice.is_empty()).then(|| self.voice.summary());
        if let Some(v) = &voice {
            guild_ids.extend(v.by_key.keys().filter_map(|k| match k {
                VoiceKey::Guild(g) => Some(g.clone()),
                _ => None,
            }));
        }

        let mut servers: Vec<ServerInfo> = guild_ids
            .into_iter()
            .map(|g| {
                let mut invites = vec![];
                if let Some(list) = self.invites.get(&g) {
                    let mut list = list.clone();
                    // Logged invites first (they are known to point at this server), newest first.
                    list.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
                    for (_, _, code) in list {
                        if !invites.contains(&code) {
                            invites.push(code);
                        }
                        if invites.len() == INVITES_PER_SERVER {
                            break;
                        }
                    }
                }
                ServerInfo {
                    name: self.server_names.get(&g).cloned().unwrap_or_else(|| "Server no longer available".into()),
                    name_known: self.server_names.contains_key(&g),
                    id: g,
                    invites,
                }
            })
            .collect();
        servers.sort_by(|a, b| a.id.cmp(&b.id));

        let voice = voice.map(|v| {
            let mut places: Vec<VoicePlace> = v
                .by_key
                .into_iter()
                .filter_map(|(k, months)| match k {
                    VoiceKey::Guild(id) => Some(VoicePlace { kind: "guild", id, months }),
                    VoiceKey::Channel(id) if !id.is_empty() => Some(VoicePlace { kind: "channel", id, months }),
                    _ => None,
                })
                .collect();
            places.sort_by(|a, b| a.id.cmp(&b.id));
            let key = |k: VoiceKey| match k {
                VoiceKey::Guild(id) => ("guild", id),
                VoiceKey::Channel(id) => ("channel", id),
            };
            let longest = v
                .longest
                .into_iter()
                .map(|(start_ms, end_ms, k)| {
                    let (kind, id) = key(k);
                    VoiceSession { start_ms, end_ms, kind, id }
                })
                .collect();
            VoiceFacts { hours: v.hours, sessions: v.sessions, days: v.days, hours_by_hour: v.by_hour, places, longest }
        });

        let mut top: Vec<(&Emoji, &u64)> = self.emoji_counts.iter().collect();
        top.sort_by(|a, b| b.1.cmp(a.1));
        let top = top
            .into_iter()
            .take(20)
            .map(|(e, &count)| match e {
                Emoji::Unicode(t) => EmojiCount { text: Some(t.clone()), name: None, id: None, animated: false, count },
                Emoji::Custom { name, id, animated } => {
                    EmojiCount { text: None, name: Some(name.clone()), id: Some(id.clone()), animated: *animated, count }
                }
            })
            .collect();
        let emojis = EmojiStats {
            total: self.emojis_in_messages + self.reactions,
            in_messages: self.emojis_in_messages,
            reactions: self.reactions,
            top,
        };

        let game_totals = self.games.totals(i64::MIN, i64::MAX);
        let from_analytics = self.games.from_analytics();
        let games = (!game_totals.is_empty()).then(|| GameStats {
            source: if from_analytics { "analytics" } else { "voice" },
            distinct: game_totals.len(),
            hours: from_analytics.then(|| game_totals.iter().filter_map(|g| g.hours).sum()),
            top: game_totals
                .iter()
                .take(30)
                .map(|g| GameCount {
                    name: g.name.clone(),
                    id: g.id.clone(),
                    sessions: g.sessions,
                    hours: g.hours,
                    months: g.months.clone(),
                    first_ms: g.first_ms,
                    last_ms: g.last_ms,
                })
                .collect(),
            by_hour: self.games.by_hour(),
        });

        let profile = self.user.as_ref().map(|u| {
            let meta = u.user_profile_metadata.as_ref();
            Profile {
                id: u.id.clone(),
                username: u.username.clone(),
                display_name: u.global_name.clone().unwrap_or_else(|| u.username.clone()),
                avatar: u.avatar_hash.clone(),
                created_ms: u.id.parse().map(time::snowflake_ms).unwrap_or(0),
                flags: flag_names(&u.flags),
                premium_until: u.premium_until.clone(),
                premium_since: meta.and_then(|m| m.premium_started_at.clone()),
                boosting_since: meta.and_then(|m| m.boosting_started_at.clone()),
                legacy_username: meta.and_then(|m| m.legacy_username.clone()),
                orbs: u.current_orbs_balance,
                friends,
                blocked,
                widgets: meta.map(|m| widgets(&m.widgets)).unwrap_or_default(),
            }
        });

        let sorted = |m: HashMap<u32, u32>| {
            let mut v: Vec<(u32, u32)> = m.into_iter().collect();
            v.sort_unstable();
            v
        };
        let mut people: Vec<PersonInfo> = people.into_values().collect();
        people.sort_by(|a, b| a.id.cmp(&b.id));

        Facts {
            profile,
            time_zone: self.time_zones.into_iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))).map(|(tz, _)| tz),
            alive_hours: sorted(self.alive_hours),
            deleted_hours: sorted(self.deleted_hours),
            lost_hours: sorted(self.lost_hours),
            has_send_log: !self.sent_ids.is_empty(),
            channels,
            people,
            servers,
            voice,
            emojis,
            games,
            quests_completed: self.quests_completed,
            text: self.text.finish(),
            tags: {
                let mut t = self.tags;
                t.sort_by_key(|e| e.ms);
                t
            },
            guild_events: {
                let mut g = self.guild_events;
                g.sort_by_key(|e| e.ms);
                g
            },
            friend_events: {
                let mut f = self.friend_events;
                f.sort_unstable();
                f
            },
            data_requests: {
                let mut d = self.data_requests;
                d.sort_unstable();
                d
            },
            guild_counts: {
                let mut g = self.guild_counts;
                g.sort_unstable();
                g.dedup();
                g
            },
            friend_counts: {
                let mut f = self.friend_counts;
                f.sort_unstable();
                f.dedup();
                f
            },
            devices: (!self.devices.is_empty()).then(|| self.devices.finish()),
            money: self.money.finish(),
            activities: {
                let mut a: Vec<(String, u32, i64, i64)> = self.activities.into_iter().map(|(k, (n, f, l))| (k, n, f, l)).collect();
                a.sort_by(|x, y| y.1.cmp(&x.1));
                a
            },
            account: self.account,
            expressions: self.expressions,
            ad_profile: self.ad_profile,
            poker: self.poker,
            tns: Some(self.tns.finish()).filter(|t| t.since_ms != 0),
        }
    }
}

/// Activity-log events worth parsing (besides `send_message`, which is read field by field).
const WANTED_EVENTS: &[&str] = &[
    "activity_session_joined", "payment_succeeded", "external_payment_succeeded", "transaction_completed",
    "gift_code_created", "gift_code_sent", "gift_code_resolved", "gift_accept_step", "sku_entitlement_created",
    "leave_voice_channel", "voice_disconnect", "start_speaking", "start_listening", "join_voice_channel",
    "accepted_instant_invite", "invite_sent", "add_reaction",
    "guild_joined", "leave_guild", "create_guild", "delete_guild",
    "premium_guild_subscription_created", "premium_guild_subscription_removed",
    "moderation_action", "guild_bot_added", "guild_tag_updated", "update_relationship", "data_request_initiated",
    "guild_joined_pending", "friends_list_viewed", "friends_list_clicked",
    // Only in the trust & safety log.
    "video_stream_ended", "message_edited", "enable_totp", "disable_totp", "start_call", "create_emoji",
];

const VOICE_EVENTS_WITH_GAME: &[&str] =
    &["start_speaking", "start_listening", "join_voice_channel", "leave_voice_channel", "voice_disconnect"];

/// Server channel types: text, voice, announcement, stage, forum, media, and threads.
fn is_server_channel(kind: &Value) -> bool {
    match kind {
        Value::String(s) => s.starts_with("GUILD_") || s.ends_with("_THREAD"),
        Value::Number(n) => matches!(n.as_u64(), Some(0 | 2 | 4 | 5 | 10 | 11 | 12 | 13 | 15 | 16)),
        _ => false,
    }
}

/// Invite codes in a message: `discord.gg/abc`, `discord.com/invite/abc`, `discordapp.com/invite/abc`.
fn invite_codes(text: &str) -> impl Iterator<Item = &str> {
    const PREFIXES: [&str; 3] = ["discord.gg/", "discord.com/invite/", "discordapp.com/invite/"];
    PREFIXES.iter().flat_map(move |p| {
        text.match_indices(p).filter_map(move |(i, _)| {
            let rest = &text[i + p.len()..];
            let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).unwrap_or(rest.len());
            (2..=32).contains(&end).then(|| &rest[..end])
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A snowflake id for a Unix ms timestamp.
    fn sf(ms: i64) -> u64 {
        ((ms - 1_420_070_400_000) as u64) << 22
    }

    #[test]
    fn classifies_entries() {
        assert_eq!(wanted("Account/user.json"), Some(Entry::User));
        assert_eq!(wanted("Messages/c123/messages.json"), Some(Entry::ChannelMessages));
        assert_eq!(wanted("Activity/reporting/events-2026-00000-of-00001.json"), Some(Entry::Events));
        assert_eq!(wanted("Activity/tns/events-2026-00000-of-00001.json"), Some(Entry::EventsFallback));
        assert_eq!(wanted("Activity/analytics/events-2026-00000-of-00001.json"), Some(Entry::Analytics));
        assert_eq!(wanted("Activity/modeling/events-2026-00000-of-00001.json"), None);
        assert_eq!(wanted("Servers/1/guild.json"), None);
    }

    #[test]
    fn reads_raw_fields() {
        let line = br#"{"event_type":"send_message","message_id":"42","channel":"7","channel_type":"0","timestamp":"\"2025-03-11T09:50:54Z\""}"#;
        assert_eq!(event_type(line), Some(&b"send_message"[..]));
        assert_eq!(raw_field(line, &Finder::new(b"\"message_id\":\"")), Some(&b"42"[..]));
        assert_eq!(raw_field(line, &Finder::new(b"\"channel\":\"")), Some(&b"7"[..]));
        assert_eq!(raw_field(line, &Finder::new(b"\"timestamp\":\"")), Some(&b"2025-03-11T09:50:54Z"[..]));
    }

    #[test]
    fn end_to_end_small_package() {
        let t1 = time::date_ms(2025, 3, 1) + 10 * time::HOUR_MS;
        let t2 = time::date_ms(2024, 3, 1) + 10 * time::HOUR_MS;
        let t3 = time::date_ms(2025, 6, 1) + 10 * time::HOUR_MS;
        let mut e = Engine::new();
        e.feed_file(
            Entry::User,
            br#"{"id":"1","username":"me","global_name":"Me","avatar_hash":null,"flags":["HYPESQUAD_ONLINE_HOUSE_3"],
                 "relationships":[{"type":"FRIEND","user":{"id":"2","username":"bff","global_name":"Best Friend","avatar":"abc"}},
                                  {"type":"BLOCKED","user":{"id":"3","username":"nope","global_name":null,"avatar":null}}]}"#,
        )
        .unwrap();
        e.feed_file(Entry::ServersIndex, br#"{"g1":"Test Server"}"#).unwrap();
        e.feed_channel(
            br#"{"id":"dm1","type":"DM","recipients":["1","2"]}"#,
            format!(
                "[{{\"ID\":{},\"Timestamp\":\"x\",\"Contents\":\"hi \u{1F62D}\"}},{{\"ID\":\"{}\",\"Timestamp\":\"x\",\"Contents\":\"old\"}}]",
                sf(t1),
                sf(t2)
            )
            .as_bytes(),
        )
        .unwrap();
        e.feed_channel(
            br#"{"id":"c1","type":"GUILD_TEXT","guild":{"id":"g1","name":"Nest"}}"#,
            format!(
                "[{{\"ID\":{},\"Timestamp\":\"x\",\"Contents\":\"<:wow:123456789012345678> join https://discord.gg/nest-42\"}}]",
                sf(t3)
            )
            .as_bytes(),
        )
        .unwrap();

        let events = format!(
            concat!(
                r#"{{"event_type":"send_message","message_id":"{alive}","channel":"c1","time_zone":"Europe/Warsaw","timestamp":"\"2025-06-01T10:00:00Z\""}}"#, "\n",
                r#"{{"event_type":"send_message","message_id":"{gone}","channel":"c1","time_zone":"Europe/Warsaw","timestamp":"\"2025-06-02T10:00:00Z\""}}"#, "\n",
                r#"{{"event_type":"send_message","message_id":"{gone}","channel":"c1","time_zone":"Europe/Warsaw","timestamp":"\"2025-06-02T10:00:00Z\""}}"#, "\n",
                r#"{{"event_type":"send_message","message_id":"{lost}","channel":"c999","time_zone":"Europe/Zagreb","timestamp":"\"2021-06-02T10:00:00Z\""}}"#, "\n",
                r#"{{"event_type":"leave_voice_channel","event_id":"e1","timestamp":"\"2025-05-01T02:00:00Z\"","guild_id":"g1","duration":"3600000","game_name":"Celeste"}}"#, "\n",
                r#"{{"event_type":"voice_disconnect","event_id":"e2","timestamp":"\"2025-05-01T02:00:30Z\"","channel_id":"dm1","duration_connected_ms":"1800000"}}"#, "\n",
                r#"{{"event_type":"add_reaction","timestamp":"\"2025-05-02T00:00:00Z\"","emoji_name":"😭"}}"#, "\n",
                r#"{{"event_type":"accepted_instant_invite","timestamp":"\"2021-05-02T00:00:00Z\"","guild":"g1","invite":"joined"}}"#, "\n",
                r#"{{"event_type":"guild_joined","timestamp":"\"2021-05-02T00:00:01Z\"","guild_id":"g1","join_method":"USER","join_type":"invite"}}"#, "\n",
                r#"{{"event_type":"guild_joined","timestamp":"\"2021-05-02T00:00:02Z\"","guild_id":"g1","join_method":"MEMBERSHIP GATING"}}"#, "\n",
                r#"{{"event_type":"update_relationship","timestamp":"\"2022-01-01T00:00:00Z\"","type":"1","is_initiator":true}}"#, "\n",
                r##"{{"event_type":"guild_tag_updated","timestamp":"\"2024-03-14T12:00:00Z\"","guild_id":"g1","new_tag":"NEST","badge_name":"HEART","badge_color_primary":"#2d7d46","badge_color_secondary":"#7ed69a"}}"##, "\n",
                r#"{{"event_type":"friends_list_viewed","timestamp":"\"2023-01-01T00:00:00Z\"","num_friends":"42"}}"#, "\n",
                r#"{{"event_type":"friends_list_viewed","timestamp":"\"2023-01-02T00:00:00Z\"","num_friends":"0"}}"#, "\n",
                r#"{{"event_type":"video_stream_ended","timestamp":"\"2024-10-15T19:10:54Z\"","participant_type":"streamer","duration":"3600"}}"#
            ),
            alive = sf(t3),
            gone = sf(t3) + 1,
            lost = 77,
        );
        // Awkward chunk sizes exercise the line carry-over.
        for chunk in events.as_bytes().chunks(7) {
            e.feed_events(chunk);
        }
        assert_eq!(e.finish_events(), 15);
        let p = e.progress();
        assert_eq!((p.messages, p.deleted, p.lost, p.sends), (3, 1, 1, 3));
        // The hour in voice and the half-hour call overlap (60.5 minutes together); the gone
        // messages are on two days.
        let day = |y, m, d| (time::date_ms(y, m, d) / time::DAY_MS) as u32;
        assert_eq!(e.activity_days(), vec![day(2021, 6, 2), 0, 1, day(2025, 5, 1), 61, 0, day(2025, 6, 2), 0, 1]);

        let f = e.finish();
        let p = f.profile.unwrap();
        assert_eq!((p.friends, p.blocked), (1, 1));
        assert_eq!(p.flags, vec!["HYPESQUAD_ONLINE_HOUSE_3"]);
        assert_eq!(f.alive_hours.iter().map(|x| x.1).sum::<u32>(), 3);
        assert_eq!(f.alive_hours[0].0, time::hour_index(t2));
        assert_eq!(f.deleted_hours.iter().map(|x| x.1).sum::<u32>(), 1);
        assert_eq!(f.lost_hours, vec![(time::hour_index(time::date_ms(2021, 6, 2) + 10 * time::HOUR_MS), 1)]);
        assert_eq!(f.time_zone.as_deref(), Some("Europe/Warsaw"));

        let dm = f.channels.iter().find(|c| c.id == "dm1").unwrap();
        assert_eq!((dm.kind, dm.person.as_deref()), ("dm", Some("2")));
        assert_eq!(dm.months, vec![(time::month_index(t2), 1), (time::month_index(t1), 1)]);
        assert_eq!(f.people.iter().find(|p| p.id == "2").unwrap().display_name, "Best Friend");

        assert_eq!(f.emojis.total, 3);
        assert_eq!(f.emojis.top[0].text.as_deref(), Some("😭"));
        let v = f.voice.unwrap();
        assert!((v.hours - (1.0 + 30.0 / 3600.0)).abs() < 1e-6);
        assert_eq!(v.places.len(), 2);
        assert_eq!(f.servers[0].name, "Test Server");
        assert_eq!(f.servers[0].invites, vec!["joined", "nest-42"]);
        assert_eq!(f.games.unwrap().top[0].name, "Celeste");
        let joins: Vec<_> = f.guild_events.iter().filter(|g| g.kind == "join").collect();
        assert_eq!((joins.len(), joins[0].detail.as_deref()), (1, Some("invite")));
        assert_eq!(f.friend_events, vec![(time::date_ms(2022, 1, 1), 1, true)]);
        assert_eq!((f.tags[0].kind, f.tags[0].tag.as_deref()), ("created", Some("NEST")));
        // The friends list that hadn't loaded (0) is left out.
        assert_eq!(f.friend_counts, vec![(time::date_ms(2023, 1, 1), 42)]);
        let end = time::date_ms(2024, 10, 15) + 19 * time::HOUR_MS + 10 * 60_000 + 54_000;
        assert_eq!(f.tns.unwrap().streamed, vec![(end - time::HOUR_MS, end)]);
    }

    #[test]
    fn analytics_games() {
        let mut e = Engine::new();
        let analytics = concat!(
            r#"{"event_type":"launch_game","event_id":"a1","timestamp":"\"2025-05-01T00:00:00Z\"","game":"Slay the Spire","game_id":"700"}"#, "\n",
            r#"{"event_type":"running_game_heartbeat","event_id":"a2","timestamp":"\"2025-05-01T00:05:00Z\"","game_name":"Slay the Spire","game_id":"700","game_session_id":"s","duration_tracked_ms":"300000"}"#,
        );
        for chunk in analytics.as_bytes().chunks(11) {
            e.feed_analytics(chunk);
        }
        assert_eq!(e.finish_events(), 2);
        assert_eq!(e.game_days(), vec![(time::date_ms(2025, 5, 1) / time::DAY_MS) as u32, 5]);
        let g = e.finish().games.unwrap();
        assert_eq!((g.source, g.distinct), ("analytics", 1));
        assert!((g.top[0].hours.unwrap() - 5.0 / 60.0).abs() < 1e-9);
    }
}
