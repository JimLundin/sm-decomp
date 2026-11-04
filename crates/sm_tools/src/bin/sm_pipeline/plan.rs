//! Plan one ci.yml Verify run as a single job matrix (ported from the deleted
//! `.github/scripts/plan_verify.py`; `plan_golden` holds it to that planner's plans).
//!
//! The scope selects what runs:
//! - gate: every locked scenario engine-only against the oracle lock, lockstep
//!   for the scenarios the lock cannot hold, and the ROM import check. Rows
//!   with a long input prefix await room-seed reseeding and stay in full.
//! - full: the lock jobs, the ROM import check, lockstep of the whole corpus
//!   (with the faithful-oracle sample when asked) and both routes.
//! - rows: lockstep of the selected rows.
//! - coverage: instrumented lockstep of the corpus, and the routes when asked.
//!
//! The gate and rows scopes pack their rows into jobs by measured seconds
//! (`pack`), so the longest row sets the wall time and every job starts at once.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use crate::corpus::{self, Selection, Shard};
use crate::json::{Json, Object};
use crate::pack::{Measured, Mode, Rows};
use crate::route::{self, RouteJob, round};
use crate::scenario::{self, Lock, Row};
use crate::toml;

/// Above this many prefix frames a rejected checkpoint costs more than the rest
/// of the gate, and every state layout change rejects it. These rows run in full.
const LONG_PREFIX: i64 = 20_000;

/// What one Verify run replays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Gate,
    Full,
    Rows,
    Coverage,
}

impl Scope {
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "gate" => Ok(Self::Gate),
            "full" => Ok(Self::Full),
            "rows" => Ok(Self::Rows),
            "coverage" => Ok(Self::Coverage),
            _ => Err(format!("unknown scope {name}")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Gate => "gate",
            Self::Full => "full",
            Self::Rows => "rows",
            Self::Coverage => "coverage",
        }
    }
}

/// The planner's inputs, as ci.yml's verify-plan step sets them.
#[derive(Debug, Clone)]
pub struct Settings {
    pub scope: Scope,
    /// Row ids for the rows scope (SELECTED).
    pub selected: Vec<String>,
    /// Re-record the faithful-oracle sample (ORACLE_SAMPLE=1).
    pub oracle_sample: bool,
    /// Replay the routes instrumented too (COVER_ROUTES=1).
    pub cover_routes: bool,
    /// The sample's three-hour UTC slot.
    pub slot: u64,
}

/// What the planner writes: GITHUB_OUTPUT lines, the step summary and its log.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub output: String,
    pub summary: String,
    pub log: String,
}

/// One kind of work: its restore group ("corpus" or a route), its mode and its row ids.
type Work = (String, Mode, Vec<String>);

struct Planner<'a> {
    rows: &'a [Row],
    lock: &'a Lock,
    locked: &'a BTreeSet<String>,
    packing: Rows<'a>,
    /// Each published route's ci-data asset, in route order.
    assets: Vec<(&'static str, String)>,
    long_rows: Vec<String>,
    settings: &'a Settings,
}

