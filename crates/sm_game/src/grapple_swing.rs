//! Grapple controller, swing placement, rope-length changes and swept block contact.
#![forbid(unsafe_code)]
use crate::actors::{self, api as actor_api};
use crate::combat_rules::GrappleOutcome;
use crate::constants::{buttons::*, samus_movement::*, samus_poses::*};
use crate::grapple_aim::{self, ConnectionSet};
use crate::grapple_origins::{self, OriginSet};
use crate::player_collision::operations as player_collision;
use crate::player_transitions as pt;
use crate::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::projectile_room::Axis;
use crate::ram::RawRam as _;
use crate::ram::{word, words as ram_words};
use crate::runtime_access::RuntimeAccess;
use crate::word_transfers;

/// What the grapple controller needs from the runtime besides named RAM,
/// sound, the cartridge images and the owned-Session view.
pub trait Host:
    crate::ram::CompatibilityRam
    + crate::ram::WorkRamAccess
    + crate::effects::SoundEffects
    + crate::cartridge::NativeImage
    + crate::cartridge::LoadedImage
    + crate::menus::host::MenuRuntime
    + crate::records::Records
    + RuntimeAccess
{
    /// Multiply through the hardware scratch words at $05E9/$05EB.
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32;
    /// Samus's bottom edge under the current pose.
    fn player_bottom(&self) -> u16;
    /// Settle the owned grapple state after the beam connects or releases.
    fn settle_grapple(&mut self);
}

fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

#[allow(non_snake_case)]
fn Mult8x8(a: u8, b: u8) -> u16 {
    (a as i32 * b as i32) as u16
}

/// Follow extension blocks until the selected reaction finishes; callbacks can
/// change the live block index before requesting another reaction.
fn react_to_block<H: Host + ?Sized>(game: &mut H, mut react: impl FnMut(u16, &mut H) -> u8) -> u8 {
    loop {
        let offset = 0x10002 + usize::from(word!(game.cur_block_index)) * 2;
        let block = u16::from_le_bytes([
            game.compatibility_byte(offset),
            game.compatibility_byte(offset + 1),
        ]);
        let result = react(block >> 12, game);
        if result & 0x80 == 0 {
            return result;
        }
    }
}

