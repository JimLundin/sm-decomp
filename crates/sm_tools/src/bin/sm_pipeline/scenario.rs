//! Scenarios as data (docs/SCENARIO_HARNESS.md): the scenario files, their
//! lock and the runner's rows, as `tools/scenario.py` reads them, and its
//! `lock --check` (`problems`, `check`).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use crate::toml::{self, Table, Value};

pub const LOCK: &str = "scenarios/lock.toml";
/// The observation domains every scenario compares (`tools/scenario.py` DOMAINS).
pub const DOMAINS: [&str; 8] = [
    "ram", "input", "samus", "camera", "room", "progress", "actors", "effects",
];
const FIELDS: [&str; 11] = [
    "id",
    "start",
    "prefix",
    "ticks",
    "facts",
    "after_expectations",
    "after_hits",
    "tags",
    "pending",
    "unlocked",
    "regression",
];

/// A pin: a scenario's private files by name and SHA-256, and their bundle.
pub type Pin = BTreeMap<String, String>;

/// One scenario as its file holds it.
#[derive(Debug, Clone)]
struct Scenario {
    id: String,
    start: String,
    prefix: i64,
    ticks: i64,
    facts: Vec<String>,
    tags: Vec<String>,
    after_expectations: Option<i64>,
    after_hits: Option<i64>,
    pending: Option<String>,
    unlocked: Option<String>,
    regression: Option<String>,
}

/// scenarios/lock.toml.
#[derive(Debug, Clone, Default)]
pub struct Lock {
    /// Each ci-data asset's ciphertext SHA-256.
    pub bundles: BTreeMap<String, String>,
    pub pins: BTreeMap<String, Pin>,
    /// The faithful oracle's digests, by scenario id.
    pub entries: BTreeMap<String, Table>,
}

/// A scenario as the runner's row: its pinned files, budget and facts.
#[derive(Debug, Clone, Default)]
pub struct Row {
    pub id: String,
    pub pending: Option<String>,
    /// The pinned seed capture:index; present (maybe empty) for every seeded start.
    pub seed: Option<String>,
    pub seed_sha256: Option<String>,
    pub inputs: Option<String>,
    pub input_sha256: Option<String>,
    pub checkpoint: Option<String>,
    /// Input ticks before the scenario's own.
    pub start_frame: i64,
    pub frames: i64,
    pub expect: Vec<String>,
    pub require_hit: Vec<String>,
    pub observe: Vec<String>,
    pub after_expectations: Option<i64>,
    pub after_hits: Option<i64>,
    pub unlocked: Option<String>,
    pub regression: Option<String>,
    pub bundle: Option<String>,
    pub tags: Vec<String>,
    /// The route a route room belongs to (its `route:R` tag).
    pub chain: Option<String>,
}

impl Row {
    pub fn inputs(&self) -> &str {
        self.inputs.as_deref().unwrap_or_default()
    }

    /// What an oracle lock entry was recorded from: start, inputs and budget.
    fn lock_key(&self) -> [(&'static str, Value); 3] {
        let start = match (&self.seed, &self.seed_sha256) {
            (Some(_), Some(digest)) => format!("seed:{digest}"),
            (Some(_), None) => String::new(),
            (None, _) => "boot".to_owned(),
        };
        [
            ("start", Value::String(start)),
            (
                "inputs",
                Value::String(self.input_sha256.clone().unwrap_or_default()),
            ),
            ("frames", Value::Integer(self.frames)),
        ]
    }

    /// A prefix start needs an oracle-side prefix first; a pending one has no data.
    fn lockable(&self) -> bool {
        self.pending.is_none() && self.start_frame == 0 && self.checkpoint.is_none()
    }
}

fn is_digest(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `[A-Za-z0-9-]+:[1-9][0-9]*`, a seed's capture:index.
fn is_seed_name(text: &str) -> bool {
    text.split_once(':').is_some_and(|(capture, index)| {
        !capture.is_empty()
            && capture
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && index
                .bytes()
                .next()
                .is_some_and(|b| (b'1'..=b'9').contains(&b))
            && index.bytes().all(|b| b.is_ascii_digit())
    })
}

/// `observe:([a-z]+)(?:\.[A-Za-z0-9_.\[\]]+)?@([0-9]+)`: the domain and tick.
fn observed(fact: &str) -> Option<(&str, &str)> {
    let (field, tick) = fact.strip_prefix("observe:")?.split_once('@')?;
    let (domain, path) = field
        .split_once('.')
        .map_or((field, None), |(domain, path)| (domain, Some(path)));
    let domain_ok = !domain.is_empty() && domain.bytes().all(|b| b.is_ascii_lowercase());
    let path_ok = path.is_none_or(|path| {
        !path.is_empty()
            && path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'[' | b']'))
    });
    let tick_ok = !tick.is_empty() && tick.bytes().all(|b| b.is_ascii_digit());
    (domain_ok && path_ok && tick_ok).then_some((domain, tick))
}

