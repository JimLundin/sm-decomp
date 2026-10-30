//! Mother Brain's neck: two swinging joints that bob, lower and raise the
//! brain, five segments placed along them each frame, and the third phase's
//! recoil from Samus's shots and the hyper beam.
//!
//! Was `MotherBrain_HandleNeck*`, `MotherBrain_SetupNeckForFakeAscent`,
//! `MotherBrain_RetractHead` and `MotherBrain_Phase3_Neck_*` in the original
//! decompilation (bank $A9). The body's and the brain's views name their slot
//! words through `mother_brain`'s views.
#![forbid(unsafe_code)]
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What the neck needs beyond the shared boss effects: the PPU's hardware
/// multiply, which the original drives through the Mode 7 registers.
pub trait Host: crate::actors::host::BossEffects {
    /// `length` times the sine of `angle`. Was `ComputeSinMult`.
    fn sine_times(&mut self, length: u16, angle: u16) -> u16;
    /// `length` times the cosine of `angle`. Was `ComputeCosMult`.
    fn cosine_times(&mut self, length: u16, angle: u16) -> u16;
}

use crate::actors::mother_brain::{
    BODY, BRAIN, MotherBrainBody as Body, MotherBrainBrain as Brain,
};

/// How a joint moves, in `lower_motion` and `upper_motion`.
pub mod motion {
    pub const STILL: u16 = 0;
    pub const BOB_DOWN: u16 = 2;
    pub const BOB_UP: u16 = 4;
    pub const LOWER: u16 = 6;
    pub const RAISE: u16 = 8;
}

/// Bank $A9 phase-3 neck functions, stored in the brain's `neck_function` word.
pub mod function {
    pub const NORMAL: u16 = 0xc330;
    /// Nothing to do: the normal bobbing is already set up.
    pub const IDLE: u16 = 0xc353;
    pub const SETUP_RECOIL_RECOVERY: u16 = 0xc354;
    pub const RECOIL_RECOVERY: u16 = 0xc37b;
    pub const SETUP_HYPER_BEAM_RECOIL: u16 = 0xc395;
    pub const HYPER_BEAM_RECOIL: u16 = 0xc3cd;
}

/// Bank $A9 brain instruction lists.
pub mod animation {
    /// The brain recovering from a recoil.
    pub const RECOVER: u16 = 0x9dbb;
    /// The brain struck by the hyper beam.
    pub const HYPER_BEAM_HIT: u16 = 0x9be7;
}

/// Where the neck starts relative to the body.
const NECK_BASE_X_OFFSET: u16 = 80;
const NECK_BASE_Y_OFFSET: u16 = 46;
/// Where the lower joint's segments sit relative to the neck base.
const SEGMENT_X_OFFSET: i32 = 112;
const SEGMENT_Y_OFFSET: i32 = 96;
/// Angle limits: the lower joint bobs between these, the upper stays above.
const LOWER_BOB_MIN: u16 = 0x2800;
const LOWER_MIN: u16 = 0x3000;
const LOWER_MAX: u16 = 0x9000;
const UPPER_MIN: u16 = 0x2000;
/// How far the upper joint may rise past the lower one.
const UPPER_ABOVE_LOWER: u16 = 0x800;
/// The brain stops bobbing up this high.
const BRAIN_TOP_Y: u16 = 60;
/// The upper joint bobs down while the brain is this far above Samus.
const BRAIN_SAMUS_MARGIN: i32 = 4;

/// Where each phase-3 recoil sets the neck speed, and how long it lasts.
const NORMAL_SPEED: u16 = 128;
const RECOIL_SPEED: u16 = 1280;
const RECOIL_FRAMES: u16 = 16;
const HYPER_BEAM_SPEED: u16 = 2304;
const HYPER_BEAM_FRAMES: u16 = 11;
const HYPER_BEAM_FLASH: u16 = 50;
const RETRACT_SPEED: u16 = 80;

/// Starts a brain instruction list. Was `MotherBrain_SetBrainInstrs`.
fn set_brain_instructions<H: Host + ?Sized>(game: &mut H, list: u16) {
    slot!(game, Body[BODY].brain_instructions = list);
    slot!(game, Body[BODY].brain_instruction_timer = 1);
}

