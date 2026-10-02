//! Devices, apps and places you opened Discord from, out of `session_start`,
//! `session_start_success` and `app_opened` events. Fields are read straight from the raw line
//! (there are ~200k of these events), like `send_message`.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use crate::time;

/// Platform slots, in output order.
pub const PLATFORMS: [&str; 5] = ["android", "ios", "desktop", "web", "other"];

#[derive(Default, Clone, Copy)]
struct Seen {
    count: u32,
    first: i64,
    last: i64,
}

impl Seen {
    fn add(&mut self, ms: i64) {
        if self.count == 0 || ms < self.first {
            self.first = ms;
        }
        self.last = self.last.max(ms);
        self.count += 1;
    }
}

struct Client {
    platform: usize,
    device: String,
    os: String,
    browser: String,
    seen: Seen,
}

#[derive(Default)]
pub struct Devices {
    clients: HashMap<String, Client>,
    platforms: BTreeMap<u16, [u32; 5]>,
    countries: HashMap<String, Seen>,
    /// Sessions per country per UTC day, for trips.
    country_days: HashMap<(String, u32), u32>,
    cities: HashMap<String, Seen>,
    isps: HashMap<String, Seen>,
    time_zones: HashMap<String, Seen>,
    opened_from: HashMap<String, u32>,
    open_hours: HashMap<u32, u32>,
    opens: u64,
    sessions: u64,
}

/// What one event says, field by field (empty when missing).
pub struct Sighting<'a> {
    pub ms: i64,
    pub kind: &'a str,
    pub browser: &'a str,
    pub os: &'a str,
    pub device: &'a str,
    pub country: &'a str,
    pub city: &'a str,
    pub isp: &'a str,
    pub time_zone: &'a str,
    pub opened_from: &'a str,
}

#[derive(Serialize, Debug)]
pub struct DeviceFacts {
    /// Apps and devices, most used first.
    pub clients: Vec<ClientOut>,
    /// `[month index, [android, ios, desktop, web, other]]`: sessions and app opens per platform.
    pub platforms: Vec<(u16, [u32; 5])>,
    pub countries: Vec<PlaceOut>,
    /// Stretches of days spent mostly in another country than the usual one, oldest first.
    pub trips: Vec<TripOut>,
    pub cities: Vec<PlaceOut>,
    pub isps: Vec<PlaceOut>,
    pub time_zones: Vec<PlaceOut>,
    /// How the app was opened: launcher, notification, …
    pub opened_from: Vec<(String, u32)>,
    /// App opens per UTC hour index.
    pub open_hours: Vec<(u32, u32)>,
    pub opens: u64,
    pub sessions: u64,
}

#[derive(Serialize, Debug)]
pub struct ClientOut {
    /// One of [`PLATFORMS`].
    pub platform: &'static str,
    /// Phone or tablet model code (like `a52q`), when the event has one.
    pub device: String,
    pub os: String,
    pub browser: String,
    pub count: u32,
    pub first_ms: i64,
    pub last_ms: i64,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct TripOut {
    pub country: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub days: u32,
    pub sessions: u32,
}

#[derive(Serialize, Debug)]
pub struct PlaceOut {
    pub name: String,
    pub count: u32,
    pub first_ms: i64,
    pub last_ms: i64,
}

/// Which app or browser an event came from.
fn platform(browser: &str, device: &str) -> usize {
    match browser {
        "Discord Android" => 0,
        "Discord iOS" => 1,
        "Discord Client" => 2,
        _ if device == "console" || browser.starts_with("Discord") => 4,
        "" => 4,
        _ => 3,
    }
}

impl Devices {
    pub fn add(&mut self, s: Sighting) {
        let add = |map: &mut HashMap<String, Seen>, key: String| {
            if !key.is_empty() {
                map.entry(key).or_default().add(s.ms);
            }
        };
        add(&mut self.countries, s.country.to_owned());
        if !s.country.is_empty() {
            *self.country_days.entry((s.country.to_owned(), (s.ms.div_euclid(time::DAY_MS)) as u32)).or_default() += 1;
        }
        if !s.city.is_empty() {
            add(&mut self.cities, if s.country.is_empty() { s.city.to_owned() } else { format!("{}, {}", s.city, s.country) });
        }
        add(&mut self.isps, s.isp.to_owned());
        add(&mut self.time_zones, s.time_zone.to_owned());

        if s.kind == "app_opened" {
            self.opens += 1;
            *self.open_hours.entry(time::hour_index(s.ms)).or_default() += 1;
            if !s.opened_from.is_empty() {
                *self.opened_from.entry(s.opened_from.to_owned()).or_default() += 1;
            }
        } else if s.kind == "session_start" {
            self.sessions += 1;
        }
        // Server-side session events repeat the client ones; count each client sighting once.
        if s.kind == "session_start_success" || s.browser.is_empty() {
            return;
        }
        let p = platform(s.browser, s.device);
        self.platforms.entry(time::month_index(s.ms)).or_insert([0; 5])[p] += 1;
        let key = match p {
            0 | 1 if !s.device.is_empty() => s.device.to_owned(),
            _ => format!("{}|{}|{}", PLATFORMS[p], s.browser, s.os),
        };
        let c = self.clients.entry(key).or_insert_with(|| Client {
            platform: p,
            device: if p <= 1 { s.device.to_owned() } else { String::new() },
            os: s.os.to_owned(),
            browser: s.browser.to_owned(),
            seen: Seen::default(),
        });
        if c.os.is_empty() {
            c.os = s.os.to_owned();
        }
        c.seen.add(s.ms);
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty() && self.countries.is_empty()
    }

