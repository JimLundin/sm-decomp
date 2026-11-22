//! Independently recorded scoped shot inputs, observations and production admission.

mod indexed;
pub(super) use indexed::{Expectations, Inputs, admit as admit_indexed, initial};

use super::{
    Sources, camera, ceres, data_failure, frame_origin::prelude_seed, movement, result::Outcome,
    weapons,
};
use boss_ceres_ridley::Phase;
use game::production::frame::{
    borrowed_player_preparation::{InputBindings, PhysicalButtons},
    random::RandomSeed,
    shot_opportunity::{Completion, Continuation, DrawOp, DrawStore, Environment, Origin},
};
use game::state::video::{NmiBatch, Oam, OamEntry, SpriteCount};
use serde::Deserialize;
use weapons_projectiles as beam;

pub(super) const PACKET: &str = "shot-contact-five-v1";
pub(super) const INPUT_PIN: &str =
    "43ac8e8db9480be6a750665511bb9b8a7d360497938bf60d870a0ae906d22fe7";
pub(super) const EXPECTED_PIN: &str =
    "3d02455e827f3ac8c5a6e41f3197a668f7e7f639254e521788c012da28c8e17c";
pub(super) const START_PIN: &str =
    "90d416d38b097c80bfb3cf7b0f0e95ae6514abc65788f708a603ec01c964140b";

#[derive(Deserialize)]
struct Input {
    row: String,
    admission_cut: u64,
    initial_oam_and_cursor_sha256: String,
    opportunity: Vec<Opportunity>,
}

#[derive(Deserialize)]
struct Opportunity {
    cut: u64,
    physical_pad: u16,
    presentation_odd: bool,
    before: Vec<(usize, u8, u8)>,
}

#[derive(Deserialize)]
struct Expected {
    opening: Opening,
    oam: Vec<ExpectedOam>,
    opportunity: Vec<Current>,
    shot: Vec<Shot>,
}

#[derive(Deserialize)]
struct ExpectedOam {
    cut: u64,
    pre_power_sha256: String,
    final_backing_and_cursor_sha256: String,
}

#[derive(Deserialize)]
struct Opening {
    cut: u64,
    slot: u8,
    after_wait: u16,
}

#[derive(Deserialize)]
struct Current {
    cut: u64,
    #[serde(rename = "current_pose_id")]
    pose_id: u16,
    prior_pose_id: u16,
    inherited: [u16; 9],
    camera_before: [u16; 2],
    camera_presentation: [u16; 2],
    controller_held: u16,
    controller_new: u16,
    previous_new: u16,
    door_timers_after: [u16; 2],
    door_opened: bool,
}

#[derive(Deserialize)]
struct Shot {
    #[serde(flatten)]
    drawing: super::shot_drawing::ExpectedCut,
    count: u16,
    cooldown: u16,
    slots: Vec<Slot>,
}

#[derive(Deserialize)]
struct Slot {
    #[serde(rename = "slot")]
    index: u8,
    x: u16,
    y: u16,
    x_fraction: u16,
    y_fraction: u16,
    x_speed: i16,
    y_speed: i16,
    x_radius: u8,
    y_radius: u8,
    heading_and_consumed: u16,
    damage: u16,
    hold: u16,
    trail_countdown: u16,
    next_clip_frame: u16,
    picture: u16,
}

fn word(wram: &[u8], offset: usize) -> Result<u16, Outcome> {
    wram.get(offset..offset.saturating_add(2))
        .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(None, "initial shot word"))
}

fn admit(wram: &[u8], power: &weapons::Content) -> Result<Origin, Outcome> {
    let actor0 = ceres::Slot::try_from(0)?;
    let actor1 = ceres::Slot::try_from(1)?;
    let weapon = weapons::ordinary_input(wram, power)?;
    let control = |offset| word(wram, offset).map(PhysicalButtons);
    let player = movement::seed(wram)?;
    Ok(Origin {
        shots: weapons::seed(wram, power)?,
        pose: player.pose,
        prior_pose: data::assets::samus::map::key_of(data::pose::PoseId(word(wram, 0x0a24)?))
            .ok_or_else(|| data_failure(None, "prior shot pose"))?,
        horizontal_speed: player.run.speed,
        previous_pad: [word(wram, 0x0dfe)?, word(wram, 0x0e00)?],
        bindings: InputBindings {
            shoot: control(0x09b2)?,
            jump: control(0x09b4)?,
            run: control(0x09b6)?,
            cancel: control(0x09b8)?,
            aim_up: control(0x09bc)?,
            aim_down: control(0x09be)?,
        },
        weapon: weapon.update,
        room: weapon.world,
        doors: [ceres::seed(wram, actor0)?, ceres::seed(wram, actor1)?],
        contacts: weapon
            .opportunities
            .try_into()
            .map_err(|_| data_failure(None, "current ordinary door count"))?,
        roster: prelude_seed::actors(wram)
            .map_err(|_| data_failure(None, "current ordinary actor roster"))?,
        door_inputs: ceres::input(wram, actor1)?,
        random: RandomSeed(word(wram, 0x05e5)?),
    })
}

const fn wait(phase: Phase) -> u16 {
    match phase {
        Phase::Open { wait }
        | Phase::Closed { wait }
        | Phase::Closing { wait, .. }
        | Phase::Opening { wait, .. }
        | Phase::Starting { wait } => wait.frames(),
        Phase::Placed | Phase::Wall => 1,
    }
}

const OAM_BYTES: usize = 544;

fn batch_from_bytes(bytes: &[u8]) -> Result<NmiBatch, Outcome> {
    let bad = || data_failure(None, "initial shot drawing backing");
    let (tables, cursor) = bytes.split_at_checked(OAM_BYTES).ok_or_else(bad)?;
    let cursor = u16::from_le_bytes(cursor.try_into().map_err(|_| bad())?);
    if !cursor.is_multiple_of(4) || cursor > 512 {
        return Err(bad());
    }
    let (low, high) = tables.split_at_checked(512).ok_or_else(bad)?;
    let mut batch = NmiBatch {
        sprites: SpriteCount(cursor / 4),
        ..NmiBatch::default()
    };
    for (entry, &[x, y, tile, attributes]) in batch.oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    batch.oam.high.copy_from_slice(high);
    Ok(batch)
}

fn oam_bytes(oam: &Oam) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(OAM_BYTES + 2);
    for entry in &oam.low {
        bytes.extend([entry.x, entry.y, entry.tile, entry.attributes]);
    }
    bytes.extend(oam.high);
    bytes
}

impl Opportunity {
    fn draw_stores(&self) -> Result<Vec<DrawStore>, Outcome> {
        self.before
            .iter()
            .map(|&(offset, kind, operand)| {
                let operation = match kind {
                    0 => DrawOp::Set(operand),
                    1 => DrawOp::Or(operand),
                    _ => return Err(data_failure(Some(self.cut), "external OAM store operation")),
                };
                Ok(DrawStore { offset, operation })
            })
            .collect()
    }

    const fn phase(&self) -> beam::FlickerPhase {
        if self.presentation_odd {
            beam::FlickerPhase::Odd
        } else {
            beam::FlickerPhase::Even
        }
    }
}
