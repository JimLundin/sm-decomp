//! Fixed, named original-controller scenario; no native host-step conversion.
use crate::{Result, hash_bytes};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter, write::SimpleFileOptions};

pub const FRAMES: usize = 8850;
pub const PREFIX: usize = 8639;
pub const FIRE: usize = 8660;
pub const TURN: usize = 8650;
pub const SCENARIO: &str = "original-df45-ordinary-shot-v2";
pub const BUTTON: &str = "X";
pub const INPUT_SHA256: &str = "cf18e063be3e419b5150c086ebb3c4d062f32a634408c4a742fc86be6c3dc35c";
const MEMBERS: [&str; 15] = [
    "authors",
    "branchname.0",
    "controlsversion",
    "coreversion",
    "gametype",
    "input",
    "projectid",
    "rerecords",
    "rom.hint",
    "rom.sha256",
    "rrdata",
    "starttime.second",
    "starttime.subsecond",
    "subtitles",
    "systemid",
];
const BUTTONS: &[u8; 12] = b"BYsSudlrAXLR";
const NEUTRAL: &[u8; 20] = b"F. 0 0|............\n";

fn read_members(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut archive = ZipArchive::new(File::open(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    if archive.len() != MEMBERS.len() {
        return Err("original movie must contain exactly the pinned 15 reset-only members".into());
    }
    let mut entries = BTreeMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if !MEMBERS.contains(&file.name()) || file.size() > 4_000_000 || file.is_dir() {
            return Err(format!("unexpected or oversized movie member {}", file.name()));
        }
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| e.to_string())?; // Validates ZIP CRC.
        if entries.insert(name, bytes).is_some() {
            return Err("duplicate original movie member".into());
        }
    }
    for (name, expected) in [
        ("rom.sha256", crate::ROM_SHA256),
        ("controlsversion", "0"),
        ("coreversion", "bsnes v085 (Compatibility core)"),
        ("gametype", "snes_ntsc"),
        ("systemid", "lsnes-rr1"),
        ("starttime.second", "1000000000"),
        ("starttime.subsecond", "0"),
    ] {
        if std::str::from_utf8(&entries[name]).map_err(|e| e.to_string())?.trim() != expected {
            return Err(format!("original movie {name} identity differs"));
        }
    }
    Ok(entries)
}

fn scenario_input(original: &[u8]) -> Result<Vec<u8>> {
    if original.len() != 129712 * 20 || hash_bytes(original) != INPUT_SHA256 {
        return Err("pinned original input member identity differs".into());
    }
    for row in original.as_chunks::<20>().0 {
        if &row[..7] != b"F. 0 0|"
            || row[19] != b'\n'
            || row[7..19].iter().zip(BUTTONS).any(|(&v, &button)| v != b'.' && v != button)
        {
            return Err("unsupported movie subframe/reset/controller encoding".into());
        }
    }
    let mut input = original[..PREFIX * 20].to_vec();
    for frame in PREFIX + 1..=FRAMES {
        let mut row = *NEUTRAL;
        if frame == FIRE {
            row[7 + 9] = b'X';
        } else if frame == TURN {
            row[7 + 7] = b'r';
        }
        input.extend_from_slice(&row);
    }
    Ok(input)
}

pub fn author(source: &Path, destination: &Path) -> Result<BTreeMap<String, String>> {
    let before = read_members(source)?;
    let input = scenario_input(&before["input"])?;
    let mut source = ZipArchive::new(File::open(source).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut output =
        ZipWriter::new(File::options().write(true).create_new(true).open(destination).map_err(|e| e.to_string())?);
    for i in 0..source.len() {
        let member = source.by_index(i).map_err(|e| e.to_string())?;
        if member.name() == "input" {
            output
                .start_file(
                    "input",
                    SimpleFileOptions::default()
                        .compression_method(CompressionMethod::Deflated)
                        .last_modified_time(DateTime::from_date_and_time(2000, 1, 1, 0, 0, 0).unwrap()),
                )
                .map_err(|e| e.to_string())?;
            output.write_all(&input).map_err(|e| e.to_string())?;
        } else {
            output.raw_copy_file(member).map_err(|e| e.to_string())?;
        }
    }
    output.finish().map_err(|e| e.to_string())?.sync_all().map_err(|e| e.to_string())?;
    let after = read_members(destination)?;
    for (name, bytes) in &before {
        let expected = if name == "input" { &input } else { bytes };
        if after.get(name) != Some(expected) {
            return Err(format!("authored movie member verification failed: {name}"));
        }
    }
    Ok(after.iter().map(|(name, bytes)| (name.clone(), hash_bytes(bytes))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_x_and_right_use_hardware_button_order() {
        let position = BUTTONS.iter().position(|&b| b == BUTTON.as_bytes()[0]).unwrap();
        assert_eq!(1_u16 << (15 - position), 0x0040);
        let right = BUTTONS.iter().position(|&b| b == b'r').unwrap();
        assert_eq!(1_u16 << (15 - right), 0x0100);
        assert_eq!(FIRE - TURN - 1, 9);
        assert_eq!(FIRE - PREFIX - 1, 20);
    }
    #[test]
    fn rejects_unidentified_input_before_authoring() {
        assert!(scenario_input(NEUTRAL).is_err());
    }
}
