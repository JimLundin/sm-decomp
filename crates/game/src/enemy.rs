//! The enemies' gameplay phases: which slots process this frame, the main enemy routine's
//! per-slot bookkeeping, and each species' AI by its body (D14).
//!
//! Bank $A0 log (<https://patrickjohnston.org/bank/A0>): "Determine which enemies to
//! process" (`$A0:8EB6`) builds the active list from the slots on screen; the main enemy
//! routine (`$A0:8FD4`) walks it, choosing the AI from the slot's AI handler bits, then
//! counts the frame, and counts the hurt flash down after drawing.

use crate::state::room::{
    AiHandler, ENEMY_SLOTS, Elevator, ElevatorArming, ElevatorDirection, ElevatorRide,
    ElevatorStatus, EnemyBody, EnemyCommon, EnemyExtraProperty, EnemyProperty,
};
use crate::state::samus::{Fixed, Samus, Speed};
use crate::state::slots::{Frames, Occupancy, Slot};
use crate::state::{Button, Controller, State};
use data::enemy::program::EnemyPrograms;
use data::id::SpeciesId;

pub mod instructions;

/// The slots processing this frame, in table order (the original's active enemy indices).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Active {
    slots: Vec<usize>,
}

/// Which of the AI routines a species header names the main enemy routine runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiRoutine {
    /// No handler bit set.
    Main,
    /// Grappled.
    Grapple,
    /// Hurt.
    Hurt,
    /// Frozen.
    Frozen,
}

impl AiRoutine {
    /// The routine for the lowest handler bit set, the main AI for none (`$A0:9070`).
    fn of(slot: &EnemyCommon) -> Self {
        [
            (AiHandler::Grapple, Self::Grapple),
            (AiHandler::Hurt, Self::Hurt),
            (AiHandler::Frozen, Self::Frozen),
        ]
        .into_iter()
        .find(|&(handler, _)| slot.ai_handlers.contains(handler))
        .map_or(Self::Main, |(_, routine)| routine)
    }
}

/// Whether a 16-bit compare's difference is negative: the original's `CMP` and `BMI`.
const fn negative(difference: u16) -> bool {
    (difference & 0x8000) != 0
}

/// Whether the slot's centre is in the processing window around the camera (`$A0:8F00`):
/// its radius past the screen's sides, 8 pixels above and $F8 below the camera's top.
fn on_screen(slot: &EnemyCommon, camera_x: u16, camera_y: u16) -> bool {
    let x = slot.position.x.pixels;
    let y = slot.position.y.pixels;
    let left = x.wrapping_add(slot.radius_x).wrapping_sub(camera_x);
    let right = camera_x
        .wrapping_add(0x100)
        .wrapping_add(slot.radius_x)
        .wrapping_sub(x);
    let top = y.wrapping_add(8).wrapping_sub(camera_y);
    let bottom = camera_y.wrapping_add(0xf8).wrapping_sub(y);
    ![left, right, top, bottom].into_iter().any(negative)
}

/// The enemy selection phase (`$A0:8EB6`): free the deleted slots and list the live ones on
/// screen, or processed off screen, or frozen.
// open: global off-screen processing (the bosses' rooms), the respawning placeholder and
// the interactive list join with their first rows.
pub fn select(state: &mut State) -> Active {
    let camera_x = state.room.camera.x.pixels;
    let camera_y = state.room.camera.y.pixels;
    let mut active = Active::default();
    for (index, slot) in state.room.enemies.slots.iter_mut().enumerate() {
        match slot.occupancy {
            Occupancy::Free => {}
            Occupancy::Deleting(_) => {
                slot.occupancy = Occupancy::Free;
                slot.common.species = SpeciesId::default();
            }
            Occupancy::Live(_) => {
                let common = &slot.common;
                if common
                    .properties
                    .contains(EnemyProperty::ProcessedOffScreen)
                    || common.ai_handlers.contains(AiHandler::Frozen)
                    || on_screen(common, camera_x, camera_y)
                {
                    active.slots.push(index);
                }
            }
        }
    }
    active
}

