//! Select and shard the corpus scenarios (every scenario but the route rooms)
//! by the seed bundle or movie route they restore (the deleted `.github/scripts/plan_corpus.py`),
//! and the rotating faithful-oracle sample (`lock_gate.oracle_sample`).

use std::collections::{BTreeMap, BTreeSet};

use crate::json::{Json, Object};
use crate::scenario::{Lock, Row};

/// Where a shard's private data comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A configured checkpoint prefix; each row is its own shard.
    Checkpoint,
    /// Captured inputs from a seed bundle.
    Captured,
    /// The any% movie, converted on the runner.
    Route,
    /// Original-emulator room seeds from a seed bundle.
    Seed,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Checkpoint => "checkpoint",
            Self::Captured => "captured",
            Self::Route => "route",
            Self::Seed => "seed",
        }
    }
}

/// One shard: rows restored together from one seed bundle or movie route.
#[derive(Debug, Clone, PartialEq)]
pub struct Shard {
    pub name: String,
    pub record_oracle: bool,
    pub cost: i64,
    pub kind: Kind,
    pub inputs: String,
    pub seed_asset: String,
    pub seed_sha256: String,
    pub route: String,
    pub segments: Vec<String>,
}

impl Shard {
    /// What a replay job restores for it (plan_verify.py SHARD).
    pub fn restore(&self) -> Object {
        Object::new()
            .with("kind", self.kind.as_str())
            .with("seed_asset", self.seed_asset.as_str())
            .with("seed_sha256", self.seed_sha256.as_str())
            .with("segments", self.segments.join(" "))
            .with("route", self.route.as_str())
            .with("inputs", self.inputs.as_str())
    }

    /// The restores, as the matrix's `shards` string.
    pub fn restores<'a>(shards: impl IntoIterator<Item = &'a Shard>) -> String {
        Json::from(shards.into_iter().map(Shard::restore).collect::<Vec<_>>()).to_string()
    }
}

/// The ci-data bundle with the row's private data, as scenarios/lock.toml pins it.
fn seed_bundle<'a>(row: &'a Row, lock: &'a Lock) -> Result<(&'a str, &'a str), String> {
    let name = row
        .bundle
        .as_deref()
        .ok_or_else(|| format!("{}: no pinned bundle", row.id))?;
    let digest = lock
        .bundles
        .get(name)
        .ok_or_else(|| format!("{}: bundle {name} has no SHA-256", row.id))?;
    Ok((name, digest))
}

/// Replay frames, plus, for a checkpoint row, the boot prefix it regenerates
/// when its checkpoint is rejected; that prefix replays about 5 times faster.
fn cost(kind: Kind, row: &Row) -> i64 {
    row.frames
        + if kind == Kind::Checkpoint {
            row.start_frame.div_euclid(5)
        } else {
            0
        }
}

/// The oracle sample's three-hour UTC slot.
pub fn oracle_slot(now: std::time::SystemTime) -> u64 {
    now.duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / (3 * 60 * 60))
}