/// `[A-Za-z0-9_-]+\.map\.csv\.toml`, a flat movie-map manifest name.
fn is_movie_map_name(text: &str) -> bool {
    text.strip_suffix(".map.csv.toml").is_some_and(|stem| {
        !stem.is_empty()
            && stem
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    })
}

fn string(value: &Value, what: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{what} must be a string"))
}

fn integer(value: &Value, what: &str) -> Result<i64, String> {
    value
        .as_integer()
        .ok_or_else(|| format!("{what} must be an integer"))
}

fn strings(value: &Value, what: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{what} must be an array"))?
        .iter()
        .map(|item| string(item, what))
        .collect()
}

pub fn tables<'a>(document: &'a Table, key: &str) -> Result<Vec<&'a Table>, String> {
    match document.get(key) {
        None => Ok(Vec::new()),
        Some(value) => value
            .as_array()
            .ok_or_else(|| format!("{key} must be an array of tables"))?
            .iter()
            .map(|item| {
                item.as_table()
                    .ok_or_else(|| format!("{key} must be an array of tables"))
            })
            .collect(),
    }
}

/// Every scenario, in file order, with its file's tags applied.
fn load(root: &Path) -> Result<Vec<Scenario>, String> {
    let directory = root.join("scenarios");
    let mut paths: Vec<_> = std::fs::read_dir(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .filter(|path| path.file_name().is_some_and(|name| name != "lock.toml"))
        .collect();
    paths.sort();
    let mut scenarios: Vec<Scenario> = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        let file = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let data = toml::read(&path)?;
        if data
            .keys()
            .any(|key| !matches!(key, "version" | "domains" | "tags" | "scenario"))
            || data.get("version") != Some(&Value::Integer(1))
        {
            return Err(format!(
                "{file}: needs version = 1 and only domains, tags and scenario"
            ));
        }
        let domains = data
            .get("domains")
            .map(|value| strings(value, "domains"))
            .transpose()?
            .unwrap_or_default();
        if domains != DOMAINS {
            return Err(format!(
                "{file}: domains must name every compared domain {DOMAINS:?}"
            ));
        }
        let file_tags = data
            .get("tags")
            .map(|value| strings(value, "tags"))
            .transpose()?
            .unwrap_or_default();
        for entry in tables(&data, "scenario")? {
            let id = entry
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if entry.keys().any(|key| !FIELDS.contains(&key))
                || id.is_empty()
                || !seen.insert(id.clone())
            {
                return Err(format!(
                    "{file}: scenario {id:?} has unknown fields or a duplicate id"
                ));
            }
            let start = entry
                .get("start")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if start != "boot" && !start.strip_prefix("seed:").is_some_and(is_digest) {
                return Err(format!("{id}: start must be boot or seed:<SHA-256>"));
            }
            let optional_string =
                |key: &str| entry.get(key).map(|value| string(value, key)).transpose();
            let optional_integer =
                |key: &str| entry.get(key).map(|value| integer(value, key)).transpose();
            let mut tags = file_tags.clone();
            tags.extend(
                entry
                    .get("tags")
                    .map(|value| strings(value, "tags"))
                    .transpose()?
                    .unwrap_or_default(),
            );
            scenarios.push(Scenario {
                start,
                prefix: optional_integer("prefix")?.unwrap_or(0),
                ticks: optional_integer("ticks")?.ok_or_else(|| format!("{id}: needs ticks"))?,
                facts: entry
                    .get("facts")
                    .map(|value| strings(value, "facts"))
                    .transpose()?
                    .unwrap_or_default(),
                tags,
                after_expectations: optional_integer("after_expectations")?,
                after_hits: optional_integer("after_hits")?,
                pending: optional_string("pending")?,
                unlocked: optional_string("unlocked")?,
                regression: optional_string("regression")?,
                id,
            });
        }
    }
    Ok(scenarios)
}

