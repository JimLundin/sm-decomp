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
    if standard_background_done(current) && !standard_background_done(previous) {
        return Ok(Readiness::StandardBackground);
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
fn standard_background_done(record: &FrameRecord) -> bool {
    let Some([_, bus, from_lo, from_hi, bank, left_lo, left_hi, ..]) = record.ppu.dma.get(1) else {
        return false;
    };
    matches!(record.cut, Cut::VblankWithoutNmi)
        && record.cpu.k == 0x82
        && record.cpu.pc == 0x8327
        && *bus == 0x18
        && *bank == 0x9a
        && u16::from_le_bytes([*from_lo, *from_hi]) >= 0xd200
        && u16::from_le_bytes([*left_lo, *left_hi]) != 0
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