fn project_axis(origin: u16, sample: i16, length: u16) -> u16 {
    let distance = if sample == -256 || sample == 256 {
        length
    } else {
        (u16::from(length as u8) * u16::from(sample.unsigned_abs() as u8)) >> 8
    };
    if sample < 0 {
        origin.wrapping_sub(distance)
    } else {
        origin.wrapping_add(distance)
    }
}
fn place_sample<H: Host + ?Sized>(angle: u16, length: u16, game: &mut H) {
    {
        let index = (angle >> 1) as usize;
        if word!(game.grapple_beam_flags) & 0x8000 == 0 {
            let offset = if kSinCosTable8bit_Sext[index + 64] < 0 {
                7
            } else {
                8
            };
            word!(
                game.grapple_beam_end_x_pos = word!(game.grapple_beam_end_x_pos) & 0xfff0 | offset
            );
            let offset = if kSinCosTable8bit_Sext[index] < 0 {
                7
            } else {
                8
            };
            word!(
                game.grapple_beam_end_y_pos = word!(game.grapple_beam_end_y_pos) & 0xfff0 | offset
            );
        }
        let x = project_axis(
            word!(game.grapple_beam_end_x_pos),
            kSinCosTable8bit_Sext[index + 64],
            length,
        );
        word!(game.grapple_beam_grapple_start_x = x);
        word!(game.grapple_beam_grapple_start_block_x = (x >> 4) as u8 as u16);
        let y = project_axis(
            word!(game.grapple_beam_end_y_pos),
            kSinCosTable8bit_Sext[index],
            length,
        );
        word!(game.grapple_beam_grapple_start_y = y);
        word!(game.grapple_beam_grapple_start_block_y = (y >> 4) as u8 as u16);
    }
}
fn contact<H: Host + ?Sized>(game: &mut H) -> u8 {
    let x = word!(game.grapple_beam_grapple_start_block_x);
    let row = Mult8x8(
        word!(game.grapple_beam_grapple_start_block_y) as u8,
        word!(game.room_width_in_blocks) as u8,
    );
    word!(game.cur_block_index = row.wrapping_add(x));
    react_to_block(game, |block_type, game| match block_type {
        0 | 3 | 4 | 6 | 7 => 0,
        1 | 8 | 9 | 11 | 12 | 14 | 15 => 1,
        2 => spike_contact(false, game),
        5 => crate::player_collision::grapple::redirect_block(&mut game.runtime(), Axis::X),
        10 => spike_contact(true, game),
        13 => crate::player_collision::grapple::redirect_block(&mut game.runtime(), Axis::Y),
        _ => unreachable!(),
    })
}
static AIR_DAMAGE: [u16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static SOLID_DAMAGE: [u16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn spike_contact<H: Host + ?Sized>(solid: bool, game: &mut H) -> u8 {
    if word!(game.samus_invincibility_timer) == 0 {
        // The original reads a word spanning adjacent BTS bytes, not just one byte.
        let at = 0x16402 + usize::from(word!(game.cur_block_index));
        let kind = i16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
        if kind >= 0 {
            let damage = if solid {
                SOLID_DAMAGE[kind as usize]
            } else {
                AIR_DAMAGE[kind as usize]
            };
            let amount = ((damage as i16 as i32) << 16) as u32;
            if amount != 0 {
                let low =
                    u16::from_ne_bytes([game.ram().read_byte(0xa4e), game.ram().read_byte(0xa4f)])
                        as u32;
                let high =
                    u16::from_ne_bytes([game.ram().read_byte(0xa50), game.ram().read_byte(0xa51)])
                        as u32;
                let sum = (low | (high << 16)).wrapping_add(amount);
                word_transfers!(game, ascending, 1;
                    native(0xa4e) <- value(sum as u16),
                    native(0xa50) <- value((sum >> 16) as u16));
                word! {game; samus_invincibility_timer = 60; samus_knockback_timer = 10; }
            }
        }
    }
    solid as u8
}
fn check_clearance<H: Host + ?Sized>(angle: u16, game: &mut H) -> u8 {
    word!(game.g_word_7E0D98 = 6);
    let mut length = word!(game.grapple_beam_length).wrapping_add(8);
    loop {
        place_sample(angle, length, game);
        let blocked = contact(game) & 1;
        if blocked != 0 {
            return blocked;
        }
        length = length.wrapping_add(8);
        if word!(game.g_word_7E0D98 -= 1, reread) == 0 {
            return 0;
        }
    }
}
fn dispatch_connection<H: Host + ?Sized>(ea: u32, game: &mut H) {
    let (pose, transition_graphics) = match ea {
        10205657 => (kPose_B2_FaceR_Grapple_Air as u16, 9),
        10205666 => (kPose_B3_FaceL_Grapple_Air as u16, 9),
        10205674 => (kPose_A8_FaceR_Grappling as u16, 10),
        10205683 => (kPose_AA_FaceR_Grappling_AimDR as u16, 10),
        10205692 => (kPose_AB_FaceL_Grappling_AimDL as u16, 10),
        10205701 => (kPose_A9_FaceL_Grappling as u16, 10),
        10205710 => (kPose_B4_FaceR_Grappling_Crouch as u16, 10),
        10205719 => (kPose_B6_FaceR_Grappling_Crouch_AimDR as u16, 10),
        10205728 => (kPose_B7_FaceL_Grappling_Crouch_AimDL as u16, 10),
        10205737 => (kPose_B5_FaceL_Grappling_Crouch as u16, 10),
        _ => {
            game.rtl_unreachable();
            return;
        }
    };
    begin_connection(pose, transition_graphics, game);
}
pub fn connect_beam<H: Host + ?Sized>(game: &mut H) {
    if game.ram().read_byte(0xa1f) as i32 == kMovementType_1A_GrabbedByDraygon as i32 {
        word!(game.grapple_beam_function = (0x9bc77ei32) as u16);
        word!(game.grapple_beam_length_delta = 0);
    } else {
        let v1: i32 = 2 * word!(game.grapple_beam_direction) as i32;
        let set = if word!(game.samus_y_speed) != 0 || word!(game.samus_y_subspeed) != 0 {
            ConnectionSet::Airborne
        } else if game.ram().read_byte(0xa1f) == 5 {
            ConnectionSet::Crouching
        } else {
            ConnectionSet::Standard
        };
        let connection = grapple_aim::connection(game, set, v1 as isize);
        word!(game.grapple_beam_function = connection.beam_function);
        dispatch_connection(u32::from(connection.handler) | 0x9b0000, game);
    };
}
// Each field reacquires the registered native image at its original read point.
macro_rules! special_angle_word {
    ($game:ident, $row:ident, $offset:literal) => {
        $game.native_image().table_word(
            sm_data::cartridge::lorom_offset(0x9bc43e) as usize + 10 * usize::from($row) + $offset,
            0,
        )
    };
}
macro_rules! clamp_previous_grapple_position {
    ($game:ident, $current:ident, $previous:ident) => {
        let delta: i16 = (word!($game.$current) as i16).wrapping_sub(word!($game.$previous) as i16);
        if (word!($game.$current) as i16).wrapping_sub(word!($game.$previous) as i16) < 0 {
            if delta as i32 + 12 & 0x8000 != 0 {
                word!($game.$previous = word!($game.$current).wrapping_add(12));
            }
        } else if delta as i32 - 13 & 0x8000 == 0 {
            word!($game.$previous = word!($game.$current).wrapping_sub(12));
        }
    };
}
pub fn apply_special_angle<H: Host + ?Sized>(game: &mut H) -> u8 {
    let mut row: u16 = 7;
    while word!(game.grapple_beam_end_angle16) != special_angle_word!(game, row, 0) {
        row = row.wrapping_sub(1);
        if row & 0x8000 != 0 {
            return 0;
        }
    }
    word! { game;
        samus_new_pose_interrupted = special_angle_word!(game, row, 2);
        samus_x_pos = word!(game.grapple_beam_end_x_pos)
            .wrapping_add(special_angle_word!(game, row, 4));
        samus_y_pos = word!(game.grapple_beam_end_y_pos)
            .wrapping_add(special_angle_word!(game, row, 6));
        grapple_beam_function = special_angle_word!(game, row, 8);
    }
    word! { game, 0; samus_special_transgfx_index, slow_grabble_scrolling_flag }
    clamp_previous_grapple_position!(game, samus_x_pos, samus_prev_x_pos);
    clamp_previous_grapple_position!(game, samus_y_pos, samus_prev_y_pos);
    1
}

fn begin_connection<H: Host + ?Sized>(pose: u16, transition_graphics: u16, game: &mut H) {
    word!(game.samus_new_pose_interrupted = pose);
    word!(game.samus_special_transgfx_index = transition_graphics);
    let angle = u16::swap_bytes(actor_api::angle_from_delta(
        (game.player_x()).wrapping_sub(word!(game.grapple_beam_end_x_pos)),
        (game.player_y()).wrapping_sub(word!(game.grapple_beam_end_y_pos)),
    ));
    word!(game.grapple_beam_end_angle16 = angle);
    word!(game.grapple_beam_end_angles_mirror = angle);
    word!(game.grapple_beam_length_delta = 0);
    if word!(game.grapple_beam_length) as i32 - 64i32 & 0x8000 == 0 {
        word!(game.grapple_beam_length, wrapping_sub(24));
    }
    place_rope_origin(game);
}

pub fn place_rope_origin<H: Host + ?Sized>(game: &mut H) {
    let angle = 2 * game.ram().read_byte(0xcfb) as u16;
    let length = word!(game.grapple_beam_length);
    place_sample(angle, length, game);
    word!(game.x_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_x));
    word!(game.y_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_y));
}
pub fn change_length<H: Host + ?Sized>(game: &mut H) -> u8 {
    if word!(game.grapple_beam_length_delta) == 0 {
        return 0;
    }
    let mut end =
        word!(game.grapple_beam_length_delta).wrapping_add(word!(game.grapple_beam_length));
    let shortening = word!(game.grapple_beam_length_delta) & 0x8000 != 0;
    if shortening && end < 8 {
        word!(game.grapple_beam_length_delta = 0);
        end = 8;
    } else if !shortening && end >= 63 {
        word!(game.grapple_beam_length_delta = 0);
        end = 63;
    }
    let angle = 2 * game.ram().read_byte(0xcfb) as u16;
    let mut length = word!(game.grapple_beam_length);
    while length != end {
        let sample_length = length.wrapping_add(if shortening { 7 } else { 57 });
        place_sample(angle, sample_length, game);
        if contact(game) != 0 {
            word!(game.grapple_beam_length = length);
            return 1;
        }
        length = if shortening {
            length.wrapping_sub(1)
        } else {
            length.wrapping_add(1)
        };
    }
    word!(game.grapple_beam_length = end);
    0
}
pub fn apply_input<H: Host + ?Sized>(game: &mut H) {
    if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
        if word!(game.grapple_beam_length) != 0 {
            word!(game.grapple_beam_length_delta = (-2i16) as u16);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        if (i32::from(word!(game.grapple_beam_length)) - 64) & 0x8000 != 0 {
            word!(game.grapple_beam_length_delta = 2);
        } else {
            word!(game.grapple_beam_length = 64);
        }
    }
    let angle = u16::from(game.ram().read_byte(0xcfb)) << 8;
    if (0x4000..0xc000).contains(&angle) {
        let direction = if word!(game.joypad1_lastkeys) & 0x200 != 0 {
            1i16
        } else if word!(game.joypad1_lastkeys) & 0x100 != 0 {
            -1
        } else {
            0
        };
        if direction != 0 {
            if angle == 0x8000 && word!(game.grapple_beam_unkD26) == 0 {
                word!(game.grapple_beam_unkD26 = (direction * 256) as u16);
            }
            let acceleration =
                if word!(game.grapple_beam_flags) != 0 && word!(game.grapple_beam_flags) & 1 != 0 {
                    6
                } else {
                    12
                };
            word!(game.grapple_beam_unkD2A = (direction * acceleration) as u16);
            return;
        }
    }
    word!(game.grapple_beam_unkD2A = 0);
}

