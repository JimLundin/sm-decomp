//! Play the scheduled flight from the station blackout using the indexed production recipe.

use super::{Report, Stop, rows::Rows};
use crate::replay::{data_failure, result::Outcome, zebes};
use oracle_record::FrameRecord;
use std::path::Path;

const SEED: &[&str] =
    &["station blackout, carried health, landing destination, palette and sprite buffer"];
const BORROWED: &[&str] = &["work completion (T1)"];
const MASKED: &[&str] = &[
    "partial setup writes during lag (E2 and T1)",
    "off-screen OAM retention and composited pixels",
    "Landing Site room initialization (R2)",
];

pub(super) fn play(
    rows: &mut Rows<'_>,
    rom: &Path,
    mut previous: FrameRecord,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let (mut production, _, _) = zebes::admit_seed(rom, &previous)?;
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: SEED,
        borrowed: BORROWED,
        masked: MASKED,
    };
    let mut held = 0u64;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "played flight row join"));
        }
        if let Err(outcome) = zebes::play_cut(&mut production, &previous, &current, cut) {
            return Ok(report(held, Stop::At(outcome)));
        }
        held = held.saturating_add(1);
        previous = current;
    }
    Ok(report(held, Stop::End))
}
