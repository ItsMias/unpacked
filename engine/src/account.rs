//! Account details and saved favourites from `Account/user.json`: what Discord keeps about you
//! (for the "What Discord knows" page) and your expressions (favourite GIFs, most-used stickers,
//! sounds and commands).

use serde::Serialize;
use serde_json::Value;

use crate::time;

#[derive(Serialize, Debug, Default)]
pub struct AccountInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    /// The last IP address Discord saw.
    pub ip: Option<String>,
    pub verified: Option<bool>,
    pub has_mobile: Option<bool>,
    pub date_of_birth: Option<String>,
    pub age_assurance: Value,
    pub predicted_age: Value,
    pub predicted_gender: Value,
    pub temp_banned_until: Value,
    /// Logged-in sessions Discord still knows about.
    pub sessions: Vec<LoginSession>,
    pub connections: Vec<Connection>,
    /// Notes you wrote on people: `[user id, note]`.
    pub notes: Vec<(String, String)>,
    pub server_settings: usize,
    pub muted_servers: usize,
    pub privacy: Value,
    /// Discord's own playtime totals per app.
    pub app_stats: Vec<AppStat>,
    pub library: Vec<LibraryApp>,
}

#[derive(Serialize, Debug)]
pub struct LoginSession {
    pub created_ms: i64,
    pub last_used_ms: i64,
    pub os: Option<String>,
    pub platform: Option<String>,
    pub ip: Option<String>,
    pub mfa: bool,
}

#[derive(Serialize, Debug)]
pub struct Connection {
    pub kind: String,
    pub name: String,
    pub visible: bool,
    pub verified: bool,
}

#[derive(Serialize, Debug)]
pub struct AppStat {
    pub id: String,
    pub seconds: i64,
    pub first_ms: Option<i64>,
    pub last_ms: Option<i64>,
}

#[derive(Serialize, Debug)]
pub struct LibraryApp {
    pub id: Option<String>,
    pub name: Option<String>,
    pub ms: Option<i64>,
}

#[derive(Serialize, Debug, Default)]
pub struct Expressions {
    pub favorite_gifs: Vec<Gif>,
    /// `[sticker id, uses]`, most used first.
    pub stickers: Vec<(String, u64)>,
    /// `[sound id, plays]`, most played first.
    pub sounds: Vec<(String, u64)>,
    /// The time span those plays cover: Discord only keeps recent ones.
    pub sounds_span: Option<(i64, i64)>,
    pub favorite_sounds: Vec<String>,
    pub commands: Vec<Command>,
    /// `[application id, uses]`: apps and activities you opened.
    pub apps: Vec<(String, u64)>,
}

#[derive(Serialize, Debug)]
pub struct Gif {
    /// The page it came from (tenor, giphy, a Discord attachment…).
    pub url: String,
    /// Playable media, through Discord's media proxy.
    pub src: String,
    pub video: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Debug)]
pub struct Command {
    pub app: String,
    pub name: String,
    pub uses: u64,
}

const STICKERS: usize = 40;
const COMMANDS: usize = 40;

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).filter(|x| !x.is_empty()).map(str::to_owned)
}

fn ms(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(Value::as_str).and_then(time::parse_ms)
}

/// `{ id: { totalUses } }` frecency maps, as `[id, uses]` most used first.
fn uses(v: Option<&Value>, limit: usize) -> Vec<(String, u64)> {
    let mut out: Vec<(String, u64)> = v
        .and_then(Value::as_object)
        .map(|m| m.iter().filter_map(|(k, x)| Some((k.clone(), x.get("totalUses")?.as_u64()?))).collect())
        .unwrap_or_default();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out.truncate(limit);
    out
}