pub fn apply_gravity<H: Host + ?Sized>(game: &mut H) {
    let angle = word!(game.grapple_beam_end_angle16);
    if angle & 0xff00 == 0x8000 {
        word! { game, 0; grapple_beam_unkD28, grapple_beam_unkD2C }
        if abs16(word!(game.grapple_beam_unkD26)).to_ne_bytes()[1] == 0 {
            word!(game.grapple_beam_unkD26 = 0);
        }
        return;
    }
    let direction = if angle & 0x8000 != 0 { -1i16 } else { 1 };
    let shift = if matches!(angle & 0xc000, 0 | 0xc000) {
        2
    } else {
        0
    };
    word!(game.grapple_beam_unkD2C = (direction * (5 >> shift)) as u16);
    let shift = shift
        + u32::from(word!(game.grapple_beam_flags) != 0 && word!(game.grapple_beam_flags) & 1 != 0);
    word!(game.grapple_beam_unkD28 = (direction * (24 >> shift)) as u16);
}

pub fn accelerate<H: Host + ?Sized>(game: &mut H) {
    word!(
        checked_i32,
        game.grapple_beam_unkD26 +=
            (word!(game.grapple_beam_unkD2A) as i32 + word!(game.grapple_beam_unkD28) as i32)
    );
    if (word!(game.grapple_beam_unkD26) ^ word!(game.grapple_beam_end_angle16)) & 0x8000 != 0 {
        word!(game.grapple_beam_unkD26 += word!(game.grapple_beam_unkD2C));
    }
    let velocity = word!(game.grapple_beam_unkD26) as i16;
    if velocity.unsigned_abs() >= 0x480 {
        word!(game.grapple_beam_unkD26 = velocity.clamp(-0x480, 0x480) as u16);
    }
}

