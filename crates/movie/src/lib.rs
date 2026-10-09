//! TAS movie → `.inputs` file: one line per frame, P1 joypad as 4 hex digits,
//! using the SNES auto-joypad bit layout ($4218/$4219):
//! B Y Select Start Up Down Left Right | A X L R 0 0 0 0
//!
//! Supported: BizHawk `.bk2` (or its extracted `Input Log.txt`).
//! TODO: lsnes `.lsmv` (many Super Metroid TASes on TASVideos use it).

use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};

pub fn button_bit(name: &str) -> Option<u16> {
    Some(match name {
        "B" => 0x8000,
        "Y" => 0x4000,
        "Select" => 0x2000,
        "Start" => 0x1000,
        "Up" => 0x0800,
        "Down" => 0x0400,
        "Left" => 0x0200,
        "Right" => 0x0100,
        "A" => 0x0080,
        "X" => 0x0040,
        "L" => 0x0020,
        "R" => 0x0010,
        _ => return None,
    })
}

/// Read the BizHawk input log from a .bk2 (zip) or a plain text file.
pub fn read_input_log(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    if bytes.starts_with(b"PK") {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
        let mut f = z.by_name("Input Log.txt").context("no 'Input Log.txt' in .bk2")?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(String::from_utf8(bytes)?)
    }
}

/// Parse a BizHawk SNES input log into P1 joypad words.
///
/// The `LogKey:` line lists button names per group, groups separated by `#`
/// and buttons by `|`. Each frame line is `|group|group|...|`, one character
/// per button, `.` meaning released.
pub fn parse_bk2_log(log: &str) -> Result<Vec<u16>> {
    let key = log
        .lines()
        .find_map(|l| l.strip_prefix("LogKey:"))
        .context("no LogKey line")?;
    let groups: Vec<Vec<&str>> = key
        .split('#')
        .filter(|g| !g.is_empty())
        .map(|g| g.split('|').filter(|b| !b.is_empty()).collect())
        .collect();

    let mut frames = Vec::new();
    for line in log.lines().filter(|l| l.starts_with('|')) {
        let fields: Vec<&str> = line.trim_matches('|').split('|').collect();
        if fields.len() != groups.len() {
            bail!("frame {} has {} groups, LogKey has {}", frames.len(), fields.len(), groups.len());
        }
        let mut pad = 0u16;
        for (g, field) in groups.iter().zip(&fields) {
            // Button-only groups have one char per button; analog/other groups are skipped.
            if field.chars().count() != g.len() {
                continue;
            }
            for (name, c) in g.iter().zip(field.chars()) {
                if c == '.' || c == ' ' {
                    continue;
                }
                if let Some(btn) = name.strip_prefix("P1 ") {
                    if let Some(bit) = button_bit(btn) {
                        pad |= bit;
                    }
                }
            }
        }
        frames.push(pad);
    }
    Ok(frames)
}

pub fn write_inputs(frames: &[u16]) -> String {
    let mut s = String::with_capacity(frames.len() * 5);
    for f in frames {
        s.push_str(&format!("{f:04X}\n"));
    }
    s
}

pub fn read_inputs(text: &str) -> Result<Vec<u16>> {
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| u16::from_str_radix(l.trim(), 16).with_context(|| format!("bad input line {l:?}")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bizhawk_log() {
        let log = "[Input]\nLogKey:#Reset|Power|#P1 Up|P1 Down|P1 Left|P1 Right|P1 Select|P1 Start|P1 Y|P1 B|P1 X|P1 A|P1 L|P1 R|\n|..|............|\n|..|...R.S.B....|\n[/Input]\n";
        let f = parse_bk2_log(log).unwrap();
        assert_eq!(f, vec![0x0000, 0x0100 | 0x1000 | 0x8000]);
        assert_eq!(read_inputs(&write_inputs(&f)).unwrap(), f);
    }
}
