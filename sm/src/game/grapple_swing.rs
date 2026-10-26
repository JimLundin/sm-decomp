//! Grapple swing placement, rope-length changes and swept block contact.
use crate::game::actors::api as actor_api;
use crate::game::music::QueueSfx1_Max15;
use crate::game::player_transitions as pt;
use crate::game::prelude::*;
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::projectile_room::Axis;
use crate::game::session_runtime::{Runtime, player_command, settle_grapple};

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
fn place_sample(angle: u16, length: u16, game: &mut Game) {
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
fn contact(game: &mut Game) -> u8 {
    let x = word!(game.grapple_beam_grapple_start_block_x);
    let row = Mult8x8(
        word!(game.grapple_beam_grapple_start_block_y) as u8,
        word!(game.room_width_in_blocks) as u8,
    );
    word!(game.cur_block_index = row.wrapping_add(x));
    game.react_to_block(|block_type, game| match block_type {
        0 | 3 | 4 | 6 | 7 => 0,
        1 | 8 | 9 | 11 | 12 | 14 | 15 => 1,
        2 => spike_contact(false, game),
        5 => {
            crate::game::player_collision::grapple::redirect_block(&mut Runtime::new(game), Axis::X)
        }
        10 => spike_contact(true, game),
        13 => {
            crate::game::player_collision::grapple::redirect_block(&mut Runtime::new(game), Axis::Y)
        }
        _ => unreachable!(),
    })
}
static AIR_DAMAGE: [u16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static SOLID_DAMAGE: [u16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn spike_contact(solid: bool, game: &mut Game) -> u8 {
    if word!(game.samus_invincibility_timer) == 0 {
        // The original reads a word spanning adjacent BTS bytes, not just one byte.
        let at = 0x16402 + usize::from(word!(game.cur_block_index));
        let kind = i16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
        if kind >= 0 {
            let damage = if solid {
                SOLID_DAMAGE[kind as usize]
            } else {
                AIR_DAMAGE[kind as usize]
            };
            let amount = ((damage as i16 as i32) << 16) as u32;
            if amount != 0 {
                let low = u16::from_ne_bytes([game.g_ram[0xa4e], game.g_ram[0xa4f]]) as u32;
                let high = u16::from_ne_bytes([game.g_ram[0xa50], game.g_ram[0xa51]]) as u32;
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
fn check_clearance(angle: u16, game: &mut Game) -> u8 {
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
#[forbid(unsafe_code)]
fn dispatch_connection(ea: u32, game: &mut Game) {
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
            Unreachable(game);
            return;
        }
    };
    begin_connection(pose, transition_graphics, game);
}
#[forbid(unsafe_code)]
pub(crate) fn connect_beam(game: &mut Game) {
    if game.g_ram[0xa1f] as i32 == kMovementType_1A_GrabbedByDraygon as i32 {
        word!(game.grapple_beam_function = (0x9bc77ei32) as u16);
        word!(game.grapple_beam_length_delta = 0);
    } else {
        let v1: i32 = 2 * word!(game.grapple_beam_direction) as i32;
        let table = if word!(game.samus_y_speed) != 0 || word!(game.samus_y_subspeed) != 0 {
            0x9bc3ee
        } else if game.g_ram[0xa1f] == 5 {
            0x9bc416
        } else {
            0x9bc3c6
        };
        word!(game.grapple_beam_function = crate::rtl::rom_table_word!(game, table, v1 as isize));
        dispatch_connection(
            u32::from(crate::rtl::rom_table_word!(game, table, (v1 + 1) as isize)) | 0x9b0000,
            game,
        );
    };
}
// Each field reacquires the registered native image at its original read point.
macro_rules! special_angle_word {
    ($game:ident, $row:ident, $offset:literal) => {
        $game.capture_native_rom().table_word(
            crate::rtl::lorom_offset(0x9bc43e) as usize + 10 * usize::from($row) + $offset,
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
#[forbid(unsafe_code)]
pub(crate) fn apply_special_angle(game: &mut Game) -> u8 {
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

fn begin_connection(pose: u16, transition_graphics: u16, game: &mut Game) {
    word!(game.samus_new_pose_interrupted = pose);
    word!(game.samus_special_transgfx_index = transition_graphics);
    let angle = u16::swap_bytes(actor_api::angle_from_delta(
        (word!(game.samus_x_pos)).wrapping_sub(word!(game.grapple_beam_end_x_pos)),
        (word!(game.samus_y_pos)).wrapping_sub(word!(game.grapple_beam_end_y_pos)),
    ));
    word!(game.grapple_beam_end_angle16 = angle);
    word!(game.grapple_beam_end_angles_mirror = angle);
    word!(game.grapple_beam_length_delta = 0);
    if word!(game.grapple_beam_length) as i32 - 64i32 & 0x8000 == 0 {
        word!(game.grapple_beam_length, wrapping_sub(24));
    }
    place_rope_origin(game);
}

pub fn place_rope_origin(game: &mut Game) {
    let angle = 2 * game.g_ram[0xcfb] as u16;
    let length = word!(game.grapple_beam_length);
    place_sample(angle, length, game);
    word!(game.x_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_x));
    word!(game.y_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_y));
}
pub fn change_length(game: &mut Game) -> u8 {
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
    let angle = 2 * game.g_ram[0xcfb] as u16;
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
pub(crate) fn apply_input(game: &mut Game) {
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
    let angle = u16::from(game.g_ram[0xcfb]) << 8;
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

pub(crate) fn apply_gravity(game: &mut Game) {
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

pub(crate) fn accelerate(game: &mut Game) {
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

pub(crate) fn rebound_jump(game: &mut Game) {
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

pub fn advance_swing(game: &mut Game) -> u8 {
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
    let mut angle = 2 * game.g_ram[0xcfb] as u16;
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

pub(crate) fn cancel(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if game.g_ram[0xa1f] == kMovementType_16_Grappling as u8 {
        pt::request_fallback_pose(&mut Runtime::new(game));
    } else {
        player_command(0x1c, game);
    }
    finish_release(game);
}

pub(crate) fn release_stationary(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    let pose = if word!(game.samus_pose) == kPose_B2_FaceR_Grapple_Air as u16 {
        kPose_01_FaceR_Normal as u16
    } else if word!(game.samus_pose) == kPose_B3_FaceL_Grapple_Air as u16 {
        kPose_02_FaceL_Normal as u16
    } else {
        let crouching = (i32::from(word!(game.samus_y_radius)) - 17) & 0x8000 != 0;
        let directions = crate::rtl::lorom_offset(0x91b629) as usize + 3;
        if game
            .capture_native_rom()
            .read(directions + 8 * usize::from(word!(game.samus_pose)))
            & 0xf0
            != 0
        {
            match (crouching, game.g_ram[0xa1e] == 4) {
                (true, true) => kPose_28_FaceL_Crouch as u16,
                (true, false) => kPose_27_FaceR_Crouch as u16,
                (false, true) => kPose_02_FaceL_Normal as u16,
                (false, false) => kPose_01_FaceR_Normal as u16,
            }
        } else {
            let table = if crouching { 0x9bc9c4 } else { 0x9bc9ba };
            // Capture the destination image before the fresh native pose-byte read.
            u16::from(
                game.capture_native_rom().read(
                    crate::rtl::lorom_offset(table) as usize
                        + usize::from(
                            game.capture_native_rom()
                                .read(directions + 8 * usize::from(word!(game.samus_pose))),
                        ),
                ),
            )
        }
    };
    word!(game.samus_new_pose_transitional = pose);
    word! {game; samus_hurt_switch_index = 0; input_to_pose_calc = 1; }
    word! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
    }
    finish_release(game);
}

pub(crate) fn release_walljump(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if game.g_ram[0xa1e] == 8 {
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

pub(crate) fn release_swing(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if word!(game.grapple_beam_unkD26) & 0x8000 == 0 {
        word!(game.samus_new_pose_transitional = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
    } else {
        word!(game.samus_new_pose_transitional = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
    }
    word!(game.samus_hurt_switch_index = 7);
    finish_release(game);
}

fn finish_release(game: &mut Game) {
    word! { game, 0;
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
        grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    let beams = word!(game.equipped_beams) & 0xfff;
    crate::game::samus_palette::write_beam_palette(
        &mut crate::game::session_runtime::Runtime::new(game),
        beams,
    );
    word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    settle_grapple(game);
    if word!(game.samus_auto_cancel_hud_item_index) != 0 {
        word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}

pub(crate) fn react_beam(game: &mut Game) -> u8 {
    crate::game::player_collision::grapple::react_beam(&mut Runtime::new(game))
}
pub(crate) fn extend_beam(game: &mut Game) -> u8 {
    crate::game::player_collision::grapple::extend_beam(&mut Runtime::new(game))
}
pub(crate) fn start_rope_animation(game: &mut Game) {
    crate::game::grapple_rope::start(&mut Runtime::new(game));
}
pub(crate) fn draw_rope(game: &mut Game) {
    crate::game::grapple_rope::draw(&mut Runtime::new(game));
}
