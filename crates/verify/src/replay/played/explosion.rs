//! Play the station blackout from one elevator intake using its indexed production entry.

use super::{Report, Stop, rows::Rows};
use crate::replay::{compared, data_failure, explosion, result::Outcome};
use game::production::ProductionOutputs;
use oracle_record::FrameRecord;
use std::path::Path;

pub(super) fn play(
    rows: &mut Rows<'_>,
    path: &Path,
    mut previous: FrameRecord,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let (mut production, _, _) = explosion::admit_seed(path, &previous)?;
    let mut observer = explosion::Observed::default();
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: &[
            "elevator blackout, completed save, health, destination, caption, palette and OAM",
        ],
        borrowed: &["T1 work completion", "post-service music queue reply"],
        masked: &[
            "partial setup writes",
            "flash scanline composition and off-screen sprite entries",
        ],
    };
    let mut held = 0_u64;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "station row join"));
        }
        let result = explosion::advance(&mut production, explosion::timing(&current)?, cut);
        let output = match result {
            Ok(ProductionOutputs::Explosion(output)) => output,
            Err(why) => return Ok(report(held, Stop::At(why))),
            _ => return Err(data_failure(Some(cut), "station output scope")),
        };
        let actual = explosion::actual(&production, Some(&output))?;
        let expected = observer.read(&previous, &current)?;
        if let Err(why) = compared(cut, &expected.fields, &actual) {
            return Ok(report(held, Stop::At(why)));
        }
        held = held.saturating_add(1);
        previous = current;
    }
    Ok(report(held, Stop::End))
}