pub fn rebound_jump<H: Host + ?Sized>(game: &mut H) {
    if word!(game.grapple_beam_unkD30) == 0
        || word!(game.button_config_jump_a) & word!(game.joypad1_newkeys) == 0
    {
        return;
    }
    let velocity = word!(game.grapple_beam_unkD26);
    let impulse = if velocity == 0 {
        0
    } else {
        let magnitude =
            if word!(game.grapple_beam_flags) != 0 && word!(game.grapple_beam_flags) & 1 != 0 {
                0x180u16
            } else {
                0x300
            };
        if velocity & 0x8000 != 0 {
            magnitude.wrapping_neg()
        } else {
            magnitude
        }
    };
    word!(game.grapple_beam_unkD2E = impulse);
}

pub fn advance_swing<H: Host + ?Sized>(game: &mut H) -> u8 {
    let scale = if word!(game.grapple_beam_flags) & 1 != 0 {
        160
    } else {
        256
    };
    let velocity = word!(game.grapple_beam_unkD2E).wrapping_add(word!(game.grapple_beam_unkD26));
    let backwards = velocity & 0x8000 != 0;
    let magnitude = if backwards {
        velocity.wrapping_neg()
    } else {
        velocity
    };
    let distance = (game.multiply_with_scratch(magnitude, scale) >> 8) as u16;
    if distance == 0 {
        return 0;
    }
    let delta = if backwards {
        distance.wrapping_neg()
    } else {
        distance
    };
    let target = 2 * (word!(game.grapple_beam_end_angle16).wrapping_add(delta) >> 8);
    let mut angle = 2 * game.ram().read_byte(0xcfb) as u16;
    while angle != target {
        let next = (if backwards {
            angle.wrapping_sub(2)
        } else {
            angle.wrapping_add(2)
        }) & 0x1ff;
        if check_clearance(next, game) & 1 != 0 {
            let stopped = ((angle >> 1) << 8) | 0x80;
            word!(game.grapple_beam_end_angle16 = stopped);
            word!(game.grapple_beam_end_angles_mirror = stopped);
            if (word!(game.g_word_7E0D98) == 6 || word!(game.g_word_7E0D98) == 5)
                && word!(game.grapple_beam_length) == 8
            {
                word!(game.grapple_beam_unkD36 |= 0x8000);
                word! {game, 0; grapple_beam_unkD26, grapple_beam_unkD2E}
            } else {
                word! {game;
                    grapple_beam_unkD30 = 16;
                    grapple_beam_unkD26 = -(word!(game.grapple_beam_unkD26) as i16 as i32 >> 1) as u16;
                    grapple_beam_unkD2E = -(word!(game.grapple_beam_unkD2E) as i16 as i32 >> 1) as u16;
                }
            }
            return 1;
        }
        angle = next;
    }
    word!(game.grapple_beam_end_angle16 += delta);
    word!(game.grapple_beam_end_angles_mirror = word!(game.grapple_beam_end_angle16));
    word!(game.grapple_beam_unkD36 &= !0x8000);
    if word!(game.grapple_beam_unkD30 -= 1, reread) & 0x8000 != 0 {
        word!(game.grapple_beam_unkD30 = 0);
    }
    let impulse = word!(game.grapple_beam_unkD2E) as i16;
    let decayed = if impulse >= 0 {
        impulse.wrapping_sub(6).max(0)
    } else {
        impulse.wrapping_add(6).min(0)
    };
    word!(game.grapple_beam_unkD2E = decayed as u16);
    0
}

pub fn launch_from_swing<H: Host + ?Sized>(game: &mut H) {
    let backwards = word!(game.grapple_beam_unkD26) & 0x8000 != 0;
    let magnitude = if backwards {
        (-2i32 * i32::from(word!(game.grapple_beam_unkD26))) as u16
    } else {
        2u16.wrapping_mul(word!(game.grapple_beam_unkD26))
    };
    let sample = kSinCosTable8bit_Sext[usize::from(game.ram().read_byte(0xcfb)) + 64];
    let vertical = game.multiply_with_scratch(sample.unsigned_abs(), magnitude);
    word_transfers!(game, ascending, 1;
        native(ram_words::samus_y_subspeed) <- value(vertical as u16),
        native(ram_words::samus_y_speed) <- value((vertical >> 16) as u16));
    word!(game.samus_y_dir = if (sample < 0) == backwards { 2 } else { 1 });
    word!(game.samus_x_accel_mode = 2);
    let adjustment = 64u16.wrapping_sub((3 * (i32::from(magnitude) >> 9)) as u16);
    let sample = abs16(
        kSinCosTable8bit_Sext
            [usize::from(game.ram().read_byte(0xcfb).wrapping_sub(adjustment as u8)) + 64]
            as u16,
    );
    let horizontal = game.multiply_with_scratch(sample, magnitude);
    word_transfers!(game, ascending, 1;
        native(ram_words::samus_x_base_subspeed) <- value(horizontal as u16),
        native(ram_words::samus_x_base_speed) <- value((horizontal >> 16) as u16));
}

macro_rules! initialize_grapple_endpoints {
    ($game:ident, $set:ident, $pose_index:ident, $y_adjustment:ident $(,)?) => {
        let origin = grapple_origins::origin($game, OriginSet::$set, $pose_index as isize);
        word! { $game;
            grapple_beam_origin_x_offset = origin.x;
            grapple_beam_origin_y_offset = origin.y.wrapping_sub($y_adjustment);
            grapple_beam_end_x_pos = origin.x.wrapping_add($game.player_x());
            x_pos_of_start_of_grapple_beam = word!($game.grapple_beam_end_x_pos);
            x_pos_of_start_of_grapple_beam_prevframe =
                origin.previous_x.wrapping_add($game.player_x());
            grapple_beam_end_y_pos = (origin.y as i32 + $game.player_y() as i32
                - $y_adjustment as i32) as u16;
            y_pos_of_start_of_grapple_beam = word!($game.grapple_beam_end_y_pos);
            y_pos_of_start_of_grapple_beam_prevframe = (origin.previous_y as i32
                + $game.player_y() as i32
                - $y_adjustment as i32) as u16;
        }
    };
}

