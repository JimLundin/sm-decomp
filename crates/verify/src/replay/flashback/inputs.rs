//! Recorded scene timing and after-motion room play. A missile consumed by the encounter is
//! still supplied as in-flight: its position comes from the preceding flight or ammo debit.

use super::{data_failure, word};
use crate::replay::result::Outcome;
use core::num::Wrapping;
use game::production::{
    AdvanceInput, Production, ProductionOutputs,
    played::flashback::{
        Cut, Phase, ReturnDissolve, RoomBackgroundY, RoomPlay, ShotCandidate, ShotId, ShotKind,
    },
};
use oracle_record::FrameRecord;
use types::{Pair, Parity, Px};

const CINEMATIC_FRAME: usize = 0x1a51;
const FRAME_COUNT: usize = 0x05b6;
const RETURN_DISSOLVE: usize = 0x1a4b;
const MISSILES: usize = 0x09c6;
const SAMUS_X: usize = 0x0af6;
const SAMUS_POSE: usize = 0x0a1c;
const SHOT_TYPE: usize = 0x0c18;
const SHOT_KIND: u16 = 0x0fff;
const SHOT_X: usize = 0x0b64;
const SHOT_SUB_X: usize = 0x0b8c;
const SHOT_VELOCITY: usize = 0x0bdc;
const SHOT_HEADING: usize = 0x0c04;
const SHOT_DAMAGE: usize = 0x0c2c;
const MISSILE: u16 = 0x0100;

#[derive(Clone, Copy)]
enum Permission {
    Start,
    Actor,
    Held,
}

#[derive(Clone, Copy)]
pub(super) struct Timing {
    permission: Permission,
    pub(super) finished: bool,
}

impl Timing {
    pub(super) const fn takes_turn(self) -> bool {
        matches!(self.permission, Permission::Actor)
    }

    pub(super) const fn starting(self) -> bool {
        matches!(self.permission, Permission::Start)
    }
}

pub(super) fn timing(
    previous: &FrameRecord,
    current: &FrameRecord,
    cut: u64,
) -> Result<Timing, Outcome> {
    if cut == 0 {
        return Ok(Timing {
            permission: Permission::Start,
            finished: false,
        });
    }
    let actor = word(current, CINEMATIC_FRAME) != word(previous, CINEMATIC_FRAME);
    let finished = super::super::played::platform::finished(current);
    if finished != (word(current, FRAME_COUNT) != word(previous, FRAME_COUNT))
        || (!finished && actor && word(current, 0x1b5d) == word(previous, 0x1b5d))
    {
        return Err(data_failure(Some(cut), "flashback actor phase alignment"));
    }
    Ok(Timing {
        permission: if actor {
            Permission::Actor
        } else {
            Permission::Held
        },
        finished,
    })
}

enum Work {
    Start(RoomBackgroundY),
    Turn {
        parity: Parity,
        dissolve: ReturnDissolve,
        play: RoomPlay,
    },
    Held,
}

struct Input {
    work: Work,
    camera: Option<Pair<Px>>,
}

pub(in crate::replay) struct Inputs {
    background_y: RoomBackgroundY,
    tape: Vec<Input>,
}

impl Inputs {
    pub(super) fn new(handoff: &FrameRecord) -> Result<Self, Outcome> {
        if word(handoff, 0x1f51) != 0xaeb8 || word(handoff, 0x008f) == 0 {
            return Err(data_failure(Some(0), "flashback start press"));
        }
        Ok(Self {
            background_y: RoomBackgroundY(word(handoff, 0x1997).cast_signed()),
            tape: Vec::new(),
        })
    }

    pub(super) fn read(
        &mut self,
        previous_turn: &FrameRecord,
        current: &FrameRecord,
        timing: Timing,
        cut: u64,
    ) -> Result<(), Outcome> {
        let work = match timing.permission {
            Permission::Start => Work::Start(self.background_y),
            Permission::Actor => Work::Turn {
                parity: if word(current, CINEMATIC_FRAME) & 1 == 0 {
                    Parity::Even
                } else {
                    Parity::Odd
                },
                dissolve: if word(current, RETURN_DISSOLVE) == 0 {
                    ReturnDissolve::ReachesZero
                } else {
                    ReturnDissolve::Continuing
                },
                play: RoomPlay {
                    samus_x: Wrapping(word(current, SAMUS_X)),
                    shots: missiles(previous_turn, current, cut)?,
                },
            },
            Permission::Held => Work::Held,
        };
        let camera = timing.finished.then(|| Pair {
            x: Wrapping(word(current, 0x0911)),
            y: Wrapping(word(current, 0x0915)),
        });
        self.tape.push(Input { work, camera });
        Ok(())
    }

    pub(in crate::replay) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let input = usize::try_from(cut)
            .ok()
            .and_then(|cut| self.tape.get(cut))
            .ok_or_else(|| data_failure(Some(cut), "flashback input cut"))?;
        let phase = match &input.work {
            Work::Start(y) => Phase::Start(*y),
            Work::Held => Phase::Held,
            Work::Turn {
                parity,
                dissolve,
                play,
            } => Phase::Turn {
                parity: *parity,
                dissolve: *dissolve,
                play,
            },
        };
        production
            .advance(AdvanceInput::Flashback(Cut {
                phase,
                drawing: input.camera,
            }))
            .map_err(|fault| super::super::production_failure(cut, fault))
    }
}

fn centre(record: &FrameRecord, slot: usize) -> u32 {
    (u32::from(word(record, SHOT_X.saturating_add(slot))) << 16)
        | u32::from(word(record, SHOT_SUB_X.saturating_add(slot)))
}

fn missiles(
    previous: &FrameRecord,
    current: &FrameRecord,
    cut: u64,
) -> Result<Vec<ShotCandidate>, Outcome> {
    let bad = || data_failure(Some(cut), "flashback missile candidate alignment");
    let debit = word(previous, MISSILES)
        .checked_sub(word(current, MISSILES))
        .ok_or_else(bad)?;
    let born = match debit {
        0 => None,
        1 if word(previous, SAMUS_POSE) == 2 => Some(
            (0..5_u8)
                .find(|&slot| {
                    word(
                        previous,
                        SHOT_DAMAGE.saturating_add(usize::from(slot).saturating_mul(2)),
                    ) == 0
                })
                .ok_or_else(bad)?,
        ),
        _ => return Err(bad()),
    };
    let mut shots = Vec::new();
    for id in (0..5_u8).rev() {
        let slot = usize::from(id).saturating_mul(2);
        let (x, velocity) = if born == Some(id) {
            (
                u32::from(word(previous, SAMUS_X).wrapping_sub(11)) << 16,
                -256_i32,
            )
        } else if word(previous, SHOT_TYPE.saturating_add(slot)) & SHOT_KIND == MISSILE {
            if word(previous, SHOT_HEADING.saturating_add(slot)) & 15 != 7 {
                return Err(bad());
            }
            (
                centre(previous, slot),
                i32::from(word(previous, SHOT_VELOCITY.saturating_add(slot)).cast_signed())
                    .wrapping_sub(80),
            )
        } else {
            continue;
        };
        let next_x = x.wrapping_add_signed(velocity << 8);
        if next_x != centre(current, slot) {
            return Err(bad());
        }
        let [_, _, lo, hi] = next_x.to_le_bytes();
        shots.push(ShotCandidate {
            id: ShotId(id),
            kind: ShotKind::Missile,
            x: Wrapping(u16::from_le_bytes([lo, hi])),
        });
    }
    Ok(shots)
}
