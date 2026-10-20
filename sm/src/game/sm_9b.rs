use crate::c_library::memcpy;
pub use crate::game::constants::bank_9b::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::samus_movement::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{Multiply16x16, QueueSfx1_Max1, QueueSfx1_Max15, QueueSfx1_Max6};
use crate::game::sm_81::DrawBeamGrappleSpritemap;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game::sm_90::{
    CallSomeSamusCode, LoadProjectilePalette, Samus_DrawStartingDeathAnim, Samus_GetBottom_R18,
    Samus_GrappleWallJumpCheck, Samus_PostGrappleCollisionDetect,
};
use crate::game::sm_91::{
    SamusFunc_F433, Samus_DealDamage, Samus_LoadSuitPalette, Samus_Pose_CancelGrapple,
    Samus_SetAnimationFrameIfPoseChanged,
};
use crate::game::sm_92::DrawSamusSuitExploding;
use crate::game::sm_93::ProjectileInsts_GetValue;
use crate::game::sm_94::{
    kSinCosTable8bit_Sext, BlockCollGrappleBeam, BlockFunc_AC11, BlockFunc_AC31, BlockReactGrapple,
    GrappleFunc_AF87, HandleMovementAndCollForSamusGrapple,
};
use crate::game::sm_a0::{CalculateAngleFromXY, GrappleBeam_CollDetect_Enemy};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::SetHiLo;
use crate::rtl::{RomPtr, Unreachable};
pub use crate::types::{size_t, LongPtr, PairU16, Point16U, SamusPose, VoidP, VramWriteEntry};
pub use crate::types::{SamusMovementType, SamusPoseParams};
use ::core::ffi::c_void;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct GrappleBeamSpecialAngles {
        pub field_0, field_2, field_4, field_6: u16;
        pub field_8: VoidP;
    }
}
pub use crate::types::Buttons;
pub use crate::types::Consts_9B;
const RomPtr_9B: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x9b>;
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 0x90946e, fnSamus_MovementHandler_Normal = 0x90a337,
    fnSamusDrawHandler_Default = 0x90eb52, fnsub_90EB86 = 0x90eb86,
}
crate::game::constants::typed_constants! { i32;
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
    fnGrappleBeamFunc_Firing = 0x9bc703, fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
}
crate::game::constants::typed_constants! { i32;
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
pub unsafe fn ProjectileTrail_Func5(k: u16, j: u16, game: &mut Game) {
    let R22: u16 = ProjectileInsts_GetValue(k, game);
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    if (ram_word!(game, ceres_status) & 0x8000) == 0 {
        let v2: i32 = k as i32 >> 1;
        r18 = ram_word!(game, projectile_x_pos[v2]);
        r20 = ram_word!(game, projectile_y_pos[v2]);
    } else {
        let pt: Point16U = CalcExplosion_Mode7(k, game);
        r18 = (ram_word!(game, layer1_x_pos)).wrapping_add(pt.x);
        r20 = (ram_word!(game, layer1_y_pos)).wrapping_add(pt.y);
    }
    let v3: i32 = k as i32 >> 1;
    let v4: u16 = ram_word!(game, projectile_type[v3]);
    let mut v5: u16 = 0;
    if (v4 & 0x20) != 0 {
        v5 = (crate::rtl::rom_table_word!(
            game,
            0x9ba4e3,
            (ram_word!(game, projectile_type[v3]) as i32 & 0xf) as isize
        ) as i32
            + 2 * (ram_word!(game, projectile_dir[v3]) as i32 & 0xf)) as u16;
    } else if (v4 & 0x10) != 0 {
        v5 = (crate::rtl::rom_table_word!(
            game,
            0x9ba4cb,
            (ram_word!(game, projectile_type[v3]) as i32 & 0xf) as isize
        ) as i32
            + 2 * (ram_word!(game, projectile_dir[v3]) as i32 & 0xf)) as u16;
    } else {
        v5 = (crate::rtl::rom_table_word!(
            game,
            0x9ba4b3,
            (ram_word!(game, projectile_type[v3]) as i32 & 0xf) as isize
        ) as i32
            + 2 * (ram_word!(game, projectile_dir[v3]) as i32 & 0xf)) as u16;
    }
    let v6: u16 = (*(RomPtr_9B(v5, game) as *mut u16) as i32 + 4 * R22 as i32) as u16;
    let p = RomPtr_9B(v6, game);
    let v7: i32 = j as i32 >> 1;
    set_ram_word!(
        game.projectiletrail_left_y_pos[v7] =
            (r20 as i32 + *p.offset(1) as i8 as i32 - 4i32) as u16
    );
    set_ram_word!(
        game.projectiletrail_left_x_pos[v7] =
            (r18 as i32 + *p.offset(0) as i8 as i32 - 4i32) as u16
    );
    set_ram_word!(
        game.projectiletrail_right_y_pos[v7] =
            (r20 as i32 + *p.offset(3) as i8 as i32 - 4i32) as u16
    );
    set_ram_word!(
        game.projectiletrail_right_x_pos[v7] =
            (r18 as i32 + *p.offset(2) as i8 as i32 - 4i32) as u16
    );
}
pub unsafe fn StartSamusDeathAnimation(game: &mut Game) {
    let v0: u16 = *ram_ptr!(game; 0xa1f) as u16;
    if *ram_ptr!(game; 0xa1f) == 3 {
        QueueSfx1_Max6(0x32, game);
    }
    let v1: u16 = kDeathAnimationFrames[v0 as usize] as u16;
    if *ram_ptr!(game; 0xa1e) == 4 {
        set_ram_word!(game.samus_pose = kPose_D8_FaceL_CrystalFlashEnd as u16);
    } else {
        set_ram_word!(game.samus_pose = kPose_D7_FaceR_CrystalFlashEnd as u16);
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
    *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
    set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
    *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
    set_ram_word!(game.samus_anim_frame_skip = 0);
    set_ram_word!(game.samus_anim_frame = v1);
    update_ram_word!(
        game.samus_x_pos,
        wrapping_sub(ram_word!(game, layer1_x_pos))
    );
    update_ram_word!(
        game.samus_y_pos,
        wrapping_sub(ram_word!(game, layer1_y_pos))
    );
}
pub unsafe fn DrawSamusStartingDeathAnim_(game: &mut Game) {
    Samus_DrawStartingDeathAnim(game);
}
pub unsafe fn HandleSamusDeathSequence(game: &mut Game) -> u16 {
    if ram_word!(game, g_word_7E0DE6) as i32 - 4i32 & 0x8000 != 0 {
        QueueTransferOfSamusDeathSequence(
            (2u16).wrapping_mul(ram_word!(game, g_word_7E0DE6)),
            game,
        );
    }
    if update_ram_word!(game.g_word_7E0DE6, wrapping_add(1), reread) as i32 - 60i32 & 0x8000 != 0 {
        let v0: bool = (update_ram_word!(game.game_options_screen_index, wrapping_sub(1), reread)
            & 0x8000)
            != 0;
        if ram_word!(game, game_options_screen_index) == 0 || v0 {
            if ram_word!(game, g_word_7E0DE4) != 0 {
                set_ram_word!(game.g_word_7E0DE4 = 0);
                set_ram_word!(game.game_options_screen_index = 3);
            } else {
                set_ram_words! { game, 1; g_word_7E0DE4, game_options_screen_index }
            }
            CopyPalettesForSamusDeath((ram_word!(game, g_word_7E0DE4)).wrapping_mul(2), game);
        }
        0
    } else {
        HandleSamusDeathSequence_Helper2(game);
        set_ram_word!(
            game.substate = (ram_word!(game, joypad2_last) as i32 & 0xb0
                == kButton_A as i32 | kButton_L as i32 | kButton_R as i32)
                as u16
        );
        1
    }
}
pub unsafe fn HandleSamusDeathSequence_Helper2(game: &mut Game) {
    let v0 = RomPtr_9B(
        crate::rtl::rom_table_word!(
            game,
            0x9bb5c8,
            (ram_word!(game, samus_suit_palette_index) >> 1) as isize
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
    set_ram_word!(
        game.game_options_screen_index =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bb823)) as *mut u8).offset(0) as u16
    );
    set_ram_words! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
    GameState_24_SamusNoHealth_Explosion_2(game);
}
pub unsafe fn CopyPalettesForSamusDeath(v0: u16, game: &mut Game) {
    let r20: i32 = crate::rtl::rom_table_word!(
        game,
        0x9bb6d2,
        (ram_word!(game, samus_suit_palette_index) >> 1) as isize
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
pub unsafe fn QueueTransferOfSamusDeathSequence(v0: u16, game: &mut Game) {
    let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 1024;
    v1 = (v1).wrapping_add(2);
    let v2: i32 = v0 as i32 >> 1;
    (*gVramWriteEntry(v1 as i32, game)).size = g_word_9BB7BF[v2 as usize];
    v1 = (v1).wrapping_add(2);
    crate::update_place! { previous fresh7 = v1 => v1.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh7 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = 0x9b;
    (*gVramWriteEntry(v1 as i32, game)).size = g_word_9BB7C9[v2 as usize];
    set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(2));
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_Helper(game: &mut Game) -> u16 {
    GameState_24_SamusNoHealth_Explosion_1(game);
    GameState_24_SamusNoHealth_Explosion_2(game)
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_1(game: &mut Game) {
    if ram_word!(game, substate) == 0 && ram_word!(game, g_word_7E0DE4) != 0 {
        let v0: i32 = ram_word!(game, g_word_7E0DE6) as i32;
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
        if ram_word!(game, g_word_7E0DE6) as i32 - 20i32 & 0x8000 != 0 {
            update_ram_word!(game.g_word_7E0DE6, wrapping_add(1));
        }
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_2(game: &mut Game) -> u16 {
    let v0: bool =
        (update_ram_word!(game.game_options_screen_index, wrapping_sub(1), reread) & 0x8000) != 0;
    if ram_word!(game, game_options_screen_index) == 0 || v0 {
        if update_ram_word!(game.g_word_7E0DE4, wrapping_add(1), reread) as i32 - 9i32 & 0x8000 == 0
        {
            set_ram_word!(game.g_word_7E0DE6 = 21);
            GameState_24_SamusNoHealth_Explosion_1(game);
            set_ram_word!(game.substate = 0);
            return 1;
        }
        if ram_word!(game, substate) == 0
            || ram_word!(game, g_word_7E0DE4) as i32 - 2i32 & 0x8000 != 0
        {
            set_ram_word!(
                game.game_options_screen_index =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bb823)) as *mut u8)
                        .offset((2 * ram_word!(game, g_word_7E0DE4) as i32) as isize,)
                        as u16
            );
            CopyPalettesForSamusDeath(
                (2u16).wrapping_mul(
                    (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bb823)) as *mut u8)
                        .offset((2 * ram_word!(game, g_word_7E0DE4) as i32 + 1) as isize))
                        as u16,
                ),
                game,
            );
        } else {
            set_ram_word!(
                game.game_options_screen_index =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bb823)) as *mut u8)
                        .offset((2 * ram_word!(game, g_word_7E0DE4) as i32) as isize,)
                        as u16
            );
        }
    }
    DrawSamusSuitExploding(game);
    0
}
pub unsafe fn CancelGrappleBeamIfIncompatiblePose(game: &mut Game) {
    if (kIsGrappleBannedForMovementType[*ram_ptr!(game; 0xa1f) as usize] == 0) {
        if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
            && ram_word!(game, grapple_beam_function) as i32 + 0x3882 & 0x8000 != 0
        {
            let mut v0: i16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .direction_shots_fired as i16;
            if (v0 as u16 & 0xf0) == 0 {
                if v0 as i32 == ram_word!(game, grapple_beam_direction) as i32 {
                    return;
                }
                if ram_word!(game, grapple_varCF6) != 0 {
                    QueueSfx1_Max6(7, game);
                    set_ram_word!(game.grapple_beam_function = ((0x9bc51ei32) as u16));
                    return;
                }
            }
        } else {
            return;
        }
    }
    if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
    }
}
pub unsafe fn CheckIfGrappleIsConnectedToBlock(game: &mut Game) -> u8 {
    set_ram_words! { game, 0;
        grapple_beam_extension_x_velocity, grapple_beam_extension_y_velocity,
    }
    if (BlockReactGrapple(game) & 1) == 0 {
        return 0;
    }
    set_ram_word!(game.samus_grapple_flags = 1);
    1
}
unsafe fn ProcessEnemyGrappleBeamColl(a: u16, mut r18: u16, game: &mut Game) -> i32 {
    match a as i32 {
        0 | 2 => -1i32,
        3 => 1,
        1 | 4 | 5 => 0,
        6 => {
            r18 = *(RomPtr_A0(r18, game) as *mut u16).offset(3);
            if (ram_word!(game, equipped_items) & 0x20) != 0 {
                r18 = (r18 >> 2);
            } else if ram_word!(game, equipped_items) & 1 != 0 {
                r18 = (r18 >> 1);
            }
            Samus_DealDamage(r18, game);
            set_ram_word!(game.samus_invincibility_timer = 96);
            set_ram_word!(game.samus_knockback_timer = 5);
            set_ram_word!(game.knockback_x_dir = (*ram_ptr!(game; 0xa1e) == 4) as u16);
            1
        }
        _ => {
            Unreachable(game);
            0
        }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallGrappleNextFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10205657 => GrappleNext_SwingClockwise; 10205666 => GrappleNext_SwingAntiClockwise;
        10205674 => GrappleNext_StandAimRight; 10205683 => GrappleNext_StandAimDownRight;
        10205692 => GrappleNext_StandAimDownLeft; 10205701 => GrappleNext_StandAimLeft;
        10205710 => GrappleNext_CrouchAimRight; 10205719 => GrappleNext_CrouchAimDownRight;
        10205728 => GrappleNext_CrouchAimDownLeft; 10205737 => GrappleNext_CrouchAimLeft;
    }
}
pub unsafe fn HandleConnectingGrapple(game: &mut Game) {
    if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_1A_GrabbedByDraygon as i32 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc77ei32) as u16));
        set_ram_word!(game.grapple_beam_length_delta = 0);
    } else {
        let v1: i32 = 2 * ram_word!(game, grapple_beam_direction) as i32;
        if ram_word!(game, samus_y_speed) != 0 || ram_word!(game, samus_y_subspeed) != 0 {
            set_ram_word!(
                game.grapple_beam_function =
                    crate::rtl::rom_table_word!(game, 0x9bc3ee, v1 as isize)
            );
            CallGrappleNextFunc(
                ((crate::rtl::rom_table_word!(game, 0x9bc3ee, (v1 + 1) as isize)) as u32)
                    | (0x9b0000u32),
                game,
            );
        } else if *ram_ptr!(game; 0xa1f) == 5 {
            set_ram_word!(
                game.grapple_beam_function =
                    crate::rtl::rom_table_word!(game, 0x9bc416, v1 as isize)
            );
            CallGrappleNextFunc(
                ((crate::rtl::rom_table_word!(game, 0x9bc416, (v1 + 1) as isize)) as u32)
                    | (0x9b0000u32),
                game,
            );
        } else {
            set_ram_word!(
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
pub unsafe fn GrappleNext_SwingClockwise(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B2_FaceR_Grapple_Air as u16);
    HandleConnectingGrapple_Swinging(game);
}
pub unsafe fn GrappleNext_SwingAntiClockwise(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B3_FaceL_Grapple_Air as u16);
    HandleConnectingGrapple_Swinging(game);
}
pub unsafe fn GrappleNext_StandAimRight(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_A8_FaceR_Grappling as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimDownRight(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_AA_FaceR_Grappling_AimDR as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimDownLeft(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_AB_FaceL_Grappling_AimDL as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimLeft(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_A9_FaceL_Grappling as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimRight(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B4_FaceR_Grappling_Crouch as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimDownRight(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B6_FaceR_Grappling_Crouch_AimDR as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimDownLeft(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B7_FaceL_Grappling_Crouch_AimDL as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimLeft(game: &mut Game) {
    set_ram_word!(game.samus_new_pose_interrupted = kPose_B5_FaceL_Grappling_Crouch as u16);
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn HandleConnectingGrapple_Swinging(game: &mut Game) {
    set_ram_word!(game.samus_special_transgfx_index = 9);
    let v0: u16 = u16::swap_bytes(CalculateAngleFromXY(
        (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, grapple_beam_end_x_pos)),
        (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, grapple_beam_end_y_pos)),
        game,
    ));
    set_ram_word!(game.grapple_beam_end_angle16 = v0);
    set_ram_word!(game.grapple_beam_end_angles_mirror = v0);
    set_ram_word!(game.grapple_beam_length_delta = 0);
    if ram_word!(game, grapple_beam_length) as i32 - 64i32 & 0x8000 == 0 {
        update_ram_word!(game.grapple_beam_length, wrapping_sub(24));
    }
    BlockFunc_AC11(game);
}
pub unsafe fn HandleConnectingGrapple_StuckInPlace(game: &mut Game) {
    set_ram_word!(game.samus_special_transgfx_index = 10);
    let v0: u16 = u16::swap_bytes(CalculateAngleFromXY(
        (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, grapple_beam_end_x_pos)),
        (ram_word!(game, samus_y_pos)).wrapping_sub(ram_word!(game, grapple_beam_end_y_pos)),
        game,
    ));
    set_ram_word!(game.grapple_beam_end_angle16 = v0);
    set_ram_word!(game.grapple_beam_end_angles_mirror = v0);
    set_ram_word!(game.grapple_beam_length_delta = 0);
    if ram_word!(game, grapple_beam_length) as i32 - 64i32 & 0x8000 == 0 {
        update_ram_word!(game.grapple_beam_length, wrapping_sub(24));
    }
    BlockFunc_AC11(game);
}
pub unsafe fn HandleSpecialGrappleBeamAngles(game: &mut Game) -> u8 {
    let mut v0: u16 = 7;
    while ram_word!(game, grapple_beam_end_angle16)
        != (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc43e))
            as *mut GrappleBeamSpecialAngles)
            .offset(v0 as isize))
        .field_0
    {
        v0 = v0.wrapping_sub(1);
        if (v0 & 0x8000) != 0 {
            return 0;
        }
    }
    set_ram_word!(
        game.samus_new_pose_interrupted = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc43e))
            as *mut GrappleBeamSpecialAngles)
            .offset(v0 as isize))
        .field_2
    );
    set_ram_word!(
        game.samus_x_pos = (ram_word!(game, grapple_beam_end_x_pos)).wrapping_add(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc43e))
                as *mut GrappleBeamSpecialAngles)
                .offset(v0 as isize))
            .field_4,
        )
    );
    set_ram_word!(
        game.samus_y_pos = (ram_word!(game, grapple_beam_end_y_pos)).wrapping_add(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc43e))
                as *mut GrappleBeamSpecialAngles)
                .offset(v0 as isize))
            .field_6,
        )
    );
    set_ram_word!(
        game.grapple_beam_function = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc43e))
            as *mut GrappleBeamSpecialAngles)
            .offset(v0 as isize))
        .field_8
    );
    set_ram_words! { game, 0; samus_special_transgfx_index, slow_grabble_scrolling_flag }
    let v2: i16 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0
    {
        if v2 as i32 + 12 & 0x8000 != 0 {
            set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(12));
        }
    } else if v2 as i32 - 13i32 & 0x8000 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(12));
    }
    let v3: i16 = ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16);
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        < 0
    {
        if v3 as i32 + 12 & 0x8000 != 0 {
            set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(12));
        }
    } else if v3 as i32 - 13i32 & 0x8000 == 0 {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(12));
    }
    1
}
pub unsafe fn GrappleBeamFunc_BB64(game: &mut Game) {
    let continue_grapple_setup: bool;
    if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Up as i32) != 0 {
        if ram_word!(game, grapple_beam_length) != 0 {
            set_ram_word!(game.grapple_beam_length_delta = -2i32 as u16);
        }
    } else if (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0 {
        if ram_word!(game, grapple_beam_length) as i32 - 64i32 & 0x8000 != 0 {
            set_ram_word!(game.grapple_beam_length_delta = 2);
        } else {
            set_ram_word!(game.grapple_beam_length = 64);
        }
    }
    if !(((*ram_ptr!(game; 0xcfb) as i32) << 8) - 0x4000i32 & 0x8000 != 0
        || ((*ram_ptr!(game; 0xcfb) as i32) << 8) + 0x4000 & 0x8000 == 0)
    {
        continue_grapple_setup = if (ram_word!(game, joypad1_lastkeys) & 0x200) != 0 {
            if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000
                && ram_word!(game, grapple_beam_unkD26) == 0
            {
                set_ram_word!(game.grapple_beam_unkD26 = 256);
            }
            if ram_word!(game, grapple_beam_flags) != 0
                && (ram_word!(game, grapple_beam_flags) & 1) != 0
            {
                set_ram_word!(game.grapple_beam_unkD2A = (g_word_9BC11A >> 1));
            } else {
                set_ram_word!(game.grapple_beam_unkD2A = g_word_9BC11A);
            }
            false
        } else {
            ((ram_word!(game, joypad1_lastkeys) & 0x100) == 0)
                || ({
                    if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000
                        && ram_word!(game, grapple_beam_unkD26) == 0
                    {
                        set_ram_word!(game.grapple_beam_unkD26 = -256i32 as u16);
                    }
                    if ram_word!(game, grapple_beam_flags) != 0
                        && (ram_word!(game, grapple_beam_flags) & 1) != 0
                    {
                        set_ram_word!(
                            game.grapple_beam_unkD2A = -(g_word_9BC11A as i32 >> 1) as u16
                        );
                    } else {
                        set_ram_word!(game.grapple_beam_unkD2A = -(g_word_9BC11A as i32) as u16);
                    }
                    false
                })
        };
        if !(continue_grapple_setup) {
            return;
        }
    }
    set_ram_word!(game.grapple_beam_unkD2A = 0);
}
pub unsafe fn GrappleBeamFunc_BC1F(game: &mut Game) {
    if (ram_word!(game, grapple_beam_end_angle16) & 0xc000) == 0xc000 {
        set_ram_word!(game.grapple_beam_unkD2C = -(g_word_9BC11C as i32 >> 2) as u16);
        if ram_word!(game, grapple_beam_flags) != 0
            && (ram_word!(game, grapple_beam_flags) & 1) != 0
        {
            set_ram_word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 3) as u16);
        } else {
            set_ram_word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 2) as u16);
        }
    } else if ram_word!(game, grapple_beam_end_angle16) & 0x8000 != 0 {
        if ((*ram_ptr!(game; 0xcfb) as i32) << 8) == 0x8000 {
            set_ram_words! { game, 0; grapple_beam_unkD28, grapple_beam_unkD2C }
            let mut v0: u16 = ram_word!(game, grapple_beam_unkD26);
            if (ram_word!(game, grapple_beam_unkD26) & 0x8000) != 0 {
                v0 = !(ram_word!(game, grapple_beam_unkD26) as i32 - 1i32) as u16;
            }
            if *(&raw mut v0 as *mut u8).offset(1) as i32 - 1i32 & 0x8000 != 0 {
                set_ram_word!(game.grapple_beam_unkD26 = 0);
            }
        } else {
            set_ram_word!(game.grapple_beam_unkD2C = -(g_word_9BC11C as i32) as u16);
            if ram_word!(game, grapple_beam_flags) != 0
                && (ram_word!(game, grapple_beam_flags) & 1) != 0
            {
                set_ram_word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32 >> 1) as u16);
            } else {
                set_ram_word!(game.grapple_beam_unkD28 = -(g_word_9BC118 as i32) as u16);
            }
        }
    } else if (ram_word!(game, grapple_beam_end_angle16) & 0x4000) != 0 {
        set_ram_word!(game.grapple_beam_unkD2C = g_word_9BC11C);
        if ram_word!(game, grapple_beam_flags) != 0
            && (ram_word!(game, grapple_beam_flags) & 1) != 0
        {
            set_ram_word!(game.grapple_beam_unkD28 = (g_word_9BC118 >> 1));
        } else {
            set_ram_word!(game.grapple_beam_unkD28 = g_word_9BC118);
        }
    } else {
        set_ram_word!(game.grapple_beam_unkD2C = (g_word_9BC11C >> 2));
        if ram_word!(game, grapple_beam_flags) != 0
            && (ram_word!(game, grapple_beam_flags) & 1) != 0
        {
            set_ram_word!(game.grapple_beam_unkD28 = (g_word_9BC118 >> 3));
        } else {
            set_ram_word!(game.grapple_beam_unkD28 = (g_word_9BC118 >> 2));
        }
    };
}
pub unsafe fn GrappleBeamFunc_BCFF(game: &mut Game) {
    update_ram_word!(game.grapple_beam_unkD26, |fresh14| (fresh14 as i32
        + (ram_word!(game, grapple_beam_unkD2A) as i32
            + ram_word!(game, grapple_beam_unkD28) as i32))
        as u16);
    if ((ram_word!(game, grapple_beam_unkD26) ^ ram_word!(game, grapple_beam_end_angle16)) & 0x8000)
        != 0
    {
        update_ram_word!(
            game.grapple_beam_unkD26,
            wrapping_add(ram_word!(game, grapple_beam_unkD2C))
        );
    }
    if (ram_word!(game, grapple_beam_unkD26) & 0x8000) == 0 {
        if ram_word!(game, grapple_beam_unkD26) >= g_word_9BC11E {
            set_ram_word!(game.grapple_beam_unkD26 = g_word_9BC11E);
        }
    } else if (-(ram_word!(game, grapple_beam_unkD26) as i32) as u16) >= g_word_9BC11E {
        set_ram_word!(game.grapple_beam_unkD26 = -(g_word_9BC11E as i32) as u16);
    }
}
pub unsafe fn GrappleBeamFunc_BD44(game: &mut Game) {
    if ram_word!(game, grapple_beam_unkD30) != 0
        && (ram_word!(game, button_config_jump_a) & ram_word!(game, joypad1_newkeys)) != 0
    {
        if ram_word!(game, grapple_beam_unkD26) != 0 {
            if (ram_word!(game, grapple_beam_unkD26) & 0x8000) != 0 {
                if ram_word!(game, grapple_beam_flags) != 0
                    && (ram_word!(game, grapple_beam_flags) & 1) != 0
                {
                    set_ram_word!(game.grapple_beam_unkD2E = -(g_word_9BC120 as i32 >> 1) as u16);
                } else {
                    set_ram_word!(game.grapple_beam_unkD2E = -(g_word_9BC120 as i32) as u16);
                }
            } else if ram_word!(game, grapple_beam_flags) != 0
                && (ram_word!(game, grapple_beam_flags) & 1) != 0
            {
                set_ram_word!(game.grapple_beam_unkD2E = (g_word_9BC120 >> 1));
            } else {
                set_ram_word!(game.grapple_beam_unkD2E = g_word_9BC120);
            }
        } else {
            set_ram_word!(game.grapple_beam_unkD2E = 0);
        }
    }
}
pub unsafe fn GrappleBeamFunc_BD95(game: &mut Game) {
    let v0: u16 = abs16(ram_word!(game, grapple_beam_unkD26));
    let mut v1: u16 = 0;
    let start_grapple_animation: bool = if v0 as i32 - 64i32 & 0x8000 == 0 {
        set_ram_word!(game.slow_grabble_scrolling_flag = 1);
        true
    } else {
        set_ram_word!(game.slow_grabble_scrolling_flag = 0);
        (((*ram_ptr!(game; 0xcfb) as i32) << 8) != 0x8000)
            || ({
                if ram_word!(game, samus_anim_frame) as i32 - 64i32 & 0x8000 != 0 {
                    set_ram_word!(game.samus_anim_frame_timer = 8);
                    set_ram_word!(game.samus_anim_frame = 64);
                }
                v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc1c2)) as *mut u8)
                    .offset(*(ram_ptr!(game, u16; 0xcfc) as *mut u8).offset(1) as isize)
                    as u16;
                false
            })
    };
    if start_grapple_animation {
        set_ram_word!(game.samus_anim_frame_timer = 15);
        v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc1c2)) as *mut u8)
            .offset(*(ram_ptr!(game, u16; 0xcfc) as *mut u8).offset(1) as isize)
            as u16;
        set_ram_word!(game.samus_anim_frame = v1);
    }
    let v2: u16 = (2u16).wrapping_mul(v1);
    if (abs16(ram_word!(game, grapple_beam_unkD2E)) & 0xff00) == 256 {
        let mut v3: u16 = ram_word!(game, samus_anim_frame);
        if ram_word!(game, samus_anim_frame) as i32 - 64i32 & 0x8000 == 0 {
            v3 = 16;
        }
        set_ram_word!(game.samus_anim_frame = (v3).wrapping_add(32));
    }
    if *ram_ptr!(game; 0xa1e) == 4 {
        set_ram_word!(
            game.samus_x_pos = (ram_word!(game, x_pos_of_start_of_grapple_beam)).wrapping_add(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc2c2)) as *mut u8)
                    .offset(v2 as isize) as i8) as u16,
            )
        );
        set_ram_word!(
            game.samus_y_pos = (ram_word!(game, y_pos_of_start_of_grapple_beam)).wrapping_add(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc2c2)) as *mut u8)
                    .offset((v2 as i32 + 1) as isize) as i8) as u16,
            )
        );
    } else {
        set_ram_word!(
            game.samus_x_pos = (ram_word!(game, x_pos_of_start_of_grapple_beam)).wrapping_add(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc302)) as *mut u8)
                    .offset(v2 as isize) as i8) as u16,
            )
        );
        set_ram_word!(
            game.samus_y_pos = (ram_word!(game, y_pos_of_start_of_grapple_beam)).wrapping_add(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc302)) as *mut u8)
                    .offset((v2 as i32 + 1) as isize) as i8) as u16,
            )
        );
    }
    set_ram_word!(
        game.x_pos_of_start_of_grapple_beam_prevframe =
            ram_word!(game, x_pos_of_start_of_grapple_beam)
    );
    set_ram_word!(
        game.y_pos_of_start_of_grapple_beam_prevframe =
            ram_word!(game, y_pos_of_start_of_grapple_beam)
    );
    GrappleBeamFunc_BE98(game);
}
pub unsafe fn GrappleBeamFunc_BE98(game: &mut Game) {
    let mut v0: i16 = ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16);
    if ((ram_word!(game, samus_x_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_x_pos)) as i16)
        < 0
    {
        if v0 as i32 + 12 & 0x8000 != 0 {
            set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_add(12));
        }
    } else if v0 as i32 - 13i32 & 0x8000 == 0 {
        set_ram_word!(game.samus_prev_x_pos = (ram_word!(game, samus_x_pos)).wrapping_sub(12));
    }
    let mut v1: i16 = ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16);
    if ((ram_word!(game, samus_y_pos)) as i16)
        .wrapping_sub((ram_word!(game, samus_prev_y_pos)) as i16)
        < 0
    {
        if v1 as i32 + 12 & 0x8000 != 0 {
            set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(12));
        }
    } else if v1 as i32 - 13i32 & 0x8000 == 0 {
        set_ram_word!(game.samus_prev_y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(12));
    }
}
pub unsafe fn GrappleBeamFunc_BEEB(game: &mut Game) {
    let v0: i32 = ram_word!(game, grapple_beam_direction) as i32;
    set_ram_word!(
        game.samus_x_pos = (ram_word!(game, x_pos_of_start_of_grapple_beam))
            .wrapping_sub(crate::rtl::rom_table_word!(game, 0x9bc122, v0 as isize),)
    );
    set_ram_word!(
        game.x_pos_of_start_of_grapple_beam_prevframe =
            (crate::rtl::rom_table_word!(game, 0x9bc14a, v0 as isize))
                .wrapping_add(ram_word!(game, samus_x_pos))
    );
    set_ram_word!(
        game.samus_y_pos = (ram_word!(game, y_pos_of_start_of_grapple_beam))
            .wrapping_sub(crate::rtl::rom_table_word!(game, 0x9bc136, v0 as isize),)
    );
    set_ram_word!(
        game.y_pos_of_start_of_grapple_beam_prevframe =
            (crate::rtl::rom_table_word!(game, 0x9bc15e, v0 as isize))
                .wrapping_add(ram_word!(game, samus_y_pos))
    );
}
pub unsafe fn GrappleBeamFunc_BF1B(game: &mut Game) {
    let r22: u16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
        as *mut SamusPoseParams)
        .offset(ram_word!(game, samus_pose) as isize))
    .y_offset_to_gfx as u16;
    let v0: u16 = (2u16).wrapping_mul(ram_word!(game, grapple_beam_direction));
    if ram_word!(game, samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
        || ram_word!(game, samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
        || *ram_ptr!(game; 0xa1f) != 1
    {
        let v1: i32 = v0 as i32 >> 1;
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!(game, 0x9bc122, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc14a, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!(game, 0x9bc136, v1 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc15e, v1 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
    } else {
        let v2: i32 = v0 as i32 >> 1;
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!(game, 0x9bc172, v2 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc19a, v2 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam =
                (crate::rtl::rom_table_word!(game, 0x9bc186, v2 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc1ae, v2 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
    };
}
pub unsafe fn UpdateGrappleBeamTilesAndIncrFlameCtr(game: &mut Game) {
    UpdateGrappleBeamTiles(game);
    if ram_word!(game, flare_counter) as i32 - 120i32 & 0x8000 != 0 {
        update_ram_word!(game.flare_counter, wrapping_add(1));
    }
}
pub unsafe fn UpdateGrappleBeamTiles(game: &mut Game) {
    if (update_ram_word!(game.grapple_point_anim_timer, wrapping_sub(1), reread) & 0x8000) != 0 {
        set_ram_word!(game.grapple_point_anim_timer = 5);
        update_ram_word!(game.grapple_point_anim_ptr, wrapping_add(512));
        if ((ram_word!(game, grapple_point_anim_ptr)) as i16)
            .wrapping_sub((crate::rtl::rom_table_word!(game, 0x9bc344, 0)) as i16)
            >= 0
        {
            set_ram_word!(
                game.grapple_point_anim_ptr = crate::rtl::rom_table_word!(game, 0x9bc342, 0)
            );
        }
    }
    let mut v0: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 32;
    v0 = (v0).wrapping_add(2);
    (*gVramWriteEntry(v0 as i32, game)).size = ram_word!(game, grapple_point_anim_ptr);
    v0 = (v0).wrapping_add(2);
    crate::update_place! { previous fresh27 = v0 => v0.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh27 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = -102i32 as u8;
    (*gVramWriteEntry(v0 as i32, game)).size = 25088;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
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
    .size as *mut u8)
        .offset(0) = -102i32 as u8;
    (*gVramWriteEntry(v0 as i32, game)).size = 25104;
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(2));
}
pub unsafe fn HandleGrappleBeamFlare(game: &mut Game) {
    if ram_word!(game, flare_counter) != 0 {
        if ram_word!(game, flare_counter) == 1 {
            set_ram_word!(game.flare_animation_frame = 16);
            set_ram_word!(game.flare_animation_timer = 3);
        }
        if (update_ram_word!(game.flare_animation_timer, wrapping_sub(1), reread) & 0x8000) != 0 {
            let mut v0: u16 = update_ram_word!(game.flare_animation_frame, wrapping_add(1), reread);
            if *(game.g_rom.offset(crate::rtl::lorom_offset(0x90c487)) as *mut u8)
                .offset(ram_word!(game, flare_animation_frame) as isize)
                == 254
            {
                update_ram_word!(
                    game.flare_animation_frame,
                    wrapping_sub(
                        (*(game.g_rom.offset(crate::rtl::lorom_offset(0x90c487)) as *mut u8)
                            .offset(
                                ((ram_word!(game, flare_animation_frame)).wrapping_add(1)) as isize,
                            )) as u16
                    )
                );
                v0 = ram_word!(game, flare_animation_frame);
            }
            set_ram_word!(
                game.flare_animation_timer = *(game.g_rom.offset(crate::rtl::lorom_offset(0x90c487))
                    as *mut u8)
                    .offset(v0 as isize) as u16
            );
        }
        let mut r22: u16 = 0;
        if *ram_ptr!(game; 0xa1e) == 4 {
            r22 = (ram_word!(game, flare_animation_frame))
                .wrapping_add(crate::rtl::rom_table_word!(game, 0x93a22b, 0));
        } else {
            r22 = (ram_word!(game, flare_animation_frame))
                .wrapping_add(crate::rtl::rom_table_word!(game, 0x93a225, 0));
        }
        let r20: u16 = (ram_word!(game, x_pos_of_start_of_grapple_beam_prevframe))
            .wrapping_sub(ram_word!(game, layer1_x_pos));
        let r18: u16 = (ram_word!(game, y_pos_of_start_of_grapple_beam_prevframe))
            .wrapping_sub(ram_word!(game, layer1_y_pos));
        if ((ram_word!(game, y_pos_of_start_of_grapple_beam_prevframe) as i32
            - ram_word!(game, layer1_y_pos) as i32
            & 0xff00)
            == 0)
        {
            DrawBeamGrappleSpritemap(r22, r20, r18, game);
        }
    }
}
pub unsafe fn GrappleBeamHandler(game: &mut Game) {
    let mut r18: u16 = 0;
    if ram_word!(game, grapple_varCF6) != 0 {
        update_ram_word!(game.grapple_varCF6, wrapping_sub(1));
    }
    update_ram_word!(game.samus_grapple_flags, |fresh9| (fresh9 & !(1)));
    CancelGrappleBeamIfIncompatiblePose(game);
    CallGrappleBeamFunc(
        ((ram_word!(game, grapple_beam_function)) as u32) | (0x9b0000u32),
        game,
    );
    if ram_word!(game, grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
        && ram_word!(game, grapple_beam_function) as i32 - (0x9bc856 & 0xffff) & 0x8000 != 0
        && (ram_word!(game, samus_suit_palette_index) & 4) == 0
        && ram_word!(game, fx_type) != 0
        && {
            r18 = Samus_GetBottom_R18(game);
            (ram_word!(game, fx_y_pos) & 0x8000) == 0
        }
        && ram_word!(game, fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
    {
        update_ram_word!(game.grapple_beam_flags, |fresh10| (fresh10) | (1u16));
    } else {
        update_ram_word!(game.grapple_beam_flags, |fresh11| (fresh11 & !(1)));
    };
}
pub unsafe fn GrappleBeamFunc_Inactive(game: &mut Game) {
    if (ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_newkeys)) != 0
        || (ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_newinput_samusfilter))
            != 0
    {
        GrappleBeamFunc_FireGoToCancel(game);
    } else if ram_word!(game, flare_counter) != 0 {
        set_ram_words! { game, 0;
            flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
            flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
            flare_fast_sparks_anim_timer,
        }
        Samus_LoadSuitPalette(game);
    }
}
pub unsafe fn GrappleBeamFunc_FireGoToCancel(game: &mut Game) {
    let mut r22: u16 = 0;
    let mut v0: i32 = 0;
    if ram_word!(game, samus_pose) as i32 == kPose_F0_FaceR_Draygon_Move as i32
        || ram_word!(game, samus_pose) as i32 == kPose_BE_FaceL_Draygon_Move as i32
    {
        v0 = CheckBannedDraygonGrappleDirs(ram_word!(game, samus_pose), game) as i32;
        r22 = 6;
    } else {
        r22 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .y_offset_to_gfx as u16;
        v0 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
        .direction_shots_fired as i32;
        if (v0 & 0xf0) != 0 {
            set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
            return;
        }
    }
    set_ram_word!(game.grapple_beam_direction = v0 as u16);
    let v1: i32 = v0;
    set_ram_word!(
        game.grapple_beam_extension_x_velocity =
            crate::rtl::rom_table_word!(game, 0x9bc0db, v1 as isize)
    );
    set_ram_word!(
        game.grapple_beam_extension_y_velocity =
            crate::rtl::rom_table_word!(game, 0x9bc0ef, v1 as isize)
    );
    set_ram_word!(
        game.grapple_beam_end_angle16 = crate::rtl::rom_table_word!(game, 0x9bc104, v1 as isize)
    );
    set_ram_word!(game.grapple_beam_end_angles_mirror = ram_word!(game, grapple_beam_end_angle16));
    set_ram_word!(game.grapple_varCF6 = 10);
    if ram_word!(game, samus_pose) as i32 == kPose_49_FaceL_Moonwalk as i32
        || ram_word!(game, samus_pose) as i32 == kPose_4A_FaceR_Moonwalk as i32
        || *ram_ptr!(game; 0xa1f) != 1
    {
        set_ram_word!(
            game.grapple_beam_origin_x_offset =
                crate::rtl::rom_table_word!(game, 0x9bc122, v1 as isize)
        );
        set_ram_word!(
            game.grapple_beam_origin_y_offset =
                (crate::rtl::rom_table_word!(game, 0x9bc136, v1 as isize)).wrapping_sub(r22)
        );
        set_ram_word!(
            game.grapple_beam_end_x_pos =
                (crate::rtl::rom_table_word!(game, 0x9bc122, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_end_x_pos)
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc14a, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.grapple_beam_end_y_pos = (crate::rtl::rom_table_word!(game, 0x9bc136, v1 as isize)
                as i32
                + ram_word!(game, samus_y_pos) as i32
                - r22 as i32) as u16
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_end_y_pos)
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc15e, v1 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
    } else {
        set_ram_word!(
            game.grapple_beam_origin_x_offset =
                crate::rtl::rom_table_word!(game, 0x9bc172, v1 as isize)
        );
        set_ram_word!(
            game.grapple_beam_origin_y_offset =
                (crate::rtl::rom_table_word!(game, 0x9bc186, v1 as isize)).wrapping_sub(r22)
        );
        set_ram_word!(
            game.grapple_beam_end_x_pos =
                (crate::rtl::rom_table_word!(game, 0x9bc172, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_end_x_pos)
        );
        set_ram_word!(
            game.x_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc19a, v1 as isize))
                    .wrapping_add(ram_word!(game, samus_x_pos))
        );
        set_ram_word!(
            game.grapple_beam_end_y_pos = (crate::rtl::rom_table_word!(game, 0x9bc186, v1 as isize)
                as i32
                + ram_word!(game, samus_y_pos) as i32
                - r22 as i32) as u16
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_end_y_pos)
        );
        set_ram_word!(
            game.y_pos_of_start_of_grapple_beam_prevframe =
                (crate::rtl::rom_table_word!(game, 0x9bc1ae, v1 as isize) as i32
                    + ram_word!(game, samus_y_pos) as i32
                    - r22 as i32) as u16
        );
    }
    set_ram_words! { game, 0;
        grapple_beam_end_x_subpos, grapple_beam_end_y_subpos, grapple_beam_end_x_suboffset,
        grapple_beam_end_x_offset, grapple_beam_end_y_suboffset, grapple_beam_end_y_offset,
        grapple_beam_flags,
    }
    set_ram_word!(game.grapple_beam_length_delta = 12);
    set_ram_words! { game, 0;
        grapple_beam_length, grapple_beam_unkD26, grapple_beam_unkD28, grapple_beam_unkD2A,
        grapple_beam_unkD2C, grapple_beam_unkD2E, grapple_beam_unkD30, grapple_beam_unkD1E,
        grapple_beam_unkD20,
    }
    set_ram_word!(game.grapple_beam_unkD3A = 2);
    set_ram_word!(game.grapple_beam_unkD3C = 0);
    set_ram_word!(game.grapple_point_anim_timer = 5);
    set_ram_word!(game.grapple_point_anim_ptr = crate::rtl::rom_table_word!(game, 0x9bc342, 0));
    set_ram_words! { game, 0;
        grapple_beam_grapple_start_x, grapple_beam_unkD38, grapple_beam_unkD36,
        slow_grabble_scrolling_flag,
    }
    GrappleFunc_AF87(game);
    set_ram_word!(game.samus_draw_handler = ((0x90eb86i32) as u16));
    set_ram_word!(game.grapple_walljump_timer = 0);
    LoadProjectilePalette(2, game);
    set_ram_word!(game.palette_buffer[223] = 32657);
    set_ram_word!(game.grapple_beam_function = ((0x9bc703i32) as u16));
    QueueSfx1_Max1(5, game);
    set_ram_word!(game.flare_counter = 1);
    set_ram_word!(game.play_resume_charging_beam_sfx = 0);
    if ram_word!(game, samus_movement_handler) as i32 == 0x90946e & 0xffff {
        set_ram_word!(game.samus_movement_handler = ((0x90a337i32) as u16));
    }
}
pub unsafe fn CheckBannedDraygonGrappleDirs(a: u16, game: &mut Game) -> u16 {
    if a as i32 == kPose_BE_FaceL_Draygon_Move as i32 {
        if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Left as i32) != 0 {
            if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Down as i32) != 0 {
                return 6;
            }
            if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Up as i32) != 0 {
                return 8;
            }
        }
        return 7;
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Right as i32) == 0 {
        return 2;
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Down as i32) != 0 {
        return 3;
    }
    if (ram_word!(game, joypad1_lastkeys) as i32 & kButton_Up as i32) == 0 {
        return 2;
    }
    1
}
pub unsafe fn GrappleBeamFunc_Firing(game: &mut Game) {
    if (ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_lastkeys)) == 0 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
        return;
    }
    update_ram_word!(
        game.grapple_beam_length,
        wrapping_add(ram_word!(game, grapple_beam_length_delta))
    );
    if ram_word!(game, grapple_beam_length) as i32 - 128i32 & 0x8000 == 0 {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
        return;
    }
    let pair: PairU16 = GrappleBeam_CollDetect_Enemy(game);
    let v1: i32 = ProcessEnemyGrappleBeamColl(pair.k, pair.j, game);
    if v1 >= 0 {
        if v1 != 0 {
            set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
            return;
        }
    } else {
        let v2: u8 = BlockCollGrappleBeam(game);
        if (v2 & 1) == 0 {
            return;
        }
        if (v2 & 0x40) == 0 {
            set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
            return;
        }
    }
    QueueSfx1_Max6(6, game);
    HandleConnectingGrapple(game);
    set_ram_word!(game.grapple_beam_length_delta = -8i32 as u16);
    update_ram_word!(game.samus_grapple_flags, |fresh17| (fresh17) | (1u16));
}
pub unsafe fn GrappleBeamFunc_ConnectedLockedInPlace(game: &mut Game) {
    if !((ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_lastkeys)) != 0
        && (GrappleBeam_CollDetect_Enemy(game).k != 0
            || CheckIfGrappleIsConnectedToBlock(game) != 0))
    {
        set_ram_word!(game.grapple_beam_function = ((0x9bc856i32) as u16));
    }
}
pub unsafe fn GrappleBeamFunc_Connected_Swinging(game: &mut Game) {
    let keep_swinging: bool;
    if (ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_lastkeys)) != 0 {
        GrappleBeamFunc_BB64(game);
        if ram_word!(game, grapple_beam_length_delta) != 0 {
            BlockFunc_AC31(game);
        }
        GrappleBeamFunc_BC1F(game);
        GrappleBeamFunc_BCFF(game);
        GrappleBeamFunc_BD44(game);
        HandleMovementAndCollForSamusGrapple(game);
        if (ram_word!(game, grapple_beam_unkD36) & 0x8000) != 0
            && HandleSpecialGrappleBeamAngles(game) & 1 != 0
        {
            return;
        }
        keep_swinging = if GrappleBeam_CollDetect_Enemy(game).k != 0 {
            update_ram_word!(game.grapple_beam_flags, |fresh13| (fresh13) | (0x8000u16));
            false
        } else {
            (CheckIfGrappleIsConnectedToBlock(game) & 1 == 0)
        };
        if keep_swinging {
        } else {
            BlockFunc_AC11(game);
            GrappleBeamFunc_BD95(game);
            return;
        }
    }
    if ram_word!(game, grapple_beam_unkD26) != 0
        || ram_word!(game, grapple_beam_end_angle16) != 0x8000
    {
        PropelSamusFromGrappleSwing(game);
        set_ram_word!(game.grapple_beam_function = ((0x9bcb8bi32) as u16));
        set_ram_word!(game.samus_movement_handler = ((0x90946ei32) as u16));
    } else {
        set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
    };
}
pub unsafe fn GrappleBeamFunc_Wallgrab(game: &mut Game) {
    if !((ram_word!(game, button_config_shoot_x) & ram_word!(game, joypad1_lastkeys)) != 0
        && CheckIfGrappleIsConnectedToBlock(game) & 1 != 0)
    {
        set_ram_word!(game.grapple_walljump_timer = 30);
        set_ram_word!(game.grapple_beam_function = ((0x9bc832i32) as u16));
    }
}
pub unsafe fn GrappleBeamFunc_C832(game: &mut Game) {
    if (update_ram_word!(game.grapple_walljump_timer, wrapping_sub(1), reread) & 0x8000) == 0 {
        if Samus_GrappleWallJumpCheck((16i16 as i32) << 16, game) != 0 {
            set_ram_word!(game.grapple_beam_function = ((0x9bc9cei32) as u16));
        }
    } else {
        set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
    };
}
pub unsafe fn GrappleBeamFunc_Cancel(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if *ram_ptr!(game; 0xa1f) as i32 == kMovementType_16_Grappling as i32 {
        Samus_Pose_CancelGrapple(game);
    } else {
        CallSomeSamusCode(0x1c, game);
    }
    set_ram_words! { game, 0;
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
        grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    LoadProjectilePalette(ram_word!(game, equipped_beams), game);
    set_ram_word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Samus_PostGrappleCollisionDetect(game);
    if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
        set_ram_words! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}
pub unsafe fn GrappleBeam_Func2(game: &mut Game) {
    let mut current_block: u64;
    QueueSfx1_Max15(7, game);
    if ram_word!(game, samus_pose) as i32 == kPose_B2_FaceR_Grapple_Air as i32 {
        current_block = 16041378612104114408;
    } else {
        if ram_word!(game, samus_pose) as i32 == kPose_B3_FaceL_Grapple_Air as i32 {
            current_block = 13151459208701579741;
        } else if ram_word!(game, samus_y_radius) as i32 - 17i32 & 0x8000 != 0 {
            if ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629)) as *mut SamusPoseParams)
                .offset(ram_word!(game, samus_pose) as isize))
            .direction_shots_fired
                & 0xf0)
                != 0
            {
                if *ram_ptr!(game; 0xa1e) == 4 {
                    set_ram_word!(game.samus_new_pose_transitional = kPose_28_FaceL_Crouch as u16);
                } else {
                    set_ram_word!(game.samus_new_pose_transitional = kPose_27_FaceR_Crouch as u16);
                }
            } else {
                set_ram_word!(
                    game.samus_new_pose_transitional =
                        *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc9c4)) as *mut u8).offset(
                            *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                                as *mut SamusPoseParams)
                                .offset(0))
                            .direction_shots_fired)
                                .offset((8 * ram_word!(game, samus_pose) as i32) as isize,)
                                as isize,
                        ) as u16
                );
            }
            current_block = 8968942556654489080;
        } else if ((*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
            as *mut SamusPoseParams)
            .offset(ram_word!(game, samus_pose) as isize))
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
            set_ram_word!(
                game.samus_new_pose_transitional =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0x9bc9ba)) as *mut u8).offset(
                        *(&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0x91b629))
                            as *mut SamusPoseParams)
                            .offset(0))
                        .direction_shots_fired)
                            .offset((8 * ram_word!(game, samus_pose) as i32) as isize,)
                            as isize,
                    ) as u16
            );
            current_block = 8968942556654489080;
        }
        match current_block {
            8968942556654489080 => {}
            16041378612104114408 => {}
            _ => {
                set_ram_word!(game.samus_new_pose_transitional = kPose_02_FaceL_Normal as u16);
                current_block = 8968942556654489080;
            }
        }
    }
    if current_block == 16041378612104114408 {
        set_ram_word!(game.samus_new_pose_transitional = kPose_01_FaceR_Normal as u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 0);
    set_ram_word!(game.input_to_pose_calc = 1);
    set_ram_words! { game, 0;
        samus_x_base_speed, samus_x_base_subspeed, samus_y_subspeed, samus_y_speed,
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
        grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    LoadProjectilePalette(ram_word!(game, equipped_beams), game);
    set_ram_word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Samus_PostGrappleCollisionDetect(game);
    if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
        set_ram_words! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}
pub unsafe fn GrappleBeamFunc_C9CE(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if *ram_ptr!(game; 0xa1e) == 8 {
        set_ram_word!(game.samus_new_pose_transitional = kPose_84_FaceL_Walljump as u16);
    } else {
        set_ram_word!(game.samus_new_pose_transitional = kPose_83_FaceR_Walljump as u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 6);
    set_ram_words! { game, 0;
        samus_x_accel_mode, samus_collides_with_solid_enemy, samus_is_falling_flag,
        UNUSED_word_7E0B1A, UNUSED_word_7E0B2A, samus_y_subspeed, samus_y_speed, samus_y_dir,
        UNUSED_word_7E0B38, samus_x_base_speed, samus_x_base_subspeed, grapple_beam_unkD1E,
        grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36, grapple_walljump_timer,
        slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags, flare_counter,
        flare_animation_frame, flare_slow_sparks_anim_frame, flare_fast_sparks_anim_frame,
        flare_animation_timer, flare_slow_sparks_anim_timer, flare_fast_sparks_anim_timer,
    }
    LoadProjectilePalette(ram_word!(game, equipped_beams), game);
    set_ram_word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Samus_PostGrappleCollisionDetect(game);
    if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
        set_ram_words! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}
pub unsafe fn PropelSamusFromGrappleSwing(game: &mut Game) {
    let mut v0: u16 = 0;
    if (ram_word!(game, grapple_beam_unkD26) & 0x8000) == 0 {
        v0 = (2u16).wrapping_mul(ram_word!(game, grapple_beam_unkD26));
        let mut v2: i16 = kSinCosTable8bit_Sext
            [(*(ram_ptr!(game, u16; 0xcfa) as *mut u8).offset(1) as i32 + 64) as usize];
        if v2 >= 0 {
            SetHiLo(
                ram_ptr!(game, u16; 0xb2e),
                ram_ptr!(game, u16; 0xb2c),
                Multiply16x16(v2 as u16, v0, game),
            );
            set_ram_word!(game.samus_y_dir = 2);
        } else {
            SetHiLo(
                ram_ptr!(game, u16; 0xb2e),
                ram_ptr!(game, u16; 0xb2c),
                Multiply16x16(-(v2 as i32) as u16, v0, game),
            );
            set_ram_word!(game.samus_y_dir = 1);
        }
    } else {
        v0 = (-2i32 * ram_word!(game, grapple_beam_unkD26) as i32) as u16;
        let mut v1: i16 = kSinCosTable8bit_Sext
            [(*(ram_ptr!(game, u16; 0xcfa) as *mut u8).offset(1) as i32 + 64) as usize];
        if v1 < 0 {
            SetHiLo(
                ram_ptr!(game, u16; 0xb2e),
                ram_ptr!(game, u16; 0xb2c),
                Multiply16x16(-(v1 as i32) as u16, v0, game),
            );
            set_ram_word!(game.samus_y_dir = 2);
        } else {
            SetHiLo(
                ram_ptr!(game, u16; 0xb2e),
                ram_ptr!(game, u16; 0xb2c),
                Multiply16x16(v1 as u16, v0, game),
            );
            set_ram_word!(game.samus_y_dir = 1);
        }
    }
    set_ram_word!(game.samus_x_accel_mode = 2);
    let mut r18: u16 = (3 * (v0 as i32 >> 9)) as u16;
    r18 = (64u16).wrapping_sub(r18);
    let v3: u16 = abs16(
        kSinCosTable8bit_Sext
            [(((*ram_ptr!(game; 0xcfb)).wrapping_sub((r18) as u8)) as i32 + 64) as usize]
            as u16,
    );
    SetHiLo(
        ram_ptr!(game, u16; 0xb46),
        ram_ptr!(game, u16; 0xb48),
        Multiply16x16(v3, v0, game),
    );
}
pub unsafe fn GrappleBeamFunc_ReleaseFromSwing(game: &mut Game) {
    QueueSfx1_Max15(7, game);
    if (ram_word!(game, grapple_beam_unkD26) & 0x8000) == 0 {
        set_ram_word!(game.samus_new_pose_transitional = kPose_52_FaceL_Jump_NoAim_MoveF as u16);
    } else {
        set_ram_word!(game.samus_new_pose_transitional = kPose_51_FaceR_Jump_NoAim_MoveF as u16);
    }
    set_ram_word!(game.samus_hurt_switch_index = 7);
    set_ram_words! { game, 0;
        grapple_beam_unkD1E, grapple_beam_unkD20, grapple_beam_direction, grapple_beam_unkD36,
        grapple_walljump_timer, slow_grabble_scrolling_flag, grapple_varCF6, grapple_beam_flags,
        flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
        flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
        flare_fast_sparks_anim_timer,
    }
    LoadProjectilePalette(ram_word!(game, equipped_beams), game);
    set_ram_word!(game.grapple_beam_function = ((0x9bc4f0i32) as u16));
    set_ram_word!(game.samus_draw_handler = ((0x90eb52i32) as u16));
    Samus_PostGrappleCollisionDetect(game);
    if ram_word!(game, samus_auto_cancel_hud_item_index) != 0 {
        set_ram_words! { game, 0; hud_item_index, samus_auto_cancel_hud_item_index }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallGrappleBeamFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10206052 => GrappleBeamFunc_BB64; 10206239 => GrappleBeamFunc_BC1F;
        10206463 => GrappleBeamFunc_BCFF; 10206532 => GrappleBeamFunc_BD44;
        10206613 => GrappleBeamFunc_BD95; 10206872 => GrappleBeamFunc_BE98;
        10206955 => GrappleBeamFunc_BEEB; 10207003 => GrappleBeamFunc_BF1B;
        10208496 => GrappleBeamFunc_Inactive; 10208542 => GrappleBeamFunc_FireGoToCancel;
        10209027 => GrappleBeamFunc_Firing; 10209150 => GrappleBeamFunc_ConnectedLockedInPlace;
        10209181 => GrappleBeamFunc_Connected_Swinging; 10209300 => GrappleBeamFunc_Wallgrab;
        10209330 => GrappleBeamFunc_C832; 10209366 => GrappleBeamFunc_Cancel;
        10209742 => GrappleBeamFunc_C9CE; 10210187 => GrappleBeamFunc_ReleaseFromSwing;
        10209477 => GrappleBeam_Func2;
    }
}
