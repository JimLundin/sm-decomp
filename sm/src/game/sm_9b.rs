use crate::game::actors;
use crate::game::combat_rules::GrappleOutcome;
use crate::game::display_refresh::queue_native_transfer;
use crate::game::grapple_swing::{self, advance_swing, change_length, place_rope_origin};
use crate::game::player_transitions as pt;
pub use crate::game::prelude::*;
use crate::game::session_access::SessionAccess;
use crate::game::session_runtime::Runtime;
use crate::game::{
    player_collision::operations as player_collision, session_runtime::Runtime as CollisionRuntime,
};

pub use crate::game::constants::bank_9b::*;

use crate::game::grapple_swing::{
    extend_beam as BlockCollGrappleBeam, react_beam as BlockReactGrapple,
    start_rope_animation as GrappleFunc_AF87,
};
use crate::game::music::{QueueSfx1_Max1, QueueSfx1_Max6};
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session_runtime::player_bottom;

const RomPtr_9B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x9b>;
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 0x90946e, fnSamus_MovementHandler_Normal = 0x90a337,
    fnSamusDrawHandler_Default = 0x90eb52, fnsub_90EB86 = 0x90eb86,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
    fnGrappleBeamFunc_Firing = 0x9bc703, fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
    fnGrappleBeamFunc_Cancel = 0x9bc856,
    fnGrappleBeam_Func2 = 0x9bc8c5,
}
static kIsGrappleBannedForMovementType: [u8; 28] = [
    0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1,
];
pub unsafe fn CancelGrappleBeamIfIncompatiblePose(game: &mut Game) {
    unsafe {
        if kIsGrappleBannedForMovementType[*ram_ptr!(game; 0xa1f) as usize] == 0 {
            if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
                && word!(game.grapple_beam_function) as i32 + 0x3882 & 0x8000 != 0
            {
                let mut v0: i16 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                    .offset(word!(game.samus_pose) as isize))
                .direction_shots_fired as i16;
                if (v0 as u16 & 0xf0) == 0 {
                    if v0 as i32 == word!(game.grapple_beam_direction) as i32 {
                        return;
                    }
                    if word!(game.grapple_varCF6) != 0 {
                        QueueSfx1_Max6(7, game);
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
}
pub fn CheckIfGrappleIsConnectedToBlock(game: &mut Game) -> u8 {
    {
        word! { game, 0;
            grapple_beam_extension_x_velocity, grapple_beam_extension_y_velocity,
        }
        if (BlockReactGrapple(game) & 1) == 0 {
            return 0;
        }
        word!(game.samus_grapple_flags = 1);
        1
    }
}
macro_rules! place_samus_relative_to_grapple_origin {
    ($game:ident, $offset_table:literal, $offset_index:ident) => {
        word! { $game;
            samus_x_pos = (word!($game.x_pos_of_start_of_grapple_beam)).wrapping_add(
                (*($game.legacy_native_rom_ptr().offset(crate::rtl::lorom_offset($offset_table)) as *mut u8)
                    .offset($offset_index as isize) as i8) as u16,
            );
            samus_y_pos = (word!($game.y_pos_of_start_of_grapple_beam)).wrapping_add(
                (*($game.legacy_native_rom_ptr().offset(crate::rtl::lorom_offset($offset_table)) as *mut u8)
                    .offset(($offset_index as i32 + 1) as isize) as i8) as u16,
            );
        }
    };
}

macro_rules! restore_grapple_start_positions {
    (
        $pose_offset:ident, $game:ident, $origin_x_table:literal, $previous_x_table:literal,
        $origin_y_table:literal, $y_adjustment:ident, $previous_y_table:literal $(,)?
    ) => {
        let v1: i32 = $pose_offset as i32 >> 1;
        word! { $game;
            x_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!($game, $origin_x_table, v1 as isize))
                    .wrapping_add(word!($game.samus_x_pos));
            x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!($game, $previous_x_table, v1 as isize))
                    .wrapping_add(word!($game.samus_x_pos));
            y_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!($game, $origin_y_table, v1 as isize) as i32
                    + word!($game.samus_y_pos) as i32
                    - $y_adjustment as i32) as u16;
            y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!($game, $previous_y_table, v1 as isize) as i32
                    + word!($game.samus_y_pos) as i32
                    - $y_adjustment as i32) as u16;
        }
    };
}