fn draygon_direction<H: Host + ?Sized>(pose: u16, game: &mut H) -> u16 {
    let (toward, down, up, straight) = if pose as i32 == kPose_BE_FaceL_Draygon_Move as i32 {
        (kButton_Left, 6, 8, 7)
    } else {
        (kButton_Right, 3, 1, 2)
    };
    if word!(game.joypad1_lastkeys) as i32 & toward as i32 == 0 {
        return straight;
    }
    if word!(game.joypad1_lastkeys) as i32 & kButton_Down as i32 != 0 {
        return down;
    }
    if word!(game.joypad1_lastkeys) as i32 & kButton_Up as i32 != 0 {
        return up;
    }
    straight
}
// Capture the native allocation before evaluating each fresh byte offset.
macro_rules! native_grapple_byte {
    ($game:ident, $address:literal, $offset:expr) => {
        $game
            .native_image()
            .read(sm_data::cartridge::lorom_offset($address) as usize + $offset)
    };
}
macro_rules! native_pose_parameter {
    ($game:ident, $offset:literal) => {
        native_grapple_byte!(
            $game,
            0x91b629,
            8 * usize::from(word!($game.samus_pose)) + $offset
        )
    };
}
macro_rules! place_samus_relative_to_grapple_origin {
    ($game:ident, $offset_table:literal, $offset_index:ident) => {
        word! { $game;
            samus_x_pos = (word!($game.x_pos_of_start_of_grapple_beam)).wrapping_add(
                native_grapple_byte!($game, $offset_table, usize::from($offset_index)) as i8 as u16,
            );
            samus_y_pos = (word!($game.y_pos_of_start_of_grapple_beam)).wrapping_add(
                native_grapple_byte!($game, $offset_table, usize::from($offset_index) + 1) as i8 as u16,
            );
        }
    };
}

macro_rules! restore_grapple_start_positions {
    ($pose_offset:ident, $game:ident, $set:ident, $y_adjustment:ident $(,)?) => {
        let v1: i32 = $pose_offset as i32 >> 1;
        let origin = grapple_origins::origin($game, OriginSet::$set, v1 as isize);
        word! { $game;
            x_pos_of_start_of_grapple_beam = origin.x.wrapping_add($game.player_x());
            x_pos_of_start_of_grapple_beam_prevframe =
                origin.previous_x.wrapping_add($game.player_x());
            y_pos_of_start_of_grapple_beam = (origin.y as i32 + $game.player_y() as i32
                - $y_adjustment as i32) as u16;
            y_pos_of_start_of_grapple_beam_prevframe = (origin.previous_y as i32
                + $game.player_y() as i32
                - $y_adjustment as i32) as u16;
        }
    };
}

pub fn position_swing<H: Host + ?Sized>(game: &mut H) {
    let v0: u16 = abs16(word!(game.grapple_beam_unkD26));
    let mut v1: u16 = 0;
    let start_grapple_animation: bool = if v0 as i32 - 64i32 & 0x8000 == 0 {
        word!(game.slow_grabble_scrolling_flag = 1);
        true
    } else {
        word!(game.slow_grabble_scrolling_flag = 0);
        (((game.ram().read_byte(0xcfb) as i32) << 8) != 0x8000)
            || ({
                if word!(game.samus_anim_frame) as i32 - 64i32 & 0x8000 != 0 {
                    word! {game; samus_anim_frame_timer = 8; samus_anim_frame = 64; }
                }
                v1 = native_grapple_byte!(game, 0x9bc1c2, usize::from(game.ram().read_byte(0xcfd)))
                    as u16;
                false
            })
    };
    if start_grapple_animation {
        word!(game.samus_anim_frame_timer = 15);
        v1 = native_grapple_byte!(game, 0x9bc1c2, usize::from(game.ram().read_byte(0xcfd))) as u16;
        word!(game.samus_anim_frame = v1);
    }
    let v2: u16 = (2u16).wrapping_mul(v1);
    if (abs16(word!(game.grapple_beam_unkD2E)) & 0xff00) == 256 {
        let mut v3: u16 = word!(game.samus_anim_frame);
        if word!(game.samus_anim_frame) as i32 - 64i32 & 0x8000 == 0 {
            v3 = 16;
        }
        word!(game.samus_anim_frame = (v3).wrapping_add(32));
    }
    if game.ram().read_byte(0xa1e) == 4 {
        place_samus_relative_to_grapple_origin!(game, 0x9bc2c2, v2);
    } else {
        place_samus_relative_to_grapple_origin!(game, 0x9bc302, v2);
    }
    word! { game;
        x_pos_of_start_of_grapple_beam_prevframe = word!(game.x_pos_of_start_of_grapple_beam);
        y_pos_of_start_of_grapple_beam_prevframe = word!(game.y_pos_of_start_of_grapple_beam);
    }
    clamp_swing_position(game);
}

pub fn clamp_swing_position<H: Host + ?Sized>(game: &mut H) {
    let mut v0: i16 = ((game.player_x()) as i16).wrapping_sub((game.previous_player_x()) as i16);
    pt::clamp_previous_position(
        crate::session_access::SessionAccess::session_mut(&mut game.runtime()),
        v0,
    );
}