/// The main enemy routine (`$A0:8FD4`) over the active slots.
// open: the shake request, Samus's and projectiles' collisions (`$A0:9758`), time frozen
// (the X-ray scope, which skips the frame count and the instructions), the grapple paralysis
// death, drawing, the sprite objects and the solid-enemy collision reset join with their
// lanes.
pub fn run(state: &mut State, active: &Active, programs: &EnemyPrograms) {
    for &index in active.slots.iter().filter(|&&index| index < ENEMY_SLOTS) {
        let Some(slot) = state.room.enemies.slots.get_mut(index) else {
            continue;
        };
        if !slot.common.properties.contains(EnemyProperty::Intangible)
            && slot.common.invincibility != Frames(0)
        {
            slot.common.invincibility = Frames(slot.common.invincibility.0.wrapping_sub(1));
        } else if slot
            .common
            .extra_properties
            .contains(EnemyExtraProperty::Paralysed)
            && !slot.common.properties.contains(EnemyProperty::Intangible)
        {
            count_flash(slot);
            continue;
        }
        let routine = AiRoutine::of(&slot.common);
        ai(
            slot,
            routine,
            &mut state.elevator,
            &mut state.samus,
            &state.controller,
        );
        slot.common.frame_counter = slot.common.frame_counter.wrapping_add(1);
        if slot
            .common
            .properties
            .contains(EnemyProperty::ProcessesInstructions)
        {
            instructions::process(slot, programs);
        }
        count_flash(slot);
    }
}

/// The hurt flash's countdown, which ends the hurt AI under 8 frames left (`$A0:9118`).
fn count_flash(slot: &mut Slot<EnemyCommon, EnemyBody>) {
    let flash = slot.common.flash.0;
    if flash == 0 {
        return;
    }
    let left = flash.wrapping_sub(1);
    slot.common.flash = Frames(left);
    if negative(left.wrapping_sub(8)) {
        slot.common.ai_handlers.remove(AiHandler::Hurt);
    }
}

/// The species' AI for `routine` (`$A0:908E`), by its body.
// open: the eye's AI joins with C8, the face block's with its lane.
fn ai(
    slot: &mut Slot<EnemyCommon, EnemyBody>,
    routine: AiRoutine,
    ride: &mut ElevatorRide,
    samus: &mut Samus,
    controller: &Controller,
) {
    let (Occupancy::Live(body) | Occupancy::Deleting(body)) = &mut slot.occupancy else {
        return;
    };
    match body {
        EnemyBody::Elevator(elevator) => match routine {
            // The header names one routine for all three (`$A3:952A`).
            AiRoutine::Main | AiRoutine::Grapple | AiRoutine::Frozen => {
                elevator_ai(&mut slot.common, *elevator, ride, samus, controller);
            }
            // open: the elevator's hurt AI, as its header names it.
            AiRoutine::Hurt => {}
        },
        EnemyBody::MorphBallEye { .. } | EnemyBody::FaceBlock { .. } => {}
    }
}

/// How far the platform moves a frame: 1.8000h pixels (`$A3:9587`).
const RIDE_SPEED: Fixed = Fixed {
    pixels: 1,
    subpixels: 0x8000,
};

/// How far above the platform's centre Samus stands (`$A3:9618`).
const SAMUS_ABOVE_PLATFORM: u16 = 0x1a;

/// `y` moved down by the ride speed, carrying the subpixel into the pixel.
fn descend(y: Fixed) -> Fixed {
    let (subpixels, carry) = y.subpixels.overflowing_add(RIDE_SPEED.subpixels);
    Fixed {
        pixels: y
            .pixels
            .wrapping_add(RIDE_SPEED.pixels)
            .wrapping_add(u16::from(carry)),
        subpixels,
    }
}

/// `y` moved up by the ride speed, borrowing the subpixel from the pixel.
fn ascend(y: Fixed) -> Fixed {
    let (subpixels, borrow) = y.subpixels.overflowing_sub(RIDE_SPEED.subpixels);
    Fixed {
        pixels: y
            .pixels
            .wrapping_sub(RIDE_SPEED.pixels)
            .wrapping_sub(u16::from(borrow)),
        subpixels,
    }
}

/// The way the platform goes when called: its first parameter, doubled by its init, is zero
/// for down (`$A3:957C`).
const fn direction(slot: &EnemyCommon) -> ElevatorDirection {
    if slot.parameter_1 == 0 {
        ElevatorDirection::Down
    } else {
        ElevatorDirection::Up
    }
}

/// The button that calls the platform its way (`$A3:94E2`).
const fn call_button(direction: ElevatorDirection) -> Button {
    match direction {
        ElevatorDirection::Down => Button::Down,
        ElevatorDirection::Up => Button::Up,
    }
}

