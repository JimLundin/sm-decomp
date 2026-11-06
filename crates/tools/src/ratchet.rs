//! `tools lockstep --check-ratchet LOCK CACHE ROM RATCHET`: lockstep over every row the ratchet
//! file names, held against its numbers ([`verify::ratchet::check`]).

use std::path::Path;

use verify::lockstep::Report;

pub const USAGE: &str = "tools lockstep --check-ratchet LOCK CACHE ROM RATCHET";

/// Lockstep over one row's cache for the frames in `window`.
pub fn run_row(
    lock: &str,
    cache: &str,
    rom: &str,
    row: &str,
    window: std::ops::Range<u64>,
) -> Result<Report, String> {
    verify::ratchet::run_row(
        Path::new(lock),
        Path::new(cache),
        Path::new(rom),
        row,
        window,
    )
}

/// Runs the check; the report when every row holds its numbers, the regressions otherwise.
pub fn check(args: &[String]) -> Result<String, String> {
    let [lock, cache, rom, ratchet] = args else {
        return Err(USAGE.to_owned());
    };
    verify::ratchet::check(
        Path::new(lock),
        Path::new(cache),
        Path::new(rom),
        Path::new(ratchet),
    )
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn wrong_arguments_are_an_error() {
        assert!(check(&["a".to_owned()]).is_err());
    }
}
