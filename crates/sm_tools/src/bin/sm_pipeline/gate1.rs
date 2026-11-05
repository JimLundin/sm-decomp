//! Gate 1 evidence (`sm_pipeline gate1`): the slice's headless replay against
//! the oracle, row by row (docs/handoff/gate1-slice.md).
//!
//! For each row: restore its bundle, record the oracle's digests where the lock
//! has none, prepare its start with the oracle-side importer, replay it with
//! `sm_replay` (no CPU emulator or native code), and compare each machine
//! stream with the lock. It prints names and verdicts only; the ROM and the row
//! data stay on the runner.
//!
//! The verifier (`tools/verify_dll.py`) and the bundle restore
//! (`tools/ci_seed_data.py restore`) are still Python, so they run as
//! subprocesses, as does `curl` for a bundle download.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::scenario::{self, Lock};
use crate::toml::{self, Table, Value};

pub const USAGE: &str =
    "sm_pipeline gate1 [--shard I/N] [--slice ROW]... ROW... (a ROW may be `all`)";

/// The machine streams a lock entry and a replay both digest.
const MACHINE: [&str; 8] = [
    "wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu",
];
/// The regions of an `SM_DIGEST_LOG` line, after its tick.
const LOGGED: [&str; 5] = ["wram", "vram", "cgram", "oam", "high oam"];
const RELEASES: &str = "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/";
const SM_REPLAY: &str = "target/release/sm_replay";
const SM_SLICE: &str = "target/release/sm_slice";
/// At most this many differing rows get a divergence location.
const DIAGNOSED: usize = 3;

/// One region of an `SM_DUMP_OUT` dump: its name and byte range.
struct Region {
    name: &'static str,
    first: usize,
    end: usize,
}

const REGIONS: [Region; 4] = [
    Region {
        name: "wram",
        first: 0,
        end: 0x2_0000,
    },
    Region {
        name: "vram",
        first: 0x2_0000,
        end: 0x3_0000,
    },
    Region {
        name: "cgram",
        first: 0x3_0000,
        end: 0x3_0200,
    },
    Region {
        name: "oam",
        first: 0x3_0200,
        end: 0x3_0400,
    },
];

/// `--shard I/N`: the I-th (from 1) of N interleaved parts of the rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shard {
    index: usize,
    count: usize,
}

impl Shard {
    const WHOLE: Self = Self { index: 1, count: 1 };

    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = || format!("--shard {text} is not I/N with 1 <= I <= N");
        let (index, count) = text.split_once('/').ok_or_else(bad)?;
        let shard = Self {
            index: index.parse().map_err(|_| bad())?,
            count: count.parse().map_err(|_| bad())?,
        };
        if shard.index == 0 || shard.index > shard.count {
            return Err(bad());
        }
        Ok(shard)
    }

    /// Every N-th row from the I-th, as Python's `rows[I - 1::N]`.
    pub fn take<T>(self, rows: Vec<T>) -> Vec<T> {
        rows.into_iter()
            .skip(self.index - 1)
            .step_by(self.count)
            .collect()
    }
}

impl fmt::Display for Shard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.index, self.count)
    }
}

/// The command line.
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments {
    rows: Vec<String>,
    shard: Shard,
    slices: Vec<String>,
}

impl Arguments {
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut parsed = Self {
            rows: Vec::new(),
            shard: Shard::WHOLE,
            slices: Vec::new(),
        };
        let mut arguments = arguments.iter();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--shard" => parsed.shard = Shard::parse(arguments.next().ok_or(USAGE)?)?,
                "--slice" => parsed.slices.push(arguments.next().ok_or(USAGE)?.clone()),
                flag if flag.starts_with("--") => return Err(USAGE.into()),
                row => parsed.rows.push(row.to_owned()),
            }
        }
        if parsed.rows.is_empty() {
            return Err(USAGE.into());
        }
        Ok(parsed)
    }
}

/// The named rows in order, then for `all` every corpus row the lock holds
/// (the room-chain entries are left to the corpus jobs), each once.
pub fn select(named: &[String], corpus: &[String], lock: &Lock) -> Vec<String> {
    let mut rows: Vec<String> = named.iter().filter(|row| *row != "all").cloned().collect();
    if named.iter().any(|row| row == "all") {
        for row in corpus {
            if lock.entries.contains_key(row) && !rows.contains(row) {
                rows.push(row.clone());
            }
        }
    }
    rows
}