/// At least `minimum` locked ids, rotating equally through the corpus and each
/// route every slot.
pub fn oracle_sample(
    rows: &[Row],
    locked: &BTreeSet<String>,
    slot: u64,
    minimum: usize,
) -> Result<Vec<String>, String> {
    let mut pools: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows.iter().filter(|row| locked.contains(&row.id)) {
        pools
            .entry(row.chain.as_deref().unwrap_or("corpus"))
            .or_default()
            .push(&row.id);
    }
    if pools.values().map(Vec::len).sum::<usize>() < minimum {
        return Err(format!(
            "oracle sample needs at least {minimum} locked scenarios"
        ));
    }
    // Where a pool's rotation starts this slot.
    let rotate = |slot: u64, quota: usize, length: usize| -> usize {
        let wide = |count: usize| u64::try_from(count).unwrap_or(u64::MAX);
        usize::try_from((slot * wide(quota)) % wide(length)).unwrap_or_default()
    };
    let mut selected: Vec<String> = Vec::new();
    let count = pools.len();
    for (index, pool) in pools.values_mut().enumerate() {
        pool.sort_unstable();
        let quota = minimum / count + usize::from(index < minimum % count);
        let start = rotate(slot, quota, pool.len());
        selected
            .extend((0..quota.min(pool.len())).map(|i| pool[(start + i) % pool.len()].to_owned()));
    }
    // Small pools contribute all their rows; fill the remaining places from the others.
    let chosen: BTreeSet<&str> = selected.iter().map(String::as_str).collect();
    let remaining: Vec<String> = rows
        .iter()
        .map(|row| row.id.as_str())
        .filter(|id| locked.contains(*id) && !chosen.contains(id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let short = minimum.saturating_sub(selected.len());
    if short > 0 {
        if remaining.is_empty() {
            return Err("oracle sample has no rows left to fill it".into());
        }
        let start = rotate(slot, short, remaining.len());
        selected.extend((0..short).map(|i| remaining[(start + i) % remaining.len()].clone()));
    }
    Ok(selected)
}

/// The locked ids whose entries lack the Session streams, once any entry
/// carries them (docs/handoff/session-compare.md, slice 5); none before.
/// The oracle sample records them all beside its rotation, so the scheduled
/// run after the first streams are committed completes the rest, and slice
/// 6 can drop `wram`.
pub fn lacking_session_streams(lock: &Lock, locked: &BTreeSet<String>) -> Vec<String> {
    let streams = |id: &str| {
        lock.entries
            .get(id)
            .and_then(|entry| entry.get("digests"))
            .and_then(crate::toml::Value::as_table)
            .is_some_and(|digests| digests.keys().any(|name| name.starts_with("session_")))
    };
    if !locked.iter().any(|id| streams(id)) {
        return Vec::new();
    }
    locked.iter().filter(|id| !streams(id)).cloned().collect()
}

/// What to shard: the selected corpus rows (all when empty) and, with a sample,
/// the faithful-oracle sample's rows.
pub struct Selection<'a> {
    pub selected: &'a [String],
    pub sample: Option<&'a [String]>,
}

/// The shards, heaviest first (`plan_corpus.py`).
pub fn shards(
    all_rows: &[Row],
    lock: &Lock,
    selection: &Selection<'_>,
) -> Result<Vec<Shard>, String> {
    let rows: Vec<&Row> = all_rows.iter().filter(|row| row.chain.is_none()).collect();
    let wanted = selection.selected;
    if !wanted.is_empty() {
        let known: BTreeSet<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        let unknown: BTreeSet<&str> = wanted
            .iter()
            .map(String::as_str)
            .filter(|id| !known.contains(id))
            .collect();
        if !unknown.is_empty() {
            return Err(format!("unknown rows: {unknown:?}"));
        }
    }
    // (records the oracle, kind, bundle or inputs) -> rows.
    let mut groups: BTreeMap<(bool, &str, &str), (Kind, Vec<&Row>)> = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|row| wanted.is_empty() || wanted.contains(&row.id))
    {
        if row.pending.is_some() {
            continue;
        }
        let kind = if row.checkpoint.is_some() {
            Kind::Checkpoint
        } else if row.seed.is_some() {
            Kind::Seed
        } else if row.inputs().starts_with("corpus-") {
            Kind::Route
        } else {
            Kind::Captured
        };
        let identity = if matches!(kind, Kind::Seed | Kind::Checkpoint) {
            seed_bundle(row, lock)?.0
        } else {
            row.inputs()
        };
        groups
            .entry((false, kind.as_str(), identity))
            .or_insert((kind, Vec::new()))
            .1
            .push(row);
    }
    if let Some(sampled) = selection.sample {
        let sampled: BTreeSet<&str> = sampled.iter().map(String::as_str).collect();
        // Route rooms are seeds too; the runtime-only restore supports explicitly
        // selected chain ids and checks their pinned bundle members.
        for row in all_rows
            .iter()
            .filter(|row| sampled.contains(row.id.as_str()))
        {
            let kind = if row.seed.is_some() {
                Kind::Seed
            } else if row.inputs().starts_with("corpus-") {
                Kind::Route
            } else {
                Kind::Captured
            };
            let identity = if kind == Kind::Seed {
                seed_bundle(row, lock)?.0
            } else {
                row.inputs()
            };
            groups
                .entry((true, kind.as_str(), identity))
                .or_insert((kind, Vec::new()))
                .1
                .push(row);
        }
    }
    let frame_cap = groups
        .iter()
        .filter(|((record, _, _), _)| !record)
        .flat_map(|(_, (_, members))| members.iter().map(|row| row.frames))
        .sum::<i64>() as f64
        / 12.0;
    let mut shards = Vec::new();
    for ((record, _, identity), (kind, mut members)) in groups {
        members.sort_by_key(|row| row.start_frame);
        for (index, chunk) in chunks(kind, members, frame_cap).into_iter().enumerate() {
            let first = chunk[0];
            let inputs = first.inputs().to_owned();
            let (seed_asset, seed_sha256) = if kind == Kind::Route {
                ("", "")
            } else {
                seed_bundle(first, lock)?
            };
            let mut name = if matches!(kind, Kind::Seed | Kind::Checkpoint) {
                let stem = identity.strip_prefix("layout150-").unwrap_or(identity);
                let stem = stem.strip_prefix("seeds-v1-").unwrap_or(stem);
                stem.split('-').next().unwrap_or_default().to_owned()
            } else {
                identity
                    .strip_suffix(".inputs")
                    .unwrap_or(identity)
                    .to_owned()
            };
            if record && let Some(chain) = &first.chain {
                name.clone_from(chain);
            }
            let route = if kind == Kind::Route {
                let stem = inputs.strip_prefix("corpus-").unwrap_or(&inputs);
                stem.strip_suffix(".inputs").unwrap_or(stem).to_owned()
            } else {
                String::new()
            };
            shards.push(Shard {
                name: format!(
                    "{}{}-{name}-{index}",
                    if record { "oracle-" } else { "" },
                    kind.as_str()
                ),
                record_oracle: record,
                cost: chunk.iter().map(|row| cost(kind, row)).sum(),
                kind,
                inputs,
                seed_asset: seed_asset.to_owned(),
                seed_sha256: seed_sha256.to_owned(),
                route,
                segments: chunk.iter().map(|row| row.id.clone()).collect(),
            });
        }
    }
    // Start the heaviest shards first; the workflow bounds simultaneous jobs.
    shards.sort_by_key(|shard| -shard.cost);
    if shards.is_empty() {
        return Err("no runnable rows selected".into());
    }
    Ok(shards)
}

