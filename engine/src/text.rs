//! What your messages say: words, lengths, links, attachments. Counted per calendar year so the
//! page can filter by range.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

/// Year slots: index 0 is 2015 (Discord's launch), the last one catches anything later.
pub const YEARS: usize = 16;
pub const FIRST_YEAR: i64 = 2015;

/// Message length buckets, in words: none (attachment or sticker only), 1, 2, 3–5, 6–10, 11–20, 21–50, 51+.
const LENGTH_EDGES: [usize; 7] = [0, 1, 2, 5, 10, 20, 50];

const TOP_WORDS: usize = 400;
const TOP_DOMAINS: usize = 60;
const QUOTE_CHARS: usize = 2000;
/// How many of the longest messages to keep.
const LONGEST: usize = 20;
/// "Walls of text": runs of long messages (at least this many characters) in one channel,
/// each sent within `WALL_GAP_MS` of the previous long one.
const LONG_CHARS: usize = 200;
const WALL_GAP_MS: i64 = 10 * 60_000;
const WALLS: usize = 10;
const WALL_PARTS: usize = 12;
const WALL_PART_CHARS: usize = 700;

/// Filler words left out of the word ranking: English and Dutch function words. Slang and
/// catchphrases ("lol", "xd", "haha") stay in, they're the interesting part.
const FILLER: &[&str] = &[
    // English
    "a", "about", "above", "after", "again", "against", "all", "also", "am", "an", "and", "any", "are",
    "aren't", "as", "at", "be", "because", "been", "before", "being", "below", "between", "both", "but",
    "by", "can", "can't", "cannot", "could", "couldn't", "did", "didn't", "do", "does", "doesn't", "doing",
    "don't", "down", "during", "each", "even", "ever", "few", "for", "from", "further", "get", "got", "had",
    "hadn't", "has", "hasn't", "have", "haven't", "having", "he", "he's", "her", "here", "here's", "hers",
    "herself", "him", "himself", "his", "how", "i", "i'd", "i'll", "i'm", "i've", "if", "im", "in", "into",
    "is", "isn't", "it", "it's", "its", "itself", "just", "let's", "me", "more", "most", "much", "my",
    "myself", "no", "nor", "not", "now", "of", "off", "on", "once", "only", "or", "other", "our", "ours",
    "ourselves", "out", "over", "own", "same", "she", "she's", "should", "shouldn't", "so", "some", "such",
    "than", "that", "that's", "the", "their", "theirs", "them", "themselves", "then", "there", "there's",
    "these", "they", "they'll", "they're", "they've", "this", "those", "through", "to", "too", "under",
    "until", "up", "very", "was", "wasn't", "we", "we'll", "we're", "we've", "were", "weren't", "what",
    "what's", "when", "where", "which", "while", "who", "who's", "whom", "why", "will", "with", "won't",
    "would", "wouldn't", "you", "you'd", "you'll", "you're", "you've", "your", "yours", "yourself",
    "yourselves", "dont", "didnt", "doesnt", "cant", "wont", "isnt", "thats", "theres", "youre", "ive",
    "ill", "id", "its", "u", "ur", "s", "t", "d", "ll", "re", "ve", "m",
    // Dutch
    "aan", "al", "alle", "als", "ben", "bij", "da", "dan", "dat", "de", "der", "deze", "die", "dit", "doch",
    "doen", "door", "dus", "een", "en", "er", "ge", "geen", "had", "heb", "hebben", "heeft", "hem", "het",
    "hier", "hij", "hoe", "hun", "ik", "in", "is", "je", "jij", "jou", "jouw", "jullie", "kan", "kon",
    "kunnen", "maar", "me", "meer", "men", "met", "mij", "mijn", "moet", "na", "naar", "niet", "niets",
    "nog", "nu", "of", "om", "omdat", "ons", "onze", "ook", "op", "over", "te", "tegen", "toch", "toen",
    "tot", "uit", "van", "veel", "voor", "want", "was", "wat", "we", "wel", "werd", "wezen", "wie", "wij",
    "wil", "worden", "word", "wordt", "zal", "ze", "zei", "zelf", "zich", "zij", "zijn", "zo", "zou",
    "'t", "t'", "'s", "m'n", "z'n", "d'r",
];