/// The elevator's main AI (`$A3:952A`), by the ride's status.
// open: the door transition check (`$A3:952A`; slice 0's gameplay pass is never in one),
// the sounds (C11), the Samus commands that set her up for the ride and unlock her facing
// (bank $90, `$90:F084`) and the projectile reset (`$90:AD22`) join with their lanes.
pub fn elevator_ai(
    slot: &mut EnemyCommon,
    elevator: Elevator,
    ride: &mut ElevatorRide,
    samus: &mut Samus,
    controller: &Controller,
) {
    let way = direction(slot);
    match ride.status {
        ElevatorStatus::Inactive if matches!(ride.armed, ElevatorArming::Disarmed) => {}
        ElevatorStatus::Inactive => {
            if controller.pressed.contains(call_button(way)) {
                place_samus(slot, samus);
                ride.status = ElevatorStatus::LeavingRoom;
            } else {
                ride.armed = ElevatorArming::Disarmed;
            }
        }
        ElevatorStatus::LeavingRoom => {
            ride.direction = way;
            slot.position.y = match way {
                ElevatorDirection::Down => descend(slot.position.y),
                ElevatorDirection::Up => ascend(slot.position.y),
            };
            place_samus(slot, samus);
        }
        ElevatorStatus::DoorTransition | ElevatorStatus::EnteringRoom => {
            ride.status = ElevatorStatus::EnteringRoom;
            enter(slot, elevator, ride, samus, way);
        }
    }
}

/// Entering the room (`$A3:95BC`): an up elevator comes in from above, a down one from
/// below, and the ride ends at the target.
fn enter(
    slot: &mut EnemyCommon,
    elevator: Elevator,
    ride: &mut ElevatorRide,
    samus: &mut Samus,
    way: ElevatorDirection,
) {
    let (y, arrived) = match way {
        ElevatorDirection::Up => {
            let y = descend(slot.position.y);
            (y, y.pixels >= elevator.target_y)
        }
        ElevatorDirection::Down => {
            let y = ascend(slot.position.y);
            (y, y.pixels < elevator.target_y)
        }
    };
    slot.position.y = y;
    if arrived {
        ride.status = ElevatorStatus::Inactive;
        ride.armed = ElevatorArming::Disarmed;
        slot.position.y.pixels = elevator.target_y;
    }
    place_samus(slot, samus);
}

