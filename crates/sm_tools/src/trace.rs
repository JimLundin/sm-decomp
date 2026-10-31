//! Single-side runs (docs/SCENARIO_HARNESS.md, free run and determinism):
//! one record per Tick, its trace file, and the running digests an oracle
//! digest lock commits (REVIEW, lane S condition 8).
//!
//! A Tick record holds one digest per machine stream (what the lockstep
//! compares as memory and registers, with its normalized RAM masked) and,
//! when observation is on, the typed observation of the domains the side
//! projects. Traces are run artifacts and are never committed; a lock keeps
//! only the running digests, which reveal nothing of the ROM.

use crate::engine::{Domains, Projection};
use crate::observation::{Domain, Observation};
use std::fmt::Write as _;
use std::hash::Hasher;
use std::io::{BufWriter, Write as _};
use std::path::Path;

/// The machine streams, in record order.
pub const MACHINE: [&str; 8] = [
    "wram", "sram", "vram", "cgram", "oam", "ppu", "video", "apu",
];

/// Every stream a run compares: the machine's, then each typed domain.
pub const STREAMS: usize = MACHINE.len() + Domain::ALL.len();

/// Bump when a stream's content or the digest changes; it refreshes every lock.
pub const FORMAT: u32 = 1;

/// Ticks between the combined digest's marks.
pub const MARK_TICKS: u32 = 1000;

const MAGIC: &[u8; 8] = b"SMTRACE\x01";

/// Stream `stream`'s name: a machine stream or a typed domain.
pub fn stream_name(stream: usize) -> &'static str {
    match stream.checked_sub(MACHINE.len()) {
        None => MACHINE[stream],
        Some(domain) => Domain::ALL[domain].name(),
    }
}

/// A typed domain's stream.
pub fn typed_stream(domain: Domain) -> usize {
    MACHINE.len()
        + Domain::ALL
            .iter()
            .position(|&other| other == domain)
            .expect("listed domain")
}

/// A word-at-a-time 64-bit digest of bytes or `Hash` values. It detects
/// change; it is not meant to resist anyone choosing the input.
#[derive(Clone, Copy, Debug)]
pub struct Digest(u64);

impl Default for Digest {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for Digest {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.0 = (self.0.rotate_left(23) ^ u64::from_le_bytes(word))
                .wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

impl Digest {
    /// The digest of `bytes`.
    pub fn of(bytes: &[u8]) -> u64 {
        let mut digest = Self::default();
        digest.write(bytes);
        digest.finish()
    }
}

/// Fold one Tick's digest into a running digest.
pub fn fold(running: u64, value: u64) -> u64 {
    let mut digest = Digest(running);
    digest.write_u64(value);
    digest.finish()
}

/// One Tick of a single-side run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tick {
    /// The input frame the Tick consumed: runs align on it.
    pub index: u32,
    /// The frame counter as the lockstep names the Tick.
    pub tick: i32,
    pub machine: [u64; MACHINE.len()],
    /// The typed observation, absent when observation is off or the side had none.
    pub typed: Option<Projection>,
}

impl Tick {
    /// The Tick's digest of `stream`, when the side has it.
    pub fn digest(&self, stream: usize) -> Option<u64> {
        match stream.checked_sub(MACHINE.len()) {
            None => Some(self.machine[stream]),
            Some(domain) => {
                let domain = Domain::ALL[domain];
                let typed = self.typed.as_ref()?;
                typed
                    .domains
                    .contains(domain)
                    .then(|| typed.observation.hash(domain))
            }
        }
    }
}

/// Writes a run's Ticks to a new trace file.
pub struct TraceWriter(BufWriter<std::fs::File>);

impl TraceWriter {
    pub fn create(path: &Path) -> Result<Self, String> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(MAGIC).map_err(|error| error.to_string())?;
        writer
            .write_all(&FORMAT.to_le_bytes())
            .map_err(|error| error.to_string())?;
        Ok(Self(writer))
    }

    pub fn write(&mut self, tick: &Tick) -> Result<(), String> {
        let mut record = Vec::new();
        record.extend(tick.index.to_le_bytes());
        record.extend(tick.tick.to_le_bytes());
        for digest in tick.machine {
            record.extend(digest.to_le_bytes());
        }
        match &tick.typed {
            None => record.push(0),
            Some(projection) => {
                let domains = Domain::ALL
                    .iter()
                    .enumerate()
                    .filter(|&(_, &domain)| projection.domains.contains(domain))
                    .fold(1u8, |mask, (bit, _)| mask | (2 << bit));
                record.push(domains);
                projection.observation.encode(&mut record);
            }
        }
        let length = u32::try_from(record.len()).map_err(|_| "trace record too long")?;
        self.0
            .write_all(&length.to_le_bytes())
            .and_then(|()| self.0.write_all(&record))
            .map_err(|error| error.to_string())
    }

    pub fn finish(mut self) -> Result<(), String> {
        self.0.flush().map_err(|error| error.to_string())
    }
}

/// Read every Tick of a trace file.
pub fn read(path: &Path) -> Result<Vec<Tick>, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let invalid = || format!("{}: not a format {FORMAT} trace", path.display());
    let header = MAGIC.iter().copied().chain(FORMAT.to_le_bytes());
    let mut rest = bytes
        .strip_prefix(header.collect::<Vec<_>>().as_slice())
        .ok_or_else(invalid)?;
    let mut ticks = Vec::new();
    while let Some((length, after)) = rest.split_first_chunk::<4>() {
        let length = usize::try_from(u32::from_le_bytes(*length)).map_err(|_| invalid())?;
        let (record, after) = after.split_at_checked(length).ok_or_else(invalid)?;
        rest = after;
        ticks.push(decode(record).ok_or_else(invalid)?);
    }
    if rest.is_empty() {
        Ok(ticks)
    } else {
        Err(invalid())
    }
}

