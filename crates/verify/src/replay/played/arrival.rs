//! Her arrival as a new game begins: its seed, its compared fields and the borrowed readiness of
//! the start.

use super::word;
use crate::replay::compare::Observation;
use crate::replay::{data_failure, result::Outcome};
use crate::source;
use game::production::played;
use oracle_record::{Cut, FrameRecord};
use types::arrival::{ArrivalStage, ArrivalState, FadeFrames};
use types::brightness::{Blanking, Brightness, Level};

/// The screen fade's counter, the frames left before its next step.
const FADE_COUNTER: source::WramWord = source::WramWord {
    name: "SCREEN_FADE_COUNTER",
    offset: 0x0725,
};
/// The game states: the new game's start being made ready, the fade in, play, and the door
/// passage's three.
const PREPARING: u16 = 0x1F;
const FADING_IN: u16 = 0x07;
const PLAYING: core::ops::RangeInclusive<u16> = 0x08..=0x0B;
/// The brightness mirror's blank bit and its level's bits. The bits between drive nothing on
/// the screen, and a door's passage leaves one of them set.
const BLANK: u8 = 0x80;
const LEVEL: u8 = 0x0F;

/// The arrival's stage as the record's game state says it.
fn stage(wram: &[u8]) -> Result<ArrivalStage, Outcome> {
    match word(wram, source::GAME_STATE)? {
        PREPARING => Ok(ArrivalStage::Preparing),
        FADING_IN => Ok(ArrivalStage::FadingIn),
        state if PLAYING.contains(&state) => Ok(ArrivalStage::Playing),
        _ => Err(Outcome::Unsupported {
            cut: None,
            dependency: "played game state outside the arrival and play".into(),
        }),
    }
}

/// The screen's brightness as the record's mirror of it holds it.
fn brightness(wram: &[u8]) -> Result<Brightness, Outcome> {
    let bad = || data_failure(None, "played brightness");
    let mirror = source::REG_INIDISP.read(wram).map_err(|_| bad())?;
    Ok(Brightness {
        level: Level::new(mirror & LEVEL).ok_or_else(bad)?,
        blanking: if mirror & BLANK == 0 {
            Blanking::Shown
        } else {
            Blanking::Blanked
        },
    })
}

/// The arrival as the record holds it.
fn arrival(wram: &[u8]) -> Result<ArrivalState, Outcome> {
    Ok(ArrivalState {
        stage: stage(wram)?,
        brightness: brightness(wram)?,
        count: FadeFrames::new(word(wram, FADE_COUNTER)?)
            .ok_or_else(|| data_failure(None, "played fade count"))?,
    })
}

/// The arrival as a seed in the preparation or in play holds it; a seed in play has had its
/// preparation's frame of work.
pub(super) fn seed(wram: &[u8]) -> Result<played::arrival::State, Outcome> {
    let arrival = arrival(wram)?;
    Ok(played::arrival::State {
        preparation: if arrival.stage == ArrivalStage::Preparing {
            played::arrival::Begun::Not
        } else {
            played::arrival::Begun::UnderWay
        },
        setup: if arrival.stage == ArrivalStage::Preparing {
            played::arrival::Setup::BeforeMusic
        } else {
            played::arrival::Setup::Published
        },
        arrival,
    })
}

/// The borrowed readiness of the start (the start's loading, R2 and C1, with T1's lag): the
/// game state turns from the preparation's to the fade in's between `previous` and `current`.
pub(super) fn start(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<played::arrival::Readiness, Outcome> {
    use played::arrival::Readiness;
    if word(&previous.wram, source::GAME_STATE)? != PREPARING {
        return Ok(Readiness::NotYet);
    }
    if let Some(accepted) = standard_objects_started(current) {
        return Ok(Readiness::StandardBackground { accepted });
    }
    if standard_objects_started(previous).is_some() && standard_objects_finished(current) {
        return Ok(Readiness::StandardObjectsFinished);
    }
    if startup_beam_published(previous, current) {
        return Ok(Readiness::BeamGraphics);
    }
    if room_effects_initialized(current) && !room_effects_initialized(previous) {
        return Ok(Readiness::RoomEffects);
    }
    if enemy_tiles_written(previous, current)?.is_some() {
        return Ok(Readiness::EnemyGraphics);
    }
    if super::finished(current)
        && source::ENEMY_LOAD_PROGRESS.read(&previous.wram) == Ok(0xfffe)
        && source::ENEMY_LOAD_PROGRESS.read(&current.wram) == Ok(0xffff)
    {
        return Ok(Readiness::EnemyActors);
    }
    if word(&current.wram, source::GAME_STATE)? == FADING_IN {
        return Ok(Readiness::Ready);
    }
    Ok(
        if final_music_wait(current) && !final_music_wait(previous) {
            Readiness::FinalMusicWait
        } else {
            Readiness::NotYet
        },
    )
}

// Ceres preparation reaches the sprite DMA after the fixed background transfer.
fn standard_objects_started(record: &FrameRecord) -> Option<u16> {
    let [_, bus, from_lo, from_hi, bank, left_lo, left_hi, ..] = *record.ppu.dma.get(1)?;
    let source = u16::from_le_bytes([from_lo, from_hi]);
    let remaining = u16::from_le_bytes([left_lo, left_hi]);
    let accepted = 0x4000_u16.checked_sub(remaining)?;
    (matches!(record.cut, Cut::VblankWithoutNmi)
        && record.cpu.k == 0x82
        && record.cpu.pc == 0x8327
        && bus == 0x18
        && bank == 0x9a
        && remaining != 0
        && source == 0xd200_u16.checked_add(accepted)?.checked_add(1)?)
    .then_some(accepted)
}

// The tilemap transfers follow the standard object upload in the same loading work.
fn standard_objects_finished(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::VblankWithoutNmi)
        && record
            .ppu
            .dma
            .get(1)
            .is_some_and(|channel| matches!(channel, [1, 0x18, 0, 0x48, 0x7e, 0, 0, ..]))
}