/// Stand Samus on the platform, still (`$A3:9612`).
const fn place_samus(slot: &EnemyCommon, samus: &mut Samus) {
    samus.position.y = Fixed {
        pixels: slot.position.y.pixels.wrapping_sub(SAMUS_ABOVE_PLATFORM),
        subpixels: 0,
    };
    samus.position.x.pixels = slot.position.x.pixels;
    if let Some(vertical) = samus.movement.vertical_mut() {
        vertical.speed = Speed::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::{Active, elevator_ai, run, select};
    use crate::state::room::{
        Elevator, ElevatorArming, ElevatorDirection, ElevatorRide, ElevatorStatus, EnemyBody,
        EnemyCommon, EnemyExtraProperty,
    };
    use crate::state::samus::{Fixed, Samus};
    use crate::state::slots::{Flag, Occupancy};
    use crate::state::{Button, Controller, State};

    fn at(pixels: u16, subpixels: u16) -> Fixed {
        Fixed { pixels, subpixels }
    }

    /// An up elevator entering the room, as the Morph Ball Room's is on arrival.
    fn arriving(y: Fixed, target_y: u16) -> (EnemyCommon, Elevator, ElevatorRide) {
        let mut slot = EnemyCommon {
            parameter_1: 2,
            ..EnemyCommon::default()
        };
        slot.position.x.pixels = 0x80;
        slot.position.y = y;
        let elevator = Elevator {
            target_y,
            unused: [0; 5],
        };
        let ride = ElevatorRide {
            status: ElevatorStatus::EnteringRoom,
            armed: ElevatorArming::Disarmed,
            direction: ElevatorDirection::Up,
        };
        (slot, elevator, ride)
    }

    #[test]
    fn entering_moves_the_platform_down_a_pixel_and_a_half_and_carries_samus() {
        // $1B.0000 + 1.8000 = $1C.8000; Samus stands $1A above, on a whole pixel.
        let (mut slot, elevator, mut ride) = arriving(at(0x1b, 0), 0x80);
        let mut samus = Samus::default();
        samus.position.y = at(0x40, 0x1234);
        elevator_ai(
            &mut slot,
            elevator,
            &mut ride,
            &mut samus,
            &Controller::default(),
        );
        assert_eq!(slot.position.y, at(0x1c, 0x8000));
        assert_eq!(samus.position.y, at(0x02, 0));
        assert_eq!(samus.position.x.pixels, 0x80);
        assert_eq!(ride.status, ElevatorStatus::EnteringRoom);
        // $1C.8000 + 1.8000 = $1E.0000: the subpixel carries.
        elevator_ai(
            &mut slot,
            elevator,
            &mut ride,
            &mut samus,
            &Controller::default(),
        );
        assert_eq!(slot.position.y, at(0x1e, 0));
    }

    #[test]
    fn reaching_the_target_ends_the_ride_on_the_target() {
        // $7F.0000 + 1.8000 = $80.8000, at or past $80: it snaps to $80.8000's pixel, $80.
        let (mut slot, elevator, mut ride) = arriving(at(0x7f, 0), 0x80);
        let mut samus = Samus::default();
        elevator_ai(
            &mut slot,
            elevator,
            &mut ride,
            &mut samus,
            &Controller::default(),
        );
        assert_eq!(slot.position.y, at(0x80, 0x8000));
        assert_eq!(ride.status, ElevatorStatus::Inactive);
        assert_eq!(ride.armed, ElevatorArming::Disarmed);
        assert_eq!(samus.position.y, at(0x66, 0));
    }

    #[test]
    fn the_door_transition_status_enters_on_the_same_frame() {
        let (mut slot, elevator, mut ride) = arriving(at(0x1b, 0), 0x80);
        ride.status = ElevatorStatus::DoorTransition;
        elevator_ai(
            &mut slot,
            elevator,
            &mut ride,
            &mut Samus::default(),
            &Controller::default(),
        );
        assert_eq!(ride.status, ElevatorStatus::EnteringRoom);
        assert_eq!(slot.position.y, at(0x1c, 0x8000));
    }

    #[test]
    fn an_idle_armed_elevator_leaves_on_its_button_and_disarms_without_it() {
        let (mut slot, elevator, mut ride) = arriving(at(0x80, 0), 0x80);
        ride.status = ElevatorStatus::Inactive;
        ride.armed = ElevatorArming::InDoorway;
        let mut samus = Samus::default();
        let mut controller = Controller::default();
        controller.pressed.insert(Button::Down);
        elevator_ai(&mut slot, elevator, &mut ride, &mut samus, &controller);
        // Down doesn't call an up elevator: it disarms and stays.
        assert_eq!(ride.armed, ElevatorArming::Disarmed);
        assert_eq!(ride.status, ElevatorStatus::Inactive);
        ride.armed = ElevatorArming::InDoorway;
        controller.pressed.remove(Button::Down);
        controller.pressed.insert(Button::Up);
        elevator_ai(&mut slot, elevator, &mut ride, &mut samus, &controller);
        assert_eq!(ride.status, ElevatorStatus::LeavingRoom);
        assert_eq!(samus.position.y, at(0x66, 0));
        // Leaving upward: $80.0000 - 1.8000 = $7E.8000.
        elevator_ai(&mut slot, elevator, &mut ride, &mut samus, &controller);
        assert_eq!(slot.position.y, at(0x7e, 0x8000));
        assert_eq!(ride.direction, ElevatorDirection::Up);
    }

    #[test]
    fn the_main_routine_counts_the_frame_of_each_active_slot_only() {
        let mut state = State::default();
        let (common, elevator, ride) = arriving(at(0x1b, 0), 0x80);
        state.elevator = ride;
        for slot in state.room.enemies.slots.iter_mut().take(2) {
            slot.common = common;
            slot.occupancy = Occupancy::Live(EnemyBody::Elevator(elevator));
        }
        // Slot 1 is far below the camera's window.
        if let Some(slot) = state.room.enemies.slots.get_mut(1) {
            slot.common.position.y.pixels = 0x400;
            slot.common.extra_properties.bits = EnemyExtraProperty::GraphicUpdated.bit();
        }
        let active = select(&mut state);
        assert_eq!(active, Active { slots: vec![0] });
        run(
            &mut state,
            &active,
            &data::enemy::program::EnemyPrograms::default(),
        );
        let counters: Vec<u16> = state
            .room
            .enemies
            .slots
            .iter()
            .take(2)
            .map(|slot| slot.common.frame_counter)
            .collect();
        assert_eq!(counters, [1, 0]);
        assert_eq!(state.samus.position.y, at(0x02, 0));
    }
}
