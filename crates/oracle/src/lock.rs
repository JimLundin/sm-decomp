//! The scenario lock (`scenarios/lock.toml`): its pins and its entries'
//! running digests. A small reader for the lock's own line format.

use std::collections::BTreeMap;

/// One `[[scenario]]` entry.
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub id: String,
    pub oracle: String,
    pub start: String,
    pub inputs: String,
    pub ticks: u32,
    pub combined: String,
    pub digests: BTreeMap<String, String>,
}

/// One pin: where a scenario's private data lives.
#[derive(Clone, Debug, Default)]
pub struct Pin {
    pub id: String,
    pub bundle: String,
    pub seed: Option<String>,
    pub inputs: String,
    pub checkpoint: Option<String>,
}

/// The parsed lock.
#[derive(Clone, Debug, Default)]
pub struct Lock {
    pub pins: Vec<Pin>,
    pub entries: Vec<Entry>,
}

/// The `key = "value"` pairs of an inline table `{ ... }`.
fn inline_table(text: &str) -> BTreeMap<String, String> {
    let body = text
        .trim()
        .trim_start_matches('{')
        .trim_end_matches(',')
        .trim_end_matches('}');
    body.split(',')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            let unquote = |text: &str| text.trim().trim_matches('"').to_owned();
            Some((unquote(key), unquote(value)))
        })
        .collect()
}

fn unquote(text: &str) -> String {
    text.trim().trim_matches('"').to_owned()
}

impl Lock {
    pub fn parse(text: &str) -> Self {
        let mut lock = Self::default();
        let mut entry: Option<Entry> = None;
        for line in text.lines() {
            let line = line.trim();
            if line == "[[scenario]]" {
                lock.entries.extend(entry.take());
                entry = Some(Entry::default());
                continue;
            }
            if let Some(pin) = line.strip_prefix("{ id = ") {
                let table = inline_table(&format!("{{ id = {pin}"));
                let get = |key: &str| table.get(key).cloned();
                lock.pins.push(Pin {
                    id: get("id").unwrap_or_default(),
                    bundle: get("bundle").unwrap_or_default(),
                    seed: get("seed"),
                    inputs: get("inputs").unwrap_or_default(),
                    checkpoint: get("checkpoint"),
                });
                continue;
            }
            let (Some(current), Some((key, value))) = (entry.as_mut(), line.split_once(" = "))
            else {
                continue;
            };
            match key {
                "id" => current.id = unquote(value),
                "oracle" => current.oracle = unquote(value),
                "start" => current.start = unquote(value),
                "inputs" => current.inputs = unquote(value),
                "ticks" => current.ticks = value.trim().parse().unwrap_or(0),
                "combined" => current.combined = unquote(value),
                "digests" => current.digests = inline_table(value),
                _ => (),
            }
        }
        lock.entries.extend(entry);
        lock
    }

    /// The pin of scenario `id`.
    pub fn pin(&self, id: &str) -> Option<&Pin> {
        self.pins.iter().find(|pin| pin.id == id)
    }

    /// The entry of scenario `id` from `oracle`.
    pub fn entry(&self, id: &str, oracle: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|entry| entry.id == id && entry.oracle == oracle)
    }
}
