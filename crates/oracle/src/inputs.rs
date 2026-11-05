//! Scenario input files: one hexadecimal JOY1 word per line.

use crate::record::Pad;
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
