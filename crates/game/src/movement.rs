//! Samus's horizontal motion on the ground: the movement handler's standing, running and
//! turning movements, and the speed rules they share.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): the normal movement handler
//! (`$90:A337`) runs one routine a movement type, before the pose animation and the pose
//! update (`$90:E725` "Samus new state handler - normal"). Running (`$90:A3E5`) and turning
//! on the ground (`$90:A67C`) both call "Samus X movement" (`$90:8E64`): the extra run speed
//! (`$90:973E`), the base speed from the movement type's speed table entry (`$90:9BD1`,
//! `$90:9A7E`), then the move (`$90:8EA9`). Standing (`$90:A383`) moves by the extra run speed
//! alone (`$90:9348`) and then stops her. The jumps and the fall are [`air`]'s.

mod air;

pub use air::make_jump;

use data::movement::{HorizontalSpeeds, Speed, SpeedEntry};
use data::pose::transition::Control;

use crate::pose::{Content, controls};
use crate::state::Controller;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{
    Acceleration, Facing, Fixed, Horizontal, Item, Movement, PoseId, Samus, Turn,
};

/// The poses that face the screen: standing in them is the elevator's, which takes no
/// horizontal motion (`$90:A383`).
const FACING_FORWARD: [PoseId; 2] = [PoseId(0), PoseId(0x9B)];

/// The movement handler's horizontal part for the frame (`$90:A337`).
// open: the ground's vertical part ("Samus Y movement - no speed calculations", `$90:923F`),
// the elevator's descent, the footsteps and the standing shot's animation reset join with C4,
// C6 and C7; the morph ball, the spring ball and the air turns join with their movements.
pub fn run(samus: &mut Samus, controller: &Controller, content: &Content) {
    if samus.movement.vertical().is_some() {
        air::run(samus, controller, content);
        return;
    }
    let facing = samus.facing;
    let pose = samus.pose;
    let equipped_booster = samus.equipment.equipped.contains(Item::SpeedBooster);
    let run_held = controls(controller.held).contains(Control::Run);
    let movement = samus.movement;
    let Some(speed) = samus.movement.horizontal_mut() else {
        return;
    };
    let mut position = samus.position.x;
    match movement {
        Movement::Standing { .. } => {
            if !FACING_FORWARD.contains(&pose) {
                stand(speed, facing, &mut position);
            }
        }
        Movement::Running { .. } => {
            let rules = Rules {
                speeds: &content.speeds,
                movement,
                run_held,
                equipped_booster,
            };
            rules.x_movement(speed, facing, &mut position);
        }
        Movement::TurningAround {
            place: Turn::OnGround,
            ..
        } => {
            let rules = Rules {
                speeds: &content.speeds,
                movement,
                run_held,
                equipped_booster,
            };
            rules.x_movement(speed, facing, &mut position);
            cancel_speed_boosting(speed);
            speed.extra_run = Speed::ZERO;
        }
        // open: turning in the air moves her the same way, with her vertical motion (C3b).
        Movement::TurningAround { .. }
        | Movement::Crouching
        | Movement::Jumping { .. }
        | Movement::Falling { .. }
        | Movement::MorphBall { .. }
        | Movement::SpringBall { .. }
        | Movement::KnockbackOrCrystalFlashEnding
        | Movement::Transition
        | Movement::Moonwalking
        | Movement::WallJumping { .. }
        | Movement::RanIntoWall
        | Movement::Grappling
        | Movement::DamageBoost
        | Movement::GrabbedByDraygon
        | Movement::Shinespark
        | Movement::Unused(_) => return,
    }
    samus.position.x = position;
}

/// What "Samus X movement" reads besides the motion itself.
struct Rules<'c> {
    speeds: &'c HorizontalSpeeds,
    movement: Movement,
    run_held: bool,
    equipped_booster: bool,
}

impl Rules<'_> {
    /// "Samus X movement" (`$90:8E64`): the extra run speed, the base speed, then the move.
    fn x_movement(&self, speed: &mut Horizontal, facing: Facing, x: &mut Fixed) {
        self.extra_run(speed);
        // A movement type past the table moves her by her extra run speed alone.
        if let Some(entry) = self.speeds.normal.of(self.movement.kind()) {
            base_speed(speed, entry);
        }
        move_horizontally(speed, speed.base, facing, x);
    }

    /// "Handle Samus X extra run speed" (`$90:973E`): running with the run button held adds
    /// the extra run acceleration up to its cap; anything else drops it unless she kept her
    /// momentum.
    // open: the speed booster's palette, its boost timer and counter, and the liquids that
    // stop the extra run speed join with the speed booster and liquids.
    const fn extra_run(&self, speed: &mut Horizontal) {
        let running = matches!(self.movement, Movement::Running { .. });
        if !(running && self.run_held) {
            if !speed.momentum {
                speed.extra_run = Speed::ZERO;
            }
            return;
        }
        speed.momentum = true;
        let extra = &self.speeds.extra_run;
        let max = if self.equipped_booster {
            extra.booster_max
        } else {
            extra.max
        };
        let current = speed.extra_run;
        // Each word compares on its own, as the original's two CMPs do.
        if below(current.pixels(), max.pixels()) || below(current.subpixels(), max.subpixels()) {
            speed.extra_run = Speed(current.0.wrapping_add(extra.acceleration.0));
        } else {
            speed.extra_run = max;
        }
    }
}