/// scenarios/lock.toml; an absent lock is empty.
pub fn read_lock(root: &Path) -> Result<Lock, String> {
    read_lock_file(&root.join(LOCK))
}

/// A lock file at any path; an absent one is empty.
pub fn read_lock_file(path: &Path) -> Result<Lock, String> {
    if !path.exists() {
        return Ok(Lock::default());
    }
    let data = toml::read(path)?;
    let mut lock = Lock::default();
    for bundle in tables(&data, "bundles")? {
        let field = |key: &str| {
            bundle.get(key).map_or_else(
                || Err(format!("a bundle needs {key}")),
                |value| string(value, key),
            )
        };
        lock.bundles.insert(field("name")?, field("sha256")?);
    }
    for pin in tables(&data, "pins")? {
        let fields: Pin = pin
            .iter()
            .map(|(key, value)| Ok((key.to_owned(), string(value, key)?)))
            .collect::<Result<_, String>>()?;
        let id = fields.get("id").cloned().ok_or("a pin needs its id")?;
        lock.pins.insert(id, fields);
    }
    for entry in tables(&data, "scenario")? {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or("a lock entry needs its id")?;
        lock.entries.insert(id.to_owned(), entry.clone());
    }
    Ok(lock)
}

/// Each scenario as the runner's row (`scenario.rows`).
pub fn rows(root: &Path, lock: &Lock) -> Result<Vec<Row>, String> {
    let empty = Pin::new();
    load(root)?
        .into_iter()
        .map(|entry| {
            let pin = lock.pins.get(&entry.id).unwrap_or(&empty);
            let pinned = |key: &str| pin.get(key).cloned();
            let mut row = Row {
                id: entry.id.clone(),
                pending: entry.pending,
                ..Row::default()
            };
            if entry.start != "boot" {
                row.seed = Some(pinned("seed").unwrap_or_default());
                row.seed_sha256 = entry.start.strip_prefix("seed:").map(str::to_owned);
            }
            // A pinned seed names the row's seed even for a boot start, as in scenario.py.
            if let Some(seed) = pinned("seed") {
                row.seed = Some(seed);
            }
            row.inputs = pinned("inputs");
            row.input_sha256 = pinned("input_sha256");
            row.checkpoint = pinned("checkpoint");
            row.start_frame = entry.prefix;
            row.frames = entry.ticks;
            if pin.contains_key("movie_map") || pin.contains_key("movie_map_sha256") {
                let name = pin.get("movie_map").map(String::as_str).unwrap_or_default();
                let digest = pin
                    .get("movie_map_sha256")
                    .map(String::as_str)
                    .unwrap_or_default();
                if entry.start != "boot" || !is_movie_map_name(name) || !is_digest(digest) {
                    return Err(format!(
                        "{}: movie map requires a boot start and flat SHA-pinned manifest",
                        entry.id
                    ));
                }
                // scenario.py checks a present map's work events against its inputs
                // (movie_inputs.check_work_event_map); that check is not ported yet.
                if root.join("movies").join(name).exists() {
                    return Err(format!(
                        "{}: movie-map checks run only in tools/scenario.py so far",
                        entry.id
                    ));
                }
            }
            for fact in entry.facts {
                if let Some(probe) = fact.strip_prefix("probe:") {
                    row.require_hit.push(probe.to_owned());
                } else if fact.starts_with("observe:") {
                    row.observe.push(fact);
                } else {
                    row.expect.push(fact);
                }
            }
            row.after_expectations = entry.after_expectations;
            row.after_hits = entry.after_hits;
            row.unlocked = entry.unlocked;
            row.regression = entry.regression;
            row.bundle = pinned("bundle");
            row.chain = entry
                .tags
                .iter()
                .find_map(|tag| tag.strip_prefix("route:"))
                .map(str::to_owned);
            row.tags = entry.tags;
            Ok(row)
        })
        .collect()
}