pub fn UpdateGrappleBeamTilesAndIncrFlameCtr(game: &mut Game) {
    if (word!(game.grapple_point_anim_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.grapple_point_anim_timer = 5);
        word!(game.grapple_point_anim_ptr += 512);
        if ((word!(game.grapple_point_anim_ptr)) as i16)
            .wrapping_sub((crate::rtl::rom_table_word!(game, 0x9bc344, 0)) as i16)
            >= 0
        {
            word!(game.grapple_point_anim_ptr = crate::rtl::rom_table_word!(game, 0x9bc342, 0));
        }
    }
    let cursor = queue_native_transfer(
        word!(game.vram_write_queue_tail),
        32,
        |game| word!(game.grapple_point_anim_ptr),
        0x9a,
        |_| 0x6200,
        game,
    );
    let index = (game.g_ram[0xcfb] >> 1 & 0xfe) >> 1;
    queue_native_transfer(
        cursor,
        128,
        |game| crate::rtl::rom_table_word!(game, 0x9bc346, isize::from(index)),
        0x9a,
        |_| 0x6210,
        game,
    );
    if word!(game.flare_counter) as i32 - 120i32 & 0x8000 != 0 {
        word!(game.flare_counter += 1);
    }
}
pub unsafe fn GrappleBeamHandler(game: &mut Game) {
    unsafe {
        let mut r18: u16 = 0;
        if word!(game.grapple_varCF6) != 0 {
            word!(game.grapple_varCF6 -= 1);
        }
        word!(game.samus_grapple_flags &= !(1));
        CancelGrappleBeamIfIncompatiblePose(game);
        CallGrappleBeamFunc(
            ((word!(game.grapple_beam_function)) as u32) | (0x9b0000u32),
            game,
        );
        if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
            && word!(game.grapple_beam_function) as i32 - (0x9bc856 & 0xffff) & 0x8000 != 0
            && (word!(game.samus_suit_palette_index) & 4) == 0
            && word!(game.fx_type) != 0
            && {
                r18 = player_bottom(game);
                (word!(game.fx_y_pos) & 0x8000) == 0
            }
            && word!(game.fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
        {
            word!(game.grapple_beam_flags |= (1u16));
        } else {
            word!(game.grapple_beam_flags &= !(1));
        };
    }
}
macro_rules! initialize_grapple_endpoints {
    (
        $game:ident, $origin_x_table:literal, $pose_index:ident, $origin_y_table:literal,
        $y_adjustment:ident, $previous_x_table:literal, $previous_y_table:literal $(,)?
    ) => {
        word! { $game;
            grapple_beam_origin_x_offset =
                crate::rtl::rom_table_word!($game, $origin_x_table, $pose_index as isize);
            grapple_beam_origin_y_offset =
                (crate::rtl::rom_table_word!($game, $origin_y_table, $pose_index as isize))
                    .wrapping_sub($y_adjustment);
            grapple_beam_end_x_pos =
                (crate::rtl::rom_table_word!($game, $origin_x_table, $pose_index as isize))
                    .wrapping_add(word!($game.samus_x_pos));
            x_pos_of_start_of_grapple_beam = word!($game.grapple_beam_end_x_pos);
            x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!($game, $previous_x_table, $pose_index as isize))
                    .wrapping_add(word!($game.samus_x_pos));
            grapple_beam_end_y_pos =
                (crate::rtl::rom_table_word!($game, $origin_y_table, $pose_index as isize) as i32
                    + word!($game.samus_y_pos) as i32
                    - $y_adjustment as i32) as u16;
            y_pos_of_start_of_grapple_beam = word!($game.grapple_beam_end_y_pos);
            y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!($game, $previous_y_table, $pose_index as isize) as i32
                    + word!($game.samus_y_pos) as i32
                    - $y_adjustment as i32) as u16;
        }
    };
}