/// Moves both joints, then places the five segments along them.
/// Was `MotherBrain_HandleNeck`.
pub fn handle_neck<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Body[BODY].neck_base_x =
            slot!(game, Body[BODY].base.x_pos).wrapping_sub(NECK_BASE_X_OFFSET)
    );
    slot!(
        game,
        Body[BODY].neck_base_y =
            slot!(game, Body[BODY].base.y_pos).wrapping_add(NECK_BASE_Y_OFFSET)
    );
    if slot!(game, Brain[BRAIN].neck_moving) != 0 {
        move_lower_joint(game);
        move_upper_joint(game);
    }
    let mut angle: u16 = slot!(byte game, Brain[BRAIN].lower_angle[1]) as u16;
    slot!(
        game,
        Brain[BRAIN].segment_1_x = (slot!(game, Body[BODY].neck_base_x) as i32
            + {
                let length = slot!(game, Brain[BRAIN].segment_1_length);
                game.sine_times(length, angle)
            } as i32
            + SEGMENT_X_OFFSET) as u16
    );
    slot!(
        game,
        Brain[BRAIN].segment_1_y = (slot!(game, Body[BODY].neck_base_y) as i32 + {
            let length = slot!(game, Brain[BRAIN].segment_1_length);
            game.cosine_times(length, angle)
        } as i32
            - SEGMENT_Y_OFFSET) as u16
    );
    slot!(
        game,
        Brain[BRAIN].segment_2_x = (slot!(game, Body[BODY].neck_base_x) as i32
            + {
                let length = slot!(game, Brain[BRAIN].segment_2_length);
                game.sine_times(length, angle)
            } as i32
            + SEGMENT_X_OFFSET) as u16
    );
    slot!(
        game,
        Brain[BRAIN].segment_2_y = (slot!(game, Body[BODY].neck_base_y) as i32 + {
            let length = slot!(game, Brain[BRAIN].segment_2_length);
            game.cosine_times(length, angle)
        } as i32
            - SEGMENT_Y_OFFSET) as u16
    );
    slot!(
        game,
        Brain[BRAIN].segment_3_x = (slot!(game, Body[BODY].neck_base_x) as i32
            + {
                let length = slot!(game, Brain[BRAIN].segment_3_length);
                game.sine_times(length, angle)
            } as i32
            + SEGMENT_X_OFFSET) as u16
    );
    slot!(
        game,
        Brain[BRAIN].segment_3_y = (slot!(game, Body[BODY].neck_base_y) as i32 + {
            let length = slot!(game, Brain[BRAIN].segment_3_length);
            game.cosine_times(length, angle)
        } as i32
            - SEGMENT_Y_OFFSET) as u16
    );
    angle = slot!(byte game, Brain[BRAIN].upper_angle[1]) as u16;
    slot!(
        game,
        Brain[BRAIN].segment_4_x = slot!(game, Brain[BRAIN].segment_3_x).wrapping_add({
            let length = slot!(game, Brain[BRAIN].segment_4_length);
            game.sine_times(length, angle)
        })
    );
    slot!(
        game,
        Brain[BRAIN].segment_4_y = slot!(game, Brain[BRAIN].segment_3_y).wrapping_add({
            let length = slot!(game, Brain[BRAIN].segment_4_length);
            game.cosine_times(length, angle)
        })
    );
    slot!(
        game,
        Brain[BRAIN].segment_5_x = slot!(game, Brain[BRAIN].segment_3_x).wrapping_add({
            let length = slot!(game, Brain[BRAIN].segment_5_length);
            game.sine_times(length, angle)
        })
    );
    slot!(
        game,
        Brain[BRAIN].segment_5_y = slot!(game, Brain[BRAIN].segment_3_y).wrapping_add({
            let length = slot!(game, Brain[BRAIN].segment_5_length);
            game.cosine_times(length, angle)
        })
    );
}

