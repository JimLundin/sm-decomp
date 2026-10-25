use crate::game::actors;
use crate::game::actors::api as actor_api;
use crate::game::combat_rules::GrappleOutcome;
use crate::game::grapple_swing::{advance_swing, change_length, place_rope_origin};
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
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::samus_animation::DrawSamusSuitExploding;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::sm_80::{Multiply16x16, QueueSfx1_Max1, QueueSfx1_Max6, QueueSfx1_Max15};
use crate::game::sm_90::{Samus_GetBottom_R18, Samus_PostGrappleCollisionDetect};

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct GrappleBeamSpecialAngles {
        pub field_0, field_2, field_4, field_6: u16;
        pub field_8: VoidP;
    }
}

const RomPtr_9B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x9b>;
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 0x90946e, fnSamus_MovementHandler_Normal = 0x90a337,
    fnSamusDrawHandler_Default = 0x90eb52, fnsub_90EB86 = 0x90eb86,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
    fnGrappleBeamFunc_Firing = 0x9bc703, fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
    fnGrappleBeamFunc_Cancel = 0x9bc856,
    fnGrappleBeam_Func2 = 0x9bc8c5,
}
static kDeathAnimationFrames: [u8; 28] = [
    5, 5, 5, 5, 1, 5, 5, 0, 1, 0, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5,
];
static g_word_9BC118: u16 = 24;
static g_word_9BC11A: u16 = 0xc;
static g_word_9BC11C: u16 = 5;
static g_word_9BC11E: u16 = 0x480;
static g_word_9BC120: u16 = 0x300;
static g_word_9BB7BF: [u16; 5] = [0x8400, 0x8800, 0x8c00, 0x9000, 0x8000];
static g_word_9BB7C9: [u16; 5] = [0x6200, 0x6400, 0x6600, 0x6800, 0x6000];
static kShadesOfWhite: [u16; 22] = [
    0x421, 0xc63, 0x14a5, 0x1ce7, 0x2529, 0x2d6b, 0x35ad, 0x4210, 0x4a52, 0x4e73, 0x5294, 0x56b5,
    0x5ad6, 0x5ef7, 0x6318, 0x6739, 0x6b5a, 0x6f7b, 0x739c, 0x77bd, 0x7bde, 0x7fff,
];
static kIsGrappleBannedForMovementType: [u8; 28] = [
    0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1,
];
pub unsafe fn StartSamusDeathAnimation(game: &mut Game) {
    unsafe {
        let v0: u16 = *ram_ptr!(game; 0xa1f) as u16;
        if *ram_ptr!(game; 0xa1f) == 3 {
            QueueSfx1_Max6(0x32, game);
        }
        let v1: u16 = kDeathAnimationFrames[v0 as usize] as u16;
        if *ram_ptr!(game; 0xa1e) == 4 {
            word!(game.samus_pose = kPose_D8_FaceL_CrystalFlashEnd as u16);
        } else {
            word!(game.samus_pose = kPose_D7_FaceR_CrystalFlashEnd as u16);
        }
        pt::finish_pose_change(&mut Runtime::new(game));
        word! {game; samus_anim_frame_skip = 0; samus_anim_frame = v1; }
        word!(game.samus_x_pos -= word!(game.layer1_x_pos));
        word!(game.samus_y_pos -= word!(game.layer1_y_pos));
    }
}
pub unsafe fn HandleSamusDeathSequence(game: &mut Game) -> u16 {
    unsafe {
        if word!(game.g_word_7E0DE6) as i32 - 4i32 & 0x8000 != 0 {
            QueueTransferOfSamusDeathSequence((2u16).wrapping_mul(word!(game.g_word_7E0DE6)), game);
        }
        if word!(game.g_word_7E0DE6 += 1, reread) as i32 - 60i32 & 0x8000 != 0 {
            let v0: bool = (word!(game.game_options_screen_index -= 1, reread) & 0x8000) != 0;
            if word!(game.game_options_screen_index) == 0 || v0 {
                if word!(game.g_word_7E0DE4) != 0 {
                    word! {game; g_word_7E0DE4 = 0; game_options_screen_index = 3; }
                } else {
                    word! { game, 1; g_word_7E0DE4, game_options_screen_index }
                }
                CopyPalettesForSamusDeath((word!(game.g_word_7E0DE4)).wrapping_mul(2), game);
            }
            0
        } else {
            HandleSamusDeathSequence_Helper2(game);
            word!(
                game.substate = (word!(game.joypad2_last) as i32 & 0xb0
                    == kButton_A as i32 | kButton_L as i32 | kButton_R as i32)
                    as u16
            );
            1
        }
    }
}
pub unsafe fn HandleSamusDeathSequence_Helper2(game: &mut Game) {
    unsafe {
        let v0 = RomPtr_9B(
            crate::rtl::rom_table_word!(
                game,
                0x9bb5c8,
                (word!(game.samus_suit_palette_index) >> 1) as isize
            ),
            game,
        ) as *const u16;
        memcpy(
            (ram_ptr!(game, u16; 0xc000)).offset(192) as *mut c_void,
            RomPtr_9B(*v0, game) as *const c_void,
            32 as size_t,
        );
        memcpy(
            (ram_ptr!(game, u16; 0xc000)).offset(240) as *mut c_void,
            RomPtr_9B(addr_word_9BA120 as u16, game) as *const c_void,
            32 as size_t,
        );
        QueueTransferOfSamusDeathSequence(8, game);
        word!(game.game_options_screen_index = *(rom_ptr!(game, u8; 0x9bb823)) as u16);
        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
        GameState_24_SamusNoHealth_Explosion_2(game);
    }
}
pub unsafe fn CopyPalettesForSamusDeath(v0: u16, game: &mut Game) {
    unsafe {
        let r20: i32 = crate::rtl::rom_table_word!(
            game,
            0x9bb6d2,
            (word!(game.samus_suit_palette_index) >> 1) as isize
        ) as i32;
        let v1 = RomPtr_9B((r20 + v0 as i32) as u16, game) as *const u16;
        memcpy(
            (ram_ptr!(game, u16; 0xc000)).offset(192) as *mut c_void,
            RomPtr_9B(*v1, game) as *const c_void,
            32 as size_t,
        );
        memcpy(
            (ram_ptr!(game, u16; 0xc000)).offset(240) as *mut c_void,
            RomPtr_9B(
                crate::rtl::rom_table_word!(game, 0x9bb80f, (v0 >> 1) as isize),
                game,
            ) as *const c_void,
            32 as size_t,
        );
    }
}
pub unsafe fn QueueTransferOfSamusDeathSequence(v0: u16, game: &mut Game) {
    unsafe {
        let mut v1: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 1024;
        v1 = (v1).wrapping_add(2);
        let v2: i32 = v0 as i32 >> 1;
        (*gVramWriteEntry(v1 as i32, game)).size = g_word_9BB7BF[v2 as usize];
        v1 = (v1).wrapping_add(2);
        crate::update_place! { previous fresh7 = v1 => v1.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh7 as i32,
            game,
        ))
        .size as *mut u8) = 0x9b;
        (*gVramWriteEntry(v1 as i32, game)).size = g_word_9BB7C9[v2 as usize];
        word!(game.vram_write_queue_tail = (v1).wrapping_add(2));
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_Helper(game: &mut Game) -> u16 {
    unsafe {
        GameState_24_SamusNoHealth_Explosion_1(game);
        GameState_24_SamusNoHealth_Explosion_2(game)
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_1(game: &mut Game) {
    unsafe {
        if word!(game.substate) == 0 && word!(game.g_word_7E0DE4) != 0 {
            let v0: i32 = word!(game.g_word_7E0DE6) as i32;
            let dst: *mut u16 = ram_ptr!(game, u16; 0xc000);
            let mut i: i32 = 0;
            while i < 384 / 2 {
                *dst.offset(i as isize) = kShadesOfWhite[v0 as usize];
                i += 1;
            }
            let mut i_0: i32 = 416 / 2;
            while i_0 < 480 / 2 {
                *dst.offset(i_0 as isize) = kShadesOfWhite[v0 as usize];
                i_0 += 1;
            }
            if word!(game.g_word_7E0DE6) as i32 - 20i32 & 0x8000 != 0 {
                word!(game.g_word_7E0DE6 += 1);
            }
        }
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_2(game: &mut Game) -> u16 {
    unsafe {
        let v0: bool = (word!(game.game_options_screen_index -= 1, reread) & 0x8000) != 0;
        if word!(game.game_options_screen_index) == 0 || v0 {
            if word!(game.g_word_7E0DE4 += 1, reread) as i32 - 9i32 & 0x8000 == 0 {
                word!(game.g_word_7E0DE6 = 21);
                GameState_24_SamusNoHealth_Explosion_1(game);
                word!(game.substate = 0);
                return 1;
            }
            if word!(game.substate) == 0 || word!(game.g_word_7E0DE4) as i32 - 2i32 & 0x8000 != 0 {
                word!(
                    game.game_options_screen_index = *(rom_ptr!(game, u8; 0x9bb823))
                        .offset((2 * word!(game.g_word_7E0DE4) as i32) as isize,)
                        as u16
                );
                CopyPalettesForSamusDeath(
                (2u16).wrapping_mul(
                    (byte!(@pointer rom_ptr!(game; 0x9bb823); (2 * word!(game.g_word_7E0DE4) as i32 + 1) as isize))
                        as u16,
                ),
                game,
            );
            } else {
                word!(
                    game.game_options_screen_index = *(rom_ptr!(game, u8; 0x9bb823))
                        .offset((2 * word!(game.g_word_7E0DE4) as i32) as isize,)
                        as u16
                );
            }
        }
        DrawSamusSuitExploding(game);
        0
    }
}
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
pub unsafe fn CheckIfGrappleIsConnectedToBlock(game: &mut Game) -> u8 {
    unsafe {
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
crate::game::callback_dispatch! {
    pub unsafe fn CallGrappleNextFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10205657] GrappleNext_SwingClockwise [] {
        word!(game.samus_new_pose_interrupted = kPose_B2_FaceR_Grapple_Air as u16);
        HandleConnectingGrapple_Swinging(game);
    };
    [10205666] GrappleNext_SwingAntiClockwise [] {
        word!(game.samus_new_pose_interrupted = kPose_B3_FaceL_Grapple_Air as u16);
        HandleConnectingGrapple_Swinging(game);
    };
    [10205674] GrappleNext_StandAimRight [] {
        word!(game.samus_new_pose_interrupted = kPose_A8_FaceR_Grappling as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205683] GrappleNext_StandAimDownRight [] {
        word!(game.samus_new_pose_interrupted = kPose_AA_FaceR_Grappling_AimDR as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205692] GrappleNext_StandAimDownLeft [] {
        word!(game.samus_new_pose_interrupted = kPose_AB_FaceL_Grappling_AimDL as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205701] GrappleNext_StandAimLeft [] {
        word!(game.samus_new_pose_interrupted = kPose_A9_FaceL_Grappling as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205710] GrappleNext_CrouchAimRight [] {
        word!(game.samus_new_pose_interrupted = kPose_B4_FaceR_Grappling_Crouch as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205719] GrappleNext_CrouchAimDownRight [] {
        word!(game.samus_new_pose_interrupted = kPose_B6_FaceR_Grappling_Crouch_AimDR as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205728] GrappleNext_CrouchAimDownLeft [] {
        word!(game.samus_new_pose_interrupted = kPose_B7_FaceL_Grappling_Crouch_AimDL as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
    [10205737] GrappleNext_CrouchAimLeft [] {
        word!(game.samus_new_pose_interrupted = kPose_B5_FaceL_Grappling_Crouch as u16);
        HandleConnectingGrapple_StuckInPlace(game);
    };
}
pub unsafe fn HandleConnectingGrapple(game: &mut Game) {
    unsafe {
        if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_1A_GrabbedByDraygon as i32 {
            word!(game.grapple_beam_function = (0x9bc77ei32) as u16);
            word!(game.grapple_beam_length_delta = 0);
        } else {
            let v1: i32 = 2 * word!(game.grapple_beam_direction) as i32;
            if word!(game.samus_y_speed) != 0 || word!(game.samus_y_subspeed) != 0 {
                word!(
                    game.grapple_beam_function =
                        crate::rtl::rom_table_word!(game, 0x9bc3ee, v1 as isize)
                );
                CallGrappleNextFunc(
                    ((crate::rtl::rom_table_word!(game, 0x9bc3ee, (v1 + 1) as isize)) as u32)
                        | (0x9b0000u32),
                    game,
                );
            } else if *ram_ptr!(game; 0xa1f) == 5 {
                word!(
                    game.grapple_beam_function =
                        crate::rtl::rom_table_word!(game, 0x9bc416, v1 as isize)
                );
                CallGrappleNextFunc(
                    ((crate::rtl::rom_table_word!(game, 0x9bc416, (v1 + 1) as isize)) as u32)
                        | (0x9b0000u32),
                    game,
                );
            } else {
                word!(
                    game.grapple_beam_function =
                        crate::rtl::rom_table_word!(game, 0x9bc3c6, v1 as isize)
                );
                CallGrappleNextFunc(
                    ((crate::rtl::rom_table_word!(game, 0x9bc3c6, (v1 + 1) as isize)) as u32)
                        | (0x9b0000u32),
                    game,
                );
            }
        };
    }
}
macro_rules! initialize_connected_grapple_angle {
    ($game:ident, $transition_graphics:literal) => {
        word!($game.samus_special_transgfx_index = $transition_graphics);
        let v0: u16 = u16::swap_bytes(actor_api::angle_from_delta(
            (word!($game.samus_x_pos)).wrapping_sub(word!($game.grapple_beam_end_x_pos)),
            (word!($game.samus_y_pos)).wrapping_sub(word!($game.grapple_beam_end_y_pos)),
        ));
        word!($game.grapple_beam_end_angle16 = v0);
        word!($game.grapple_beam_end_angles_mirror = v0);
        word!($game.grapple_beam_length_delta = 0);
        if word!($game.grapple_beam_length) as i32 - 64i32 & 0x8000 == 0 {
            word!($game.grapple_beam_length, wrapping_sub(24));
        }
    };
}

pub unsafe fn HandleConnectingGrapple_Swinging(game: &mut Game) {
    unsafe {
        initialize_connected_grapple_angle!(game, 9);
        place_rope_origin(game);
    }
}
pub unsafe fn HandleConnectingGrapple_StuckInPlace(game: &mut Game) {
    unsafe {
        initialize_connected_grapple_angle!(game, 10);
        place_rope_origin(game);
    }
}
pub unsafe fn HandleSpecialGrappleBeamAngles(game: &mut Game) -> u8 {
    unsafe {
        let mut v0: u16 = 7;
        while word!(game.grapple_beam_end_angle16)
            != (*(rom_ptr!(game, GrappleBeamSpecialAngles; 0x9bc43e)).offset(v0 as isize)).field_0
        {
            v0 = v0.wrapping_sub(1);
            if (v0 & 0x8000) != 0 {
                return 0;
            }
        }
        word! { game;
            samus_new_pose_interrupted =
                (*(rom_ptr!(game, GrappleBeamSpecialAngles; 0x9bc43e)).offset(v0 as isize)).field_2;
            samus_x_pos = (word!(game.grapple_beam_end_x_pos)).wrapping_add(
                (*(rom_ptr!(game, GrappleBeamSpecialAngles; 0x9bc43e)).offset(v0 as isize)).field_4,
            );
            samus_y_pos = (word!(game.grapple_beam_end_y_pos)).wrapping_add(
                (*(rom_ptr!(game, GrappleBeamSpecialAngles; 0x9bc43e)).offset(v0 as isize)).field_6,
            );
            grapple_beam_function =
                (*(rom_ptr!(game, GrappleBeamSpecialAngles; 0x9bc43e)).offset(v0 as isize)).field_8;
        }
        word! { game, 0; samus_special_transgfx_index, slow_grabble_scrolling_flag }
        let v2: i16 =
            ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16);
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub((word!(game.samus_prev_x_pos)) as i16)
            < 0
        {
            if v2 as i32 + 12 & 0x8000 != 0 {
                word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_add(12));
            }
        } else if v2 as i32 - 13i32 & 0x8000 == 0 {
            word!(game.samus_prev_x_pos = (word!(game.samus_x_pos)).wrapping_sub(12));
        }
        let v3: i16 =
            ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16);
        if ((word!(game.samus_y_pos)) as i16).wrapping_sub((word!(game.samus_prev_y_pos)) as i16)
            < 0
        {
            if v3 as i32 + 12 & 0x8000 != 0 {
                word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_add(12));
            }
        } else if v3 as i32 - 13i32 & 0x8000 == 0 {
            word!(game.samus_prev_y_pos = (word!(game.samus_y_pos)).wrapping_sub(12));
        }
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

pub unsafe fn UpdateGrappleBeamTilesAndIncrFlameCtr(game: &mut Game) {
    unsafe {
        UpdateGrappleBeamTiles(game);
        if word!(game.flare_counter) as i32 - 120i32 & 0x8000 != 0 {
            word!(game.flare_counter += 1);
        }
    }
}
pub unsafe fn UpdateGrappleBeamTiles(game: &mut Game) {
    unsafe {
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
        let mut v0: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 32;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, game)).size = word!(game.grapple_point_anim_ptr);
        v0 = (v0).wrapping_add(2);
        crate::update_place! { previous fresh27 = v0 => v0.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh27 as i32,
            game,
        ))
        .size as *mut u8) = -102i32 as u8;
        (*gVramWriteEntry(v0 as i32, game)).size = 25088;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
        let v1: u16 = (*ram_ptr!(game; 0xcfb) as i32 >> 1 & 0xfe) as u16;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, game)).size = 128;
        v0 = (v0).wrapping_add(2);
        (*gVramWriteEntry(v0 as i32, game)).size =
            crate::rtl::rom_table_word!(game, 0x9bc346, (v1 >> 1) as isize);
        v0 = (v0).wrapping_add(2);
        crate::update_place! { previous fresh28 = v0 => v0.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh28 as i32,
            game,
        ))
        .size as *mut u8) = -102i32 as u8;
        (*gVramWriteEntry(v0 as i32, game)).size = 25104;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
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
                r18 = Samus_GetBottom_R18(game);
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

pub unsafe fn CheckBannedDraygonGrappleDirs(a: u16, game: &mut Game) -> u16 {
    unsafe {
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
macro_rules! clear_grapple_state_after_release {
    ($game:ident) => {
        word! { $game, 0;
            grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
            grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
            flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
            flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
            flare_fast_sparks_anim_timer,
        }
        let beams = word!($game.equipped_beams) & 0xfff;
        crate::game::samus_palette::write_beam_palette(
            &mut crate::game::session_runtime::Runtime::new($game),
            beams,
        );
        word!($game.grapple_beam_function = ((0x9bc4f0i32) as u16));
        word!($game.samus_draw_handler = ((0x90eb52i32) as u16));
        Samus_PostGrappleCollisionDetect($game);
        if word!($game.samus_auto_cancel_hud_item_index) != 0 {
            word! { $game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    };
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
                    samus_y_subspeed = Multiply16x16(v2 as u16, v0, game)
                );
                word!(game.samus_y_dir = 2);
            } else {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = Multiply16x16(-(v2 as i32) as u16, v0, game)
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
                    samus_y_subspeed = Multiply16x16(-(v1 as i32) as u16, v0, game)
                );
                word!(game.samus_y_dir = 2);
            } else {
                words!(
                    game.samus_y_speed,
                    samus_y_subspeed = Multiply16x16(v1 as u16, v0, game)
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
            samus_x_base_subspeed = Multiply16x16(v3, v0, game)
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallGrappleBeamFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10206052] GrappleBeamFunc_BB64 [] {
        if (word!(game.joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
            if word!(game.grapple_beam_length) != 0 {
                word!(game.grapple_beam_length_delta = -2i32 as u16);
            }
        } else if (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
            if word!(game.grapple_beam_length) as i32 - 64i32 & 0x8000 != 0 {
                word!(game.grapple_beam_length_delta = 2);
            } else {
                word!(game.grapple_beam_length = 64);
            }
        }
        if !(((*ram_ptr!(game; 0xcfb) as i32) << 8) - 0x4000i32 & 0x8000 != 0
            || ((*ram_ptr!(game; 0xcfb) as i32) << 8) + 0x4000 & 0x8000 == 0)
        {
            let continue_grapple_setup: bool = if (word!(game.joypad1_lastkeys) & 0x200) != 0 {
                if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000
                    && word!(game.grapple_beam_unkD26) == 0
                {
                    word!(game.grapple_beam_unkD26 = 256);
                }
                if word!(game.grapple_beam_flags) != 0 && (word!(game.grapple_beam_flags) & 1) != 0
                {
                    word!(game.grapple_beam_unkD2A = g_word_9BC11A >> 1);
                } else {
                    word!(game.grapple_beam_unkD2A = g_word_9BC11A);
                }
                false
            } else {
                ((word!(game.joypad1_lastkeys) & 0x100) == 0)
                    || ({
                        if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000
                            && word!(game.grapple_beam_unkD26) == 0
                        {
                            word!(game.grapple_beam_unkD26 = -256i32 as u16);
                        }
                        if word!(game.grapple_beam_flags) != 0
                            && (word!(game.grapple_beam_flags) & 1) != 0
                        {
                            word!(game.grapple_beam_unkD2A = -(g_word_9BC11A as i32 >> 1) as u16);
                        } else {
                            word!(game.grapple_beam_unkD2A = -(g_word_9BC11A as i32) as u16);
                        }
                        false
                    })
            };
            if !(continue_grapple_setup) {
                return;
            }
        }
        word!(game.grapple_beam_unkD2A = 0);
    };
    [10206239] GrappleBeamFunc_BC1F [] {
        if (word!(game.grapple_beam_end_angle16) & 0xc000) == 0xc000 {
            word!(game.grapple_beam_unkD2C = -(g_word_9BC11C as i32 >> 2) as u16);
            if word!(game.grapple_beam_flags) != 0 && (word!(game.grapple_beam_flags) & 1) != 0 {
                word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 3) as u16);
            } else {
                word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 2) as u16);
            }
        } else if word!(game.grapple_beam_end_angle16) & 0x8000 != 0 {
            if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000 {
                word! { game, 0; grapple_beam_unkD28, grapple_beam_unkD2C }
                let mut v0: u16 = word!(game.grapple_beam_unkD26);
                if (word!(game.grapple_beam_unkD26) & 0x8000) != 0 {
                    v0 = !(word!(game.grapple_beam_unkD26) as i32 - 1i32) as u16;
                }
                if byte!(v0; 1) as i32 - 1i32 & 0x8000 != 0 {
                    word!(game.grapple_beam_unkD26 = 0);
                }
            } else {
                word!(game.grapple_beam_unkD2C = -(g_word_9BC11C as i32) as u16);
                if word!(game.grapple_beam_flags) != 0 && (word!(game.grapple_beam_flags) & 1) != 0
                {
                    word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 1) as u16);
                } else {
                    word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32) as u16);
                }
            }
        } else if (word!(game.grapple_beam_end_angle16) & 0x4000) != 0 {
            word!(game.grapple_beam_unkD2C = g_word_9BC11C);
            if word!(game.grapple_beam_flags) != 0 && (word!(game.grapple_beam_flags) & 1) != 0 {
                word!(game.grapple_beam_unkD28 = g_word_9BC118 >> 1);
            } else {
                word!(game.grapple_beam_unkD28 = g_word_9BC118);
            }
        } else {
            word!(game.grapple_beam_unkD2C = g_word_9BC11C >> 2);
            if word!(game.grapple_beam_flags) != 0 && (word!(game.grapple_beam_flags) & 1) != 0 {
                word!(game.grapple_beam_unkD28 = g_word_9BC118 >> 3);
            } else {
                word!(game.grapple_beam_unkD28 = g_word_9BC118 >> 2);
            }
        };
    };
    [10206463] GrappleBeamFunc_BCFF [] {
        word!(checked_i32, game.grapple_beam_unkD26 += (word!(game.grapple_beam_unkD2A) as i32 + word!(game.grapple_beam_unkD28) as i32));
        if ((word!(game.grapple_beam_unkD26) ^ word!(game.grapple_beam_end_angle16)) & 0x8000) != 0
        {
            word!(game.grapple_beam_unkD26 += word!(game.grapple_beam_unkD2C));
        }
        if (word!(game.grapple_beam_unkD26) & 0x8000) == 0 {
            if word!(game.grapple_beam_unkD26) >= g_word_9BC11E {
                word!(game.grapple_beam_unkD26 = g_word_9BC11E);
            }
        } else if (-(word!(game.grapple_beam_unkD26) as i32) as u16) >= g_word_9BC11E {
            word!(game.grapple_beam_unkD26 = -(g_word_9BC11E as i32) as u16);
        }
    };
    [10206532] GrappleBeamFunc_BD44 [] {
        if word!(game.grapple_beam_unkD30) != 0
            && (word!(game.button_config_jump_a) & word!(game.joypad1_newkeys)) != 0
        {
            if word!(game.grapple_beam_unkD26) != 0 {
                if (word!(game.grapple_beam_unkD26) & 0x8000) != 0 {
                    if word!(game.grapple_beam_flags) != 0
                        && (word!(game.grapple_beam_flags) & 1) != 0
                    {
                        word!(game.grapple_beam_unkD2E = -(g_word_9BC120 as i32 >> 1) as u16);
                    } else {
                        word!(game.grapple_beam_unkD2E = -(g_word_9BC120 as i32) as u16);
                    }
                } else if word!(game.grapple_beam_flags) != 0
                    && (word!(game.grapple_beam_flags) & 1) != 0
                {
                    word!(game.grapple_beam_unkD2E = g_word_9BC120 >> 1);
                } else {
                    word!(game.grapple_beam_unkD2E = g_word_9BC120);
                }
            } else {
                word!(game.grapple_beam_unkD2E = 0);
            }
        }
    };
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
        HandleConnectingGrapple(game);
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
            GrappleBeamFunc_BB64(game);
            if word!(game.grapple_beam_length_delta) != 0 {
                change_length(game);
            }
            GrappleBeamFunc_BC1F(game);
            GrappleBeamFunc_BCFF(game);
            GrappleBeamFunc_BD44(game);
            advance_swing(game);
            if (word!(game.grapple_beam_unkD36) & 0x8000) != 0
                && HandleSpecialGrappleBeamAngles(game) & 1 != 0
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
    [10209366] GrappleBeamFunc_Cancel [] {
        QueueSfx1_Max15(7, game);
        if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_16_Grappling as i32 {
            pt::request_fallback_pose(&mut Runtime::new(game));
        } else {
            CallSomeSamusCode(0x1c, game);
        }
        clear_grapple_state_after_release!(game);
    };
    [10209477] GrappleBeam_Func2 [] {
        let mut current_block: u64;
        QueueSfx1_Max15(7, game);
        if word!(game.samus_pose) as i32 == kPose_B2_FaceR_Grapple_Air as i32 {
            current_block = 16041378612104114408;
        } else {
            if word!(game.samus_pose) as i32 == kPose_B3_FaceL_Grapple_Air as i32 {
                current_block = 13151459208701579741;
            } else if word!(game.samus_y_radius) as i32 - 17i32 & 0x8000 != 0 {
                if ((*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                    .offset(word!(game.samus_pose) as isize))
                .direction_shots_fired
                    & 0xf0)
                    != 0
                {
                    if *ram_ptr!(game; 0xa1e) == 4 {
                        word!(game.samus_new_pose_transitional = kPose_28_FaceL_Crouch as u16);
                    } else {
                        word!(game.samus_new_pose_transitional = kPose_27_FaceR_Crouch as u16);
                    }
                } else {
                    word!(
                        game.samus_new_pose_transitional = *(rom_ptr!(game, u8; 0x9bc9c4)).offset(
                            *(&raw mut (*(rom_ptr!(game, SamusPoseParams; 0x91b629)))
                                .direction_shots_fired)
                                .offset((8 * word!(game.samus_pose) as i32) as isize,)
                                as isize,
                        ) as u16
                    );
                }
                current_block = 8968942556654489080;
            } else if ((*(rom_ptr!(game, SamusPoseParams; 0x91b629))
                .offset(word!(game.samus_pose) as isize))
            .direction_shots_fired
                & 0xf0)
                != 0
            {
                if *ram_ptr!(game; 0xa1e) != 4 {
                    current_block = 16041378612104114408;
                } else {
                    current_block = 13151459208701579741;
                }
            } else {
                word!(
                    game.samus_new_pose_transitional = *(rom_ptr!(game, u8; 0x9bc9ba)).offset(
                        *(&raw mut (*(rom_ptr!(game, SamusPoseParams; 0x91b629)))
                            .direction_shots_fired)
                            .offset((8 * word!(game.samus_pose) as i32) as isize,)
                            as isize,
                    ) as u16
                );
                current_block = 8968942556654489080;
            }
            match current_block {
                8968942556654489080 => {}
                16041378612104114408 => {}
                _ => {
                    word!(game.samus_new_pose_transitional = kPose_02_FaceL_Normal as u16);
                    current_block = 8968942556654489080;
                }
            }
        }
        if current_block == 16041378612104114408 {
            word!(game.samus_new_pose_transitional = kPose_01_FaceR_Normal as u16);
        }
        word! {game; samus_hurt_switch_index = 0; input_to_pose_calc = 1; }
        word! { game, 0;
            samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
            grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
            grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
            flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
            flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
            flare_fast_sparks_anim_timer,
        }
        let beams = word!(game.equipped_beams) & 0xfff;
        crate::game::samus_palette::write_beam_palette(
            &mut crate::game::session_runtime::Runtime::new(game), beams,
        );
        word!(game.grapple_beam_function = (0x9bc4f0i32) as u16);
        word!(game.samus_draw_handler = (0x90eb52i32) as u16);
        Samus_PostGrappleCollisionDetect(game);
        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    };
    [10209742] GrappleBeamFunc_C9CE [] {
        QueueSfx1_Max15(7, game);
        if *ram_ptr!(game; 0xa1e) == 8 {
            word!(game.samus_new_pose_transitional = kPose_84_FaceL_Walljump as u16);
        } else {
            word!(game.samus_new_pose_transitional = kPose_83_FaceR_Walljump as u16);
        }
        word!(game.samus_hurt_switch_index = 6);
        word! { game, 0;
            samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
            UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed, samus_y_dir,
            UNUSED_word_7E0B38, samus_x_base_speed, samus_x_base_subspeed, grapple_beam_unkD1E,
            grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36, grapple_walljump_timer,
            slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags, flare_counter,
            flare_animation_frame, flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
            flare_animation_timer, flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
        }
        let beams = word!(game.equipped_beams) & 0xfff;
        crate::game::samus_palette::write_beam_palette(
            &mut crate::game::session_runtime::Runtime::new(game), beams,
        );
        word!(game.grapple_beam_function = (0x9bc4f0i32) as u16);
        word!(game.samus_draw_handler = (0x90eb52i32) as u16);
        Samus_PostGrappleCollisionDetect(game);
        if word!(game.samus_auto_cancel_hud_item_index) != 0 {
            word! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
        }
    };
    [10210187] GrappleBeamFunc_ReleaseFromSwing [] {
        QueueSfx1_Max15(7, game);
        if (word!(game.grapple_beam_unkD26) & 0x8000) == 0 {
            word!(game.samus_new_pose_transitional = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
        } else {
            word!(game.samus_new_pose_transitional = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
        }
        word!(game.samus_hurt_switch_index = 7);
        clear_grapple_state_after_release!(game);
    };
}
