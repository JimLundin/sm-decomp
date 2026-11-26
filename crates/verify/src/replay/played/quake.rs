//! The room's quake: its seed and compared fields.
//!
//! The quake's State is decoded once from the seed record and carried. At every cut whose frame
//! of work finished the run compares the shake's frames left, the phase and, while a shake is
//! under way, its kind. On a frame of play with the room's update (the last cut in main
//! gameplay or its fade in, and both cuts finished) it compares the layers' offsets and, where
//! the record shows the update asked the actors to shake, that every Ceres door was asked.
//!
//! The record has no offset of its own: an offset is what the frame added to a layer's scroll
//! past where its owner placed the layer. On an axis whose back layer the room holds still, the
//! game writes no place, so the offset is the scroll's change since the last cut. A slot's shake
//! timer reads 2 for a frame after its request too, so where the record shows no request none
//! is compared.

use super::platform::finished;
use super::word;
use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::source::{self, WramByte, WramWord};
use game::production::played;
use oracle_record::FrameRecord;
use world_quake::{
    Effects, Offset, QuakeDirection, QuakeKind, QuakeReach, QuakeSize, Shake, State,
};

/// The shake's kind, read while frames of it are left.
const KIND: WramWord = WramWord {
    name: "earthquake_type",
    offset: 0x183E,
};
/// The frames of shake left.
const REMAINING: WramWord = WramWord {
    name: "earthquake_timer",
    offset: 0x1840,
};
/// The room's updates, counted round.
const PHASE: WramWord = WramWord {
    name: "earthquake_phase",
    offset: 0x1842,
};

/// Kind codes count the reaches by 9, the sizes by 3 and the directions by 1; 36 kinds in all.
const REACH_STEP: u16 = 9;
const SIZE_STEP: u16 = 3;
const KINDS: u16 = 36;
/// The first kind code whose shake reaches the actors.
const REACHES_ACTORS: u16 = 0x12;

/// The game states whose frames run the room's update: main gameplay, and its fade in.
const PLAY_FADING_IN: u16 = 7;
const PLAY: u16 = 8;

/// An enemy's shake timer as the update's request leaves it.
const ASKED: u16 = 2;

/// The scroll rate at which the room holds its back layer still on an axis.
const HELD: u8 = 1;

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.into(),
    }
}

fn signed(wram: &[u8], at: WramWord) -> Result<i16, Outcome> {
    Ok(word(wram, at)?.cast_signed())
}

fn byte(wram: &[u8], at: WramByte) -> Result<u8, Outcome> {
    at.read(wram)
        .map_err(|_| crate::replay::data_failure(None, at.name))
}

/// The quake as the record holds it. A kind code past the 36 is no kind the quake has: refused.
pub(super) fn seed(wram: &[u8]) -> Result<State, Outcome> {
    let phase = world_quake::QuakePhase(word(wram, PHASE)?);
    let shake = match core::num::NonZeroU16::new(word(wram, REMAINING)?) {
        None => None,
        Some(remaining) => Some(Shake {
            kind: kind(word(wram, KIND)?)?,
            remaining,
        }),
    };
    Ok(State { phase, shake })
}

/// The kind a code names: `9 × reach + 3 × (size − 1) + direction`, the reaches in the order
/// front, scenery, whole, back and the directions sideways, up and down, diagonal.
fn kind(code: u16) -> Result<QuakeKind, Outcome> {
    if code >= KINDS {
        return Err(unsupported("played quake kind outside the 36"));
    }
    let reach = match code / REACH_STEP {
        0 => QuakeReach::Front,
        1 => QuakeReach::Scenery,
        2 => QuakeReach::Whole,
        _ => QuakeReach::Back,
    };
    let direction = match code % SIZE_STEP {
        0 => QuakeDirection::Sideways,
        1 => QuakeDirection::UpAndDown,
        _ => QuakeDirection::Diagonal,
    };
    let pixels = match code % REACH_STEP / SIZE_STEP {
        0 => 1,
        1 => 2,
        _ => 3,
    };
    let size = QuakeSize::new(pixels)
        .ok_or_else(|| crate::replay::data_failure(None, "played quake size"))?;
    Ok(QuakeKind {
        reach,
        size,
        direction,
    })
}

