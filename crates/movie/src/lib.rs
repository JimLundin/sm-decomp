//! TAS movie → `.inputs` file: one line per frame, P1 joypad as 4 hex digits,
//! using the SNES auto-joypad bit layout ($4218/$4219):
//! B Y Select Start Up Down Left Right | A X L R 0 0 0 0
//!
//! Supported: BizHawk `.bk2` (or extracted logs) and clean-start lsnes `.lsmv`
//! with one SNES P1 gamepad record per frame. Unsupported initialization,
//! reset, subframe and extra-controller records are rejected explicitly.

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};

pub mod corpus;

/// Imported input and the ROM identity recorded by its container, if available.
pub struct Movie {
    pub frames: Vec<u16>,
    pub rom_sha256: Option<String>,
}

/// Import a movie or the plain hexadecimal `.inputs` format.
pub fn load(path: &Path) -> Result<Movie> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    if bytes.starts_with(b"PK") {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
        if archive.file_names().any(|name| name == "input") {
            return read_lsmv(&mut archive);
        }
        anyhow::ensure!(
            !archive.file_names().any(|name| matches!(name, "Core.bin" | "SaveRAM.bin")),
            "stateful BK2 initialization is not supported; require a clean start"
        );
        let log = zip_text(&mut archive, "Input Log.txt")?;
        return Ok(Movie { frames: parse_bk2_log(&log)?, rom_sha256: None });
    }
    let text = String::from_utf8(bytes)?;
    let frames = if text.contains("LogKey:") { parse_bk2_log(&text)? } else { read_inputs(&text)? };
    Ok(Movie { frames, rom_sha256: None })
}

fn zip_text<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>, name: &str) -> Result<String> {
    let mut text = String::new();
    archive.by_name(name).with_context(|| format!("missing {name:?} in movie"))?.read_to_string(&mut text)?;
    Ok(text)
}

fn read_lsmv<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> Result<Movie> {
    // Source: https://tasvideos.org/EmulatorResources/Lsnes/LSMV
    for name in archive.file_names() {
        if matches!(name, "savestate" | "savestate.anchor" | "movie.bin")
            || name.starts_with("moviesram.")
            || name.starts_with("ramcontent.")
            || name.starts_with("setting.")
        {
            bail!("unsupported LSMV initialization member {name:?}; require a clean default start");
        }
        if name.starts_with("port") && name[4..].chars().all(|c| c.is_ascii_digit()) {
            bail!("explicit LSMV controller configuration {name:?} is not supported");
        }
    }
    let game_type = zip_text(archive, "gametype")?;
    anyhow::ensure!(game_type.trim() == "snes_ntsc", "unsupported LSMV game type {:?}", game_type.trim());
    let controls = zip_text(archive, "controlsversion")?;
    anyhow::ensure!(controls.trim() == "0", "unsupported LSMV controls version {:?}", controls.trim());
    let hash = zip_text(archive, "rom.sha256")?.trim().to_lowercase();
    anyhow::ensure!(hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()), "invalid LSMV ROM SHA-256");
    let frames = parse_lsmv_log(&zip_text(archive, "input")?)?;
    Ok(Movie { frames, rom_sha256: Some(hash) })
}

