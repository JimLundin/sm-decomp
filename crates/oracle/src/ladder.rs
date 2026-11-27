//! A movie's ladder: lsnes states saved every N frames, so that a seed cut or a
//! measurement resumes from the nearest rung instead of playing from power-on.
//!
//! The ladder of `movies/NAME.inputs` lives in `DIR/NAME/`. Rung F is
//! `FFFFFF.lss`, the state before step F (the convention of native seeds),
//! and `FFFFFF.record`, the raw record of step F-1, always with the memories.
//! `ladder.txt` lists each rung's frame and the SHA-256 of its record.
//!
//! A rung reproduces the next when the oracle resumed from its state and
//! stepped to the next rung yields a record byte-identical to the stored one:
//! the cut, the CPU, the pad, the frame counter and all five memories. The
//! states themselves are not compared: lsnes's state bytes can differ between
//! runs that reach the same machine state.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use data::rom::sha256;

/// The manifest beside the rungs.
const MANIFEST: &str = "ladder.txt";

/// The rungs of one movie, as its manifest lists them.
pub struct Ladder {
    directory: PathBuf,
    /// Each rung's frame and its record's SHA-256, in hexadecimal.
    rungs: BTreeMap<u64, String>,
}

impl Ladder {
    /// The ladder of the movie `inputs` under `root`, empty if none is cut yet.
    pub fn open(root: &Path, inputs: &Path) -> Result<Self, String> {
        let name = inputs
            .file_stem()
            .ok_or_else(|| format!("{}: no movie name", inputs.display()))?;
        let directory = root.join(name);
        let manifest = directory.join(MANIFEST);
        let rungs = match std::fs::read_to_string(&manifest) {
            Ok(text) => text
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| {
                    let (frame, sha) = line
                        .split_once(' ')
                        .ok_or_else(|| format!("{}: {line}", manifest.display()))?;
                    let frame = frame
                        .parse()
                        .map_err(|error| format!("{}: {line}: {error}", manifest.display()))?;
                    Ok((frame, sha.trim().to_owned()))
                })
                .collect::<Result<_, String>>()?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(format!("{}: {error}", manifest.display())),
        };
        Ok(Self { directory, rungs })
    }

    pub fn state_path(&self, frame: u64) -> PathBuf {
        self.directory.join(format!("{frame:06}.lss"))
    }

    pub fn record_path(&self, frame: u64) -> PathBuf {
        self.directory.join(format!("{frame:06}.record"))
    }

    /// The rungs' frames, in order.
    pub fn frames(&self) -> impl Iterator<Item = u64> + '_ {
        self.rungs.keys().copied()
    }

    /// The SHA-256 the manifest lists for rung `frame`'s record.
    pub fn record_sha256(&self, frame: u64) -> Option<&str> {
        self.rungs.get(&frame).map(String::as_str)
    }

    /// The latest rung at or before `frame`: resumed from it, the oracle's
    /// first step is the rung's frame, so it reaches step `frame`.
    pub fn nearest(&self, frame: u64) -> Option<u64> {
        self.rungs
            .range(..=frame)
            .next_back()
            .map(|(&rung, _)| rung)
    }

    /// Store rung `frame` and list it in the manifest, replacing any rung of
    /// the same frame.
    pub fn add(&mut self, frame: u64, record: &[u8], state: &Path) -> Result<(), String> {
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| format!("{}: {error}", self.directory.display()))?;
        let record_path = self.record_path(frame);
        std::fs::write(&record_path, record)
            .map_err(|error| format!("{}: {error}", record_path.display()))?;
        let state_path = self.state_path(frame);
        std::fs::copy(state, &state_path)
            .map_err(|error| format!("{}: {error}", state.display()))?;
        self.rungs.insert(frame, hex(&sha256(record)));
        let listed: String = self
            .rungs
            .iter()
            .map(|(frame, sha)| format!("{frame} {sha}\n"))
            .collect();
        let manifest = self.directory.join(MANIFEST);
        std::fs::write(
            &manifest,
            format!("# oracle ladder: rung frame, SHA-256 of its record\n{listed}"),
        )
        .map_err(|error| format!("{}: {error}", manifest.display()))
    }
}

/// The parts of an lsnes record (`tools/oracle/lsnes_server.lua`), by byte range.
const PARTS: [(&str, usize, usize); 11] = [
    ("magic", 0, 4),
    ("cut", 4, 20),
    ("frame", 20, 24),
    ("pad", 24, 26),
    ("cpu", 26, 42),
    ("dumped", 42, 43),
    ("wram", 43, 0x2_002b),
    ("vram", 0x2_002b, 0x3_002b),
    ("cgram", 0x3_002b, 0x3_022b),
    ("oam", 0x3_022b, 0x3_044b),
    ("sram", 0x3_044b, 0x3_244b),
];

/// The parts in which two records differ, with the differing byte counts.
pub fn differences(left: &[u8], right: &[u8]) -> Vec<(&'static str, usize)> {
    let mut parts: Vec<(&'static str, usize)> = PARTS
        .iter()
        .filter_map(|&(name, start, end)| {
            fn part(bytes: &[u8], start: usize, end: usize) -> &[u8] {
                bytes.get(start..end.min(bytes.len())).unwrap_or_default()
            }
            let (a, b) = (part(left, start, end), part(right, start, end));
            let differing =
                a.iter().zip(b).filter(|(x, y)| x != y).count() + a.len().abs_diff(b.len());
            (differing > 0).then_some((name, differing))
        })
        .collect();
    if left.len() != right.len() {
        parts.push(("length", left.len().abs_diff(right.len())));
    }
    parts
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_is_the_latest_rung_at_or_before() {
        let ladder = Ladder {
            directory: PathBuf::new(),
            rungs: [(10, String::new()), (20, String::new())].into(),
        };
        assert_eq!(ladder.nearest(9), None);
        assert_eq!(ladder.nearest(10), Some(10));
        assert_eq!(ladder.nearest(25), Some(20));
    }
}
