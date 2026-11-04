//! The baby Metroid, hatched from the last Metroid (bank
//! $A9:EED1-$A9:FB6F): its hitbox tests, camera wait, screen test, steering
//! and cry.
use crate::actors::host::abs16;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::effects::SoundEffects;
use crate::ram::CompatibilityRam;
use crate::runtime_access::SessionRam;

slot_view! {
    /// The baby Metroid's slot.
    BabyMetroid, BabyMetroidEdit {
        record: [
            function = var_a, x_velocity = var_b, y_velocity = var_c,
            // $0FAE: the palette animation frame (low byte) and timer (high byte).
            palette_animation = var_d, palette_handler_delay = var_e, function_timer = var_f,
            // $0FB6: the contact reaction.
            contact_enabled = parameter_2,
        ],
        extended: [
            // $7E:7802: frames left chasing Samus's Y; $7E:7804: how long it
            // has hovered over her.
            aggro_timer = var_01, proximity_counter = var_02, cry_enabled = var_04,
            cry_timer = var_05,
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

/// Check for enemy collision with enemy ($A9:EED1): 1 when actor `k`'s and
/// actor `j`'s hitboxes do not overlap.
pub fn separated_from_enemy(
    k: u16,
    j: u16,
    game: &mut (impl SlotState + CompatibilityRam + ?Sized),
) -> u8 {
    let width = (BabyMetroid::of(game, k).actor().radius.x as i32
        + BabyMetroid::of(game, j).actor().radius.x as i32
        + 1) as u16;
    let dx = abs16(
        BabyMetroid::of(game, j)
            .actor()
            .x
            .whole
            .wrapping_sub(BabyMetroid::of(game, k).actor().x.whole),
    );
    if dx < width {
        let height = (BabyMetroid::of(game, k).actor().radius.y as i32
            + BabyMetroid::of(game, j).actor().radius.y as i32
            + 1) as u16;
        return (abs16(
            BabyMetroid::of(game, j)
                .actor()
                .y
                .whole
                .wrapping_sub(BabyMetroid::of(game, k).actor().y.whole),
        ) >= height) as u8;
    }
    1
}

/// Check for enemy collision with rectangle ($A9:EF06): 1 when actor `k`'s
/// hitbox does not overlap `rect`. `rect` is (x, y, half width, half
/// height).
pub fn separated_from_rect(
    k: u16,
    rect: (u16, u16, u16, u16),
    game: &mut (impl SlotState + CompatibilityRam + ?Sized),
) -> u8 {
    let (x, y, w, h) = rect;
    let width = (BabyMetroid::of(game, k).actor().radius.x as i32 + w as i32 + 1) as u16;
    let dx = abs16(x.wrapping_sub(BabyMetroid::of(game, k).actor().x.whole));
    if dx < width {
        let height = (BabyMetroid::of(game, k).actor().radius.y as i32 + h as i32 + 1) as u16;
        return (abs16(y.wrapping_sub(BabyMetroid::of(game, k).actor().y.whole)) >= height) as u8;
    }
    1
}

/// Baby Metroid function - wait for camera ($A9:EFE6): once the camera is left of
/// the stop, pin it there, turn scrolls 1 and 3 red, raise the invisible walls,
/// let the sidehopper live and enable the cry.
pub fn wait_for_camera(game: &mut impl crate::actors::mother_brain_fight::Lease) {
    if ((i32::from(game.camera().layer1[0]) - (i32::from(CAMERA_STOP_X) + 1)) & SIGN) != 0 {
        crate::camera::set_layer1(game, 0, CAMERA_STOP_X);
        // Red is the scroll word's low byte with its high byte cleared.
        for scroll in SCROLLS {
            let value = u16::from(SessionRam::byte(game, scroll));
            let bytes = value.to_ne_bytes();
            SessionRam::set_byte(game, scroll, bytes[0]);
            SessionRam::set_byte(game, scroll + 1, bytes[1]);
        }
        for (block_x, block_y) in INVISIBLE_WALLS {
            crate::actors::mother_brain_fight::spawn_plm(block_x, block_y, INVISIBLE_WALL, game);
        }
        let k = game.current_enemy();
        *BabyMetroidEdit::of(game, k).function() = LET_SIDEHOPPER_LIVE;
        *BabyMetroidEdit::of(game, k).cry_enabled() = 1;
    }
}

/// Check if enemy is vaguely on screen ($A9:F57A): 0 while actor `k` is within
/// the screen margins, 1 otherwise.
pub fn is_off_screen(k: u16, game: &mut (impl SlotState + CompatibilityRam + ?Sized)) -> u8 {
    let y_pos = BabyMetroid::of(game, k).actor().y.whole as i16;
    if y_pos >= 0 {
        let y =
            (y_pos as i32 + SCREEN_Y_MARGIN - game.session().room.camera.layer1[1] as i32) as i16;
        if y >= 0 && (y as i32 - SCREEN_HEIGHT) & SIGN != 0 {
            let x_pos = BabyMetroid::of(game, k).actor().x.whole as i16;
            if x_pos >= 0 {
                let x = (x_pos as i32 + SCREEN_X_MARGIN
                    - game.session().room.camera.layer1[0] as i32) as i16;
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
pub fn accelerate_towards_y(
    k: u16,
    target: u16,
    acceleration: u16,
    game: &mut (impl SlotState + CompatibilityRam + ?Sized),
) -> u16 {
    let mut result: u16 = 0;
    let offset = (BabyMetroid::of(game, k).actor().y.whole as i16).wrapping_sub(target as i16);
    if offset == 0 {
        result = result.wrapping_add(1);
    } else if offset >= 0 {
        let mut velocity = BabyMetroid::of(game, k)
            .y_velocity()
            .wrapping_sub(acceleration);
        if (velocity as i32 + i32::from(MAX_SPEED)) & SIGN != 0 {
            velocity = MAX_SPEED.wrapping_neg();
        }
        *BabyMetroidEdit::of(game, k).y_velocity() = velocity;
        if ((BabyMetroid::of(game, k).actor().y.whole as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            <= 0
        {
            *BabyMetroidEdit::of(game, k).y_velocity() = 0;
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity = acceleration.wrapping_add(BabyMetroid::of(game, k).y_velocity());
        if (velocity as i32 - i32::from(MAX_SPEED)) & SIGN == 0 {
            velocity = MAX_SPEED;
        }
        *BabyMetroidEdit::of(game, k).y_velocity() = velocity;
        if ((BabyMetroid::of(game, k).actor().y.whole as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            >= 0
        {
            *BabyMetroidEdit::of(game, k).y_velocity() = 0;
            result = result.wrapping_add(1);
        }
    }
    result
}

/// Accelerate baby Metroid towards X position ($A9:F615): accelerate the X
/// velocity by `acceleration` towards `target`, capped at 500h; 1 once there.
pub fn accelerate_towards_x(
    k: u16,
    target: u16,
    acceleration: u16,
    game: &mut (impl SlotState + CompatibilityRam + ?Sized),
) -> u16 {
    let mut result: u16 = 0;
    if (BabyMetroid::of(game, k).actor().x.whole as i16).wrapping_sub(target as i16) >= 0 {
        let mut velocity = BabyMetroid::of(game, k)
            .x_velocity()
            .wrapping_sub(acceleration);
        if (velocity as i32 + i32::from(MAX_SPEED)) & SIGN != 0 {
            velocity = MAX_SPEED.wrapping_neg();
        }
        *BabyMetroidEdit::of(game, k).x_velocity() = velocity;
        if ((BabyMetroid::of(game, k).actor().x.whole as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            <= 0
        {
            *BabyMetroidEdit::of(game, k).x_velocity() = 0;
            result = result.wrapping_add(1);
        }
    } else {
        let mut velocity = acceleration.wrapping_add(BabyMetroid::of(game, k).x_velocity());
        if (velocity as i32 - i32::from(MAX_SPEED)) & SIGN == 0 {
            velocity = MAX_SPEED;
        }
        *BabyMetroidEdit::of(game, k).x_velocity() = velocity;
        if ((BabyMetroid::of(game, k).actor().x.whole as i32 + (velocity >> 8) as i8 as i32
            - target as i32) as i16)
            >= 0
        {
            *BabyMetroidEdit::of(game, k).x_velocity() = 0;
            result = result.wrapping_add(1);
        }
    }
    result
}

/// Handle baby Metroid cry sound effect ($A9:F751): on palette frame 5, cry every
/// fourth time while the cry is enabled.
pub fn handle_cry(
    k: u16,
    a: u16,
    game: &mut (impl SlotState + CompatibilityRam + SoundEffects + ?Sized),
) -> u16 {
    if a == CRY_FRAME {
        if BabyMetroid::of(game, k).cry_enabled() != 0 {
            let count = BabyMetroid::of(game, k).cry_timer().wrapping_add(1);
            *BabyMetroidEdit::of(game, k).cry_timer() = count;
            if count >= CRY_CYCLES {
                *BabyMetroidEdit::of(game, k).cry_timer() = 0;
                let effect = if BabyMetroid::of(game, k).palette_handler_delay() < DRAINING_DELAY {
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
