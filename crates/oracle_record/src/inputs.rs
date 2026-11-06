//! Scenario input files: one hexadecimal JOY1 word per line, per Tick in
//! the lock's inputs and per video frame in a frame-pad file (docs/ORACLE.md, RELOCK b1).

use crate::Pad;
use std::path::Path;

/// Read an `.inputs` file.
pub fn read(path: &Path) -> Result<Vec<Pad>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(number, line)| {
            u16::from_str_radix(line.trim(), 16)
                .map(Pad)
                .map_err(|error| format!("{}:{}: {error}", path.display(), number + 1))
        })
        .collect()
}

/// Write `pads` in the `.inputs` format, one word per line.
pub fn write(path: &Path, pads: &[Pad]) -> Result<(), String> {
    let text: String = pads.iter().map(|pad| format!("{:04x}\n", pad.0)).collect();
    std::fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}