/// Each room's observation SHA-256 in a committed capture index, if there is one.
fn captured_seeds(
    root: &Path,
    capture: &str,
) -> Result<Option<HashMap<String, Option<String>>>, String> {
    let path = root.join(format!("movies/room-seeds-{capture}.csv"));
    if !path.exists() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut lines = text
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.is_empty());
    let header: Vec<&str> = lines
        .next()
        .map(|line| line.split(',').collect())
        .unwrap_or_default();
    let column = |name: &str| header.iter().position(|&field| field == name);
    let (index_column, digest_column) = (column("room_index"), column("observation_sha256"));
    let mut seeds = HashMap::new();
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        let field = |column: Option<usize>| {
            column
                .and_then(|at| fields.get(at))
                .map(|&value| value.to_owned())
        };
        // csv.DictReader: a missing column or a short row reads as None.
        let index = field(index_column)
            .ok_or_else(|| format!("{}: a row without room_index", path.display()))?;
        seeds.insert(index, field(digest_column));
    }
    Ok(Some(seeds))
}

/// Everything that keeps the scenarios and their lock from being complete and current.
pub fn problems(root: &Path, lock: &Lock, loaded: &[Row]) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    let names: BTreeSet<&str> = loaded.iter().map(|row| row.id.as_str()).collect();
    let mut used = BTreeSet::new();
    let mut captures: HashMap<String, Option<HashMap<String, Option<String>>>> = HashMap::new();
    let empty = Pin::new();
    for row in loaded {
        let (name, pin) = (&row.id, lock.pins.get(&row.id).unwrap_or(&empty));
        let pinned = |key: &str| pin.get(key).map(String::as_str).unwrap_or_default();
        if row.pending.is_none() {
            if pinned("inputs").is_empty() || !is_digest(pinned("input_sha256")) {
                found.push(format!("{name}: no pinned inputs"));
            }
            if row.seed.as_deref().is_some_and(|seed| !is_seed_name(seed)) {
                found.push(format!(
                    "{name}: a seeded start needs its pinned seed capture:index"
                ));
            }
            if row.require_hit.is_empty() && row.expect.is_empty() && row.observe.is_empty() {
                found.push(format!("{name}: no fact demonstrates coverage"));
            }
            let movie_route = pinned("inputs").starts_with("corpus-")
                && row.seed.is_none()
                && !pin.contains_key("checkpoint");
            if !pin.contains_key("bundle") && !movie_route {
                found.push(format!("{name}: its private data is in no bundle"));
            }
        }
        for key in ["checkpoint", "faithful_checkpoint"] {
            if pin.contains_key(key) && !is_digest(pinned(&format!("{key}_sha256"))) {
                found.push(format!("{name}: {key} needs its SHA-256"));
            }
        }
        if let Some(bundle) = pin.get("bundle") {
            used.insert(bundle.as_str());
            if !lock.bundles.contains_key(bundle) {
                found.push(format!("{name}: bundle {bundle} has no pinned SHA-256"));
            }
        }
        if let Some(seed) = row.seed.as_deref().filter(|seed| !seed.is_empty()) {
            let (capture, index) = seed
                .split_once(':')
                .filter(|(_, index)| !index.contains(':'))
                .ok_or_else(|| format!("{name}: seed {seed} is not capture:index"))?;
            if !captures.contains_key(capture) {
                captures.insert(capture.to_owned(), captured_seeds(root, capture)?);
            }
            if let Some(seeds) = captures.get(capture).and_then(Option::as_ref)
                && !seeds.is_empty()
                && seeds.values().all(Option::is_some)
                && seeds.get(index).and_then(Option::as_deref) != row.seed_sha256.as_deref()
            {
                found.push(format!(
                    "{name}: start differs from seed {seed} in movies/room-seeds-{capture}.csv"
                ));
            }
        }
        for fact in &row.observe {
            let in_window = observed(fact)
                .filter(|(domain, _)| DOMAINS.contains(domain))
                .is_some_and(|(_, tick)| {
                    tick.parse::<i64>().is_ok_and(|tick| {
                        row.start_frame <= tick && tick < row.start_frame + row.frames
                    })
                });
            if !in_window {
                found.push(format!(
                    "{name}: {fact} names no compared domain at a tick the scenario runs"
                ));
            }
        }
    }
    found.extend(
        lock.pins
            .keys()
            .filter(|name| !names.contains(name.as_str()))
            .map(|name| format!("{name}: pinned, but no such scenario")),
    );
    found.extend(
        lock.bundles
            .keys()
            .filter(|name| !used.contains(name.as_str()))
            .map(|name| format!("{name}: pinned, but no scenario uses it")),
    );
    // The oracle lock: each lockable scenario is locked or listed as unlocked.
    let keys: HashMap<&str, &Row> = loaded
        .iter()
        .filter(|row| row.lockable())
        .map(|row| (row.id.as_str(), row))
        .collect();
    for (name, entry) in &lock.entries {
        match keys.get(name.as_str()) {
            None => found.push(format!("{name}: locked, but no such lockable scenario")),
            Some(row) if row.unlocked.is_some() => {
                found.push(format!("{name}: both locked and unlocked"))
            }
            Some(row) => {
                let stale = entry.get("oracle").and_then(Value::as_str) != Some("faithful")
                    || row
                        .lock_key()
                        .iter()
                        .any(|(key, value)| entry.get(key) != Some(value));
                if stale {
                    found.push(format!(
                        "{name}: stale lock entry (start, inputs or budget changed); re-record it"
                    ));
                }
            }
        }
    }
    for row in loaded {
        let lockable = keys.contains_key(row.id.as_str());
        if row.unlocked.is_some() && !lockable {
            found.push(format!("{}: unlocked, but not lockable", row.id));
        } else if lockable && !lock.entries.contains_key(&row.id) && row.unlocked.is_none() {
            found.push(format!("{}: neither locked nor unlocked", row.id));
        }
    }
    Ok(found)
}