    pub fn finish(self) -> DeviceFacts {
        let places = |m: HashMap<String, Seen>, n: usize| {
            let mut v: Vec<PlaceOut> =
                m.into_iter().map(|(name, s)| PlaceOut { name, count: s.count, first_ms: s.first, last_ms: s.last }).collect();
            v.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
            v.truncate(n);
            v
        };
        let mut clients: Vec<ClientOut> = self
            .clients
            .into_values()
            .map(|c| ClientOut {
                platform: PLATFORMS[c.platform],
                device: c.device,
                os: c.os,
                browser: c.browser,
                count: c.seen.count,
                first_ms: c.seen.first,
                last_ms: c.seen.last,
            })
            .collect();
        clients.sort_by(|a, b| b.count.cmp(&a.count));
        let mut opened_from: Vec<(String, u32)> = self.opened_from.into_iter().collect();
        opened_from.sort_by(|a, b| b.1.cmp(&a.1));
        let mut open_hours: Vec<(u32, u32)> = self.open_hours.into_iter().collect();
        open_hours.sort_unstable();
        let home = self.countries.iter().max_by(|a, b| a.1.count.cmp(&b.1.count).then_with(|| b.0.cmp(a.0))).map(|(c, _)| c.clone());
        DeviceFacts {
            clients,
            platforms: self.platforms.into_iter().collect(),
            trips: home.map(|h| trips(self.country_days, &h)).unwrap_or_default(),
            countries: places(self.countries, 60),
            cities: places(self.cities, 40),
            isps: places(self.isps, 20),
            time_zones: places(self.time_zones, 30),
            opened_from,
            open_hours,
            opens: self.opens,
            sessions: self.sessions,
        }
    }
}

/// Trips: runs of days where most sessions came from one other country than `home` (at least
/// three that day), ended by a day back home, a different country or more than a day without
/// sessions abroad. Kept when they last two days or have twenty sessions, which skips most
/// one-off VPN sessions.
fn trips(country_days: HashMap<(String, u32), u32>, home: &str) -> Vec<TripOut> {
    // Each day's main country and how many sessions it had.
    let mut days: BTreeMap<u32, (String, u32)> = BTreeMap::new();
    for ((country, day), n) in country_days {
        let e = days.entry(day).or_insert((String::new(), 0));
        if n > e.1 || (n == e.1 && country == home) {
            *e = (country, n);
        }
    }
    let mut out = vec![];
    let mut cur: Option<(String, u32, u32, u32, u32)> = None; // country, first day, last day, days, sessions
    let close = |cur: &mut Option<(String, u32, u32, u32, u32)>, out: &mut Vec<TripOut>| {
        if let Some((country, first, last, n_days, sessions)) = cur.take() {
            if n_days >= 2 || sessions >= 20 {
                let start_ms = first as i64 * time::DAY_MS;
                out.push(TripOut { country, start_ms, end_ms: (last as i64 + 1) * time::DAY_MS - 1, days: last - first + 1, sessions });
            }
        }
    };
    for (day, (country, n)) in days {
        if country == home {
            close(&mut cur, &mut out);
            continue;
        }
        if n < 3 {
            continue;
        }
        match &mut cur {
            Some(t) if t.0 == country && day - t.2 <= 2 => {
                t.2 = day;
                t.3 += 1;
                t.4 += n;
            }
            _ => {
                close(&mut cur, &mut out);
                cur = Some((country, day, day, 1, n));
            }
        }
    }
    close(&mut cur, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips_need_a_few_days_or_sessions_abroad() {
        let mut m = HashMap::new();
        for d in (0..30).filter(|d| !(10..15).contains(d)) {
            m.insert(("NL".to_string(), d), 10);
        }
        // Four days in Italy, ten sessions each, with one quiet day in the middle.
        for d in [10, 11, 13, 14] {
            m.insert(("IT".to_string(), d), 10);
        }
        // One VPN day.
        m.insert(("US".to_string(), 20), 12);
        let t = trips(m, "NL");
        assert_eq!(t.len(), 1);
        assert_eq!((t[0].country.as_str(), t[0].days, t[0].sessions), ("IT", 5, 40));
    }
}
