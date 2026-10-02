//! Voice time, rebuilt from `leave_voice_channel` / `voice_disconnect` events.
//!
//! Both events carry how long the connection lasted, and the event timestamp is when it ended,
//! so each one gives an interval `[end - duration, end]`. The two event types overlap heavily,
//! so time is summed as the *union* of intervals, never a plain total.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::time::{DAY_MS, month_index};

/// Sessions longer than this are treated as logging glitches and clipped.
const MAX_SESSION_MS: i64 = 24 * 3_600_000;

/// Where the voice time was spent: a server, or a DM/group call channel.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VoiceKey {
    Guild(String),
    Channel(String),
}

#[derive(Default)]
pub struct Voice {
    intervals: Vec<(i64, i64, VoiceKey)>,
    seen_event_ids: HashSet<String>,
}

/// All-time voice time, bucketed so the page can filter by any date range.
pub struct VoiceSummary {
    pub hours: f64,
    pub sessions: usize,
    /// (UTC day number, hours that day, sessions that started that day).
    pub days: Vec<(u32, f32, u16)>,
    /// Hours per place, per month (see [`crate::time::month_index`]).
    pub by_key: HashMap<VoiceKey, Vec<(u16, f32)>>,
    /// (UTC hour index, hours in voice during that hour).
    pub by_hour: Vec<(u32, f32)>,
    /// The longest unbroken sessions in one place: (start, end, place), longest first.
    pub longest: Vec<(i64, i64, VoiceKey)>,
}

const LONGEST: usize = 10;

impl Voice {
    /// Records one session. `event_id` de-duplicates events that appear in several folders.
    pub fn add(&mut self, event_id: Option<&str>, end_ms: i64, duration_ms: i64, key: VoiceKey) {
        if duration_ms <= 0 {
            return;
        }
        if let Some(id) = event_id {
            if !self.seen_event_ids.insert(id.to_owned()) {
                return;
            }
        }
        let start = end_ms - duration_ms.min(MAX_SESSION_MS);
        self.intervals.push((start, end_ms, key));
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    /// Voice events recorded so far.
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    /// Hours per UTC day so far, overlaps merged, for the loading screen's calendar.
    pub fn day_hours(&self) -> Vec<(u32, f64)> {
        let mut days: BTreeMap<u32, f64> = BTreeMap::new();
        for (s, e) in union(self.intervals.iter().map(|(s, e, _)| (*s, *e)).collect()) {
            split_by_day(s, e, |d, h| *days.entry(d).or_default() += h);
        }
        days.into_iter().collect()
    }

    pub fn summary(&self) -> VoiceSummary {
        let merged = union(self.intervals.iter().map(|(s, e, _)| (*s, *e)).collect());

        let mut days: BTreeMap<u32, (f64, u16)> = BTreeMap::new();
        let mut by_hour: BTreeMap<u32, f64> = BTreeMap::new();
        for &(s, e) in &merged {
            days.entry(day(s)).or_default().1 += 1;
            split_by_day(s, e, |d, h| days.entry(d).or_default().0 += h);
            split_by_hour(s, e, |hr, h| *by_hour.entry(hr).or_default() += h);
        }

        let mut per_key: HashMap<VoiceKey, Vec<(i64, i64)>> = HashMap::new();
        for (s, e, k) in &self.intervals {
            per_key.entry(k.clone()).or_default().push((*s, *e));
        }
        let mut sessions: Vec<(i64, i64, VoiceKey)> = vec![];
        let by_key = per_key
            .into_iter()
            .map(|(k, v)| {
                let mut months: BTreeMap<u16, f64> = BTreeMap::new();
                for (s, e) in union(v) {
                    split_by_day(s, e, |d, h| {
                        *months.entry(month_index(d as i64 * DAY_MS)).or_default() += h;
                    });
                    sessions.push((s, e, k.clone()));
                }
                (k, months.into_iter().map(|(m, h)| (m, h as f32)).collect())
            })
            .collect();
        sessions.sort_by_key(|(s, e, _)| std::cmp::Reverse(e - s));
        sessions.truncate(LONGEST);

        VoiceSummary {
            hours: hours(&merged),
            sessions: merged.len(),
            days: days.into_iter().map(|(d, (h, n))| (d, h as f32, n)).collect(),
            by_key,
            by_hour: by_hour.into_iter().map(|(hr, h)| (hr, h as f32)).collect(),
            longest: sessions,
        }
    }
}

fn day(ms: i64) -> u32 {
    ms.div_euclid(DAY_MS) as u32
}

pub(crate) fn union(mut v: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    v.sort_unstable();
    let mut out: Vec<(i64, i64)> = Vec::with_capacity(v.len());
    for (s, e) in v {
        match out.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => out.push((s, e)),
        }
    }
    out
}