/// `scenario.py lock --check` over a root: the line it prints, or an error naming every problem.
pub fn lock_check(root: &Path) -> Result<String, String> {
    let lock = read_lock(root)?;
    let loaded = rows(root, &lock)?;
    check(root, &lock, &loaded).map(|(_, line)| line)
}

/// The locked ids and the line `scenario.py lock --check` prints; an error names every problem.
pub fn check(
    root: &Path,
    lock: &Lock,
    loaded: &[Row],
) -> Result<(BTreeSet<String>, String), String> {
    let found = problems(root, lock, loaded)?;
    if !found.is_empty() {
        return Err(format!(
            "Scenarios or their lock are not current:\n{}",
            found.join("\n")
        ));
    }
    let line = format!(
        "Scenarios current: {} scenarios, {} pins, {} bundles; {} locked, {} unlocked",
        loaded.len(),
        lock.pins.len(),
        lock.bundles.len(),
        lock.entries.len(),
        loaded.iter().filter(|row| row.unlocked.is_some()).count()
    );
    Ok((lock.entries.keys().cloned().collect(), line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_python_patterns() {
        assert!(is_digest(&"a".repeat(64)));
        assert!(!is_digest(&"A".repeat(64)));
        assert!(is_seed_name("100-input-phase:12"));
        assert!(!is_seed_name("100-input-phase:012"));
        assert!(!is_seed_name(":1"));
        assert_eq!(
            observed("observe:samus.x_fraction@12"),
            Some(("samus", "12"))
        );
        assert_eq!(observed("observe:ram@3"), Some(("ram", "3")));
        assert_eq!(observed("observe:Ram@3"), None);
        assert_eq!(observed("observe:ram.@3"), None);
        assert!(is_movie_map_name("any-1.map.csv.toml"));
        assert!(!is_movie_map_name("a/b.map.csv.toml"));
    }
}
