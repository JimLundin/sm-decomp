//! The baby Metroid, hatched from the last Metroid (bank
//! $A9:EED1-$A9:FB6F): its hitbox tests, camera wait, screen test, steering
//! and cry.
#![forbid(unsafe_code)]
use crate::actors::layouts::part_view;
use crate::actors::session_slots::{SlotAccess, session_slot};
use crate::effects::SoundEffects;
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess, word};

/// What the baby Metroid needs beyond work RAM and sound: hardcoded PLM spawns.
pub trait Host:
    WorkRamAccess + CompatibilityRam + SoundEffects + crate::runtime_access::RuntimeAccess + SlotAccess
{
    /// Spawn the PLM with `header` at block (`block_x`, `block_y`).
    fn spawn_hardcoded_plm(&mut self, block_x: u8, block_y: u8, header: u16);
}

part_view! {
    /// The baby Metroid's view of its slot.
    BabyMetroid: EnemyLayout40 {
        words: [
            function, x_velocity, y_velocity,
            // $0FAE: the palette animation frame (low byte) and timer (high byte).
            palette_animation, palette_handler_delay, function_timer, parameter_1,
            // $0FB6: the contact reaction.
            contact_enabled,
        ],
        pad1: [u8; 26696],
        words: [
            var_00,
            // $7E:7802: frames left chasing Samus's Y; $7E:7804: how long it
            // has hovered over her.
            aggro_timer, proximity_counter, var_03, cry_enabled, cry_timer, var_06, var_07, var_08, var_09,
            var_0A, var_0B, var_0C, var_0D, var_0E, var_0F, var_10, var_11, var_12, var_13, var_14,
            var_15, var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
    }
}

/// $A9:F02B: Shitroid function - let sidehopper live for a bit.
pub const LET_SIDEHOPPER_LIVE: u16 = 0xf02b;
/// PLM $B767, the baby Metroid invisible wall, at its two blocks.
const INVISIBLE_WALL: u16 = 0xb767;
const INVISIBLE_WALLS: [(u8, u8); 2] = [(0x30, 3), (0x1f, 3)];
/// The camera stops at X 200h.
const CAMERA_STOP_X: u16 = 0x200;
/// $7E:CD20 and $7E:CD22, scrolls 1 and 3, which the camera stop turns red.
const SCROLLS: [usize; 2] = [0xcd20, 0xcd22];

/// The screen test's margins: 60h above and below, 10h left and right of a
/// 120h by 1A0h area.
const SCREEN_Y_MARGIN: i32 = 0x60;
const SCREEN_HEIGHT: i32 = 0x1a0;
const SCREEN_X_MARGIN: i32 = 0x10;
const SCREEN_WIDTH: i32 = 0x120;
const SIGN: i32 = 0x8000;

/// Velocities are capped at 500h (1/100h px a frame).
const MAX_SPEED: u16 = 0x500;

/// The cry plays on palette frame 5 of every fourth cycle; below palette
/// handler delay 0Ah it is the draining sound instead.
const CRY_FRAME: u16 = 5;
const CRY_CYCLES: u16 = 4;
const DRAINING_DELAY: u16 = 0xa;
const CRY: u8 = 0x72;
const DRAINING: u8 = 0x78;

fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

/// Check for enemy collision with enemy ($A9:EED1): 1 when actor `k`'s and
/// actor `j`'s hitboxes do not overlap. Was `BabyMetroid_Func_1`.
pub fn separated_from_enemy(k: u16, j: u16, game: &mut impl Host) -> u8 {
    let e = i32::from(j);
    let g = i32::from(k);
    let width = (session_slot!(game, BabyMetroid[g].base.x_width) as i32
        + session_slot!(game, BabyMetroid[e].base.x_width) as i32
        + 1) as u16;
    let dx = abs16(
        session_slot!(game, BabyMetroid[e].base.x_pos)
            .wrapping_sub(session_slot!(game, BabyMetroid[g].base.x_pos)),
    );
    if dx < width {
        let height = (session_slot!(game, BabyMetroid[g].base.y_height) as i32
            + session_slot!(game, BabyMetroid[e].base.y_height) as i32
            + 1) as u16;
        return (abs16(
            session_slot!(game, BabyMetroid[e].base.y_pos)
                .wrapping_sub(session_slot!(game, BabyMetroid[g].base.y_pos)),
        ) >= height) as u8;
    }
    1
}

/// Check for enemy collision with rectangle ($A9:EF06): 1 when actor `k`'s
/// hitbox does not overlap `rect`. Was `BabyMetroid_Func_2`.
/// `rect` is (x, y, half width, half height).
pub fn separated_from_rect(k: u16, rect: (u16, u16, u16, u16), game: &mut impl Host) -> u8 {
    let (x, y, w, h) = rect;
    let e = i32::from(k);
    let width = (session_slot!(game, BabyMetroid[e].base.x_width) as i32 + w as i32 + 1) as u16;
    let dx = abs16(x.wrapping_sub(session_slot!(game, BabyMetroid[e].base.x_pos)));
    if dx < width {
        let height =
            (session_slot!(game, BabyMetroid[e].base.y_height) as i32 + h as i32 + 1) as u16;
        return (abs16(y.wrapping_sub(session_slot!(game, BabyMetroid[e].base.y_pos))) >= height)
            as u8;
    }
    1
}

/// Baby Metroid function - wait for camera ($A9:EFE6): once the camera is left of
/// the stop, pin it there, turn scrolls 1 and 3 red, raise the invisible walls,
/// let the sidehopper live and enable the cry. Was `BabyMetroid_Func_4`.
pub fn wait_for_camera(game: &mut impl Host) {
    if (crate::camera::current(game).layer1[0] as i32 - (i32::from(CAMERA_STOP_X) + 1)) & SIGN != 0
    {
        crate::camera::set_layer1(&mut game.runtime(), 0, CAMERA_STOP_X);
        // Red is the scroll word's low byte with its high byte cleared.
        for scroll in SCROLLS {
            let value = u16::from(game.ram().read_byte(scroll));
            let bytes = value.to_ne_bytes();
            game.ram().write_byte(scroll, bytes[0]);
            game.ram().write_byte(scroll + 1, bytes[1]);
        }
        for (block_x, block_y) in INVISIBLE_WALLS {
            game.spawn_hardcoded_plm(block_x, block_y, INVISIBLE_WALL);
        }
        let e = word!(game.cur_enemy_index) as i32;
        session_slot!(game, BabyMetroid[e].function = LET_SIDEHOPPER_LIVE);
        session_slot!(game, BabyMetroid[e].cry_enabled = 1);
    }
}

/// Check if enemy is vaguely on screen ($A9:F57A): 0 while actor `k` is within
/// the screen margins, 1 otherwise. Was `BabyMetroid_CheckIfOnScreen`.
pub fn is_off_screen(k: u16, game: &mut impl Host) -> u8 {
    let e = i32::from(k);
    let y_pos = session_slot!(game, BabyMetroid[e].base.y_pos) as i16;
    if y_pos >= 0 {
        let y =
            (y_pos as i32 + SCREEN_Y_MARGIN - crate::camera::current(game).layer1[1] as i32) as i16;
        if y >= 0 && y as i32 - SCREEN_HEIGHT & SIGN != 0 {
            let x_pos = session_slot!(game, BabyMetroid[e].base.x_pos) as i16;
            if x_pos >= 0 {
                let x = (x_pos as i32 + SCREEN_X_MARGIN
                    - crate::camera::current(game).layer1[0] as i32) as i16;
                if x >= 0 && (x as i32 - SCREEN_WIDTH) & SIGN != 0 {
                    return 0;
                }
            }
        }
    }
    1
}

/// Accelerate baby Metroid towards Y position ($A9:F5B5): accelerate the Y
/// velocity by `acceleration` towards `target`, capped at 500h; 1 once there.
/// Was `BabyMetroid_AccelerateTowardsY`.
pub fn accelerate_towards_y(k: u16, target: u16, acceleration: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    let mut result: u16 = 0;
    let offset =
        (session_slot!(game, BabyMetroid[e].base.y_pos) as i16).wrapping_sub(target as i16);
    if offset == 0 {
        result = result.wrapping_add(1);
    } else if offset >= 0 {
        let mut velocity =
            session_slot!(game, BabyMetroid[e].y_velocity).wrapping_sub(acceleration);
        if velocity as i32 + i32::from(MAX_SPEED) & SIGN != 0 {
            velocity = MAX_SPEED.wrapping_neg();
        }
        session_slot!(game, BabyMetroid[e].y_velocity = velocity);
        if ((session_slot!(game, BabyMetroid[e].base.y_pos) as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            <= 0
        {
            session_slot!(game, BabyMetroid[e].y_velocity = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity =
            acceleration.wrapping_add(session_slot!(game, BabyMetroid[e].y_velocity));
        if velocity as i32 - i32::from(MAX_SPEED) & SIGN == 0 {
            velocity = MAX_SPEED;
        }
        session_slot!(game, BabyMetroid[e].y_velocity = velocity);
        if ((session_slot!(game, BabyMetroid[e].base.y_pos) as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            >= 0
        {
            session_slot!(game, BabyMetroid[e].y_velocity = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}

/// Accelerate baby Metroid towards X position ($A9:F615): accelerate the X
/// velocity by `acceleration` towards `target`, capped at 500h; 1 once there.
/// Was `BabyMetroid_AccelerateTowardsX`.
pub fn accelerate_towards_x(k: u16, target: u16, acceleration: u16, game: &mut impl Host) -> u16 {
    let e = i32::from(k);
    let mut result: u16 = 0;
    if (session_slot!(game, BabyMetroid[e].base.x_pos) as i16).wrapping_sub(target as i16) >= 0 {
        let mut velocity =
            session_slot!(game, BabyMetroid[e].x_velocity).wrapping_sub(acceleration);
        if velocity as i32 + i32::from(MAX_SPEED) & SIGN != 0 {
            velocity = MAX_SPEED.wrapping_neg();
        }
        session_slot!(game, BabyMetroid[e].x_velocity = velocity);
        if ((session_slot!(game, BabyMetroid[e].base.x_pos) as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            <= 0
        {
            session_slot!(game, BabyMetroid[e].x_velocity = 0);
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity =
            acceleration.wrapping_add(session_slot!(game, BabyMetroid[e].x_velocity));
        if velocity as i32 - i32::from(MAX_SPEED) & SIGN == 0 {
            velocity = MAX_SPEED;
        }
        session_slot!(game, BabyMetroid[e].x_velocity = velocity);
        if ((session_slot!(game, BabyMetroid[e].base.x_pos) as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            >= 0
        {
            session_slot!(game, BabyMetroid[e].x_velocity = 0);
            result = result.wrapping_add(1);
        }
    }
    result
}

/// Handle baby Metroid cry sound effect ($A9:F751): on palette frame 5, cry every
/// fourth time while the cry is enabled. Was `BabyMetroid_HandleCrySoundEffect`.
pub fn handle_cry(k: u16, a: u16, game: &mut impl Host) -> u16 {
    if a == CRY_FRAME {
        let e = i32::from(k);
        if session_slot!(game, BabyMetroid[e].cry_enabled) != 0 {
            let count = session_slot!(game, BabyMetroid[e].cry_timer).wrapping_add(1);
            session_slot!(game, BabyMetroid[e].cry_timer = count);
            if count >= CRY_CYCLES {
                session_slot!(game, BabyMetroid[e].cry_timer = 0);
                let effect =
                    if session_slot!(game, BabyMetroid[e].palette_handler_delay) < DRAINING_DELAY {
                        DRAINING
                    } else {
                        CRY
                    };
                game.queue_sound(1, effect, 6);
            }
        }
        return CRY_FRAME;
    }
    a
}