#[derive(Serialize, Debug)]
pub struct TextStats {
    /// Per year slot: words, characters, messages with an attachment, messages with a link.
    pub words: Vec<u64>,
    pub chars: Vec<u64>,
    pub with_attachments: Vec<u64>,
    pub with_links: Vec<u64>,
    /// Per year slot, messages per length bucket (see `LENGTH_EDGES`).
    pub lengths: Vec<[u32; 8]>,
    pub top_words: Vec<Counted>,
    /// The filler words left out of `top_words`, counted the same way.
    pub filler_words: Vec<Counted>,
    pub top_domains: Vec<Counted>,
    pub first: Option<Quote>,
    /// The longest messages, longest first.
    pub longest: Vec<Quote>,
    /// Runs of long messages sent one after another, biggest first.
    pub walls: Vec<Wall>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Wall {
    pub channel: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub count: usize,
    /// Characters across the whole run.
    pub chars: usize,
    /// The messages, oldest first (cut short, and only the first few).
    pub parts: Vec<(i64, String)>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Counted {
    pub text: String,
    /// Count per year slot.
    pub years: Vec<u32>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Quote {
    pub ms: i64,
    pub channel: String,
    pub text: String,
    /// Full length, in characters (the text may be cut).
    pub chars: usize,
}

pub struct Text {
    filler: HashSet<&'static str>,
    words: [u64; YEARS],
    chars: [u64; YEARS],
    with_attachments: [u64; YEARS],
    with_links: [u64; YEARS],
    lengths: [[u32; 8]; YEARS],
    word_counts: HashMap<String, [u32; YEARS]>,
    filler_counts: HashMap<&'static str, [u32; YEARS]>,
    domains: HashMap<String, [u32; YEARS]>,
    first: Option<Quote>,
    longest: Vec<Quote>,
    walls: Vec<Wall>,
    /// Long messages of the channel being read, for finding walls of text.
    run: Vec<(i64, usize, String)>,
    buf: String,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            filler: FILLER.iter().copied().collect(),
            words: [0; YEARS],
            chars: [0; YEARS],
            with_attachments: [0; YEARS],
            with_links: [0; YEARS],
            lengths: [[0; 8]; YEARS],
            word_counts: HashMap::new(),
            filler_counts: HashMap::new(),
            domains: HashMap::new(),
            first: None,
            longest: Vec::new(),
            walls: Vec::new(),
            run: Vec::new(),
            buf: String::new(),
        }
    }
}

pub fn year_slot(year: i64) -> usize {
    (year - FIRST_YEAR).clamp(0, YEARS as i64 - 1) as usize
}

fn quote(ms: i64, channel: &str, text: &str, chars: usize) -> Quote {
    Quote { ms, channel: channel.to_owned(), text: text.chars().take(QUOTE_CHARS).collect(), chars }
}

impl Text {
    pub fn add(&mut self, year: i64, ms: i64, channel: &str, text: &str, has_attachment: bool) {
        let y = year_slot(year);
        let chars = text.chars().count();
        let mut words = 0;
        let mut has_link = false;
        for token in text.split_whitespace() {
            words += 1;
            if let Some(host) = link_host(token) {
                has_link = true;
                self.domains.entry(host).or_insert([0; YEARS])[y] += 1;
                continue;
            }
            // Mentions, channels and custom emoji: <@123>, <#123>, <:name:123>.
            if token.starts_with('<') && token.ends_with('>') {
                continue;
            }
            for word in token.split(|c: char| !(c.is_alphanumeric() || c == '\'')) {
                self.count_word(word, y);
            }
        }
        self.words[y] += words as u64;
        self.chars[y] += chars as u64;
        self.with_attachments[y] += has_attachment as u64;
        self.with_links[y] += has_link as u64;
        let bucket = LENGTH_EDGES.iter().filter(|&&e| words > e).count();
        self.lengths[y][bucket] += 1;

        if !text.trim().is_empty() {
            if self.first.as_ref().is_none_or(|f| ms < f.ms) {
                self.first = Some(quote(ms, channel, text, chars));
            }
            if self.longest.len() < LONGEST || chars > self.longest.last().map_or(0, |l| l.chars) {
                let at = self.longest.partition_point(|l| l.chars >= chars);
                self.longest.insert(at, quote(ms, channel, text, chars));
                self.longest.truncate(LONGEST);
            }
            if chars >= LONG_CHARS {
                self.run.push((ms, chars, text.chars().take(WALL_PART_CHARS).collect()));
            }
        }
    }

    /// Call after the last message of each channel: turns its long messages into walls of text.
    pub fn end_channel(&mut self, channel: &str) {
        let mut run = std::mem::take(&mut self.run);
        run.sort_by_key(|m| m.0);
        let mut i = 0;
        while i < run.len() {
            let mut j = i + 1;
            while j < run.len() && run[j].0 - run[j - 1].0 <= WALL_GAP_MS {
                j += 1;
            }
            if j - i >= 2 {
                let chars = run[i..j].iter().map(|m| m.1).sum();
                if self.walls.len() < WALLS || chars > self.walls.last().map_or(0, |w| w.chars) {
                    let wall = Wall {
                        channel: channel.to_owned(),
                        start_ms: run[i].0,
                        end_ms: run[j - 1].0,
                        count: j - i,
                        chars,
                        parts: run[i..j].iter().take(WALL_PARTS).map(|m| (m.0, m.2.clone())).collect(),
                    };
                    let at = self.walls.partition_point(|w| w.chars >= chars);
                    self.walls.insert(at, wall);
                    self.walls.truncate(WALLS);
                }
            }
            i = j;
        }
    }