pub fn position_anchored<H: Host + ?Sized>(game: &mut H) {
    let v0: i32 = word!(game.grapple_beam_direction) as i32;
    let origin = grapple_origins::origin(game, OriginSet::Standard, v0 as isize);
    word! { game;
        samus_x_pos = (word!(game.x_pos_of_start_of_grapple_beam)).wrapping_sub(origin.x);
        x_pos_of_start_of_grapple_beam_prevframe =
            origin.previous_x.wrapping_add(game.player_x());
        samus_y_pos = (word!(game.y_pos_of_start_of_grapple_beam)).wrapping_sub(origin.y);
        y_pos_of_start_of_grapple_beam_prevframe =
            origin.previous_y.wrapping_add(game.player_y());
    }
}

pub fn restore_start_positions<H: Host + ?Sized>(game: &mut H) {
    let r22: u16 = native_pose_parameter!(game, 4) as u16;
    let v0: u16 = (2u16).wrapping_mul(word!(game.grapple_beam_direction));
    if word!(game.samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
        || word!(game.samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
        || game.ram().read_byte(0xa1f) != 1
    {
        restore_grapple_start_positions!(v0, game, Standard, r22);
    } else {
        restore_grapple_start_positions!(v0, game, Running, r22);
    };
}

static kIsGrappleBannedForMovementType: [u8; 28] = [
    0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1,
];
pub fn cancel_incompatible_pose<H: Host + ?Sized>(game: &mut H) {
    if kIsGrappleBannedForMovementType[game.ram().read_byte(0xa1f) as usize] == 0 {
        if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
            && word!(game.grapple_beam_function) as i32 + 0x3882 & 0x8000 != 0
        {
            let mut v0: i16 = native_pose_parameter!(game, 3) as i16;
            if (v0 as u16 & 0xf0) == 0 {
                if v0 as i32 == word!(game.grapple_beam_direction) as i32 {
                    return;
                }
                if word!(game.grapple_varCF6) != 0 {
                    game.queue_sound(0, (7) as u8, 6);
                    word!(game.grapple_beam_function = (0x9bc51ei32) as u16);
                    return;
                }
            }
        } else {
            return;
        }
    }
    if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
        word!(game.grapple_beam_function = (0x9bc856i32) as u16);
    }
}

pub fn start_firing<H: Host + ?Sized>(game: &mut H) {
    let mut r22: u16 = 0;
    let mut v0: i32 = 0;
    if word!(game.samus_pose) as i32 == kPose_F0_FaceR_Draygon_Move as i32
        || word!(game.samus_pose) as i32 == kPose_BE_FaceL_Draygon_Move as i32
    {
        v0 = draygon_direction(word!(game.samus_pose), game) as i32;
        r22 = 6;
    } else {
        r22 = native_pose_parameter!(game, 4) as u16;
        v0 = native_pose_parameter!(game, 3) as i32;
        if (v0 & 0xf0) != 0 {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
            return;
        }
    }
    word!(game.grapple_beam_direction = v0 as u16);
    let v1: i32 = v0;
    let extension = grapple_aim::extension(game, v1 as isize);
    word! { game;
        grapple_beam_extension_x_velocity = extension.x_velocity;
        grapple_beam_extension_y_velocity = extension.y_velocity;
        grapple_beam_end_angle16 = extension.angle;
        grapple_beam_end_angles_mirror = word!(game.grapple_beam_end_angle16);
        grapple_varCF6 = 10;
    }
    if word!(game.samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
        || word!(game.samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
        || game.ram().read_byte(0xa1f) != 1
    {
        initialize_grapple_endpoints!(game, Standard, v1, r22);
    } else {
        initialize_grapple_endpoints!(game, Running, v1, r22);
    }
    word! { game, 0;
        grapple_beam_end_x_subpos, grapple_beam_end_y_subpos, grapple_beam_end_x_suboffset,
        grapple_beam_end_x_offset, grapple_beam_end_y_suboffset, grapple_beam_end_y_offset,
        grapple_beam_flags,
    }
    word!(game.grapple_beam_length_delta = 12);
    word! { game, 0;
        grapple_beam_length, grapple_beam_unkD26, grapple_beam_unkD28, grapple_beam_unkD2A,
        grapple_beam_unkD2C, grapple_beam_unkD2E, grapple_beam_unkD30, grapple_beam_unkD1E,
        grapple_beam_unkD20,
    }
    word! {game; grapple_beam_unkD3A = 2; grapple_beam_unkD3C = 0;
    grapple_point_anim_timer = 5;
    grapple_point_anim_ptr = grapple_aim::point_animation(game).first; }
    word! { game, 0;
        grapple_beam_grapple_start_x, grapple_beam_unkD38, grapple_beam_unkD36,
        slow_grabble_scrolling_flag,
    }
    start_rope_animation(game);
    word! {game; samus_draw_handler = ((0x90eb86i32) as u16); grapple_walljump_timer = 0; }
    crate::samus_palette::write_beam_palette(&mut game.runtime(), 2);
    word! {game; palette_buffer[223] = 32657; grapple_beam_function = ((0x9bc703i32) as u16); }
    game.queue_sound(0, (5) as u8, 1);
    word! {game; flare_counter = 1; play_resume_charging_beam_sfx = 0; }
    if word!(game.samus_movement_handler) as i32 == 0x90946e & 0xffff {
        word!(game.samus_movement_handler = (0x90a337i32) as u16);
    }
}

pub fn cancel<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, (7) as u8, 15);
    if game.ram().read_byte(0xa1f) == kMovementType_16_Grappling as u8 {
        pt::request_fallback_pose(&mut game.runtime());
    } else {
        game.player_command(0x1c);
    }
    finish_release(game);
}

pub fn release_stationary<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, (7) as u8, 15);
    let pose = if word!(game.samus_pose) == kPose_B2_FaceR_Grapple_Air as u16 {
        kPose_01_FaceR_Normal as u16
    } else if word!(game.samus_pose) == kPose_B3_FaceL_Grapple_Air as u16 {
        kPose_02_FaceL_Normal as u16
    } else {
        let crouching = (i32::from(word!(game.samus_y_radius)) - 17) & 0x8000 != 0;
        if native_pose_parameter!(game, 3) & 0xf0 != 0 {
            match (crouching, game.ram().read_byte(0xa1e) == 4) {
                (true, true) => kPose_28_FaceL_Crouch as u16,
                (true, false) => kPose_27_FaceR_Crouch as u16,
                (false, true) => kPose_02_FaceL_Normal as u16,
                (false, false) => kPose_01_FaceR_Normal as u16,
            }
        } else {
            let table = if crouching { 0x9bc9c4 } else { 0x9bc9ba };
            // Capture the destination image before the fresh native pose-byte read.
            u16::from(game.native_image().read(
                sm_data::cartridge::lorom_offset(table) as usize
                    + usize::from(native_pose_parameter!(game, 3)),
            ))
        }
    };
    word!(game.samus_new_pose_transitional = pose);
    word! {game; samus_hurt_switch_index = 0; input_to_pose_calc = 1; }
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    finish_release(game);
}