impl Planner<'_> {
    /// The corpus shards for the selected rows (every row when empty).
    fn shards(&self, selected: &[String], sample: bool) -> Result<Vec<Shard>, String> {
        let sampled = if sample {
            let mut sampled =
                corpus::oracle_sample(self.rows, self.locked, self.settings.slot, 30)?;
            for id in corpus::lacking_session_streams(self.lock, self.locked) {
                if !sampled.contains(&id) {
                    sampled.push(id);
                }
            }
            Some(sampled)
        } else {
            None
        };
        corpus::shards(
            self.rows,
            self.lock,
            &Selection {
                selected,
                sample: sampled.as_deref(),
            },
        )
    }

    fn asset(&self, route: &str) -> Result<&str, String> {
        self.assets
            .iter()
            .find(|(name, _)| *name == route)
            .map(|(_, asset)| asset.as_str())
            .ok_or_else(|| format!("route {route} has no published bundle"))
    }

    /// One lockstep or oracle-sample shard's job.
    fn corpus_job(shard: &Shard, timeout: i64) -> Object {
        let mode = if shard.record_oracle {
            Mode::Record
        } else {
            Mode::Lockstep
        };
        Object::new()
            .with("name", shard.name.as_str())
            .with("mode", mode.as_str())
            .with("steps", shard.cost)
            .with("segments", shard.segments.join(" "))
            .with("kinds", shard.kind.as_str())
            .with("timeout", timeout)
            .with("shards", Shard::restores([shard]))
    }

    /// The rooms of one route job that this mode runs, or None when it runs none.
    fn room_job(
        &self,
        name: &str,
        job: &RouteJob,
        mode: Mode,
        keep: impl Fn(&str) -> bool,
        timeout: i64,
    ) -> Result<Option<Object>, String> {
        let mut ids = Vec::new();
        for id in &job.ids {
            if self.packing.get(id)?.pending.is_none() && keep(id) {
                ids.push(id.as_str());
            }
        }
        Ok((!ids.is_empty()).then(|| {
            Object::new()
                .with("name", name)
                .with("mode", mode.as_str())
                .with("steps", job.steps)
                .with("kinds", "room")
                .with("timeout", timeout)
                .with("route", job.route)
                .with("asset", job.asset.as_str())
                .with("segments", ids.join(" "))
        }))
    }

    /// Jobs for the work, packed by measured seconds to one budget.
    ///
    /// A route job restores its route's bundle; a corpus job restores each seed
    /// bundle or movie route its rows need once, as the corpus shards them.
    fn packed(&self, work: Vec<Work>, long_rows: &[String]) -> Result<Vec<Object>, String> {
        let mut merged: Vec<((String, Mode), Vec<String>)> = Vec::new();
        for (group, mode, ids) in work {
            match merged
                .iter_mut()
                .find(|(key, _)| key.0 == group && key.1 == mode)
            {
                Some((_, existing)) => existing.extend(ids),
                None => merged.push(((group, mode), ids)),
            }
        }
        for (_, ids) in &mut merged {
            let mut seen = HashSet::new();
            ids.retain(|id| seen.insert(id.clone()));
        }
        let corpus: BTreeSet<String> = merged
            .iter()
            .filter(|((group, _), _)| group == "corpus")
            .flat_map(|(_, ids)| ids.iter().cloned())
            .collect();
        let mut shard_of: HashMap<String, Shard> = HashMap::new();
        if !corpus.is_empty() {
            // The corpus shards drop pending rows, so they never reach a job.
            for shard in self.shards(&corpus.into_iter().collect::<Vec<_>>(), false)? {
                for id in &shard.segments {
                    shard_of.insert(id.clone(), shard.clone());
                }
            }
            for ((group, _), ids) in &mut merged {
                if group == "corpus" {
                    ids.retain(|id| shard_of.contains_key(id));
                }
            }
        }
        let (limit, longest) = self.packing.budget(
            merged
                .iter()
                .map(|((_, mode), ids)| (ids.as_slice(), *mode)),
        );
        let mut jobs = Vec::new();
        for ((group, mode), ids) in &merged {
            let (floors, rest) = self.packing.floor(ids, *mode, longest);
            let mut planned: Vec<(Vec<String>, usize, bool)> = floors
                .into_iter()
                .map(|segments| (segments, 2, true))
                .collect();
            if !rest.is_empty() {
                planned.extend(
                    self.packing
                        .pack(&rest, *mode, limit)
                        .into_iter()
                        .map(|segments| (segments, mode.workers(), false)),
                );
            }
            for (index, (segments, count, first)) in planned.into_iter().enumerate() {
                let mut steps = 0;
                for id in &segments {
                    steps += self.packing.get(id)?.frames;
                }
                let mut job = Object::new()
                    .with("mode", mode.as_str())
                    .with("steps", steps)
                    .with(
                        "seconds",
                        round(self.packing.estimate(&segments, *mode, count)),
                    )
                    .with("workers", count)
                    .with("first", first)
                    .with("segments", segments.join(" "));
                if group == "corpus" {
                    let mut restores: Vec<(Shard, Vec<String>)> = Vec::new();
                    for id in &segments {
                        let shard = shard_of
                            .get(id)
                            .ok_or_else(|| format!("{id}: in no corpus shard"))?;
                        let same = |other: &Shard| {
                            (
                                other.kind,
                                &other.seed_asset,
                                &other.seed_sha256,
                                &other.route,
                                &other.inputs,
                            ) == (
                                shard.kind,
                                &shard.seed_asset,
                                &shard.seed_sha256,
                                &shard.route,
                                &shard.inputs,
                            )
                        };
                        match restores.iter_mut().find(|(other, _)| same(other)) {
                            Some((_, names)) => names.push(id.clone()),
                            None => restores.push((shard.clone(), vec![id.clone()])),
                        }
                    }
                    let kinds: BTreeSet<&str> = restores
                        .iter()
                        .map(|(shard, _)| shard.kind.as_str())
                        .collect();
                    let entries: Vec<Object> = restores
                        .iter()
                        .map(|(shard, names)| {
                            let mut entry = shard.restore();
                            entry.set("segments", names.join(" "));
                            entry
                        })
                        .collect();
                    let long = segments.iter().any(|id| long_rows.contains(id));
                    job.set(
                        "name",
                        format!(
                            "{}-corpus-{index:02}-{}-rows",
                            mode.as_str(),
                            segments.len()
                        ),
                    );
                    job.set("kinds", kinds.into_iter().collect::<Vec<_>>().join(" "));
                    job.set("timeout", if long { 240_i64 } else { 65 });
                    job.set("shards", Json::from(entries).to_string());
                } else {
                    job.set(
                        "name",
                        format!(
                            "{}-{group}-{index:02}-{}-rooms",
                            mode.as_str(),
                            segments.len()
                        ),
                    );
                    job.set("kinds", "room");
                    job.set("timeout", 60_i64);
                    job.set("route", group.as_str());
                    job.set("asset", self.asset(group)?);
                }
                jobs.push(job);
            }
        }
        Ok(jobs)
    }

    fn gate(&self) -> Result<Vec<Object>, String> {
        let runnable: Vec<&Row> = self
            .rows
            .iter()
            .filter(|row| row.pending.is_none())
            .collect();
        let corpus: Vec<&str> = runnable
            .iter()
            .filter(|row| row.chain.is_none())
            .map(|row| row.id.as_str())
            .collect();
        let locked = |id: &&str| self.locked.contains(*id);
        let owned = |ids: Vec<&str>| ids.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let mut work: Vec<Work> = vec![
            (
                "corpus".into(),
                Mode::Lock,
                owned(corpus.iter().copied().filter(locked).collect()),
            ),
            // The lock cannot hold these: unlocked scenarios and checkpoint-backed rows.
            (
                "corpus".into(),
                Mode::Lockstep,
                owned(
                    corpus
                        .iter()
                        .copied()
                        .filter(|id| !locked(id) && !self.long_rows.iter().any(|long| long == id))
                        .collect(),
                ),
            ),
        ];
        for (route, _) in &self.assets {
            let chain: Vec<&str> = runnable
                .iter()
                .filter(|row| row.chain.as_deref() == Some(route))
                .map(|row| row.id.as_str())
                .collect();
            work.push((
                (*route).into(),
                Mode::Lock,
                owned(chain.iter().copied().filter(locked).collect()),
            ));
            work.push((
                (*route).into(),
                Mode::Lockstep,
                owned(chain.iter().copied().filter(|id| !locked(id)).collect()),
            ));
        }
        let mut jobs = self.packed(work, &[])?;
        jobs.push(
            Object::new()
                .with("name", "rom-import")
                .with("mode", "rom-import")
                .with("steps", 0_i64)
                .with("seconds", 0_i64)
                .with("kinds", "")
                .with("timeout", 30_i64),
        );
        Ok(jobs)
    }

    /// The engine-only jobs: every locked room and corpus row (`lock_gate.jobs`).
    ///
    /// Rooms run as the route jobs do; corpus rows shard by seed bundle as the
    /// lockstep corpus does, and the shards pack into jobs of about a room
    /// job's frames, each restoring its shards' bundles in turn.
    fn lock_jobs(&self, routes: &[RouteJob]) -> Result<Vec<Object>, String> {
        let corpus: Vec<String> = self
            .rows
            .iter()
            .filter(|row| row.chain.is_none() && self.locked.contains(&row.id))
            .map(|row| row.id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        // lock_gate.jobs passed its own environment to plan_corpus.py, so the
        // run's oracle sample joins these shards too.
        let shards = self.shards(&corpus, self.settings.oracle_sample)?;
        let size = routes
            .iter()
            .map(|job| job.steps)
            .max()
            .ok_or("no route jobs")?;
        let total: i64 = shards.iter().map(|shard| shard.cost).sum();
        let count = usize::try_from(round(total as f64 / size as f64).max(1)).unwrap_or(1);
        let mut bins: Vec<Vec<&Shard>> = vec![Vec::new(); count];
        for shard in &shards {
            let lightest = route::lightest(&bins, |group| {
                group.iter().map(|shard| shard.cost).sum::<i64>()
            });
            bins[lightest].push(shard);
        }
        enum Planned<'r> {
            Room(&'r RouteJob),
            Corpus(Object),
        }
        let mut planned: Vec<(i64, Planned<'_>)> = routes
            .iter()
            .map(|job| (job.steps, Planned::Room(job)))
            .collect();
        for (index, group) in bins
            .iter()
            .enumerate()
            .filter(|(_, group)| !group.is_empty())
        {
            let steps: i64 = group.iter().map(|shard| shard.cost).sum();
            let rows: usize = group.iter().map(|shard| shard.segments.len()).sum();
            let segments: Vec<String> =
                group.iter().map(|shard| shard.segments.join(" ")).collect();
            let kinds: BTreeSet<&str> = group.iter().map(|shard| shard.kind.as_str()).collect();
            let job = Object::new()
                .with("kind", "corpus")
                .with("name", format!("lock-corpus-{index:02}-{rows}-rows"))
                .with("steps", steps)
                .with("segments", segments.join(" "))
                .with("kinds", kinds.into_iter().collect::<Vec<_>>().join(" "))
                .with("shards", Shard::restores(group.iter().copied()))
                .with("mode", "lock")
                .with("timeout", 60_i64);
            planned.push((steps, Planned::Corpus(job)));
        }
        planned.sort_by_key(|(steps, _)| -steps);
        let mut jobs = Vec::new();
        for (_, job) in planned {
            match job {
                Planned::Corpus(job) => jobs.push(job),
                Planned::Room(room) => {
                    let name = format!("lock-{}", room.name);
                    jobs.extend(self.room_job(
                        &name,
                        room,
                        Mode::Lock,
                        |id| self.locked.contains(id),
                        60,
                    )?);
                }
            }
        }
        Ok(jobs)
    }

    fn full(&self, routes: &[RouteJob]) -> Result<Vec<Object>, String> {
        let mut jobs = self.lock_jobs(routes)?;
        jobs.push(
            Object::new()
                .with("name", "rom-import")
                .with("mode", "rom-import")
                .with("steps", 0_i64)
                .with("kinds", "")
                .with("timeout", 30_i64),
        );
        for shard in self.shards(&[], self.settings.oracle_sample)? {
            let heavy = shard.segments.iter().any(|id| self.long_rows.contains(id));
            jobs.push(Self::corpus_job(&shard, if heavy { 240 } else { 65 }));
        }
        for job in routes {
            jobs.extend(self.room_job(&job.name, job, Mode::Lockstep, |_| true, 60)?);
        }
        Ok(jobs)
    }

    fn selected_rows(&self) -> Result<Vec<Object>, String> {
        let selected = &self.settings.selected;
        let unknown: BTreeSet<&str> = selected
            .iter()
            .map(String::as_str)
            .filter(|id| self.packing.get(id).is_err())
            .collect();
        if selected.is_empty() || !unknown.is_empty() {
            return Err(format!(
                "scope rows needs known row ids; unknown: {unknown:?}"
            ));
        }
        let mut work = Vec::new();
        for id in selected {
            let group = self
                .packing
                .get(id)?
                .chain
                .clone()
                .unwrap_or_else(|| "corpus".into());
            work.push((group, Mode::Lockstep, vec![id.clone()]));
        }
        self.packed(work, &self.long_rows)
    }

    fn coverage(&self, routes: &[RouteJob]) -> Result<Vec<Object>, String> {
        let mut jobs: Vec<Object> = self
            .shards(&[], false)?
            .iter()
            .map(|shard| Self::corpus_job(shard, 300))
            .collect();
        if self.settings.cover_routes {
            for job in routes {
                jobs.extend(self.room_job(&job.name, job, Mode::Lockstep, |_| true, 150)?);
            }
        }
        Ok(jobs)
    }
}

/// The pinned any% movie: its file name and SHA-256.
fn movie(root: &Path) -> Result<(String, String), String> {
    let manifest = toml::read(&root.join("movies/manifest.toml"))?;
    let movies = manifest
        .get("movies")
        .and_then(toml::Value::as_array)
        .unwrap_or_default();
    let movie = movies
        .iter()
        .filter_map(toml::Value::as_table)
        .find(|movie| movie.get("category").and_then(toml::Value::as_str) == Some("any%"))
        .ok_or("movies/manifest.toml lists no any% movie")?;
    let field = |key: &str| {
        movie
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("the any% movie needs {key}"))
    };
    Ok((field("file")?, field("sha256")?))
}