    fn count_word(&mut self, raw: &str, y: usize) {
        let w = raw.trim_matches('\'');
        if w.chars().count() < 2 || w.chars().all(|c| c.is_ascii_digit()) {
            return;
        }
        self.buf.clear();
        self.buf.extend(w.chars().flat_map(char::to_lowercase));
        if let Some(&f) = self.filler.get(self.buf.as_str()) {
            self.filler_counts.entry(f).or_insert([0; YEARS])[y] += 1;
            return;
        }
        match self.word_counts.get_mut(self.buf.as_str()) {
            Some(c) => c[y] += 1,
            None => {
                let mut c = [0; YEARS];
                c[y] = 1;
                self.word_counts.insert(self.buf.clone(), c);
            }
        }
    }

    pub fn finish(self) -> TextStats {
        let top = |m: HashMap<String, [u32; YEARS]>, n: usize| {
            let mut v: Vec<(String, [u32; YEARS], u64)> =
                m.into_iter().map(|(k, c)| { let t = c.iter().map(|&x| x as u64).sum(); (k, c, t) }).collect();
            v.sort_unstable_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
            v.truncate(n);
            v.into_iter().map(|(text, c, _)| Counted { text, years: c.to_vec() }).collect()
        };
        TextStats {
            words: self.words.to_vec(),
            chars: self.chars.to_vec(),
            with_attachments: self.with_attachments.to_vec(),
            with_links: self.with_links.to_vec(),
            lengths: self.lengths.to_vec(),
            top_words: top(self.word_counts, TOP_WORDS),
            filler_words: top(self.filler_counts.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(), FILLER.len()),
            top_domains: top(self.domains, TOP_DOMAINS),
            first: self.first,
            longest: self.longest,
            walls: self.walls,
        }
    }
}

/// Host of an http(s) link token, without "www.".
fn link_host(token: &str) -> Option<String> {
    let start = token.find("http://").or_else(|| token.find("https://"))?;
    let rest = &token[start..];
    let rest = &rest[rest.find("://")? + 3..];
    let end = rest.find(['/', ':', '?', '#', '>', ')', '"']).unwrap_or(rest.len());
    let host = rest[..end].trim_end_matches('.').to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    (host.contains('.') && host.len() > 3).then(|| host.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words_links_and_lengths() {
        let mut t = Text::default();
        t.add(2024, 10, "c1", "Lol the cat is SO funny lol https://www.YouTube.com/watch?v=1 <@123>", false);
        t.add(2024, 5, "c2", "", true);
        t.add(2025, 20, "c1", "ik ben er bijna, lol", false);
        let s = t.finish();
        let y24 = year_slot(2024);
        assert_eq!(s.words[y24], 9);
        assert_eq!(s.with_attachments[y24], 1);
        assert_eq!(s.with_links[y24], 1);
        assert_eq!(s.lengths[y24][0], 1); // the attachment-only message
        assert_eq!(s.lengths[y24][4], 1); // 6–10 words
        assert_eq!(s.top_words[0].text, "lol");
        assert_eq!(s.top_words[0].years.iter().sum::<u32>(), 3);
        assert!(!s.top_words.iter().any(|w| ["the", "is", "ik", "ben", "er", "123"].contains(&w.text.as_str())));
        assert!(s.filler_words.iter().any(|w| w.text == "the"));
        assert_eq!(s.top_domains[0].text, "youtube.com");
        assert_eq!(s.first.unwrap().ms, 10);
        assert_eq!(s.longest[0].channel, "c1");
    }

    #[test]
    fn walls_of_text() {
        let long = "word ".repeat(60); // 300 characters
        let mut t = Text::default();
        for (i, ms) in [0, 60_000, 120_000, 3_600_000].into_iter().enumerate() {
            t.add(2024, ms, "c1", &long, false);
            if i == 1 {
                t.add(2024, 90_000, "c1", "short reply", false);
            }
        }
        t.end_channel("c1");
        let s = t.finish();
        // Three long messages within minutes of each other; the one an hour later stands alone.
        assert_eq!(s.walls.len(), 1);
        assert_eq!((s.walls[0].count, s.walls[0].chars), (3, 900));
        assert_eq!(s.longest.len(), 5);
        assert!(s.longest.windows(2).all(|w| w[0].chars >= w[1].chars));
    }

    #[test]
    fn link_hosts() {
        assert_eq!(link_host("<https://discord.gg/abc>").as_deref(), Some("discord.gg"));
        assert_eq!(link_host("(see:https://a.b.example.org:8080/x)").as_deref(), Some("a.b.example.org"));
        assert_eq!(link_host("http://localhost"), None);
        assert_eq!(link_host("nope"), None);
    }
}