pub fn release_walljump<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, (7) as u8, 15);
    if game.ram().read_byte(0xa1e) == 8 {
        word!(game.samus_new_pose_transitional = kPose_84_FaceL_Walljump as u16);
    } else {
        word!(game.samus_new_pose_transitional = kPose_83_FaceR_Walljump as u16);
    }
    word!(game.samus_hurt_switch_index = 6);
    word! { game, 0;
        samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
        UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed, samus_y_dir,
        UNUSED_word_7E0B38, samus_x_base_speed, samus_x_base_subspeed,
    }
    finish_release(game);
}

pub fn release_swing<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, (7) as u8, 15);
    if word!(game.grapple_beam_unkD26) & 0x8000 == 0 {
        word!(game.samus_new_pose_transitional = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
    } else {
        word!(game.samus_new_pose_transitional = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
    }
    word!(game.samus_hurt_switch_index = 7);
    finish_release(game);
}

fn finish_release<H: Host + ?Sized>(game: &mut H) {
    word! { game, 0;
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
        grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    let beams = word!(game.equipped_beams) & 0xfff;
    crate::samus_palette::write_beam_palette(&mut game.runtime(), beams);
    word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    game.settle_grapple();
    if word!(game.samus_auto_cancel_hud_item_index) != 0 {
        word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}

pub fn react_beam<H: Host + ?Sized>(game: &mut H) -> u8 {
    crate::player_collision::grapple::react_beam(&mut game.runtime())
}
pub fn extend_beam<H: Host + ?Sized>(game: &mut H) -> u8 {
    crate::player_collision::grapple::extend_beam(&mut game.runtime())
}
pub fn start_rope_animation<H: Host + ?Sized>(game: &mut H) {
    crate::grapple_rope::start(&mut game.runtime());
}
pub fn draw_rope<H: Host + ?Sized>(game: &mut H) {
    crate::grapple_rope::draw(&mut game.runtime());
}

fn connected_to_block<H: Host + ?Sized>(game: &mut H) -> bool {
    word! { game, 0;
        grapple_beam_extension_x_velocity, grapple_beam_extension_y_velocity,
    }
    let connected = react_beam(game) & 1 != 0;
    if connected {
        word!(game.samus_grapple_flags = 1);
    }
    connected
}

pub fn update_tiles<H: Host + ?Sized>(game: &mut H) {
    if (word!(game.grapple_point_anim_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.grapple_point_anim_timer = 5);
        word!(game.grapple_point_anim_ptr += 512);
        if ((word!(game.grapple_point_anim_ptr)) as i16)
            .wrapping_sub(grapple_aim::point_animation(game).end as i16)
            >= 0
        {
            word!(game.grapple_point_anim_ptr = grapple_aim::point_animation(game).first);
        }
    }
    let cursor = game.queue_native_transfer(
        word!(game.vram_write_queue_tail),
        32,
        &|game: &H| word!(game.grapple_point_anim_ptr),
        0x9a,
        &|_: &H| 0x6200,
    );
    let index = (game.ram().read_byte(0xcfb) >> 1 & 0xfe) >> 1;
    game.queue_native_transfer(
        cursor,
        128,
        &|game: &H| crate::cartridge::rom_table_word!(game, 0x9bc346, isize::from(index)),
        0x9a,
        &|_: &H| 0x6210,
    );
    if word!(game.flare_counter) as i32 - 120i32 & 0x8000 != 0 {
        word!(game.flare_counter += 1);
    }
}

pub fn update<H: Host + ?Sized>(game: &mut H) {
    let mut r18: u16 = 0;
    if word!(game.grapple_varCF6) != 0 {
        word!(game.grapple_varCF6 -= 1);
    }
    word!(game.samus_grapple_flags &= !(1));
    cancel_incompatible_pose(game);
    dispatch(
        ((word!(game.grapple_beam_function)) as u32) | (0x9b0000u32),
        game,
    );
    if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
        && word!(game.grapple_beam_function) as i32 - (0x9bc856 & 0xffff) & 0x8000 != 0
        && (word!(game.samus_suit_palette_index) & 4) == 0
        && word!(game.fx_type) != 0
        && {
            r18 = game.player_bottom();
            (word!(game.fx_y_pos) & 0x8000) == 0
        }
        && word!(game.fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
    {
        word!(game.grapple_beam_flags |= (1u16));
    } else {
        word!(game.grapple_beam_flags &= !(1));
    };
}

fn shoot_held<H: Host + ?Sized>(game: &H) -> bool {
    word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys) != 0
}

fn dispatch<H: Host + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        0x9bc4f0 => {
            if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0
                || (word!(game.button_config_shoot_x) & word!(game.joypad1_newinput_samusfilter))
                    != 0
            {
                start_firing(game);
            } else if word!(game.flare_counter) != 0 {
                word! { game, 0;
                    flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
                    flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
                    flare_fast_sparks_anim_timer,
                }
                crate::samus_palette::load_suit(
                    &mut game.runtime(),
                    crate::visual_state::PaletteBuffer::Current,
                );
            }
        }
        0x9bc703 => {
            let shooting = shoot_held(game);
            if shooting {
                word!(game.grapple_beam_length += word!(game.grapple_beam_length_delta));
            }
            if !shooting || word!(game.grapple_beam_length) as i32 - 128 & 0x8000 == 0 {
                word!(game.grapple_beam_function = 0xc856);
                return;
            }
            let contact = actors::contacts::grapple_contact(&mut game.runtime());
            let outcome = match contact {
                Some(contact) => actors::combat::resolve_grapple(&mut game.runtime(), contact),
                None => GrappleOutcome::Miss,
            };
            let attached = match outcome {
                GrappleOutcome::Attach => true,
                GrappleOutcome::Cancel => false,
                GrappleOutcome::Miss => {
                    let v2: u8 = extend_beam(game);
                    if (v2 & 1) == 0 {
                        return;
                    }
                    v2 & 0x40 != 0
                }
            };
            if !attached {
                word!(game.grapple_beam_function = 0xc856);
                return;
            }
            game.queue_sound(0, (6) as u8, 6);
            connect_beam(game);
            word!(game.grapple_beam_length_delta = -8i32 as u16);
            word!(game.samus_grapple_flags |= (1u16));
        }
        0x9bc77e | 0x9bc814 => {
            let attached = shoot_held(game)
                && ((ea == 0x9bc77e
                    && actors::contacts::grapple_contact(&mut game.runtime())
                        .is_some_and(|contact| contact.has_reaction()))
                    || connected_to_block(game));
            if !attached {
                if ea == 0x9bc814 {
                    word!(game.grapple_walljump_timer = 30);
                    word!(game.grapple_beam_function = 0xc832);
                } else {
                    word!(game.grapple_beam_function = 0xc856);
                }
            }
        }
        0x9bc79d => {
            if shoot_held(game) {
                apply_input(game);
                if word!(game.grapple_beam_length_delta) != 0 {
                    change_length(game);
                }
                apply_gravity(game);
                accelerate(game);
                rebound_jump(game);
                advance_swing(game);
                if (word!(game.grapple_beam_unkD36) & 0x8000) != 0
                    && apply_special_angle(game) & 1 != 0
                {
                    return;
                }
                let keep_swinging: bool = if actors::contacts::grapple_contact(&mut game.runtime())
                    .is_some_and(|contact| contact.has_reaction())
                {
                    word!(game.grapple_beam_flags |= (0x8000u16));
                    false
                } else {
                    !connected_to_block(game)
                };
                if !keep_swinging {
                    place_rope_origin(game);
                    position_swing(game);
                    return;
                }
            }
            if word!(game.grapple_beam_unkD26) != 0
                || word!(game.grapple_beam_end_angle16) != 0x8000
            {
                launch_from_swing(game);
                word!(game.grapple_beam_function = (0x9bcb8bi32) as u16);
                word!(game.samus_movement_handler = (0x90946ei32) as u16);
            } else {
                word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
            };
        }
        0x9bc832 => {
            if (word!(game.grapple_walljump_timer -= 1, reread) & 0x8000) == 0 {
                let request = player_collision::JumpRequest {
                    kind: player_collision::JumpKind::Grapple,
                    amount: (16i16 as i32) << 16,
                };
                if player_collision::jump_admission(&mut game.runtime(), request) {
                    word!(game.grapple_beam_function = (0x9bc9cei32) as u16);
                }
            } else {
                word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
            };
        }
        0x9bbd95 => position_swing(game),
        0x9bbe98 => clamp_swing_position(game),
        0x9bbeeb => position_anchored(game),
        0x9bbf1b => restore_start_positions(game),
        0x9bc51e => start_firing(game),
        0x9bc856 => cancel(game),
        0x9bc8c5 => release_stationary(game),
        0x9bc9ce => release_walljump(game),
        0x9bcb8b => release_swing(game),
        0x9bbb64 => apply_input(game),
        0x9bbc1f => apply_gravity(game),
        0x9bbcff => accelerate(game),
        0x9bbd44 => rebound_jump(game),
        _ => {
            game.rtl_unreachable();
        }
    }
}