/// A rejected configured checkpoint regenerates its own complete boot prefix,
/// so each checkpoint row is its own shard. Seed groups share only a
/// seconds-long bundle restore, so one heavier than a twelfth of the corpus
/// splits; other groups split by nine rows.
fn chunks(kind: Kind, members: Vec<&Row>, frame_cap: f64) -> Vec<Vec<&Row>> {
    match kind {
        Kind::Checkpoint => members.into_iter().map(|row| vec![row]).collect(),
        Kind::Seed => {
            let mut out = Vec::new();
            let mut current: Vec<&Row> = Vec::new();
            for row in members {
                let frames: i64 = current.iter().map(|member| member.frames).sum();
                if !current.is_empty() && (frames + row.frames) as f64 > frame_cap {
                    out.push(std::mem::take(&mut current));
                }
                current.push(row);
            }
            out.push(current);
            out
        }
        Kind::Captured | Kind::Route => members.chunks(9).map(<[&Row]>::to_vec).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locked_row(id: &str, chain: Option<&str>) -> Row {
        Row {
            id: id.to_owned(),
            chain: chain.map(str::to_owned),
            ..Row::default()
        }
    }

    #[test]
    fn the_sample_rotates_through_each_pool() {
        let mut rows: Vec<Row> = (0..40)
            .map(|i| locked_row(&format!("c{i:02}"), None))
            .collect();
        rows.extend((0..3).map(|i| locked_row(&format!("r{i}"), Some("100"))));
        let locked: BTreeSet<String> = rows.iter().map(|row| row.id.clone()).collect();
        let sample = oracle_sample(&rows, &locked, 1, 30).unwrap();
        assert_eq!(sample.len(), 30);
        // Route 100 holds three rows, under its quota of fifteen: all three, then
        // fifteen corpus rows from slot * 15 and twelve more to fill.
        assert_eq!(&sample[..3], ["r0", "r1", "r2"]);
        assert_eq!(sample[3], "c15");
        assert_eq!(sample.iter().collect::<BTreeSet<_>>().len(), 30);
        assert!(oracle_sample(&rows[..10], &locked, 1, 30).is_err());
    }

    #[test]
    fn the_sample_completes_the_session_streams_once_they_start() {
        let entry = |digests: &str| {
            crate::toml::parse(&format!("digests = {{ {digests} }}")).expect("entry")
        };
        let locked: BTreeSet<String> = ["a", "b", "c"].map(str::to_owned).into();
        let mut lock = Lock::default();
        for id in &locked {
            lock.entries.insert(id.clone(), entry(r#""wram" = "0""#));
        }
        assert!(lacking_session_streams(&lock, &locked).is_empty());
        lock.entries.insert(
            "b".to_owned(),
            entry(r#""wram" = "0", "session_loop" = "1""#),
        );
        assert_eq!(lacking_session_streams(&lock, &locked), ["a", "c"]);
    }
}
