//! The station explosion the played run reads: record projections, the seed State and drawing.

mod drawing;
mod projections;

use super::data_failure;
use super::result::Outcome;
use game::production::played::explosion::{Cut, Destination, State};
use game::state::video::{Colour, Oam, OamEntry};
use oracle_record::{Cut as RecordCut, FrameRecord, NmiKind};

pub(super) use drawing::actual;
pub(super) use projections::Observed;

const STAGE: usize = 0x1f51;
const FINISHED: usize = 0x05b6;
const FLOW: usize = 0x0998;
const DESTINATION: usize = 0xd914;
const PENDING: u16 = 0xc11b;

pub(super) fn byte(record: &FrameRecord, at: usize) -> Result<u8, Outcome> {
    record
        .wram
        .get(at)
        .copied()
        .ok_or_else(|| data_failure(Some(record.index), "station record byte"))
}
pub(super) fn word(record: &FrameRecord, at: usize) -> Result<u16, Outcome> {
    Ok(u16::from_le_bytes([
        byte(record, at)?,
        byte(record, at.saturating_add(1))?,
    ]))
}

pub(super) fn pending(seed: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(seed, FLOW)? == 0x22
        && word(seed, STAGE)? == PENDING
        && word(seed, DESTINATION)? == 0x22
        && byte(seed, 0x51)? & 0x80 != 0)
}

pub(super) fn enters_from(before: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(before, STAGE)? != PENDING)
}

pub(super) fn seed_state(seed: &FrameRecord) -> Result<State, Outcome> {
    let cols: &[u16; 256] = seed
        .cgram
        .as_ref()
        .try_into()
        .map_err(|_| data_failure(Some(seed.index), "station seed palette"))?;
    let mut oam = Oam::default();
    let (low, high) = seed
        .oam
        .split_at_checked(512)
        .ok_or_else(|| data_failure(Some(seed.index), "station seed sprite entries"))?;
    oam.high = high
        .try_into()
        .map_err(|_| data_failure(Some(seed.index), "station high sprite entries"))?;
    for (entry, &[x, y, tile, attributes]) in oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    Ok(State::start(
        word(seed, FINISHED)?,
        super::health_tail::decode(&seed.wram, Some(seed.index))?,
        word(seed, 0x09c4)?,
        match word(seed, DESTINATION)? {
            0x22 => Destination::LandingSite,
            0x1f => Destination::CeresStart,
            _ => return Err(data_failure(Some(seed.index), "station entry destination")),
        },
        word(seed, 0x09e2)? != 0,
        cols.map(Colour),
        oam,
    ))
}

pub(super) fn timing(record: &FrameRecord) -> Result<Cut, Outcome> {
    if !matches!(record.cut,RecordCut::NmiReturn(nmi) if nmi.kind==NmiKind::Requested) {
        return Ok(Cut::Unfinished);
    }
    let mut busy = false;
    for timer in (0x0629..0x0639).step_by(2) {
        busy |= word(record, timer)? != 0;
    }
    Ok(Cut::Finished { music_busy: busy })
}
