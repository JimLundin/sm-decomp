//! Lockstep differential harness.
//!
//! Both sides implement [`Console`]: the reference (RustySNES running the ROM)
//! and the subject (currently fallback-only execution on the same hardware).
//! Feed both the same input each frame; stop at the first
//! frame where any compared region differs and write a report.
//!
//! This is the same idea as snesrev/sm's built-in frame compare, made into a
//! batch tool so it can run over every TAS movie unattended.

use anyhow::{Result, ensure};
use std::fmt::Write;

pub trait Console {
    fn name(&self) -> &str;
    /// Run one frame with this P1 joypad value.
    fn run_frame(&mut self, pad: u16) -> Result<()>;
    /// Memory regions to compare, in a fixed order: (name, bytes).
    /// Start with WRAM; add VRAM, CGRAM, OAM once WRAM matches.
    fn regions(&self) -> Vec<(&'static str, &[u8])>;
}

#[derive(Debug)]
pub struct Divergence {
    pub frame: usize,
    pub region: &'static str,
    /// (offset, reference, subject) — first few differing bytes.
    pub bytes: Vec<(usize, u8, u8)>,
    pub total_diffs: usize,
}

pub enum Outcome {
    Match { frames: usize },
    Diverged(Divergence),
}

pub fn run(reference: &mut dyn Console, subject: &mut dyn Console, inputs: &[u16]) -> Result<Outcome> {
    for (frame, &pad) in inputs.iter().enumerate() {
        reference.run_frame(pad)?;
        subject.run_frame(pad)?;
        let r = reference.regions();
        let s = subject.regions();
        ensure!(r.len() == s.len(), "console region counts differ at frame {frame}");
        for ((name, rb), (subject_name, sb)) in r.iter().zip(&s) {
            ensure!(name == subject_name && rb.len() == sb.len(), "console region layout differs at frame {frame}");
            if rb != sb {
                let diffs: Vec<(usize, u8, u8)> = rb
                    .iter()
                    .zip(sb.iter())
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, (a, b))| (i, *a, *b))
                    .collect();
                return Ok(Outcome::Diverged(Divergence {
                    frame,
                    region: name,
                    total_diffs: diffs.len(),
                    bytes: diffs.into_iter().take(32).collect(),
                }));
            }
        }
    }
    Ok(Outcome::Match { frames: inputs.len() })
}

/// Human/agent-readable report. Add symbol names for WRAM addresses here
/// (from the bank logs' RAM map) — that is what makes triage fast.
pub fn report(d: &Divergence, misses_csv: Option<&str>) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "DIVERGED at frame {} in {} ({} bytes differ)", d.frame, d.region, d.total_diffs);
    let _ = writeln!(s, "offset    ref  sub");
    for (o, a, b) in &d.bytes {
        let _ = writeln!(s, "${o:05X}    {a:02X}   {b:02X}");
    }
    if let Some(m) = misses_csv {
        let _ = writeln!(s, "\nfallback misses so far:\n{m}");
    }
    let _ = writeln!(
        s,
        "\ntriage: missing variant/function → config/functions.toml; jump target → \
         config/jump_tables.toml; emitter bug → fix + fuzz case; timing → scheduler/override"
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        ram: Vec<u8>,
        frame: usize,
        bug_at: Option<usize>,
    }
    impl Console for Fake {
        fn name(&self) -> &str {
            "fake"
        }
        fn run_frame(&mut self, pad: u16) -> Result<()> {
            self.ram[0] = self.ram[0].wrapping_add(pad as u8);
            if Some(self.frame) == self.bug_at {
                self.ram[5] ^= 0xff;
            }
            self.frame += 1;
            Ok(())
        }
        fn regions(&self) -> Vec<(&'static str, &[u8])> {
            vec![("wram", &self.ram)]
        }
    }

    #[test]
    fn finds_first_divergent_frame() {
        let mut r = Fake { ram: vec![0; 16], frame: 0, bug_at: None };
        let mut s = Fake { ram: vec![0; 16], frame: 0, bug_at: Some(7) };
        match run(&mut r, &mut s, &[1; 20]).unwrap() {
            Outcome::Diverged(d) => {
                assert_eq!(d.frame, 7);
                assert_eq!(d.bytes, vec![(5, 0x00, 0xff)]);
            }
            Outcome::Match { .. } => panic!("should diverge"),
        }
    }

    #[test]
    fn mismatched_region_lengths_fail_instead_of_ignoring_trailing_bytes() {
        let mut r = Fake { ram: vec![0; 16], frame: 0, bug_at: None };
        let mut s = Fake { ram: vec![0; 17], frame: 0, bug_at: None };
        assert!(run(&mut r, &mut s, &[0]).is_err());
    }
}
