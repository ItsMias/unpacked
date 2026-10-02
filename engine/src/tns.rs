//! Events only the trust & safety log (`Activity/tns`) has: Go Live streams you ran and watched,
//! edited messages, two-factor changes, calls you started and emoji you uploaded. That log only
//! goes back about two years, so `since_ms` says where it starts.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::time::{DAY_MS, month_index};
use crate::voice::{split_by_day, union};

/// Streams longer than this are logging glitches and get clipped, like voice sessions.
const MAX_STREAM_MS: i64 = 24 * 3_600_000;

#[derive(Default)]
pub struct Tns {
    streamed: Vec<(i64, i64)>,
    watched: Vec<(i64, i64)>,
    edits: BTreeMap<u16, u32>,
    two_factor: Vec<(i64, bool)>,
    calls_started: Vec<i64>,
    emoji_created: Vec<i64>,
    since_ms: i64,
}

#[derive(Serialize, Debug, Default)]
pub struct TnsFacts {
    /// Go Live streams you ran: `[start ms, end ms]`, overlaps merged, oldest first.
    pub streamed: Vec<(i64, i64)>,
    /// Hours watching other people's streams, per month (see [`crate::time::month_index`]).
    pub watched: Vec<(u16, f32)>,
    /// Messages you edited, per month.
    pub edits: Vec<(u16, u32)>,
    /// Two-factor authentication turned on (`true`) or off, oldest first.
    pub two_factor: Vec<(i64, bool)>,
    pub calls_started: Vec<i64>,
    pub emoji_created: Vec<i64>,
    /// The oldest of these events.
    pub since_ms: i64,
}

impl Tns {
    fn seen(&mut self, ms: i64) {
        if self.since_ms == 0 || ms < self.since_ms {
            self.since_ms = ms;
        }
    }

    /// A `video_stream_ended`: `role` is `streamer` for your own Go Live and `receiver` for
    /// watching someone else's. (`sender` looks like camera video and isn't counted.)
    pub fn stream_ended(&mut self, end_ms: i64, secs: i64, role: &str) {
        if secs <= 0 {
            return;
        }
        let span = (end_ms - (secs * 1000).min(MAX_STREAM_MS), end_ms);
        match role {
            "streamer" => self.streamed.push(span),
            "receiver" => self.watched.push(span),
            _ => return,
        }
        self.seen(span.0);
    }

    pub fn edited(&mut self, ms: i64) {
        *self.edits.entry(month_index(ms)).or_default() += 1;
        self.seen(ms);
    }

    pub fn two_factor(&mut self, ms: i64, on: bool) {
        self.two_factor.push((ms, on));
        self.seen(ms);
    }

    pub fn call_started(&mut self, ms: i64) {
        self.calls_started.push(ms);
        self.seen(ms);
    }

    pub fn emoji_created(&mut self, ms: i64) {
        self.emoji_created.push(ms);
        self.seen(ms);
    }

    pub fn finish(self) -> TnsFacts {
        let mut watched: BTreeMap<u16, f64> = BTreeMap::new();
        for (s, e) in union(self.watched) {
            split_by_day(s, e, |d, h| *watched.entry(month_index(d as i64 * DAY_MS)).or_default() += h);
        }
        let sorted = |mut v: Vec<i64>| {
            v.sort_unstable();
            v
        };
        let mut two_factor = self.two_factor;
        two_factor.sort_unstable();
        TnsFacts {
            streamed: union(self.streamed),
            watched: watched.into_iter().map(|(m, h)| (m, h as f32)).collect(),
            edits: self.edits.into_iter().collect(),
            two_factor,
            calls_started: sorted(self.calls_started),
            emoji_created: sorted(self.emoji_created),
            since_ms: self.since_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const H: i64 = 3_600_000;

    #[test]
    fn streams_merge_and_split_by_role() {
        let mut t = Tns::default();
        t.stream_ended(10 * H, 3600, "streamer");
        t.stream_ended(10 * H + 600_000, 1800, "streamer"); // overlaps the first
        t.stream_ended(30 * H, 7200, "receiver");
        t.stream_ended(40 * H, 7200, "sender"); // camera: not a stream
        let f = t.finish();
        assert_eq!(f.streamed, vec![(9 * H, 10 * H + 600_000)]);
        assert!((f.watched.iter().map(|x| x.1).sum::<f32>() - 2.0).abs() < 1e-6);
        assert_eq!(f.since_ms, 9 * H);
    }
}
