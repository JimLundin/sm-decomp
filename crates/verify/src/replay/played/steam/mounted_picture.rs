//! The mounted puff's scoped OAM encoder, with native actor and insertion inputs.

#[cfg(test)]
use super::super::{open_rows, read_lock, tests::sources};
use super::{decode_shaft, slots};
use crate::replay::blackout::word;
#[cfg(test)]
use crate::replay::blackout::{observe::oam_bytes, seed, standing::packed_oam};
use crate::replay::{Outcome, data_failure};
use crate::source::{ENEMY_DATA, ENEMY_FRAME_COUNTER, ENEMY_SHAKE_TIMER, SlotField};
use boss_ceres_ridley::{PuffFigure, VentFacing};
#[cfg(test)]
use data::rom::Rom;
#[cfg(test)]
use game::production::played::shaft::vent;
#[cfg(test)]
use game::state::video::{SpriteCount, Video};
use oracle_record::FrameRecord;
#[cfg(test)]
use oracle_record::{Cut, NmiKind};
use types::Pair;

#[cfg(test)]
const ROW: &str = "chain-any-0011-DF45";
#[cfg(test)]
const OAM: std::ops::Range<usize> = 0x0370..0x0590;
const PREFIX_SLOTS: u16 = 24;
#[cfg(test)]
const VENT_SLOTS: std::ops::Range<usize> = 24..128;

#[test]
#[ignore = "pinned record cache and player ROM"]
fn mounted_vents_preserve_the_hidden_sprite_suffix() -> Result<(), Outcome> {
    let sources = sources();
    let lock = read_lock(&sources)?;
    let mut rows = open_rows(&sources, &lock, ROW)?;
    let image = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let rom = image.lorom();
    let art = import::shaft::vent::import(rom).unwrap();
    let format = game::exact::save_files::Format::load(rom).unwrap();
    let mut previous = rows.record(164)?;
    let mut video = seed::read(&previous, rom, &format, false)?.video;
    for cut in 165..=241 {
        let current = rows.record(cut)?;
        if cut <= 239 {
            video.batch.sprites = SpriteCount(PREFIX_SLOTS);
            video.batch.clear_high_oam();
            for (facing, figure, origin) in observed_draws(&previous, &current)? {
                vent::append(&mut video.batch, &art, facing, figure, origin);
            }
        }
        if matches!(current.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested) {
            video.batch.finalise_oam();
            video.upload_oam_and_cgram();
        }
        compare_suffix(&current, &video)?;
        previous = current;
    }
    super::super::tests::diagnostic(format_args!(
        "MOUNTED_VENT_OAM shaft=165..241 cuts=77 slots=24..127"
    ));
    Ok(())
}

fn observed_draws(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<Vec<(VentFacing, PuffFigure, Pair<i32>)>, Outcome> {
    let bad = || data_failure(Some(current.index), "mounted vent draw inputs");
    let read = |frame: &FrameRecord, slot, field: SlotField| {
        ENEMY_DATA
            .word(field, slot)
            .and_then(|field| field.read(&frame.wram))
            .map_err(|_| bad())
    };
    let camera = Pair {
        x: word(&current.wram, 0x911)?,
        y: word(&current.wram, 0x915)?,
    };
    let mut draws = Vec::new();
    for (slot, vent) in slots(&current.wram)?
        .into_iter()
        .zip(decode_shaft(&current.wram)?)
    {
        let Some(figure) = vent.shown_puff_figure() else {
            continue;
        };
        let offset = 0x7010_usize.wrapping_add(usize::from(slot).wrapping_mul(64));
        let shake = if read(previous, slot, ENEMY_SHAKE_TIMER)? == 0 {
            0
        } else if read(current, slot, ENEMY_FRAME_COUNTER)? & 2 == 0 {
            1
        } else {
            -1
        };
        let x = vent
            .position
            .x
            .0
            .wrapping_sub(camera.x)
            .wrapping_add(word(&current.wram, offset)?)
            .wrapping_add_signed(shake)
            .cast_signed();
        let y = vent
            .position
            .y
            .0
            .wrapping_sub(camera.y)
            .wrapping_add(word(&current.wram, offset.wrapping_add(2))?)
            .cast_signed();
        if (-128..384).contains(&x) && (-128..384).contains(&y) {
            draws.push((
                vent.facing,
                figure,
                Pair {
                    x: i32::from(x),
                    y: i32::from(y),
                },
            ));
        }
    }
    Ok(draws)
}

#[cfg(test)]
fn compare_suffix(record: &FrameRecord, video: &Video) -> Result<(), Outcome> {
    compare_range(record, video, VENT_SLOTS)
}

#[cfg(test)]
fn compare_range(
    record: &FrameRecord,
    video: &Video,
    slots: std::ops::Range<usize>,
) -> Result<(), Outcome> {
    let bad = || data_failure(Some(record.index), "mounted vent OAM buffer");
    for (field, expected, actual) in [
        (
            "mounted-vent.pending-oam",
            record.wram.get(OAM).ok_or_else(bad)?,
            &video.batch.oam,
        ),
        (
            "mounted-vent.published-oam",
            &record.oam[..],
            &video.memory.oam,
        ),
    ] {
        if packed_oam(expected, slots.clone()).ok_or_else(bad)?
            != packed_oam(&oam_bytes(actual), slots.clone()).ok_or_else(bad)?
        {
            return Err(Outcome::Difference {
                cut: record.index,
                field: field.into(),
            });
        }
    }
    Ok(())
}

/// Compare all slots this producer has written while preserving the rest of the earlier batch.
#[cfg(test)]
pub(in crate::replay::played) fn compare_live_written(
    previous: &FrameRecord,
    current: &FrameRecord,
    video: &Video,
    art: &data::assets::shaft::VentPictures,
    retained: &mut usize,
) -> Result<(), Outcome> {
    *retained = (*retained).max(live_end(previous, current, art)?);
    compare_range(current, video, usize::from(PREFIX_SLOTS)..*retained)
}

pub(in crate::replay::played) fn live_end(
    previous: &FrameRecord,
    current: &FrameRecord,
    art: &data::assets::shaft::VentPictures,
) -> Result<usize, Outcome> {
    let observed = observed_draws(previous, current)?;
    let written = observed
        .iter()
        .map(|(facing, figure, _)| {
            let maps = match facing {
                VentFacing::Up => &art.up,
                VentFacing::Down => &art.down,
                VentFacing::Left => &art.left,
                VentFacing::Right => &art.right,
            };
            maps.get(usize::from(figure.get().saturating_sub(1)))
                .map(|map| map.entries.len())
                .ok_or_else(|| data_failure(Some(current.index), "mounted vent figure"))
        })
        .try_fold(0_usize, |total, count| {
            count.map(|count| total.saturating_add(count))
        })?;
    Ok(usize::from(PREFIX_SLOTS).saturating_add(written))
}