fn decode(mut record: &[u8]) -> Option<Tick> {
    let mut take = |count: usize| {
        let (front, rest) = record.split_at_checked(count)?;
        record = rest;
        Some(front)
    };
    let index = u32::from_le_bytes(take(4)?.try_into().ok()?);
    let tick = i32::from_le_bytes(take(4)?.try_into().ok()?);
    let mut machine = [0; MACHINE.len()];
    for digest in &mut machine {
        *digest = u64::from_le_bytes(take(8)?.try_into().ok()?);
    }
    let mask = take(1)?[0];
    let typed = if mask == 0 {
        None
    } else {
        let mut domains = Domains::default();
        for (bit, &domain) in Domain::ALL.iter().enumerate() {
            if mask & (2 << bit) != 0 {
                domains.insert(domain);
            }
        }
        let observation = Observation::decode(&mut record)?;
        if !record.is_empty() {
            return None;
        }
        Some(Projection {
            observation,
            domains,
        })
    };
    Some(Tick {
        index,
        tick,
        machine,
        typed,
    })
}

/// The running digests of one run: what an oracle digest lock commits.
#[derive(Clone, Debug)]
pub struct Running {
    ticks: u32,
    digests: [u64; STREAMS],
    /// Streams some Tick lacked; they have no running digest.
    missing: [bool; STREAMS],
    combined: u64,
    marks: Vec<(u32, u64)>,
}

impl Default for Running {
    fn default() -> Self {
        Self {
            ticks: 0,
            digests: [Digest::default().finish(); STREAMS],
            missing: [false; STREAMS],
            combined: Digest::default().finish(),
            marks: Vec::new(),
        }
    }
}

impl Running {
    pub fn fold(&mut self, tick: &Tick) {
        for (stream, (running, missing)) in
            self.digests.iter_mut().zip(&mut self.missing).enumerate()
        {
            match tick.digest(stream) {
                Some(digest) => *running = fold(*running, digest),
                None => *missing = true,
            }
        }
        // The combined digest folds the machine streams, which every side has.
        for &digest in &tick.machine {
            self.combined = fold(self.combined, digest);
        }
        self.ticks += 1;
        if self.ticks.is_multiple_of(MARK_TICKS) {
            self.marks.push((self.ticks, self.combined));
        }
    }

    /// The lock entry's body: the tick count, every complete stream's running
    /// digest, and the combined digest at each mark and at the end.
    pub fn lock_toml(&self) -> String {
        let mut text = String::new();
        macro_rules! line {
            ($($argument:tt)*) => {
                writeln!(text, $($argument)*).expect("String write")
            };
        }
        line!("format = {FORMAT}");
        line!("ticks = {}", self.ticks);
        line!("combined = \"{:016x}\"", self.combined);
        line!("[digests]");
        for (stream, (digest, missing)) in self.digests.iter().zip(&self.missing).enumerate() {
            if !missing {
                line!("{} = \"{digest:016x}\"", stream_name(stream));
            }
        }
        line!("[marks]");
        for (ticks, digest) in &self.marks {
            line!("\"{ticks}\" = \"{digest:016x}\"");
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(index: u32, wram: u64, samus_x: Option<u16>) -> Tick {
        Tick {
            index,
            tick: i32::try_from(index).expect("small") + 10,
            machine: [wram, 1, 2, 3, 4, 5, 6, 7],
            typed: samus_x.map(|x| {
                let mut domains = Domains::default();
                domains.insert(Domain::Samus);
                let mut observation = Observation::default();
                observation.samus.x = x;
                Projection {
                    observation,
                    domains,
                }
            }),
        }
    }

    #[test]
    fn a_trace_reads_back_every_tick() {
        let directory = std::env::temp_dir().join(format!("sm-tools-trace-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temp directory");
        let path = directory.join("run.trace");
        let _ = std::fs::remove_file(&path);
        let ticks = vec![tick(0, 9, Some(3)), tick(1, 8, None)];
        let mut writer = TraceWriter::create(&path).expect("new trace");
        for tick in &ticks {
            writer.write(tick).expect("write");
        }
        writer.finish().expect("flush");
        assert_eq!(read(&path).expect("read"), ticks);
        std::fs::remove_dir_all(&directory).expect("cleanup");
    }

    #[test]
    fn a_stream_some_tick_lacks_has_no_running_digest() {
        let mut running = Running::default();
        running.fold(&tick(0, 9, Some(3)));
        running.fold(&tick(1, 9, None));
        let lock = running.lock_toml();
        assert!(lock.contains("wram = "));
        assert!(!lock.contains("samus = "));
        assert!(lock.contains("ticks = 2"));
    }

    #[test]
    fn one_changed_tick_changes_its_stream_and_the_combined_digest() {
        let fold_all = |ticks: &[Tick]| {
            let mut running = Running::default();
            for tick in ticks {
                running.fold(tick);
            }
            running
        };
        let left = fold_all(&[tick(0, 1, Some(1)), tick(1, 2, Some(1))]);
        let right = fold_all(&[tick(0, 1, Some(1)), tick(1, 2, Some(2))]);
        let samus = typed_stream(Domain::Samus);
        assert_ne!(left.digests[samus], right.digests[samus]);
        assert_eq!(left.digests[0], right.digests[0]);
        assert_eq!(left.combined, right.combined);
        let changed = fold_all(&[tick(0, 1, Some(1)), tick(1, 3, Some(1))]);
        assert_ne!(left.combined, changed.combined);
        assert_eq!(stream_name(samus), "samus");
    }
}