pub fn CheckBannedDraygonGrappleDirs(a: u16, game: &mut Game) -> u16 {
    {
        if a as i32 == kPose_BE_FaceL_Draygon_Move as i32 {
            if (word!(game.joypad1_lastkeys) as i32 & kButton_Left as i32) != 0 {
                if (word!(game.joypad1_lastkeys) as i32 & kButton_Down as i32) != 0 {
                    return 6;
                }
                if (word!(game.joypad1_lastkeys) as i32 & kButton_Up as i32) != 0 {
                    return 8;
                }
            }
            return 7;
        }
        if (word!(game.joypad1_lastkeys) as i32 & kButton_Right as i32) == 0 {
            return 2;
        }
        if (word!(game.joypad1_lastkeys) as i32 & kButton_Down as i32) != 0 {
            return 3;
        }
        if (word!(game.joypad1_lastkeys) as i32 & kButton_Up as i32) == 0 {
            return 2;
        }
        1
    }
}
pub unsafe fn PropelSamusFromGrappleSwing(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        if (word!(game.grapple_beam_unkD26) & 0x8000) == 0 {
            v0 = (2u16).wrapping_mul(word!(game.grapple_beam_unkD26));
            let mut v2: i16 = kSinCosTable8bit_Sext
                [(byte!(@pointer ram_ptr!(game, u16; 0xcfa); 1) as i32 + 64) as usize];
            if v2 >= 0 {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = game.multiply_with_scratch(v2 as u16, v0)
                );
                word!(game.samus_y_dir = 2);
            } else {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = game.multiply_with_scratch(-(v2 as i32) as u16, v0)
                );
                word!(game.samus_y_dir = 1);
            }
        } else {
            v0 = (-2i32 * word!(game.grapple_beam_unkD26) as i32) as u16;
            let mut v1: i16 = kSinCosTable8bit_Sext
                [(byte!(@pointer ram_ptr!(game, u16; 0xcfa); 1) as i32 + 64) as usize];
            if v1 < 0 {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = game.multiply_with_scratch(-(v1 as i32) as u16, v0)
                );
                word!(game.samus_y_dir = 2);
            } else {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = game.multiply_with_scratch(v1 as u16, v0)
                );
                word!(game.samus_y_dir = 1);
            }
        }
        word!(game.samus_x_accel_mode = 2);
        let mut r18: u16 = (3 * (v0 as i32 >> 9)) as u16;
        r18 = (64u16).wrapping_sub(r18);
        let v3: u16 = abs16(
            kSinCosTable8bit_Sext
                [(((*ram_ptr!(game; 0xcfb)).wrapping_sub((r18) as u8)) as i32 + 64) as usize]
                as u16,
        );
        words!(
            game.samus_x_base_speed,
            samus_x_base_subspeed = game.multiply_with_scratch(v3, v0)
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallGrappleBeamFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10206613] GrappleBeamFunc_BD95 [] {
        let v0: u16 = abs16(word!(game.grapple_beam_unkD26));
        let mut v1: u16 = 0;
        let start_grapple_animation: bool = if v0 as i32 - 64i32 & 0x8000 == 0 {
            word!(game.slow_grabble_scrolling_flag = 1);
            true
        } else {
            word!(game.slow_grabble_scrolling_flag = 0);
            (((*ram_ptr!(game; 0xcfb) as i32) << 8) != 0x8000)
                || ({
                    if word!(game.samus_anim_frame) as i32 - 64i32 & 0x8000 != 0 {
                        word! {game; samus_anim_frame_timer = 8; samus_anim_frame = 64; }
                    }
                    v1 = byte!(@pointer rom_ptr!(game; 0x9bc1c2); *ram_ptr!(game; 0xcfc, 1) as isize)
                        as u16;
                    false
                })
        };
        if start_grapple_animation {
            word!(game.samus_anim_frame_timer = 15);
            v1 = byte!(@pointer rom_ptr!(game; 0x9bc1c2); *ram_ptr!(game; 0xcfc, 1) as isize)
                as u16;
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
        if *ram_ptr!(game; 0xa1e) == 4 {
            place_samus_relative_to_grapple_origin!(game, 0x9bc2c2, v2);
        } else {
            place_samus_relative_to_grapple_origin!(game, 0x9bc302, v2);
        }
        word! { game;
            x_pos_of_start_of_grapple_beam_prevframe = word!(game.x_pos_of_start_of_grapple_beam);
            y_pos_of_start_of_grapple_beam_prevframe = word!(game.y_pos_of_start_of_grapple_beam);
        }
        GrappleBeamFunc_BE98(game);
    };
    [10206872] GrappleBeamFunc_BE98 [] {
        let mut v0: i16 =
            ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
        pt::clamp_previous_position(Runtime::new(game).session_mut(), v0);
    };
    [10206955] GrappleBeamFunc_BEEB [] {
        let v0: i32 = word!(game.grapple_beam_direction) as i32;
        word! { game;
            samus_x_pos = (word!(game.x_pos_of_start_of_grapple_beam))
                .wrapping_sub(crate::rtl::rom_table_word!(game, 0x9bc122, v0 as isize));
            x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc14a, v0 as isize))
                    .wrapping_add(word!(game.samus_x_pos));
            samus_y_pos = (word!(game.y_pos_of_start_of_grapple_beam))
                .wrapping_sub(crate::rtl::rom_table_word!(game, 0x9bc136, v0 as isize));
            y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc15e, v0 as isize))
                    .wrapping_add(word!(game.samus_y_pos));
        }
    };
    [10207003] GrappleBeamFunc_BF1B [] {
        let r22: u16 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
            .offset(word!(game.samus_pose) as isize))
        .y_offset_to_gfx as u16;
        let v0: u16 = (2u16).wrapping_mul(word!(game.grapple_beam_direction));
        if word!(game.samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
            || word!(game.samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
            || *ram_ptr!(game; 0xa1f) != 1
        {
            restore_grapple_start_positions!(v0, game, 0x9bc122, 0x9bc14a, 0x9bc136, r22, 0x9bc15e);
        } else {
            restore_grapple_start_positions!(v0, game, 0x9bc172, 0x9bc19a, 0x9bc186, r22, 0x9bc1ae);
        };
    };
    [10208496] GrappleBeamFunc_Inactive [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0
            || (word!(game.button_config_shoot_x) & word!(game.joypad1_newinput_samusfilter)) != 0
        {
            GrappleBeamFunc_FireGoToCancel(game);
        } else if word!(game.flare_counter) != 0 {
            word! { game, 0;
                flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
                flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
                flare_fast_sparks_anim_timer,
            }
            crate::game::samus_palette::load_suit(&mut Runtime::new(game), crate::game::visual_state::PaletteBuffer::Current);
        }
    };
    [10208542] GrappleBeamFunc_FireGoToCancel [] {
        let mut r22: u16 = 0;
        let mut v0: i32 = 0;
        if word!(game.samus_pose) as i32 == kPose_F0_FaceR_Draygon_Move as i32
            || word!(game.samus_pose) as i32 == kPose_BE_FaceL_Draygon_Move as i32
        {
            v0 = CheckBannedDraygonGrappleDirs(word!(game.samus_pose), game) as i32;
            r22 = 6;
        } else {
            r22 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .y_offset_to_gfx as u16;
            v0 = (*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .direction_shots_fired as i32;
            if (v0 & 0xf0) != 0 {
                word!(game.grapple_beam_function = (0x9bc856i32) as u16);
                return;
            }
        }
        word!(game.grapple_beam_direction = v0 as u16);
        let v1: i32 = v0;
        word! { game;
            grapple_beam_extension_x_velocity =
                crate::rtl::rom_table_word!(game, 0x9bc0db, v1 as isize);
            grapple_beam_extension_y_velocity =
                crate::rtl::rom_table_word!(game, 0x9bc0ef, v1 as isize);
            grapple_beam_end_angle16 = crate::rtl::rom_table_word!(game, 0x9bc104, v1 as isize);
            grapple_beam_end_angles_mirror = word!(game.grapple_beam_end_angle16);
            grapple_varCF6 = 10;
        }
        if word!(game.samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
            || word!(game.samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
            || *ram_ptr!(game; 0xa1f) != 1
        {
            initialize_grapple_endpoints!(game, 0x9bc122, v1, 0x9bc136, r22, 0x9bc14a, 0x9bc15e);
        } else {
            initialize_grapple_endpoints!(game, 0x9bc172, v1, 0x9bc186, r22, 0x9bc19a, 0x9bc1ae);
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
            grapple_point_anim_ptr = crate::rtl::rom_table_word!(game, 0x9bc342, 0); }
        word! { game, 0;
            grapple_beam_grapple_start_x, grapple_beam_unkD38, grapple_beam_unkD36,
            slow_grabble_scrolling_flag,
        }
        GrappleFunc_AF87(game);
        word! {game; samus_draw_handler = ((0x90eb86i32) as u16); grapple_walljump_timer = 0; }
        crate::game::samus_palette::write_beam_palette(
            &mut crate::game::session_runtime::Runtime::new(game), 2,
        );
        word! {game; palette_buffer[223] = 32657; grapple_beam_function = ((0x9bc703i32) as u16); }
        QueueSfx1_Max1(5, game);
        word! {game; flare_counter = 1; play_resume_charging_beam_sfx = 0; }
        if word!(game.samus_movement_handler) as i32 == 0x90946e & 0xffff {
            word!(game.samus_movement_handler = (0x90a337i32) as u16);
        }
    };
    [10209027] GrappleBeamFunc_Firing [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) == 0 {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
            return;
        }
        word!(game.grapple_beam_length += word!(game.grapple_beam_length_delta));
        if word!(game.grapple_beam_length) as i32 - 128i32 & 0x8000 == 0 {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
            return;
        }
        let contact = actors::contacts::grapple_contact(&mut Runtime::new(game));
        let outcome = match contact {
            Some(contact) => actors::combat::resolve_grapple(&mut Runtime::new(game), contact),
            None => GrappleOutcome::Miss,
        };
        match outcome {
            GrappleOutcome::Attach => {},
            GrappleOutcome::Cancel => {
                word!(game.grapple_beam_function = (0x9bc856i32) as u16);
                return;
            }
            GrappleOutcome::Miss => {
                let v2: u8 = BlockCollGrappleBeam(game);
                if (v2 & 1) == 0 {
                    return;
                }
                if (v2 & 0x40) == 0 {
                    word!(game.grapple_beam_function = (0x9bc856i32) as u16);
                    return;
                }
            }
        }
        QueueSfx1_Max6(6, game);
        grapple_swing::connect_beam(game);
        word!(game.grapple_beam_length_delta = -8i32 as u16);
        word!(game.samus_grapple_flags |= (1u16));
    };
    [10209150] GrappleBeamFunc_ConnectedLockedInPlace [] {
        if !((word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
            && (actors::contacts::grapple_contact(&mut Runtime::new(game)).is_some_and(|contact| contact.has_reaction())
                || CheckIfGrappleIsConnectedToBlock(game) != 0))
        {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
        }
    };
    [10209181] GrappleBeamFunc_Connected_Swinging [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
            grapple_swing::apply_input(game);
            if word!(game.grapple_beam_length_delta) != 0 {
                change_length(game);
            }
            grapple_swing::apply_gravity(game);
            grapple_swing::accelerate(game);
            grapple_swing::rebound_jump(game);
            advance_swing(game);
            if (word!(game.grapple_beam_unkD36) & 0x8000) != 0
                && grapple_swing::apply_special_angle(game) & 1 != 0
            {
                return;
            }
            let keep_swinging: bool = if actors::contacts::grapple_contact(&mut Runtime::new(game)).is_some_and(|contact| contact.has_reaction()) {
                word!(game.grapple_beam_flags |= (0x8000u16));
                false
            } else {
                CheckIfGrappleIsConnectedToBlock(game) & 1 == 0
            };
            if keep_swinging {
            } else {
                place_rope_origin(game);
                GrappleBeamFunc_BD95(game);
                return;
            }
        }
        if word!(game.grapple_beam_unkD26) != 0 || word!(game.grapple_beam_end_angle16) != 0x8000 {
            PropelSamusFromGrappleSwing(game);
            word!(game.grapple_beam_function = (0x9bcb8bi32) as u16);
            word!(game.samus_movement_handler = (0x90946ei32) as u16);
        } else {
            word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
        };
    };
    [10209300] GrappleBeamFunc_Wallgrab [] {
        if !((word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
            && CheckIfGrappleIsConnectedToBlock(game) & 1 != 0)
        {
            word!(game.grapple_walljump_timer = 30);
            word!(game.grapple_beam_function = (0x9bc832i32) as u16);
        }
    };
    [10209330] GrappleBeamFunc_C832 [] {
        if (word!(game.grapple_walljump_timer -= 1, reread) & 0x8000) == 0 {
            if { let collision_request = player_collision::JumpRequest { kind: player_collision::JumpKind::Grapple, amount: (16i16 as i32) << 16 }; player_collision::jump_admission(&mut CollisionRuntime::new(game), collision_request) } {
                word!(game.grapple_beam_function = (0x9bc9cei32) as u16);
            }
        } else {
            word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
        };
    };
    @external {
        10209366 => grapple_swing::cancel;
        10209477 => grapple_swing::release_stationary;
        10209742 => grapple_swing::release_walljump;
        10210187 => grapple_swing::release_swing;
        10206052 => grapple_swing::apply_input;
        10206239 => grapple_swing::apply_gravity;
        10206463 => grapple_swing::accelerate;
        10206532 => grapple_swing::rebound_jump;
    }
}
