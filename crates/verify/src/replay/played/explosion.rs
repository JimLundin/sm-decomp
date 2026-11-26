//! Play both departing cinematics from the single elevator blackout.

mod carry;

use super::{Report, Stop, rows::Rows};
use crate::replay::{compared, data_failure, explosion, result::Outcome, zebes};
use game::production::played::departure::{Content, Fault, Output, Run};
use game::production::played::landing_site::{self, Work};
use game::state::owned::OwnedState;
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
    let (production, _, _) = explosion::admit_seed(path, &previous)?;
    let carry = carry::origin(&production, &previous)?;
    let image = std::fs::read(path).map_err(|_| data_failure(None, "departure ROM file"))?;
    let rom = data::rom::Rom::new(image).map_err(|_|data_failure(None, "departure ROM pin"))?;
    let graphics = import::landing_site::import(rom.lorom()).map_err(|_|data_failure(None, "Landing Site standard graphics"))?;
    let content = Content { flight: zebes::content(path)?, landing: graphics };
    let mut run = Run::start(production, content, carry)
        .map_err(|fault| data_failure(Some(start), &format!("departure intake {fault:?}")))?;
    let mut observer = explosion::Observed::default();
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: &[
            "elevator blackout, completed save, Health, destination, caption, shaft room, explored maps, RNG and video memory",
        ],
        borrowed: &["T1 work completion", "post-service music queue reply", "T1 accepted and selected bytes during direct DMA"],
        masked: &[
            "in-progress cinematic setup transfers",
            "flash scanline composition",
        ],
    };
    let mut held = 0_u64;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "departure row join"));
        }
        let timing = explosion::timing(&current)?;
        let landing = matches!(run.scene().state(), OwnedState::Zebes(s) if matches!(s.flow, game::production::played::zebes::Flow::Loading(_)));
        let work = if landing {
            match run.carry().landing.phase {
                landing_site::Phase::Pending => Some(landing_work(&current)?),
                _ => None,
            }
        } else { None };
        let output = match run.advance(timing, work) {
            Ok(output) => output,
            Err(Fault::Timing) => return Ok(report(held, Stop::At(Outcome::Unsupported{cut:Some(cut),dependency:"T1 progress for the unfinished Landing Site sprite transfer".into()}))),
            Err(fault) => return Err(data_failure(Some(cut), &format!("departure advance {fault:?}"))),
        };
        let scene_result = match &output {
            Output::Scene(ProductionOutputs::Explosion(output)) => compared(cut, &observer.read(&previous, &current)?.fields, &explosion::actual(run.scene(), Some(output))?),
            Output::Scene(ProductionOutputs::Zebes(output)) => compared(cut, &zebes::expected(&previous, &current, true)?.fields, &zebes::actual(run.scene(), output, true)?),
            Output::Landing => Ok(()),
            _ => return Err(data_failure(Some(cut), "departure output scope")),
        };
        if let Err(why) = scene_result.and_then(|()|carry::check(&run, &current, timing, cut)) {
            return Ok(report(held, Stop::At(why)));
        }
        held = held.saturating_add(1);
        previous = current;
    }
    Ok(report(held, Stop::End))
}


// The external T1 receipt measures source-bus work. No destination memory or game
// state from the receiving record is supplied to the simulation.
fn landing_work(record: &FrameRecord) -> Result<Work, Outcome> {
    let channel = record.ppu.dma.get(1).ok_or_else(|| data_failure(Some(record.index),"Landing Site timing channel"))?;
    let consumed = 0x4000u16.checked_sub(u16::from_le_bytes([channel[5], channel[6]])).ok_or_else(|| data_failure(Some(record.index),"Landing Site timing length"))?;
    let source = u16::from_le_bytes([channel[2],channel[3]]);
    let ahead = source.wrapping_sub(0xd200).wrapping_sub(consumed);
    if !matches!(record.cut, oracle_record::Cut::VblankWithoutNmi) || ahead > 1 { return Err(data_failure(Some(record.index), "Landing Site timing not at a direct transfer")); }
    Ok(Work::Open { accepted: consumed, selected: ahead == 1 })
}