/// Whether the frame from `previous` to `current` ran the room's update: the last cut was in
/// main gameplay or its fade in, and both found their frame of work finished.
fn updated(previous: &FrameRecord, current: &FrameRecord) -> Result<bool, Outcome> {
    let state = word(&previous.wram, source::GAME_STATE)?;
    Ok(finished(previous) && finished(current) && matches!(state, PLAY_FADING_IN | PLAY))
}

/// The quake's fields as the record at `current` holds them, after `previous`, at a cut whose
/// frame of work finished; the front offset not while the getaway draws its picture there.
pub(super) fn expected(
    previous: &FrameRecord,
    current: &FrameRecord,
    flyby_controls_front: bool,
) -> Result<Vec<Observation>, Outcome> {
    let wram = &current.wram;
    let remaining = word(wram, REMAINING)?;
    let mut fields = vec![
        remaining_observation(remaining),
        phase_observation(word(wram, PHASE)?),
    ];
    if remaining != 0 {
        fields.push(kind_observation(Some(kind(word(wram, KIND)?)?)));
    }
    if !updated(previous, current)? {
        return Ok(fields);
    }
    if !flyby_controls_front {
        fields.push(offset_observation("quake.front", Some(front(wram)?)));
    }
    fields.push(offset_observation(
        "quake.back",
        Some(back(&previous.wram, wram)?),
    ));
    // The update asks the actors while a shake reaching them was under way as it began: its
    // frames left at the last cut, or a start this frame left some.
    let before = word(&previous.wram, REMAINING)?;
    if (remaining != 0 || before != 0) && word(wram, KIND)? >= REACHES_ACTORS {
        fields.push(actors_observation(Some(every_door_asked(wram)?)));
    }
    Ok(fields)
}

/// The front layer's offset: its scroll past layer 1's place and BG1's offset.
fn front(wram: &[u8]) -> Result<Offset, Outcome> {
    Ok(Offset {
        x: past(
            wram,
            source::REG_BG_SCROLL[0][0],
            LAYER1_X,
            source::BG1_X_OFFSET,
        )?,
        y: past(
            wram,
            source::REG_BG_SCROLL[0][1],
            LAYER1_Y,
            source::BG1_Y_OFFSET,
        )?,
    })
}

/// The back layer's offset: its scroll past layer 2's place and BG2's offset, or, on an axis the
/// room holds it still, its scroll's change since `previous`.
fn back(previous: &[u8], wram: &[u8]) -> Result<Offset, Outcome> {
    let axis = |scroll: WramWord, rate: WramByte, place: WramWord, offset: WramWord| {
        if byte(wram, rate)? == HELD {
            Ok(signed(wram, scroll)?.wrapping_sub(signed(previous, scroll)?))
        } else {
            past(wram, scroll, place, offset)
        }
    };
    Ok(Offset {
        x: axis(
            source::REG_BG_SCROLL[1][0],
            source::LAYER2_SCROLL_X,
            source::LAYER2_X_POS,
            source::BG2_X_OFFSET,
        )?,
        y: axis(
            source::REG_BG_SCROLL[1][1],
            source::LAYER2_SCROLL_Y,
            source::LAYER2_Y_POS,
            source::BG2_Y_OFFSET,
        )?,
    })
}

/// Layer 1's place, in whole pixels.
const LAYER1_X: WramWord = WramWord {
    name: "layer1_x_pos",
    offset: 0x0911,
};
const LAYER1_Y: WramWord = WramWord {
    name: "layer1_y_pos",
    offset: 0x0915,
};

/// How far `scroll` lies past `place` and `offset`, signed.
fn past(wram: &[u8], scroll: WramWord, place: WramWord, offset: WramWord) -> Result<i16, Outcome> {
    let placed = word(wram, place)?.wrapping_add(word(wram, offset)?);
    Ok(word(wram, scroll)?.wrapping_sub(placed).cast_signed())
}