pub fn account(u: &Value) -> AccountInfo {
    let list = |k: &str| u.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
    let mut sessions: Vec<LoginSession> = list("user_sessions")
        .iter()
        .filter_map(|x| {
            let d = x.get("user_data")?;
            let c = d.get("client_info").cloned().unwrap_or(Value::Null);
            Some(LoginSession {
                created_ms: ms(d, "creation_time")?,
                last_used_ms: ms(d, "approx_last_used_time").unwrap_or(0),
                os: s(&c, "os"),
                platform: s(&c, "platform"),
                ip: s(&c, "ip"),
                mfa: d.get("is_mfa").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect();
    sessions.sort_by_key(|x| std::cmp::Reverse(x.last_used_ms.max(x.created_ms)));

    let connections = list("connections")
        .iter()
        .filter_map(|c| {
            Some(Connection {
                kind: s(c, "type")?,
                name: s(c, "name").unwrap_or_default(),
                visible: c.get("visibility").and_then(Value::as_i64) == Some(1),
                verified: c.get("verified").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect();

    let notes = u
        .get("notes")
        .and_then(Value::as_object)
        .map(|m| m.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned()))).collect())
        .unwrap_or_default();

    let servers = list("guild_settings");
    let mut app_stats: Vec<AppStat> = list("user_activity_application_statistics")
        .iter()
        .filter_map(|a| {
            Some(AppStat {
                id: s(a, "application_id")?,
                seconds: a.get("total_duration").and_then(Value::as_i64).unwrap_or(0),
                first_ms: ms(a, "first_played_at"),
                last_ms: ms(a, "last_played_at"),
            })
        })
        .collect();
    app_stats.sort_by_key(|a| std::cmp::Reverse(a.seconds));

    let library = list("library_applications")
        .iter()
        .map(|a| LibraryApp {
            id: a.get("application").and_then(|x| s(x, "id")),
            name: a.get("application").and_then(|x| s(x, "name")),
            ms: ms(a, "created_at"),
        })
        .collect();

    AccountInfo {
        email: s(u, "email"),
        phone: s(u, "phone"),
        ip: s(u, "ip"),
        verified: u.get("verified").and_then(Value::as_bool),
        has_mobile: u.get("has_mobile").and_then(Value::as_bool),
        date_of_birth: s(u, "date_of_birth").filter(|d| !d.starts_with("0001")),
        age_assurance: u.get("age_assurance").cloned().unwrap_or(Value::Null),
        predicted_age: u.get("predicted_age").cloned().unwrap_or(Value::Null),
        predicted_gender: u.get("predicted_gender").cloned().unwrap_or(Value::Null),
        temp_banned_until: u.get("temp_banned_until").cloned().unwrap_or(Value::Null),
        sessions,
        connections,
        notes,
        server_settings: servers.len(),
        muted_servers: servers.iter().filter(|g| g.get("muted").and_then(Value::as_bool) == Some(true)).count(),
        privacy: u.pointer("/settings/settings/privacy").cloned().unwrap_or(Value::Null),
        app_stats,
        library,
    }
}

pub fn expressions(u: &Value) -> Expressions {
    let f = u.pointer("/settings/frecency").cloned().unwrap_or(Value::Null);
    let mut favorite_gifs: Vec<(i64, Gif)> = f
        .pointer("/favoriteGifs/gifs")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(url, g)| {
                    Some((
                        g.get("order").and_then(Value::as_i64).unwrap_or(0),
                        Gif {
                            url: url.clone(),
                            src: s(g, "src")?,
                            video: s(g, "format").as_deref() == Some("VIDEO"),
                            width: g.get("width").and_then(Value::as_u64).unwrap_or(0) as u32,
                            height: g.get("height").and_then(Value::as_u64).unwrap_or(0) as u32,
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    // Newest favourite first, as in Discord's GIF picker.
    favorite_gifs.sort_by_key(|(order, _)| std::cmp::Reverse(*order));

    let commands = uses(f.pointer("/applicationCommandFrecency/applicationCommands"), usize::MAX)
        .into_iter()
        .filter_map(|(key, n)| {
            // "<application id>\0<command>[:<guild id>]"
            let (app, rest) = key.split_once('\0')?;
            let name = rest.split(':').next().unwrap_or(rest);
            Some(Command { app: app.to_owned(), name: name.to_owned(), uses: n })
        })
        .take(COMMANDS)
        .collect();

    let plays: Vec<i64> = f
        .pointer("/playedSoundFrecency/playedSounds")
        .and_then(Value::as_object)
        .map(|m| {
            m.values()
                .filter_map(|x| x.get("recentUses")?.as_array().cloned())
                .flatten()
                .filter_map(|t| t.as_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default();

    Expressions {
        sounds_span: plays.iter().min().zip(plays.iter().max()).map(|(a, b)| (*a, *b)),
        favorite_gifs: favorite_gifs.into_iter().map(|(_, g)| g).collect(),
        stickers: uses(f.pointer("/stickerFrecency/stickers"), STICKERS),
        sounds: uses(f.pointer("/playedSoundFrecency/playedSounds"), STICKERS),
        favorite_sounds: f
            .pointer("/favoriteSoundboardSounds/soundIds")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
            .unwrap_or_default(),
        commands,
        apps: uses(f.pointer("/applicationFrecency/applications"), 40).into_iter().filter(|(id, _)| id != "-1").collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_account_and_favourites() {
        let u: Value = serde_json::from_str(
            r#"{"email":"a@b.c","date_of_birth":"0001-01-01","notes":{"1":"pal"},
                "guild_settings":[{"muted":true},{"muted":false}],
                "user_sessions":[{"user_data":{"creation_time":"2026-05-03T10:04:20.337861+00:00","is_mfa":true,"client_info":{"os":"Windows","platform":"Chrome"}}}],
                "settings":{"frecency":{
                  "favoriteGifs":{"gifs":{"https://tenor.com/x":{"format":"VIDEO","src":"https://m/x.mp4","width":10,"height":20,"order":2},
                                          "https://tenor.com/y":{"format":"IMAGE","src":"https://m/y.gif","width":10,"height":10,"order":5}}},
                  "stickerFrecency":{"stickers":{"7":{"totalUses":3},"8":{"totalUses":9}}},
                  "applicationCommandFrecency":{"applicationCommands":{"42\u0000show:99":{"totalUses":5}}}}}}"#,
        )
        .unwrap();
        let a = account(&u);
        assert_eq!((a.email.as_deref(), a.date_of_birth, a.muted_servers, a.notes.len()), (Some("a@b.c"), None, 1, 1));
        assert!(a.sessions[0].mfa);
        let e = expressions(&u);
        assert_eq!(e.favorite_gifs[0].src, "https://m/y.gif");
        assert!(e.favorite_gifs[1].video);
        assert_eq!(e.stickers[0], ("8".to_owned(), 9));
        assert_eq!((e.commands[0].app.as_str(), e.commands[0].name.as_str()), ("42", "show"));
    }
}