/// Decode lsnes' SNES `BYsSudlrAXLR` order into the SNES hardware bit layout.
/// The current runner has frame-level input only, so resets and subframes fail.
pub fn parse_lsmv_log(log: &str) -> Result<Vec<u16>> {
    const BUTTONS: &[u8; 12] = b"BYsSudlrAXLR";
    const BITS: [u16; 12] =
        [0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010];
    let mut frames = Vec::new();
    for (number, line) in log.lines().enumerate() {
        let (header, pad) =
            line.split_once('|').with_context(|| format!("LSMV line {}: missing controller", number + 1))?;
        anyhow::ensure!(
            header == "F." || header == "F. 0 0",
            "LSMV line {}: unsupported reset, subframe, or timing record {header:?}",
            number + 1
        );
        anyhow::ensure!(pad.len() == 12, "LSMV line {}: expected one 12-button P1 gamepad", number + 1);
        let mut mask = 0;
        for (index, value) in pad.bytes().enumerate() {
            anyhow::ensure!(
                value == b'.' || value == b' ' || value == BUTTONS[index],
                "LSMV line {}: invalid button at column {}",
                number + 1,
                index + 1
            );
            if value == BUTTONS[index] {
                mask |= BITS[index];
            }
        }
        frames.push(mask);
    }
    anyhow::ensure!(!frames.is_empty(), "empty LSMV input");
    Ok(frames)
}

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
    let key = log.lines().find_map(|l| l.strip_prefix("LogKey:")).context("no LogKey line")?;
    let groups: Vec<Vec<&str>> =
        key.split('#').filter(|g| !g.is_empty()).map(|g| g.split('|').filter(|b| !b.is_empty()).collect()).collect();
    for name in groups.iter().flatten() {
        anyhow::ensure!(
            matches!(*name, "Reset" | "Power")
                || name.strip_prefix("P1 ").and_then(button_bit).is_some()
                || name.strip_prefix("P2 ").and_then(button_bit).is_some(),
            "unsupported BK2 control {name:?}"
        );
    }

    let mut frames = Vec::new();
    for line in log.lines().filter(|l| l.starts_with('|')) {
        let fields: Vec<&str> = line.trim_matches('|').split('|').collect();
        if fields.len() != groups.len() {
            bail!("frame {} has {} groups, LogKey has {}", frames.len(), fields.len(), groups.len());
        }
        let mut pad = 0u16;
        for (g, field) in groups.iter().zip(&fields) {
            anyhow::ensure!(field.chars().count() == g.len(), "BK2 frame {}: invalid button group", frames.len());
            for (name, c) in g.iter().zip(field.chars()) {
                if c == '.' || c == ' ' {
                    continue;
                }
                let bit = name.strip_prefix("P1 ").and_then(button_bit).with_context(|| {
                    format!("BK2 frame {}: active reset, power, or extra controller {name:?}", frames.len())
                })?;
                pad |= bit;
            }
        }
        frames.push(pad);
    }
    anyhow::ensure!(!frames.is_empty(), "empty BK2 input");
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
    let frames: Vec<u16> = text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| u16::from_str_radix(l.trim(), 16).with_context(|| format!("bad input line {l:?}")))
        .collect::<Result<_>>()?;
    anyhow::ensure!(!frames.is_empty(), "empty input");
    anyhow::ensure!(frames.iter().all(|pad| pad & 0xf == 0), "non-gamepad bits in input");
    Ok(frames)
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

    #[test]
    fn lsnes_buttons_map_to_hardware_bits() {
        let log = "F. 0 0|............\nF.|BYsSudlrAXLR\nF. 0 0|..sS....A..R\n";
        assert_eq!(parse_lsmv_log(log).unwrap(), vec![0, 0xfff0, 0x3090]);
        for (column, bit) in
            [0x8000, 0x4000, 0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40, 0x20, 0x10].into_iter().enumerate()
        {
            let mut pad = *b"............";
            pad[column] = b"BYsSudlrAXLR"[column];
            assert_eq!(parse_lsmv_log(&format!("F.|{}", String::from_utf8(pad.to_vec()).unwrap())).unwrap(), [bit]);
        }
    }

    #[test]
    fn refuses_to_flatten_subframes_resets_and_extra_ports() {
        for log in ["..|............", "FR 0 0|............", "F.|............|............", "F.|...X........", ""] {
            assert!(parse_lsmv_log(log).is_err(), "{log:?}");
        }
    }

    #[test]
    fn rejects_unsupported_bizhawk_controls_instead_of_dropping_them() {
        for log in [
            "LogKey:#Reset|Power|#P1 A|\n|R.|.|",
            "LogKey:#P1 A|#P2 A|\n|.|A|",
            "LogKey:#P1 A|\n|AA|",
            "LogKey:#P1 Analog|\n|000|",
        ] {
            assert!(parse_bk2_log(log).is_err(), "{log:?}");
        }
        assert!(read_inputs("").is_err());
        assert!(read_inputs("0001").is_err());
    }

    #[test]
    fn lsmv_container_checks_rom_identity_and_initialization() {
        use std::io::{Cursor, Write};
        fn archive(extra: Option<&str>) -> zip::ZipArchive<Cursor<Vec<u8>>> {
            let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
            let mut files = vec![
                ("gametype", "snes_ntsc\n"),
                ("controlsversion", "0\n"),
                ("rom.sha256", "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72\n"),
                ("input", "F. 0 0|...S........\n"),
            ];
            if let Some(name) = extra {
                files.push((name, "data"));
            }
            for (name, text) in files {
                zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
                zip.write_all(text.as_bytes()).unwrap();
            }
            zip::ZipArchive::new(Cursor::new(zip.finish().unwrap().into_inner())).unwrap()
        }
        let movie = read_lsmv(&mut archive(None)).unwrap();
        assert_eq!(movie.frames, [0x1000]);
        assert_eq!(
            movie.rom_sha256.as_deref(),
            Some("12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72")
        );
        for extra in ["savestate", "savestate.anchor", "moviesram.0", "ramcontent.0", "setting.region", "port1"] {
            assert!(read_lsmv(&mut archive(Some(extra))).is_err());
        }
    }
}