/// Moves the lower joint by its motion. Was `MotherBrain_HandleNeckLower`.
fn move_lower_joint<H: Host + ?Sized>(game: &mut H) {
    match slot!(game, Brain[BRAIN].lower_motion) {
        motion::STILL => {}
        motion::BOB_DOWN => {
            let mut angle: i16 = (slot!(game, Brain[BRAIN].lower_angle) as i16)
                .wrapping_sub(slot!(game, Brain[BRAIN].neck_speed) as i16);
            if (angle as u16) < LOWER_BOB_MIN {
                slot!(game, Brain[BRAIN].lower_motion = motion::BOB_UP);
                angle = LOWER_BOB_MIN as i16;
            }
            slot!(game, Brain[BRAIN].lower_angle = angle as u16);
        }
        motion::BOB_UP => {
            if (slot!(game, Brain[BRAIN].base.y_pos) as i32 - BRAIN_TOP_Y as i32) & 0x8000 != 0 {
                slot!(game, Brain[BRAIN].lower_motion = motion::BOB_DOWN);
            } else {
                let mut angle: i16 = (slot!(game, Brain[BRAIN].neck_speed) as i16)
                    .wrapping_add(slot!(game, Brain[BRAIN].lower_angle) as i16);
                if (angle as u16) >= LOWER_MAX {
                    slot!(game, Brain[BRAIN].lower_motion = motion::BOB_DOWN);
                    angle = LOWER_MAX as i16;
                }
                slot!(game, Brain[BRAIN].lower_angle = angle as u16);
            }
        }
        motion::LOWER => {
            let mut angle: i16 = (slot!(game, Brain[BRAIN].lower_angle) as i16)
                .wrapping_sub(slot!(game, Brain[BRAIN].neck_speed) as i16);
            if (angle as u16) < LOWER_MIN {
                slot!(game, Brain[BRAIN].lower_motion = motion::STILL);
                angle = LOWER_MIN as i16;
            }
            slot!(game, Brain[BRAIN].lower_angle = angle as u16);
        }
        motion::RAISE => {
            let mut angle: i16 = (slot!(game, Brain[BRAIN].neck_speed) as i16).wrapping_add(slot!(
                game,
                Brain[BRAIN].lower_angle
            )
                as i16);
            if (angle as u16) >= LOWER_MAX {
                slot!(game, Brain[BRAIN].lower_motion = motion::STILL);
                angle = LOWER_MAX as i16;
            }
            slot!(game, Brain[BRAIN].lower_angle = angle as u16);
        }
        _ => {
            game.unreachable();
            #[allow(clippy::empty_loop)]
            loop {}
        }
    }
}

/// Moves the upper joint by its motion; while bobbing it keeps the brain
/// level with Samus. Was `MotherBrain_HandleNeckUpper`.
fn move_upper_joint<H: Host + ?Sized>(game: &mut H) {
    match slot!(game, Brain[BRAIN].upper_motion) {
        motion::STILL => {}
        motion::BOB_DOWN => {
            if ((slot!(game, Brain[BRAIN].base.y_pos) as i32 + BRAIN_SAMUS_MARGIN
                - word!(game.samus_y_pos) as i32) as i16)
                < 0
            {
                let mut angle: i16 = (slot!(game, Brain[BRAIN].upper_angle) as i16)
                    .wrapping_sub(slot!(game, Brain[BRAIN].neck_speed) as i16);
                if (angle as u16) < UPPER_MIN {
                    slot!(game, Brain[BRAIN].upper_motion = motion::BOB_UP);
                    angle = UPPER_MIN as i16;
                }
                slot!(game, Brain[BRAIN].upper_angle = angle as u16);
            } else {
                slot!(game, Brain[BRAIN]; lower_motion = motion::BOB_UP, upper_motion = motion::BOB_UP);
            }
        }
        motion::BOB_UP => {
            let limit = slot!(game, Brain[BRAIN].lower_angle).wrapping_add(UPPER_ABOVE_LOWER);
            let mut angle = slot!(game, Brain[BRAIN].neck_speed)
                .wrapping_add(slot!(game, Brain[BRAIN].upper_angle));
            if angle >= limit {
                slot!(game, Brain[BRAIN].upper_motion = motion::BOB_DOWN);
                angle = limit;
            }
            slot!(game, Brain[BRAIN].upper_angle = angle);
        }
        motion::LOWER => {
            let mut angle: i16 = (slot!(game, Brain[BRAIN].upper_angle) as i16)
                .wrapping_sub(slot!(game, Brain[BRAIN].neck_speed) as i16);
            if (angle as u16) < UPPER_MIN {
                slot!(game, Brain[BRAIN].upper_motion = motion::STILL);
                angle = UPPER_MIN as i16;
            }
            slot!(game, Brain[BRAIN].upper_angle = angle as u16);
        }
        motion::RAISE => {
            let limit = slot!(game, Brain[BRAIN].lower_angle).wrapping_add(UPPER_ABOVE_LOWER);
            let mut angle = slot!(game, Brain[BRAIN].neck_speed)
                .wrapping_add(slot!(game, Brain[BRAIN].upper_angle));
            if angle >= limit {
                slot!(game, Brain[BRAIN].upper_motion = motion::STILL);
                angle = limit;
            }
            slot!(game, Brain[BRAIN].upper_angle = angle);
        }
        _ => {
            game.unreachable();
            #[allow(clippy::empty_loop)]
            loop {}
        }
    }
}

/// Folds the neck down for the fake death's ascent.
/// Was `MotherBrain_SetupNeckForFakeAscent`.
pub fn setup_for_fake_ascent<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Brain[BRAIN].segment_1_length = 2);
    slot!(game, Brain[BRAIN]; segment_2_length = 10, segment_4_length = 10);
    slot!(game, Brain[BRAIN]; segment_3_length = 20, segment_5_length = 20);
    slot!(game, Brain[BRAIN].lower_angle = 0x4800);
    slot!(game, Brain[BRAIN].upper_angle = 0x5000);
    slot!(game, Brain[BRAIN].neck_speed = 0x100);
}