/// The ids of scenarios/corpus.toml, in file order.
fn corpus_ids(root: &Path) -> Result<Vec<String>, String> {
    let corpus = toml::read(&root.join("scenarios/corpus.toml"))?;
    scenario::tables(&corpus, "scenario")?
        .into_iter()
        .map(|entry| {
            entry
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| "a corpus scenario needs its id".to_owned())
        })
        .collect()
}

/// Where a replay's lock entry parts from the locked one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    Stream(&'static str),
    Ticks,
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stream(name) => formatter.write_str(name),
            Self::Ticks => formatter.write_str("ticks"),
        }
    }
}

fn digest<'a>(entry: &'a Table, stream: &str) -> Option<&'a Value> {
    entry.get("digests")?.as_table()?.get(stream)
}

/// Each machine stream whose digest differs (or is absent on one side), then
/// the tick count when it differs.
pub fn compare(locked: &Table, ran: &Table) -> Vec<Difference> {
    let mut differing: Vec<Difference> = MACHINE
        .iter()
        .filter(|stream| digest(locked, stream) != digest(ran, stream))
        .map(|stream| Difference::Stream(stream))
        .collect();
    if locked.get("ticks") != ran.get("ticks") {
        differing.push(Difference::Ticks);
    }
    differing
}

/// The lines `new` adds to `old`, by their longest common subsequence.
pub fn added<'a>(old: &[&str], new: &[&'a str]) -> Vec<&'a str> {
    let prefix = old
        .iter()
        .zip(new)
        .take_while(|(left, right)| left == right)
        .count();
    let (old, new) = (&old[prefix..], &new[prefix..]);
    let suffix = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let (old, new) = (&old[..old.len() - suffix], &new[..new.len() - suffix]);
    // common[i][j]: the common subsequence length of old[i..] and new[j..].
    let mut common = vec![vec![0_usize; new.len() + 1]; old.len() + 1];
    for (i, left) in old.iter().enumerate().rev() {
        for (j, right) in new.iter().enumerate().rev() {
            common[i][j] = if left == right {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut lines) = (0, 0, Vec::new());
    while j < new.len() {
        if i < old.len() && old[i] == new[j] {
            i += 1;
            j += 1;
        } else if i < old.len() && common[i + 1][j] >= common[i][j + 1] {
            i += 1;
        } else {
            lines.push(new[j]);
            j += 1;
        }
    }
    lines
}

/// Each logged region's first tick where the two `SM_DIGEST_LOG`s differ, in
/// the order they are found.
pub fn first_differing(left: &[&str], right: &[&str]) -> Vec<(&'static str, i64)> {
    let mut first: Vec<(&'static str, i64)> = Vec::new();
    for (a, b) in left.iter().zip(right) {
        let (mut a_fields, mut b_fields) = (a.split_whitespace(), b.split_whitespace());
        let tick = a_fields.next().and_then(|tick| tick.parse().ok());
        b_fields.next();
        for ((name, x), y) in LOGGED.iter().zip(a_fields).zip(b_fields) {
            if let Some(tick) = tick
                && x != y
                && !first.iter().any(|(found, _)| found == name)
            {
                first.push((name, tick));
            }
        }
    }
    first
}

/// The first differing byte ranges of two dumps, named by region.
pub fn ranges(left: &[u8], right: &[u8]) -> String {
    let mut parts = Vec::new();
    for region in &REGIONS {
        let end = region.end.min(left.len()).min(right.len());
        let mut found: Vec<(usize, usize)> = Vec::new();
        for at in region.first..end {
            if left[at] != right[at] {
                match found.last_mut() {
                    Some((_, last)) if at - *last <= 2 => *last = at,
                    _ => found.push((at, at)),
                }
            }
        }
        let total: usize = found.iter().map(|(start, end)| end - start + 1).sum();
        let shown: Vec<String> = found
            .iter()
            .take(40)
            .map(|(start, end)| format!("{:05x}-{:05x}", start - region.first, end - region.first))
            .collect();
        parts.push(format!(
            "{}: {total} bytes in {} ranges: {}",
            region.name,
            found.len(),
            shown.join(", ")
        ));
    }
    parts.join("\n")
}

/// The last `count` characters of `text`.
fn tail(text: &str, count: usize) -> &str {
    let skip = text.chars().count().saturating_sub(count);
    text.char_indices()
        .nth(skip)
        .map_or("", |(at, _)| &text[at..])
}

/// A finished subprocess's exit code and its output as text.
struct Ran {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Run a command to its end, capturing its output, with extra environment.
fn run(program: &str, arguments: &[&str], environment: &[(&str, &str)]) -> Result<Ran, String> {
    let output = Command::new(program)
        .args(arguments)
        .envs(environment.iter().copied())
        .output()
        .map_err(|error| format!("{program}: {error}"))?;
    Ok(Ran {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn path_text(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("{} is not UTF-8", path.display()))
}

/// A verifier mode against a lock file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Record a lock entry; a lock records the faithful oracle, the real game
    /// (REVIEW condition 8(i)).
    Lock,
    CheckLock,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lock => "lock",
            Self::CheckLock => "check-lock",
        }
    }
}

/// The verifier on one row, headless and direct, with extra environment.
fn verify(row: &str, lock: &Path, mode: Mode, environment: &[(&str, &str)]) -> Result<Ran, String> {
    let mut arguments = vec![
        "tools/verify_dll.py",
        "rom/sm.smc",
        "--corpus",
        "scenarios",
        "--no-build",
        "--direct",
        "--segment",
        row,
        "--mode",
        mode.as_str(),
        "--lock",
        path_text(lock)?,
    ];
    if mode == Mode::Lock {
        arguments.extend(["--oracle", "faithful"]);
    }
    run("python3", &arguments, environment)
}

/// The verifier's verdict on a row, printed; its exit code.
fn verifier_entry(row: &str, lock: &Path, mode: Mode) -> Result<i32, String> {
    let ran = verify(row, lock, mode, &[])?;
    let verdicts: Vec<&str> = ran
        .stdout
        .lines()
        .filter(|line| line.starts_with(row) || line.starts_with("FAIL"))
        .collect();
    println!(
        "{row}: verifier {}: exit {}: {}",
        mode.as_str(),
        ran.code,
        verdicts.join(" | ")
    );
    if ran.code != 0 {
        // The verifier reports its own errors on stderr; names and reasons only.
        let reasons: Vec<&str> = ran
            .stderr
            .lines()
            .filter(|line| {
                ["verify:", "FAIL", "error", "usage:"]
                    .iter()
                    .any(|start| line.starts_with(start))
                    || line.contains(": error:")
            })
            .collect();
        let shown = &reasons[reasons.len().saturating_sub(5)..];
        println!(
            "{row}: verifier {} stderr: {}",
            mode.as_str(),
            shown.join(" | ")
        );
    }
    Ok(ran.code)
}

/// A file's lines, none when it is absent.
fn lines_of(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map(|text| text.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// One row's prepared replay inputs.
struct Prepared {
    row: String,
    /// The lock file holding the row's entry.
    lock: PathBuf,
    content: PathBuf,
    sealed: PathBuf,
    inputs: PathBuf,
    ticks: String,
}

impl Prepared {
    /// `sm_replay one` on the row, optionally writing its digest lines.
    fn replay_one(
        &self,
        digest_lines: Option<&Path>,
        environment: &[(&str, &str)],
    ) -> Result<Ran, String> {
        let mut arguments = vec![
            "one",
            path_text(&self.content)?,
            path_text(&self.sealed)?,
            path_text(&self.inputs)?,
            &self.ticks,
        ];
        if let Some(path) = digest_lines {
            arguments.push(path_text(path)?);
        }
        run(SM_REPLAY, &arguments, environment)
    }
}

/// One Gate 1 run over its rows.
struct Gate {
    work: PathBuf,
    lock: Lock,
}

/// Which run of a row a divergence location reads.
#[derive(Debug, Clone, Copy)]
enum Side {
    Verifier,
    Slice,
}

impl Side {
    const BOTH: [Self; 2] = [Self::Verifier, Self::Slice];

    fn as_str(self) -> &'static str {
        match self {
            Self::Verifier => "verifier",
            Self::Slice => "slice",
        }
    }
}

impl Gate {
    fn pin(&self, row: &str, field: &str) -> Result<&str, String> {
        self.lock
            .pins
            .get(row)
            .and_then(|pin| pin.get(field))
            .map(String::as_str)
            .ok_or_else(|| format!("{row}: the lock pins no {field}"))
    }

    fn restore(&self, row: &str) -> Result<(), String> {
        let asset = self.pin(row, "bundle")?;
        let sha256 = self
            .lock
            .bundles
            .get(asset)
            .ok_or_else(|| format!("{row}: the lock has no bundle {asset}"))?;
        let path = self.work.join(asset);
        let path = path_text(&path)?;
        if !Path::new(path).exists() {
            let url = format!("{RELEASES}{asset}");
            let fetched = run(
                "curl",
                &[
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--output",
                    path,
                    &url,
                ],
                &[],
            )?;
            if fetched.code != 0 {
                return Err(format!(
                    "{row}: download failed: {}",
                    tail(&fetched.stderr, 300)
                ));
            }
        }
        let restored = run(
            "python3",
            &[
                "tools/ci_seed_data.py",
                "restore",
                path,
                "--sha256",
                sha256,
                "--runtime-only",
                "--segment",
                row,
            ],
            &[],
        )?;
        if restored.code != 0 {
            return Err(format!(
                "{row}: restore failed: {}",
                tail(&restored.stderr, 300)
            ));
        }
        Ok(())
    }

    /// Record the oracle's entry for a row the lock lacks, on this runner, and
    /// print its lines (digests only) so it can be committed. The fresh lock
    /// file and the entry, or `None` when recording failed.
    fn record(&self, row: &str) -> Result<Option<(PathBuf, Table)>, String> {
        let committed = PathBuf::from(scenario::LOCK);
        let fresh = self.work.join(format!("{row}-lock.toml"));
        std::fs::copy(&committed, &fresh)
            .map_err(|error| format!("{}: {error}", fresh.display()))?;
        if verifier_entry(row, &fresh, Mode::Lock)? != 0 {
            return Ok(None);
        }
        let Some(recorded) = scenario::read_lock_file(&fresh)?.entries.remove(row) else {
            println!("{row}: the lock recorded no entry");
            return Ok(None);
        };
        let (old, new) = (lines_of(&committed), lines_of(&fresh));
        let old: Vec<&str> = old.iter().map(String::as_str).collect();
        let new: Vec<&str> = new.iter().map(String::as_str).collect();
        for line in added(&old, &new) {
            println!("{row}: LOCK+ {line}");
        }
        Ok(Some((fresh, recorded)))
    }

    /// One row: its verdict, `true` when it matches the lock.
    fn row(
        &self,
        row: &str,
        slices: &[String],
        diagnosed: &mut usize,
        window_failed: &mut usize,
    ) -> Result<bool, String> {
        self.restore(row)?;
        let (lock, locked) = match self.lock.entries.get(row) {
            Some(locked) => (PathBuf::from(scenario::LOCK), locked.clone()),
            None => match self.record(row)? {
                Some(recorded) => recorded,
                None => return Ok(false),
            },
        };
        let start = self.work.join(format!("{row}.start"));
        verify(
            row,
            &lock,
            Mode::CheckLock,
            &[("SM_EXPORT_START", path_text(&start)?)],
        )?;
        if !start.exists() {
            println!("{row}: no prepared start");
            return Ok(false);
        }
        verifier_entry(row, &lock, Mode::CheckLock)?;
        let ticks = locked
            .get("ticks")
            .and_then(Value::as_integer)
            .ok_or_else(|| format!("{row}: the lock entry has no tick count"))?;
        let prepared = Prepared {
            row: row.to_owned(),
            lock,
            content: self.work.join(format!("{row}.content")),
            sealed: self.work.join(format!("{row}.sealed")),
            inputs: Path::new("movies").join(self.pin(row, "inputs")?),
            ticks: ticks.to_string(),
        };
        let prepare = run(
            SM_REPLAY,
            &[
                "prepare",
                "rom/sm.smc",
                path_text(&start)?,
                path_text(&prepared.content)?,
                path_text(&prepared.sealed)?,
            ],
            &[],
        )?;
        if prepare.code != 0 {
            println!("{row}: prepare failed: {}", tail(&prepare.stderr, 300));
            return Ok(false);
        }
        let out = self.work.join(format!("{row}-entry.toml"));
        let replay = run(
            SM_REPLAY,
            &[
                "gate",
                path_text(&prepared.content)?,
                path_text(&prepared.sealed)?,
                path_text(&prepared.inputs)?,
                &prepared.ticks,
                path_text(&out)?,
            ],
            &[],
        )?;
        println!(
            "{row}: replay exit {}\n{}\n{}",
            replay.code,
            tail(replay.stdout.trim(), 1200),
            tail(replay.stderr.trim(), 600)
        );
        if replay.code != 0 {
            return Ok(false);
        }
        let entry = toml::read(&out)?;
        let differing = compare(&locked, &entry);
        let verdict = if differing.is_empty() {
            format!("MATCHES the oracle on {} streams", MACHINE.len())
        } else {
            let names: Vec<String> = differing.iter().map(Difference::to_string).collect();
            format!("DIFFERS in {}", names.join(", "))
        };
        let ran_ticks = entry.get("ticks").and_then(Value::as_integer).unwrap_or(-1);
        println!("{row}: {verdict} ({ran_ticks} ticks)");
        if slices.iter().any(|slice| slice == row) && !self.windowed(&prepared)? {
            *window_failed += 1;
        }
        if !differing.is_empty() && *diagnosed < DIAGNOSED {
            *diagnosed += 1;
            self.diagnose(&prepared)?;
        }
        Ok(differing.is_empty())
    }

    /// The verifier's own engine or the slice on a row, with extra environment.
    fn side(
        &self,
        prepared: &Prepared,
        side: Side,
        environment: &[(&str, &str)],
    ) -> Result<(), String> {
        match side {
            Side::Verifier => verify(&prepared.row, &prepared.lock, Mode::CheckLock, environment),
            Side::Slice => prepared.replay_one(None, environment),
        }
        .map(drop)
    }

    /// Where the slice and the verifier's own engine run part.
    fn diagnose(&self, prepared: &Prepared) -> Result<(), String> {
        let row = &prepared.row;
        let mut logs = Vec::new();
        for side in Side::BOTH {
            let log = self.work.join(format!("{row}-{}.log", side.as_str()));
            // Absent already is fine: the run writes it afresh.
            std::fs::remove_file(&log).ok();
            self.side(prepared, side, &[("SM_DIGEST_LOG", path_text(&log)?)])?;
            logs.push(lines_of(&log));
        }
        let [left, right] =
            [&logs[0], &logs[1]].map(|log| log.iter().map(String::as_str).collect::<Vec<_>>());
        println!("{row}: logs {} and {} ticks", left.len(), right.len());
        let first = first_differing(&left, &right);
        let shown: Vec<String> = first
            .iter()
            .map(|(name, tick)| format!("'{name}': {tick}"))
            .collect();
        println!(
            "{row}: first differing tick per region: {{{}}}",
            shown.join(", ")
        );
        let Some(tick) = first.iter().map(|(_, tick)| *tick).min() else {
            return Ok(());
        };
        let tick = tick.to_string();
        let mut dumps = Vec::new();
        for side in Side::BOTH {
            let path = self.work.join(format!("{row}-{}.dump", side.as_str()));
            self.side(
                prepared,
                side,
                &[("SM_DUMP_TICK", &tick), ("SM_DUMP_OUT", path_text(&path)?)],
            )?;
            match std::fs::read(&path) {
                Ok(bytes) => dumps.push(bytes),
                Err(error) => {
                    println!("{row}: no {} dump at tick {tick}: {error}", side.as_str());
                    return Ok(());
                }
            }
        }
        println!("{row}: at tick {tick}:\n{}", ranges(&dumps[0], &dumps[1]));
        Ok(())
    }

    /// The windowed app on the same row, headless, against sm_replay's run of
    /// it, compared Tick by Tick on every machine stream's digest.
    fn windowed(&self, prepared: &Prepared) -> Result<bool, String> {
        let row = &prepared.row;
        let window = self.work.join(format!("{row}-window-digests.txt"));
        let replay = self.work.join(format!("{row}-replay-digests.txt"));
        for path in [&window, &replay] {
            // Absent already is fine: the run writes it afresh.
            std::fs::remove_file(path).ok();
        }
        let save = self.work.join(format!("{row}-window.srm"));
        let shown = run(
            "xvfb-run",
            &[
                "-a",
                SM_SLICE,
                path_text(&prepared.content)?,
                path_text(&prepared.sealed)?,
                "--inputs",
                path_text(&prepared.inputs)?,
                "--ticks",
                &prepared.ticks,
                "--digests-out",
                path_text(&window)?,
                "--save",
                path_text(&save)?,
            ],
            &[("SDL_AUDIODRIVER", "dummy")],
        )?;
        println!(
            "{row}: sm_slice exit {}: {} {}",
            shown.code,
            tail(shown.stdout.trim(), 400),
            tail(shown.stderr.trim(), 400)
        );
        if shown.code != 0 {
            return Ok(false);
        }
        prepared.replay_one(Some(&replay), &[])?;
        let (left, right) = (lines_of(&window), lines_of(&replay));
        let first = left
            .iter()
            .zip(&right)
            .find(|(a, b)| a != b)
            .map(|(a, _)| a.split_whitespace().next().unwrap_or_default());
        let ticks: usize = prepared.ticks.parse().map_err(|_| "bad tick count")?;
        if first.is_none() && left.len() == ticks && right.len() == ticks {
            println!(
                "{row}: sm_slice MATCHES sm_replay on all {} machine streams at every Tick ({ticks} ticks)",
                MACHINE.len()
            );
            return Ok(true);
        }
        println!(
            "{row}: sm_slice DIFFERS from sm_replay: digest logs {} and {} of {ticks} ticks; first differing tick: {}",
            left.len(),
            right.len(),
            first.unwrap_or("None")
        );
        Ok(false)
    }
}

/// A fresh, empty working directory under the system's temporary one.
fn work_directory() -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    let path = std::env::temp_dir().join(format!("gate1-{}-{nanos:09}", std::process::id()));
    std::fs::create_dir(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(path)
}

/// `sm_pipeline gate1`, from the repository root.
pub fn command(arguments: &[String]) -> Result<ExitCode, String> {
    let arguments = Arguments::parse(arguments)?;
    let root = Path::new(".");
    let gate = Gate {
        work: work_directory()?,
        lock: scenario::read_lock(root)?,
    };
    let corpus = if arguments.rows.iter().any(|row| row == "all") {
        corpus_ids(root)?
    } else {
        Vec::new()
    };
    let rows = arguments
        .shard
        .take(select(&arguments.rows, &corpus, &gate.lock));
    println!("Gate 1 shard {}: {} rows", arguments.shard, rows.len());
    let (mut failed, mut diagnosed, mut window_failed) = (0, 0, 0);
    for row in &rows {
        if !gate.row(row, &arguments.slices, &mut diagnosed, &mut window_failed)? {
            failed += 1;
        }
    }
    println!("Gate 1 rows: {}/{} match", rows.len() - failed, rows.len());
    if !arguments.slices.is_empty() {
        println!(
            "sm_slice rows: {}/{} match sm_replay",
            arguments.slices.len() - window_failed,
            arguments.slices.len()
        );
    }
    Ok(if failed == 0 && window_failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn locked(ids: &[&str]) -> Lock {
        let mut lock = Lock::default();
        for id in ids {
            lock.entries.insert((*id).to_owned(), Table::default());
        }
        lock
    }

    #[test]
    fn all_adds_the_locked_corpus_rows_after_the_named_ones() {
        let lock = locked(&["a", "b", "d", "chain-room"]);
        let corpus = strings(&["a", "b", "c", "d"]);
        assert_eq!(
            select(&strings(&["b", "all"]), &corpus, &lock),
            strings(&["b", "a", "d"])
        );
        // A named row runs whether or not the lock holds it.
        assert_eq!(
            select(&strings(&["new-row"]), &corpus, &lock),
            strings(&["new-row"])
        );
    }

    #[test]
    fn shards_interleave_and_cover_every_row_once() {
        let rows: Vec<u32> = (0..10).collect();
        assert_eq!(
            Shard::parse("2/3").map(|shard| shard.take(rows.clone())),
            Ok(vec![1, 4, 7])
        );
        let mut covered: Vec<u32> = (1..=8)
            .flat_map(|index| Shard { index, count: 8 }.take(rows.clone()))
            .collect();
        covered.sort_unstable();
        assert_eq!(covered, rows);
        assert_eq!(Shard::parse("1/1"), Ok(Shard::WHOLE));
        for bad in ["0/8", "9/8", "1", "a/b", "1/0"] {
            assert!(Shard::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn arguments_take_rows_shard_and_slices() {
        let parsed = Arguments::parse(&strings(&[
            "--shard", "3/8", "--slice", "door", "item", "all", "--slice", "save",
        ]));
        assert_eq!(
            parsed,
            Ok(Arguments {
                rows: strings(&["item", "all"]),
                shard: Shard { index: 3, count: 8 },
                slices: strings(&["door", "save"]),
            })
        );
        assert!(Arguments::parse(&strings(&["--shard", "1/2"])).is_err());
        assert!(Arguments::parse(&strings(&["--bogus", "row"])).is_err());
    }

    fn entry(ticks: i64, wram: &str) -> Table {
        let digests: String = MACHINE
            .iter()
            .map(|stream| {
                let value = if *stream == "wram" { wram } else { "00" };
                format!("{stream} = \"{value}\"\n")
            })
            .collect();
        toml::parse(&format!(
            "format = 3\nticks = {ticks}\n[digests]\n{digests}"
        ))
        .expect("entry")
    }

    #[test]
    fn the_lock_comparison_names_each_differing_stream_and_the_ticks() {
        let lock = toml::parse(
            "pins = []\n[[scenario]]\nid = \"row\"\nticks = 5\ndigests = { \"wram\" = \"aa\", \"sram\" = \"00\", \"vram\" = \"00\", \"cgram\" = \"00\", \"oam\" = \"00\", \"ppu\" = \"00\", \"video\" = \"00\", \"apu\" = \"00\" }\n",
        )
        .expect("lock");
        let locked = scenario::tables(&lock, "scenario").expect("entries")[0].clone();
        assert_eq!(compare(&locked, &entry(5, "aa")), []);
        assert_eq!(
            compare(&locked, &entry(6, "ab")),
            [Difference::Stream("wram"), Difference::Ticks]
        );
        // A stream the replay does not digest differs.
        let partial = toml::parse("ticks = 5\n[digests]\nwram = \"aa\"\n").expect("partial");
        assert_eq!(compare(&locked, &partial).len(), MACHINE.len() - 1);
    }

    #[test]
    fn added_lines_are_the_ones_the_recording_inserted() {
        let old = ["a", "b", "c", "e"];
        let new = ["a", "b", "c", "d1", "d2", "e"];
        assert_eq!(added(&old, &new), ["d1", "d2"]);
        assert_eq!(added(&["x"], &["y", "x", "z"]), ["y", "z"]);
        assert!(added(&old, &old).is_empty());
    }

    #[test]
    fn digest_logs_name_each_regions_first_differing_tick() {
        let left = ["1 a b c d e", "2 a b c d e", "3 x b c y e"];
        let right = ["1 a b c d e", "2 a q c d e", "3 a q c d e"];
        assert_eq!(
            first_differing(&left, &right),
            [("vram", 2), ("wram", 3), ("oam", 3)]
        );
    }

    #[test]
    fn dump_ranges_merge_near_bytes_per_region() {
        let left = vec![0_u8; 0x3_0400];
        let mut right = left.clone();
        for at in [0x10, 0x12, 0x20, 0x2_0001] {
            right[at] = 1;
        }
        let shown = ranges(&left, &right);
        let lines: Vec<&str> = shown.lines().collect();
        assert_eq!(
            lines[0],
            "wram: 4 bytes in 2 ranges: 00010-00012, 00020-00020"
        );
        assert_eq!(lines[1], "vram: 1 bytes in 1 ranges: 00001-00001");
        assert_eq!(lines[3], "oam: 0 bytes in 0 ranges: ");
    }

    #[test]
    fn tails_count_characters() {
        assert_eq!(tail("abcdef", 3), "def");
        assert_eq!(tail("ab", 3), "ab");
        assert_eq!(tail("ab", 0), "");
        assert_eq!(tail("aé", 1), "é");
    }
}