fn hours(v: &[(i64, i64)]) -> f64 {
    v.iter().map(|(s, e)| (e - s) as f64).sum::<f64>() / 3_600_000.0
}

/// Calls `f(hour index, hours)` for each UTC hour the interval touches.
fn split_by_hour(mut s: i64, e: i64, mut f: impl FnMut(u32, f64)) {
    const HOUR: i64 = 3_600_000;
    while s < e {
        let next = (s.div_euclid(HOUR) + 1) * HOUR;
        let end = next.min(e);
        f(s.div_euclid(HOUR) as u32, (end - s) as f64 / HOUR as f64);
        s = end;
    }
}

/// Calls `f(day, hours)` for each UTC day the interval touches.
pub(crate) fn split_by_day(mut s: i64, e: i64, mut f: impl FnMut(u32, f64)) {
    while s < e {
        let next = (s.div_euclid(DAY_MS) + 1) * DAY_MS;
        let end = next.min(e);
        f(day(s), (end - s) as f64 / 3_600_000.0);
        s = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const H: i64 = 3_600_000;

    fn g(id: &str) -> VoiceKey {
        VoiceKey::Guild(id.into())
    }

    #[test]
    fn overlapping_events_are_not_double_counted() {
        let mut v = Voice::default();
        // leave_voice_channel and voice_disconnect for the same 2h session.
        v.add(Some("a"), 10 * H, 2 * H, g("1"));
        v.add(Some("b"), 10 * H + 60_000, 2 * H, g("1"));
        let t = v.summary();
        assert!((t.hours - (2.0 + 1.0 / 60.0)).abs() < 1e-9);
        assert_eq!(t.sessions, 1);
    }

    #[test]
    fn duplicate_event_ids_are_ignored() {
        let mut v = Voice::default();
        v.add(Some("x"), 5 * H, H, g("1"));
        v.add(Some("x"), 5 * H, H, g("1"));
        assert!((v.summary().hours - 1.0).abs() < 1e-9);
    }

    #[test]
    fn split_by_day_and_capped() {
        let mut v = Voice::default();
        v.add(None, 26 * H, 4 * H, g("1")); // 22h day 0 .. 2h day 1
        let t = v.summary();
        assert_eq!(t.days.len(), 2);
        assert!((t.days[0].1 - 2.0).abs() < 1e-6 && t.days[0].2 == 1);
        assert!((t.days[1].1 - 2.0).abs() < 1e-6 && t.days[1].2 == 0);
        let mut v = Voice::default();
        v.add(None, 100 * H, 90 * H, g("1")); // glitch, capped to 24h
        assert!((v.summary().hours - 24.0).abs() < 1e-9);
    }

    #[test]
    fn per_key_totals() {
        let mut v = Voice::default();
        v.add(None, 2 * H, H, g("1"));
        v.add(None, 2 * H, H, VoiceKey::Channel("dm".into()));
        let t = v.summary();
        assert!((t.hours - 1.0).abs() < 1e-9); // same hour, two places at once
        assert!((t.by_key[&g("1")][0].1 - 1.0).abs() < 1e-6);
        assert!((t.by_key[&VoiceKey::Channel("dm".into())][0].1 - 1.0).abs() < 1e-6);
        assert_eq!(t.longest.len(), 2);
        assert_eq!(t.by_hour, vec![(1, 1.0)]);
    }
}