// The startup beam write is the sole entry in the normal VRAM queue while loading.
fn startup_beam_published(previous: &FrameRecord, current: &FrameRecord) -> bool {
    super::finished(current)
        && source::VRAM_WRITE_QUEUE_TAIL.read(&previous.wram) == Ok(7)
        && source::VRAM_WRITE_QUEUE
            .read(&previous.wram)
            .is_ok_and(|queue| {
                queue.get(..2) == Some(&0x100_u16.to_le_bytes())
                    && queue.get(5..7) == Some(&0x6300_u16.to_le_bytes())
            })
}

// T1: enabling NMI for the second music service follows the room effects reset.
fn room_effects_initialized(record: &FrameRecord) -> bool {
    let stack_base = usize::from(record.cpu.s);
    let return_address = |offset: usize, address: u16| {
        let start = stack_base.saturating_add(offset);
        record.wram.get(start..start.saturating_add(2)) == Some(&address.to_le_bytes())
    };
    matches!(record.cut, Cut::NmiReturn(nmi) if nmi.interrupted_pc == 0x0080_835a)
        && return_address(3, 0xa131)
        && return_address(7, 0xa0d4)
}

/// Number of enemy tile words the preparation has published at a requested upload cut.
pub(super) fn enemy_tiles_written(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<Option<usize>, Outcome> {
    const SOURCE: source::WramWord = source::WramWord {
        name: "ENEMY_TILE_SOURCE",
        offset: 0x0e7c,
    };
    const DESTINATION: source::WramWord = source::WramWord {
        name: "ENEMY_TILE_DESTINATION",
        offset: 0x0e7e,
    };
    if !super::finished(current) || word(&previous.wram, source::GAME_STATE)? != PREPARING {
        return Ok(None);
    }
    let next = word(&current.wram, SOURCE)?;
    if !(0x7800..=0x9800).contains(&next) || !next.is_multiple_of(0x800) {
        return Ok(None);
    }
    let prior = if next == 0x7800 {
        0
    } else {
        next.saturating_sub(0x800)
    };
    let words = usize::from(next.saturating_sub(0x7000)) / 2;
    let to = word(&current.wram, DESTINATION)?;
    let reached = to == 0x6c00_u16.saturating_add(u16::try_from(words).unwrap_or_default())
        && word(&previous.wram, SOURCE)? == prior;
    Ok(reached.then_some(words))
}

/// The borrowed loading beat preceding the HUD by twenty waits. The other calls to
/// the generic music service have different return addresses on the native stack.
fn final_music_wait(record: &FrameRecord) -> bool {
    const WAIT_FOR_NMI: core::ops::RangeInclusive<u32> = 0x0080_8343..=0x0080_8346;
    const MUSIC_CALLER: u16 = 0xa119;
    // JSL's return address and bank, the two PHPs and the saved X precede the caller.
    const CALLER_OFFSET: usize = 9;
    let Cut::NmiReturn(nmi) = record.cut else {
        return false;
    };
    let address = usize::from(record.cpu.s).saturating_add(CALLER_OFFSET);
    super::finished(record)
        && WAIT_FOR_NMI.contains(&nmi.interrupted_pc)
        && record.wram.get(address..address.saturating_add(2)) == Some(&MUSIC_CALLER.to_le_bytes())
}

/// The arrival's fields as the record holds them: the stage and the brightness at every cut,
/// and the fade's count once the played preparation is over (`counted`): the preparation
/// writes the count as its own working value.
pub(super) fn expected(wram: &[u8], counted: bool) -> Result<Vec<Observation>, Outcome> {
    Ok(observations(arrival(wram)?, counted))
}

/// The arrival's fields as the played State holds them.
pub(super) fn actual(state: played::arrival::State) -> Vec<Observation> {
    observations(state.arrival, counted(state))
}

/// Whether the fade's count is compared: once the played preparation is over.
pub(super) fn counted(state: played::arrival::State) -> bool {
    !state.preparing()
}

fn observations(arrival: ArrivalState, counted: bool) -> Vec<Observation> {
    let part = |field: &str, value: String| Observation {
        field: field.into(),
        bytes: value.into_bytes(),
    };
    let mut fields = vec![
        part("arrival.stage", format!("{:?}", arrival.stage)),
        part("arrival.brightness", format!("{:?}", arrival.brightness)),
    ];
    if counted {
        fields.push(Observation::quantity(
            "arrival.fade-count",
            i64::from(arrival.count.get()),
        ));
    }
    fields
}

#[cfg(test)]
mod tests {
    use types::brightness::{Blanking, Brightness, Level};

    fn mirror(value: u8) -> Vec<u8> {
        let mut wram = vec![0; 0x100];
        wram[0x51] = value;
        wram
    }

    #[test]
    fn the_brightness_mirror_reads_as_a_level_and_whether_the_screen_is_blanked() {
        let read = |value| super::brightness(&mirror(value)).ok();
        assert_eq!(
            read(0x80),
            Some(Brightness {
                level: Level::BLACK,
                blanking: Blanking::Blanked
            })
        );
        assert_eq!(read(0x0F), Some(Brightness::FULLY_UP));
        assert_eq!(
            read(0x8F),
            Some(Brightness {
                level: Level::FULL,
                blanking: Blanking::Blanked
            })
        );
        assert_eq!(
            read(0x1F),
            Some(Brightness::FULLY_UP),
            "the bits between drive nothing"
        );
    }
}
