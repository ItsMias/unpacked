//! JavaScript bindings (only compiled for wasm32).

use wasm_bindgen::prelude::*;

use crate::{Engine, Entry};

const KINDS: [Entry; 11] = [
    Entry::User,
    Entry::MessagesIndex,
    Entry::ServersIndex,
    Entry::Quests,
    Entry::ChannelMeta,
    Entry::ChannelMessages,
    Entry::Events,
    Entry::EventsFallback,
    Entry::Analytics,
    Entry::Traits,
    Entry::Poker,
];

/// Entry kind codes shared with `web/analyze.js`: index into `KINDS`, or -1 for "don't read".
#[wasm_bindgen]
pub fn wanted(path: &str) -> i32 {
    crate::wanted(path).map_or(-1, |k| KINDS.iter().position(|x| *x == k).unwrap() as i32)
}

#[wasm_bindgen]
pub struct WasmEngine(Engine);

#[wasm_bindgen]
impl WasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEngine {
        WasmEngine(Engine::new())
    }

    pub fn feed_file(&mut self, kind: i32, bytes: &[u8]) -> Result<(), JsError> {
        let kind = *KINDS.get(kind as usize).ok_or_else(|| JsError::new("bad entry kind"))?;
        self.0.feed_file(kind, bytes).map_err(|e| JsError::new(&e))
    }

    pub fn feed_channel(&mut self, channel_json: &[u8], messages_json: &[u8]) -> Result<(), JsError> {
        self.0.feed_channel(channel_json, messages_json).map_err(|e| JsError::new(&e))
    }

    pub fn feed_events(&mut self, chunk: &[u8]) {
        self.0.feed_events(chunk);
    }

    pub fn feed_analytics(&mut self, chunk: &[u8]) {
        self.0.feed_analytics(chunk);
    }

    /// Returns how many events the file had.
    pub fn finish_events(&mut self) -> f64 {
        self.0.finish_events() as f64
    }

    /// Events per type since the last call, as JSON `[[type, count], …]`.
    pub fn take_event_types(&mut self) -> String {
        serde_json::to_string(&self.0.take_event_types()).unwrap_or_else(|_| "[]".into())
    }

    /// Live counters as JSON, for the loading screen.
    pub fn progress(&self) -> String {
        serde_json::to_string(&self.0.progress()).unwrap_or_else(|_| "{}".into())
    }

    /// Messages per UTC day so far: `[day, count, …]`.
    pub fn day_counts(&self) -> Vec<u32> {
        self.0.day_counts()
    }

    /// Voice minutes and gone messages per UTC day so far: `[day, minutes, gone, …]`.
    pub fn activity_days(&self) -> Vec<u32> {
        self.0.activity_days()
    }

    /// Playtime minutes per UTC day so far: `[day, minutes, …]`.
    pub fn game_days(&self) -> Vec<u32> {
        self.0.game_days()
    }

    pub fn has_voice(&self) -> bool {
        self.0.has_voice()
    }

    /// Consumes the engine and returns the facts as JSON.
    pub fn finish(self) -> String {
        serde_json::to_string(&self.0.finish()).unwrap_or_else(|_| "{}".into())
    }
}