/// Plan the run.
pub fn plan(root: &Path, settings: &Settings) -> Result<Plan, String> {
    let lock = scenario::read_lock(root)?;
    let rows = scenario::rows(root, &lock)?;
    let (locked, current) = scenario::check(root, &lock, &rows)?;
    let measured = Measured::read(root)?;
    let scope = settings.scope;
    let routes = if matches!(scope, Scope::Full | Scope::Coverage) {
        route::plan(&rows, &lock)?
    } else {
        Vec::new()
    };
    let mut assets = Vec::new();
    for name in route::ROUTES {
        if let Some((asset, _)) = route::published(&rows, &lock, name)? {
            assets.push((name, asset));
        }
    }
    let long_rows: Vec<String> = rows
        .iter()
        .filter(|row| row.chain.is_none() && row.pending.is_none() && row.start_frame > LONG_PREFIX)
        .map(|row| row.id.clone())
        .collect();
    let planner = Planner {
        rows: &rows,
        lock: &lock,
        locked: &locked,
        packing: Rows::new(&rows, &measured),
        assets,
        long_rows,
        settings,
    };
    let mut jobs = match scope {
        Scope::Gate => planner.gate()?,
        Scope::Full => planner.full(&routes)?,
        Scope::Rows => planner.selected_rows()?,
        Scope::Coverage => planner.coverage(&routes)?,
    };
    // Start the longest jobs first: by estimated seconds where packed, else by frames.
    jobs.sort_by_key(|job| {
        -job.int("seconds")
            .or_else(|| job.int("steps"))
            .unwrap_or_default()
    });
    // Names label checks and artifacts, so each one must be unique.
    let mut seen: Vec<(String, usize)> = Vec::new();
    for job in &mut jobs {
        let name = job.str("name").to_owned();
        let count = match seen.iter_mut().find(|(seen, _)| *seen == name) {
            Some((_, count)) => {
                *count += 1;
                *count
            }
            None => {
                seen.push((name.clone(), 1));
                1
            }
        };
        if count > 1 {
            job.set("name", format!("{name}-{count}"));
        }
    }
    // The gate's floor jobs set its wall time. They build their own verifier, so
    // they queue for runners with verify-build instead of behind it (ci.yml
    // verify-replay-first). At most two, one per route today, bound the extra builds.
    let first_jobs: Vec<usize> = if scope == Scope::Gate {
        jobs.iter()
            .enumerate()
            .filter(|(_, job)| job.get("first") == Some(&Json::Bool(true)))
            .map(|(index, _)| index)
            .take(2)
            .collect()
    } else {
        Vec::new()
    };
    let names: Vec<String> = jobs.iter().map(|job| job.str("name").to_owned()).collect();
    let total = jobs.len();
    let (mut first, mut rest) = (Vec::new(), Vec::new());
    for (index, job) in jobs.into_iter().enumerate() {
        if first_jobs.contains(&index) {
            first.push(job)
        } else {
            rest.push(job)
        }
    }
    let (movie_file, movie_sha256) = movie(root)?;
    let include = |jobs: Vec<Object>| Object::new().with("include", Json::from(jobs));
    let output = format!(
        "matrix={}\nfirst={}\nfirst_count={}\nmovie_file={movie_file}\nmovie_sha256={movie_sha256}\n",
        include(rest),
        include(first),
        first_jobs.len()
    );
    let mut summary = format!("Scope {}: {total} jobs\n\n", scope.name());
    if scope == Scope::Gate {
        summary.push_str(
            "PR lock gate; three long-prefix rows remain pending room-seed reseeding.\n\n",
        );
        summary.extend(planner.long_rows.iter().filter(|name| !locked.contains(*name)).map(|name| {
            format!("Excluded from fast gate: {name}; pending room-seed reseeding; retained in nightly full cold replay.\n")
        }));
    }
    let log = format!(
        "{current}\n{}: {total} jobs: {}\n",
        scope.name(),
        names.join(", ")
    );
    Ok(Plan {
        output,
        summary,
        log,
    })
}