/// Pulls the head back: raises the lower joint and lowers the upper one.
/// Was `MotherBrain_RetractHead`.
pub fn retract_head<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Brain[BRAIN].neck_speed = RETRACT_SPEED);
    slot!(game, Brain[BRAIN].neck_moving = 1);
    slot!(game, Brain[BRAIN].lower_motion = motion::RAISE);
    slot!(game, Brain[BRAIN].upper_motion = motion::LOWER);
}

/// Runs the third phase's neck function. Was `MotherBrain_Phase3_NeckHandler`.
pub fn run_phase3<H: Host + ?Sized>(game: &mut H) {
    match slot!(game, Brain[BRAIN].neck_function) {
        function::NORMAL => phase3_normal(game),
        function::IDLE => {}
        function::SETUP_RECOIL_RECOVERY => phase3_setup_recoil_recovery(game),
        function::RECOIL_RECOVERY => phase3_recoil_recovery(game),
        function::SETUP_HYPER_BEAM_RECOIL => phase3_setup_hyper_beam_recoil(game),
        function::HYPER_BEAM_RECOIL => phase3_hyper_beam_recoil(game),
        _ => game.unreachable(),
    }
}

/// Starts the normal bobbing. Was `MotherBrain_Phase3_Neck_Normal`; the
/// original stores the lower motion twice.
fn phase3_normal<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Brain[BRAIN]; lower_motion = 1, neck_speed = NORMAL_SPEED,
        lower_motion = motion::BOB_DOWN, upper_motion = motion::BOB_UP,
        neck_function = function::IDLE);
}

/// Lowers both joints fast after a hit. Was `MotherBrain_Phase3_Neck_SetupRecoilRecovery`.
fn phase3_setup_recoil_recovery<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Brain[BRAIN].neck_moving = 1);
    slot!(game, Brain[BRAIN].neck_speed = RECOIL_SPEED);
    slot!(game, Brain[BRAIN]; lower_motion = motion::LOWER, upper_motion = motion::LOWER);
    slot!(game, Brain[BRAIN].neck_function = function::RECOIL_RECOVERY);
    slot!(game, Brain[BRAIN].neck_timer = RECOIL_FRAMES);
    phase3_recoil_recovery(game);
}

/// Waits out the recoil, then recovers. Was `MotherBrain_Phase3_Neck_RecoilRecovery`.
fn phase3_recoil_recovery<H: Host + ?Sized>(game: &mut H) {
    let frames: i16 = (slot!(game, Brain[BRAIN].neck_timer) as i16).wrapping_sub(1);
    if frames < 0 {
        set_brain_instructions(game, animation::RECOVER);
        slot!(game, Brain[BRAIN].neck_function = function::NORMAL);
    } else {
        slot!(game, Brain[BRAIN].neck_timer = frames as u16);
    }
}

/// Throws the head back from the hyper beam. Was `MotherBrain_Phase3_Neck_SetupHyperBeamRecoil`.
fn phase3_setup_hyper_beam_recoil<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Brain[BRAIN].neck_moving = 1);
    slot!(game, Body[BODY].attacks_disabled = 1);
    set_brain_instructions(game, animation::HYPER_BEAM_HIT);
    slot!(game, Brain[BRAIN].shake_timer = HYPER_BEAM_FLASH);
    slot!(game, Brain[BRAIN].neck_speed = HYPER_BEAM_SPEED);
    slot!(game, Brain[BRAIN]; lower_motion = motion::RAISE, upper_motion = motion::RAISE);
    slot!(
        game,
        Brain[BRAIN].neck_function = function::HYPER_BEAM_RECOIL
    );
    slot!(game, Brain[BRAIN].neck_timer = HYPER_BEAM_FRAMES);
    phase3_hyper_beam_recoil(game);
}

/// Waits out the hyper beam recoil, then recovers. Was `MotherBrain_Phase3_Neck_HyperBeamRecoil`.
fn phase3_hyper_beam_recoil<H: Host + ?Sized>(game: &mut H) {
    let frames: i16 = (slot!(game, Brain[BRAIN].neck_timer) as i16).wrapping_sub(1);
    if frames < 0 {
        slot!(game, Brain[BRAIN].neck_speed = NORMAL_SPEED);
        slot!(game, Body[BODY].attacks_disabled = 0);
        slot!(
            game,
            Brain[BRAIN].neck_function = function::SETUP_RECOIL_RECOVERY
        );
    } else {
        slot!(game, Brain[BRAIN].neck_timer = frames as u16);
    }
}