/// Whether `value` compares below `limit` the way the 65816's CMP and BMI do: by the sign
/// of the 16-bit difference.
const fn below(value: u16, limit: u16) -> bool {
    value.wrapping_sub(limit) >= 0x8000
}

/// "Calculate Samus X base speed - deceleration allowed" (`$90:9A7E`): accelerating adds the
/// entry's acceleration up to its cap; turning or decelerating takes its deceleration away
/// until she stops, and then she accelerates again.
// open: the deceleration multiplier (`SAMUS_X_DECEL_MULT`) scales the deceleration when set;
// it joins with the blocks that set it (C4).
const fn base_speed(speed: &mut Horizontal, entry: &SpeedEntry) {
    match speed.acceleration {
        Acceleration::Accelerating => {
            let base = Speed(speed.base.0.wrapping_add(entry.acceleration.0));
            speed.base = if over_cap(base, entry.max) {
                entry.max
            } else {
                base
            };
        }
        Acceleration::TurningAround | Acceleration::Decelerating => {
            let base = Speed(speed.base.0.wrapping_sub(entry.deceleration.0));
            if base.pixels() >= 0x8000 {
                speed.base = Speed::ZERO;
                speed.acceleration = Acceleration::Accelerating;
            } else {
                speed.base = base;
            }
        }
    }
}

/// Whether a base speed is past the cap, word by word as the original compares: a pixel
/// word below the cap's never is; an equal one is when the subpixel word compares above.
/// So 2.1000 against a cap of 2.C000 is past it (the difference $5000 is positive).
const fn over_cap(base: Speed, max: Speed) -> bool {
    let pixels = base.pixels().wrapping_sub(max.pixels());
    if pixels >= 0x8000 {
        return false;
    }
    if pixels != 0 {
        return true;
    }
    let subpixels = base.subpixels().wrapping_sub(max.subpixels());
    subpixels != 0 && subpixels < 0x8000
}

/// "Move Samus horizontally" (`$90:8EA9`): she moves the way she faces, or against it while
/// her turn's momentum lasts, by her base and extra run speed (`$90:E464`, `$90:E4AD`).
// open: the speed divisor (`SAMUS_X_SPEED_DIVISOR`) and the extra displacement
// (`EXTRA_SAMUS_X_DISPLACEMENT`) are zero in slice 0's movements; they join with the
// movements that set them. The move itself goes through block and solid enemy collision
// (`$90:9350`, `$90:93B1`), which can stop her and kill her speed (C4).
fn move_horizontally(speed: &Horizontal, base: Speed, facing: Facing, x: &mut Fixed) {
    let leftwards = match speed.acceleration {
        Acceleration::Accelerating | Acceleration::Decelerating => facing == Facing::Left,
        Acceleration::TurningAround => facing == Facing::Right,
    };
    let total = base.0.wrapping_add(speed.extra_run.0);
    let displacement = clamp(if leftwards {
        total.wrapping_neg()
    } else {
        total
    });
    let position = Speed::new(x.pixels, x.subpixels);
    let moved = Speed(position.0.wrapping_add(displacement));
    x.pixels = moved.pixels();
    x.subpixels = moved.subpixels();
}

/// The displacement's pixel word, held to 15 pixels either way with its subpixels kept
/// (`$90:E464`).
fn clamp(displacement: u32) -> u32 {
    let amount = Speed(displacement);
    let pixels = amount.pixels();
    let held = if pixels < 0x8000 {
        pixels.min(0x000F)
    } else {
        pixels.max(0xFFF1)
    };
    Speed::new(held, amount.subpixels()).0
}

/// "Samus movement - standing" (`$90:A383`): her extra run speed moves her a last frame,
/// then she stops.
fn stand(speed: &mut Horizontal, facing: Facing, x: &mut Fixed) {
    move_horizontally(speed, Speed::ZERO, facing, x);
    // Cancelling the speed boost drops her momentum; the speeds go with it.
    *speed = Horizontal::default();
}

/// "Cancel speed boosting" (bank $91 log, `$91:DE53`): she loses her running momentum.
// open: the speed boost counter, timer and palettes join with the speed booster.
pub const fn cancel_speed_boosting(speed: &mut Horizontal) {
    speed.momentum = false;
}

/// Whether the elevator carries her: standing facing the screen while a ride is on. The pose
/// input handler takes no input then (`$91:805C`), and standing steps her down (`$90:A383`).
#[must_use]
pub fn carried(samus: &Samus, ride: ElevatorRide) -> bool {
    let riding = match ride.status {
        ElevatorStatus::Inactive => false,
        ElevatorStatus::LeavingRoom
        | ElevatorStatus::DoorTransition
        | ElevatorStatus::EnteringRoom => true,
    };
    riding
        && FACING_FORWARD.contains(&samus.pose)
        && matches!(samus.movement, Movement::Standing { .. })
}

/// Standing on a ride steps her a pixel down (`$90:A383`), after the platform has stood her
/// on it this frame: the next frame's placement takes it back.
// open: the step is a terrain-only collision, so a block under her stops it at the block's
// top with subpixels $FFFF (the Brinstar descent's frames 316-326 and 348-358); it joins
// with Samus's block collision.
pub fn ride(samus: &mut Samus, ride: ElevatorRide) {
    if carried(samus, ride) {
        samus.position.y.pixels = samus.position.y.pixels.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests;