/// Whether every Ceres door's slot holds the shake the update's request gives it.
fn every_door_asked(wram: &[u8]) -> Result<bool, Outcome> {
    for slot in crate::replay::ceres::doors(wram)? {
        let timer = source::ENEMY_DATA
            .word(source::ENEMY_SHAKE_TIMER, slot.index())
            .and_then(|field| field.read(wram))
            .map_err(|_| crate::replay::data_failure(None, "played quake actors"))?;
        if timer != ASKED {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The quake's fields as the played State and the frame's update hold them: a frame without an
/// update has no offsets and asks nothing.
pub(super) fn actual(state: &played::State, update: Option<Effects>) -> Vec<Observation> {
    let quake = state.quake.0;
    vec![
        remaining_observation(quake.remaining().0),
        phase_observation(quake.phase.0),
        kind_observation(quake.shake.map(|shake| shake.kind)),
        offset_observation("quake.front", update.map(|effects| effects.front)),
        offset_observation("quake.back", update.map(|effects| effects.back)),
        actors_observation(update.map(|effects| effects.actors_asked)),
    ]
}

fn remaining_observation(remaining: u16) -> Observation {
    Observation::quantity("quake.remaining", i64::from(remaining))
}

fn phase_observation(phase: u16) -> Observation {
    Observation::quantity("quake.phase", i64::from(phase))
}

/// A kind as its reach, size and direction; none as nothing.
fn kind_observation(kind: Option<QuakeKind>) -> Observation {
    let bytes = kind.map_or_else(Vec::new, |kind| {
        let reach = match kind.reach {
            QuakeReach::Whole => 0,
            QuakeReach::Back => 1,
            QuakeReach::Front => 2,
            QuakeReach::Scenery => 3,
        };
        let direction = match kind.direction {
            QuakeDirection::Diagonal => 0,
            QuakeDirection::Sideways => 1,
            QuakeDirection::UpAndDown => 2,
        };
        vec![reach, kind.size.pixels(), direction]
    });
    Observation {
        field: "quake.kind".into(),
        bytes,
    }
}

/// An offset as x then y; none, from a frame without an update, as nothing.
fn offset_observation(field: &str, offset: Option<Offset>) -> Observation {
    Observation {
        field: field.into(),
        bytes: offset.map_or_else(Vec::new, |offset| {
            [offset.x.to_le_bytes(), offset.y.to_le_bytes()].concat()
        }),
    }
}

/// Whether the actors were asked to shake; none, from a frame without an update, as nothing.
fn actors_observation(asked: Option<bool>) -> Observation {
    Observation {
        field: "quake.actors".into(),
        bytes: asked.map_or_else(Vec::new, |asked| vec![u8::from(asked)]),
    }
}

#[cfg(test)]
mod tests {
    use super::kind;
    use world_quake::{QuakeDirection, QuakeReach};

    #[test]
    fn a_kind_code_names_its_reach_size_and_direction_and_none_past_the_36() {
        // The doors' small and strong shakes, and the door of Ridley's room's.
        for (code, reach, size) in [
            (0x14, QuakeReach::Whole, 1),
            (0x1A, QuakeReach::Whole, 3),
            (0x1D, QuakeReach::Back, 1),
            (0x23, QuakeReach::Back, 3),
        ] {
            let decoded = kind(code).unwrap();
            assert_eq!(decoded.reach, reach);
            assert_eq!(decoded.size.pixels(), size);
            assert_eq!(decoded.direction, QuakeDirection::Diagonal);
        }
        assert_eq!(kind(0).unwrap().reach, QuakeReach::Front);
        assert_eq!(kind(0).unwrap().direction, QuakeDirection::Sideways);
        assert_eq!(kind(0x0A).unwrap().reach, QuakeReach::Scenery);
        assert_eq!(kind(0x0A).unwrap().direction, QuakeDirection::UpAndDown);
        assert!(kind(0x24).is_err());
    }
}
