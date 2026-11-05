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

/// SHA-256 (FIPS 180-4).
pub fn sha256(message: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut hash: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    let bits = u64::try_from(message.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    padded.extend_from_slice(&bits.to_be_bytes());
    for block in padded.as_chunks::<64>().0 {
        let mut schedule = [0u32; 64];
        for (word, bytes) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
            *word = u32::from_be_bytes(*bytes);
        }
        for t in 16..64 {
            let (w15, w2) = (schedule[t - 15], schedule[t - 2]);
            let s0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3);
            let s1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10);
            schedule[t] = schedule[t - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[t - 7])
                .wrapping_add(s1);
        }
        let mut v = hash;
        for (&k, &w) in K.iter().zip(&schedule) {
            let [a, b, c, d, e, f, g, h] = v;
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(k)
                .wrapping_add(w);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            v = [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g];
        }
        for (word, add) in hash.iter_mut().zip(v) {
            *word = word.wrapping_add(add);
        }
    }
    let mut digest = [0u8; 32];
    for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(hash) {
        *bytes = word.to_be_bytes();
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_the_standard_vectors() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

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
