//! Migration seeds: the lsnes (bsnes v085) room captures today's corpus
//! starts from (`target/seeds/<capture>/<NNNN>-<ROOM>-*`). They hold the
//! memories and a register CSV, but no SPC700, DSP or timer state, so no
//! emulator resumes them exactly. Harness-native seeds replace them.

use crate::OracleError;
use crate::lakesnes::LakeSnes;
use std::collections::BTreeMap;
use std::path::Path;

/// One lsnes capture.
#[derive(Clone, Debug)]
pub struct Capture {
    pub wram: Vec<u8>,
    pub vram: Vec<u8>,
    pub cgram: Vec<u8>,
    pub oam: Vec<u8>,
    pub sram: Vec<u8>,
    pub apu_ram: Vec<u8>,
    /// The register CSV: lsnes's names and values.
    pub registers: BTreeMap<String, i64>,
}

impl Capture {
    /// Read capture `index` (1-based) from `directory`, whatever its room.
    pub fn read(directory: &Path, index: u32) -> Result<Self, String> {
        let prefix = format!("{index:04}-");
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        let mut stem = None;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(base) = name
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix("-wram.bin"))
            {
                stem = Some(format!("{prefix}{base}"));
            }
        }
        let stem = stem.ok_or_else(|| format!("no capture {index} in {}", directory.display()))?;
        let file = |part: &str| {
            let path = directory.join(format!("{stem}-{part}"));
            std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))
        };
        let csv = String::from_utf8(file("registers.csv")?).map_err(|error| error.to_string())?;
        let registers = csv
            .lines()
            .skip(1)
            .filter_map(|line| {
                let (name, value) = line.split_once(',')?;
                Some((name.to_owned(), value.trim().parse().ok()?))
            })
            .collect();
        Ok(Self {
            wram: file("wram.bin")?,
            vram: file("vram.bin")?,
            cgram: file("cgram.bin")?,
            oam: file("oam.bin")?,
            sram: file("sram.bin")?,
            apu_ram: file("apuram.bin")?,
            registers,
        })
    }

    /// A register's value from the CSV.
    pub fn register(&self, name: &str) -> Option<i64> {
        self.registers.get(name).copied()
    }
}

/// Load `capture` into a powered-on LakeSnes.
pub(crate) fn apply(_oracle: &mut LakeSnes, _capture: &Capture) -> Result<(), OracleError> {
    Err(OracleError::Seed(
        "lsnes captures are not loaded yet; start from power-on or a LakeSnes state".into(),
    ))
}
