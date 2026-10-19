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
use crate::rtl::abs16;
use crate::rtl::SetHiLo;
use crate::rtl::{RomPtr, Unreachable};
extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, size_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, LongPtr,
    PairU16, Point16U, SamusPose, VoidP, VramWriteEntry,
};
pub use crate::types::{SamusMovementType, SamusPoseParams};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct GrappleBeamSpecialAngles {
    pub field_0: uint16,
    pub field_2: uint16,
    pub field_4: uint16,
    pub field_6: uint16,
    pub field_8: VoidP,
}
pub use crate::types::Buttons;
pub use crate::types::Consts_9B;
#[inline]
unsafe fn RomPtr_9B(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0x9b0000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomPtr_A0(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xa00000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
pub const fnSamus_MoveHandler_ReleaseFromGrapple: ::core::ffi::c_int = 0x90946e;
pub const fnSamus_MovementHandler_Normal: ::core::ffi::c_int = 0x90a337;
pub const fnSamusDrawHandler_Default: ::core::ffi::c_int = 0x90eb52;
pub const fnsub_90EB86: ::core::ffi::c_int = 0x90eb86;
pub const fnGrappleNext_SwingClockwise: uint32 = 10205657;
pub const fnGrappleNext_SwingAntiClockwise: uint32 = 10205666;
pub const fnGrappleNext_StandAimRight: uint32 = 10205674;
pub const fnGrappleNext_StandAimDownRight: uint32 = 10205683;
pub const fnGrappleNext_StandAimDownLeft: uint32 = 10205692;
pub const fnGrappleNext_StandAimLeft: uint32 = 10205701;
pub const fnGrappleNext_CrouchAimRight: uint32 = 10205710;
pub const fnGrappleNext_CrouchAimDownRight: uint32 = 10205719;
pub const fnGrappleNext_CrouchAimDownLeft: uint32 = 10205728;
pub const fnGrappleNext_CrouchAimLeft: uint32 = 10205737;
pub const fnGrappleBeamFunc_BB64: uint32 = 10206052;
pub const fnGrappleBeamFunc_BC1F: uint32 = 10206239;
pub const fnGrappleBeamFunc_BCFF: uint32 = 10206463;
pub const fnGrappleBeamFunc_BD44: uint32 = 10206532;
pub const fnGrappleBeamFunc_BD95: uint32 = 10206613;
pub const fnGrappleBeamFunc_BE98: uint32 = 10206872;
pub const fnGrappleBeamFunc_BEEB: uint32 = 10206955;
pub const fnGrappleBeamFunc_BF1B: uint32 = 10207003;
pub const fnGrappleBeamFunc_Inactive: ::core::ffi::c_int = 0x9bc4f0;
pub const fnGrappleBeamFunc_FireGoToCancel: ::core::ffi::c_int = 0x9bc51e;
pub const fnGrappleBeamFunc_Firing: ::core::ffi::c_int = 0x9bc703;
pub const fnGrappleBeamFunc_ConnectedLockedInPlace: ::core::ffi::c_int = 0x9bc77e;
pub const fnGrappleBeamFunc_Connected_Swinging: uint32 = 10209181;
pub const fnGrappleBeamFunc_Wallgrab: uint32 = 10209300;
pub const fnGrappleBeamFunc_C832: ::core::ffi::c_int = 0x9bc832;
pub const fnGrappleBeamFunc_Cancel: ::core::ffi::c_int = 0x9bc856;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5;
pub const fnGrappleBeamFunc_C9CE: ::core::ffi::c_int = 0x9bc9ce;
pub const fnGrappleBeamFunc_ReleaseFromSwing: ::core::ffi::c_int = 0x9bcb8b;
static kDeathAnimationFrames: [uint8; 28] = [
    5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8, 1 as uint8, 5 as uint8, 5 as uint8, 0 as uint8,
    1 as uint8, 0 as uint8, 5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8,
    5 as uint8, 1 as uint8, 1 as uint8, 1 as uint8, 5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8,
    5 as uint8, 5 as uint8, 5 as uint8, 5 as uint8,
];
static g_word_9BC118: uint16 = 24 as uint16;
static g_word_9BC11A: uint16 = 0xc as uint16;
static g_word_9BC11C: uint16 = 5 as uint16;
static g_word_9BC11E: uint16 = 0x480 as uint16;
static g_word_9BC120: uint16 = 0x300 as uint16;
static g_word_9BB7BF: [uint16; 5] = [
    0x8400 as uint16,
    0x8800 as uint16,
    0x8c00 as uint16,
    0x9000 as uint16,
    0x8000 as uint16,
];
static g_word_9BB7C9: [uint16; 5] = [
    0x6200 as uint16,
    0x6400 as uint16,
    0x6600 as uint16,
    0x6800 as uint16,
    0x6000 as uint16,
];
static kShadesOfWhite: [uint16; 22] = [
    0x421 as uint16,
    0xc63 as uint16,
    0x14a5 as uint16,
    0x1ce7 as uint16,
    0x2529 as uint16,
    0x2d6b as uint16,
    0x35ad as uint16,
    0x4210 as uint16,
    0x4a52 as uint16,
    0x4e73 as uint16,
    0x5294 as uint16,
    0x56b5 as uint16,
    0x5ad6 as uint16,
    0x5ef7 as uint16,
    0x6318 as uint16,
    0x6739 as uint16,
    0x6b5a as uint16,
    0x6f7b as uint16,
    0x739c as uint16,
    0x77bd as uint16,
    0x7bde as uint16,
    0x7fff as uint16,
];
static kIsGrappleBannedForMovementType: [uint8; 28] = [
    0 as uint8, 0 as uint8, 0 as uint8, 1 as uint8, 1 as uint8, 0 as uint8, 0 as uint8, 1 as uint8,
    1 as uint8, 1 as uint8, 1 as uint8, 0 as uint8, 0 as uint8, 1 as uint8, 1 as uint8, 1 as uint8,
    0 as uint8, 1 as uint8, 1 as uint8, 1 as uint8, 1 as uint8, 0 as uint8, 0 as uint8, 1 as uint8,
    1 as uint8, 1 as uint8, 0 as uint8, 1 as uint8,
];
pub unsafe fn ProjectileTrail_Func5(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut R22: uint16 = ProjectileInsts_GetValue(k, game);
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        r18 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v2 as isize);
        r20 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v2 as isize);
    } else {
        let mut pt: Point16U = CalcExplosion_Mode7(k, game);
        r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
            as ::core::ffi::c_int
            + pt.x as ::core::ffi::c_int) as uint16;
        r20 = (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
            + pt.y as ::core::ffi::c_int) as uint16;
    }
    let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v3 as isize);
    let mut v5: uint16 = 0;
    if v4 as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v5 = (*(game.g_rom.offset(
            (((0x9ba4e3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9ba4e3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)) as uint16;
    } else if v4 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v5 = (*(game.g_rom.offset(
            (((0x9ba4cb as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9ba4cb as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)) as uint16;
    } else {
        v5 = (*(game.g_rom.offset(
            (((0x9ba4b3 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9ba4b3 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)) as uint16;
    }
    let mut v6: uint16 = (*(RomPtr_9B(v5 as uint16_t, game) as *mut uint16) as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int * R22 as ::core::ffi::c_int) as uint16;
    let mut p: *const uint8 = RomPtr_9B(v6 as uint16_t, game);
    let mut v7: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd778) as *mut uint16).offset(v7 as isize) =
        (r20 as ::core::ffi::c_int + *p.offset(1) as int8 as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd730) as *mut uint16).offset(v7 as isize) =
        (r18 as ::core::ffi::c_int + *p.offset(0) as int8 as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd79c) as *mut uint16).offset(v7 as isize) =
        (r20 as ::core::ffi::c_int + *p.offset(3) as int8 as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd754) as *mut uint16).offset(v7 as isize) =
        (r18 as ::core::ffi::c_int + *p.offset(2) as int8 as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn StartSamusDeathAnimation(game: &mut Game) {
    let mut v0: uint16 = *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        == 3 as ::core::ffi::c_int
    {
        QueueSfx1_Max6(0x32 as uint16, game);
    }
    let mut v1: uint16 = kDeathAnimationFrames[v0 as usize] as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
            kPose_D8_FaceL_CrystalFlashEnd as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) =
            kPose_D7_FaceR_CrystalFlashEnd as uint16;
    }
    SamusFunc_F433(game);
    Samus_SetAnimationFrameIfPoseChanged(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) = v1;
    let ref mut fresh23 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    *fresh23 = (*fresh23 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    let ref mut fresh24 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh24 = (*fresh24 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn DrawSamusStartingDeathAnim_(game: &mut Game) {
    Samus_DrawStartingDeathAnim(game);
}
pub unsafe fn HandleSamusDeathSequence(game: &mut Game) -> uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        QueueTransferOfSamusDeathSequence(
            (2 as ::core::ffi::c_int
                * *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16,
            game,
        );
    }
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16);
    *fresh4 = (*fresh4).wrapping_add(1);
    if *fresh4 as ::core::ffi::c_int - 60 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
        *fresh6 = (*fresh6).wrapping_sub(1);
        let mut v0: bool =
            *fresh6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) == 0 || v0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 3 as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = 1 as uint16;
            }
            CopyPalettesForSamusDeath(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int) as uint16,
                game,
            );
        }
        return 0 as uint16;
    } else {
        HandleSamusDeathSequence_Helper2(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x8d) as *mut uint16)
                as ::core::ffi::c_int
                & 0xb0 as ::core::ffi::c_int
                == kButton_A as ::core::ffi::c_int
                    | kButton_L as ::core::ffi::c_int
                    | kButton_R as ::core::ffi::c_int) as uint16;
        return 1 as uint16;
    };
}
pub unsafe fn HandleSamusDeathSequence_Helper2(game: &mut Game) {
    let mut v0: *const uint16 = RomPtr_9B(
        *(game.g_rom.offset(
            (((0x9bb5c8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9bb5c8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xa74) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as uint16_t,
        game,
    ) as *const uint16;
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(192)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(*v0, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(240)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(addr_word_9BA120 as uint16_t, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    QueueTransferOfSamusDeathSequence(8 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) = *(game.g_rom.offset(
        (((0x9bb823 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bb823 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint8)
        .offset(0)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) = 0 as uint16;
    GameState_24_SamusNoHealth_Explosion_2(game);
}
pub unsafe fn CopyPalettesForSamusDeath(mut v0: uint16, game: &mut Game) {
    let mut r20: ::core::ffi::c_int = *(game.g_rom.offset(
        (((0x9bb6d2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bb6d2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xa74) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int;
    let mut v1: *const uint16 =
        RomPtr_9B((r20 + v0 as ::core::ffi::c_int) as uint16_t, game) as *const uint16;
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(192)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(*v1, game) as *const ::core::ffi::c_void,
        32 as size_t,
    );
    memcpy(
        ((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(240)
            as *mut uint16 as *mut ::core::ffi::c_void,
        RomPtr_9B(
            *(game.g_rom.offset(
                (((0x9bb80f as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bb80f as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as uint16_t,
            game,
        ) as *const ::core::ffi::c_void,
        32 as size_t,
    );
}
pub unsafe fn QueueTransferOfSamusDeathSequence(mut v0: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1024 as uint16;
    v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size = g_word_9BB7BF[v2 as usize];
    v1 = (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh7 = v1;
    v1 = v1.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh7 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = 0x9b as uint8;
    (*gVramWriteEntry(v1 as ::core::ffi::c_int, game)).size = g_word_9BB7C9[v2 as usize];
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v1 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_Helper(game: &mut Game) -> uint16 {
    GameState_24_SamusNoHealth_Explosion_1(game);
    return GameState_24_SamusNoHealth_Explosion_2(game);
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) as ::core::ffi::c_int
            != 0
    {
        let mut v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xde6)
            as *mut uint16) as ::core::ffi::c_int;
        let mut dst: *mut uint16 =
            (&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 384 as ::core::ffi::c_int / 2 as ::core::ffi::c_int {
            *dst.offset(i as isize) = kShadesOfWhite[v0 as usize];
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = 416 as ::core::ffi::c_int / 2 as ::core::ffi::c_int;
        while i_0 < 480 as ::core::ffi::c_int / 2 as ::core::ffi::c_int {
            *dst.offset(i_0 as isize) = kShadesOfWhite[v0 as usize];
            i_0 += 1;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) as ::core::ffi::c_int
            - 20 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh3 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16);
            *fresh3 = (*fresh3).wrapping_add(1);
        }
    }
}
pub unsafe fn GameState_24_SamusNoHealth_Explosion_2(game: &mut Game) -> uint16 {
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16);
    *fresh1 = (*fresh1).wrapping_sub(1);
    let mut v0: bool =
        *fresh1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) == 0 || v0 {
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16);
        *fresh2 = (*fresh2).wrapping_add(1);
        if *fresh2 as ::core::ffi::c_int - 9 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde6) as *mut uint16) = 21 as uint16;
            GameState_24_SamusNoHealth_Explosion_1(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
            return 1 as uint16;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) == 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16)
                as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bb823 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bb823 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(
                        (2 as ::core::ffi::c_int
                            * *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16)
                                as ::core::ffi::c_int) as isize,
                    ) as uint16;
            CopyPalettesForSamusDeath(
                (2 as ::core::ffi::c_int
                    * *(game.g_rom.offset(
                        (((0x9bb823 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x9bb823 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint8)
                        .offset(
                            (2 as ::core::ffi::c_int
                                * *((&raw mut game.g_ram as *mut uint8).offset(0xde4)
                                    as *mut uint16)
                                    as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int) as isize,
                        ) as ::core::ffi::c_int) as uint16,
                game,
            );
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xde2) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bb823 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bb823 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(
                        (2 as ::core::ffi::c_int
                            * *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16)
                                as ::core::ffi::c_int) as isize,
                    ) as uint16;
        }
    }
    DrawSamusSuitExploding(game);
    return 0 as uint16;
}
pub unsafe fn CancelGrappleBeamIfIncompatiblePose(game: &mut Game) {
    let mut v0: int16 = 0;
    if !(kIsGrappleBannedForMovementType
        [*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        != 0)
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) as ::core::ffi::c_int
            != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16)
                as ::core::ffi::c_int
                + 0x3882 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            v0 = (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize,
                ))
            .direction_shots_fired as int16;
            if v0 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                if v0 as ::core::ffi::c_int
                    == *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16)
                        as ::core::ffi::c_int
                {
                    return;
                }
                if *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) != 0 {
                    QueueSfx1_Max6(7 as uint16, game);
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                        (0x9bc51e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
                    return;
                }
            }
        } else {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CheckIfGrappleIsConnectedToBlock(game: &mut Game) -> uint8 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xd22) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd24) as *mut uint16) = 0 as uint16;
    if BlockReactGrapple(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 0 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16) = 1 as uint16;
    return 1 as uint8;
}
unsafe fn ProcessEnemyGrappleBeamColl(
    mut a: uint16,
    mut r18: uint16,
    game: &mut Game,
) -> ::core::ffi::c_int {
    match a as ::core::ffi::c_int {
        0 | 2 => return -(1 as ::core::ffi::c_int),
        3 => return 1 as ::core::ffi::c_int,
        1 | 4 | 5 => return 0 as ::core::ffi::c_int,
        6 => {
            r18 = *(RomPtr_A0(r18 as uint16_t, game) as *mut uint16).offset(3);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
                as ::core::ffi::c_int
                & 0x20 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                r18 = (r18 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
            } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0
            {
                r18 = (r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            }
            Samus_DealDamage(r18, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
                (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
                    == 4 as ::core::ffi::c_int) as uint16;
            return 1 as ::core::ffi::c_int;
        }
        _ => {
            Unreachable(game);
            return 0 as ::core::ffi::c_int;
        }
    };
}
pub unsafe fn CallGrappleNextFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10205657 => {
            GrappleNext_SwingClockwise(game);
            return;
        }
        10205666 => {
            GrappleNext_SwingAntiClockwise(game);
            return;
        }
        10205674 => {
            GrappleNext_StandAimRight(game);
            return;
        }
        10205683 => {
            GrappleNext_StandAimDownRight(game);
            return;
        }
        10205692 => {
            GrappleNext_StandAimDownLeft(game);
            return;
        }
        10205701 => {
            GrappleNext_StandAimLeft(game);
            return;
        }
        10205710 => {
            GrappleNext_CrouchAimRight(game);
            return;
        }
        10205719 => {
            GrappleNext_CrouchAimDownRight(game);
            return;
        }
        10205728 => {
            GrappleNext_CrouchAimDownLeft(game);
            return;
        }
        10205737 => {
            GrappleNext_CrouchAimLeft(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn HandleConnectingGrapple(game: &mut Game) {
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        == kMovementType_1A_GrabbedByDraygon as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc77e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 0 as uint16;
    } else {
        let mut v1: ::core::ffi::c_int = 2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16)
                as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) as ::core::ffi::c_int
            != 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16)
                as ::core::ffi::c_int
                != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bc3ee as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc3ee as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize);
            CallGrappleNextFunc(
                (*(game.g_rom.offset(
                    (((0x9bc3ee as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc3ee as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    | 0x9b0000 as ::core::ffi::c_int) as uint32,
                game,
            );
        } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
            == 5 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bc416 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc416 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize);
            CallGrappleNextFunc(
                (*(game.g_rom.offset(
                    (((0x9bc416 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc416 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    | 0x9b0000 as ::core::ffi::c_int) as uint32,
                game,
            );
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bc3c6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc3c6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(v1 as isize);
            CallGrappleNextFunc(
                (*(game.g_rom.offset(
                    (((0x9bc3c6 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc3c6 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((v1 + 1 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    | 0x9b0000 as ::core::ffi::c_int) as uint32,
                game,
            );
        }
    };
}
pub unsafe fn GrappleNext_SwingClockwise(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B2_FaceR_Grapple_Air as uint16;
    HandleConnectingGrapple_Swinging(game);
}
pub unsafe fn GrappleNext_SwingAntiClockwise(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B3_FaceL_Grapple_Air as uint16;
    HandleConnectingGrapple_Swinging(game);
}
pub unsafe fn GrappleNext_StandAimRight(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_A8_FaceR_Grappling as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimDownRight(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_AA_FaceR_Grappling_AimDR as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimDownLeft(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_AB_FaceL_Grappling_AimDL as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_StandAimLeft(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_A9_FaceL_Grappling as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimRight(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B4_FaceR_Grappling_Crouch as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimDownRight(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B6_FaceR_Grappling_Crouch_AimDR as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimDownLeft(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B7_FaceL_Grappling_Crouch_AimDL as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn GrappleNext_CrouchAimLeft(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) =
        kPose_B5_FaceL_Grappling_Crouch as uint16;
    HandleConnectingGrapple_StuckInPlace(game);
}
pub unsafe fn HandleConnectingGrapple_Swinging(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30) as *mut uint16) = 9 as uint16;
    let mut v0: uint16 = u16::swap_bytes(CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        game,
    ));
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
        *fresh19 = (*fresh19 as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
    }
    BlockFunc_AC11(game);
}
pub unsafe fn HandleConnectingGrapple_StuckInPlace(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30) as *mut uint16) = 10 as uint16;
    let mut v0: uint16 = u16::swap_bytes(CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                as ::core::ffi::c_int) as uint16,
        game,
    ));
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) as ::core::ffi::c_int
        - 64 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
        *fresh18 = (*fresh18 as ::core::ffi::c_int - 24 as ::core::ffi::c_int) as uint16;
    }
    BlockFunc_AC11(game);
}
pub unsafe fn HandleSpecialGrappleBeamAngles(game: &mut Game) -> uint8 {
    let mut v0: uint16 = 7 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) as ::core::ffi::c_int
        != (*(game.g_rom.offset(
            (((0x9bc43e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9bc43e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut GrappleBeamSpecialAngles)
            .offset(v0 as isize))
        .field_0 as ::core::ffi::c_int
    {
        v0 = v0.wrapping_sub(1);
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 0 as uint8;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa2a) as *mut uint16) = (*(game.g_rom.offset(
        (((0x9bc43e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc43e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut GrappleBeamSpecialAngles)
        .offset(v0 as isize))
    .field_2;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) as ::core::ffi::c_int
            + (*(game.g_rom.offset(
                (((0x9bc43e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc43e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut GrappleBeamSpecialAngles)
                .offset(v0 as isize))
            .field_4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) as ::core::ffi::c_int
            + (*(game.g_rom.offset(
                (((0x9bc43e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc43e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut GrappleBeamSpecialAngles)
                .offset(v0 as isize))
            .field_6 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) = (*(game.g_rom.offset(
        (((0x9bc43e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc43e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut GrappleBeamSpecialAngles)
        .offset(v0 as isize))
    .field_8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa30) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    let mut v2: int16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) as ::core::ffi::c_int)
        as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v2 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v2 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
    }
    let mut v3: int16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) as ::core::ffi::c_int)
        as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v3 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v3 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
    }
    return 1 as uint8;
}
pub unsafe fn GrappleBeamFunc_BB64(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) =
                -(2 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
        as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 2 as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = 64 as uint16;
        }
    }
    if !(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        - 0x4000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        || ((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int)
            + 0x4000 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0)
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if (*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
                == 0x8000 as ::core::ffi::c_int
                && *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) == 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = 256 as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                    as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) =
                    (g_word_9BC11A as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) = g_word_9BC11A;
            }
            current_block = 15089075282327824602;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            current_block = 2963438129606135224;
        } else {
            if (*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int
                == 0x8000 as ::core::ffi::c_int
                && *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) == 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) =
                    -(256 as ::core::ffi::c_int) as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                    as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) =
                    -(g_word_9BC11A as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) =
                    -(g_word_9BC11A as ::core::ffi::c_int) as uint16;
            }
            current_block = 15089075282327824602;
        }
        match current_block {
            2963438129606135224 => {}
            _ => {
                return;
            }
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) = 0 as uint16;
}
pub unsafe fn GrappleBeamFunc_BC1F(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) as ::core::ffi::c_int
        & 0xc000 as ::core::ffi::c_int
        == 0xc000 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) =
            -(g_word_9BC11C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                -(g_word_9BC118 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                -(g_word_9BC118 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if (*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            == 0x8000 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) = 0 as uint16;
            let mut v0: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                v0 = !(*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as uint16;
            }
            if *(&raw mut v0 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = 0 as uint16;
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) =
                -(g_word_9BC11C as ::core::ffi::c_int) as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                    as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                    -(g_word_9BC118 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                    -(g_word_9BC118 as ::core::ffi::c_int) as uint16;
            }
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16)
        as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) = g_word_9BC11C;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                (g_word_9BC118 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) = g_word_9BC118;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) =
            (g_word_9BC11C as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) as ::core::ffi::c_int
            != 0
            && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                (g_word_9BC118 as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as uint16;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) =
                (g_word_9BC118 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn GrappleBeamFunc_BCFF(game: &mut Game) {
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16);
    *fresh14 = (*fresh14 as ::core::ffi::c_int
        + (*((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16)
                as ::core::ffi::c_int)) as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        ^ *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        let ref mut fresh15 = *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16);
        *fresh15 = (*fresh15 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
            >= g_word_9BC11E as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = g_word_9BC11E;
        }
    } else if -(*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
        as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >= g_word_9BC11E as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) =
            -(g_word_9BC11E as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GrappleBeamFunc_BD44(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16) as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x9b4) as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
                as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) != 0 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                    as ::core::ffi::c_int
                    != 0
                    && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                        as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) =
                        -(g_word_9BC120 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) =
                        -(g_word_9BC120 as ::core::ffi::c_int) as uint16;
                }
            } else if *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                as ::core::ffi::c_int
                != 0
                && *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16)
                    as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) =
                    (g_word_9BC120 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = g_word_9BC120;
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn GrappleBeamFunc_BD95(game: &mut Game) {
    let mut current_block: u64;
    let mut v0: uint16 = abs16(*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16));
    let mut v1: uint16 = 0;
    if v0 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 1 as uint16;
        current_block = 8418170951678595278;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
        if (*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            != 0x8000 as ::core::ffi::c_int
        {
            current_block = 8418170951678595278;
        } else {
            if *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16)
                as ::core::ffi::c_int
                - 64 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa94) as *mut uint16) = 8 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) = 64 as uint16;
            }
            v1 = *(game.g_rom.offset(
                (((0x9bc1c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc1c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ) as isize,
                ) as uint16;
            current_block = 13536709405535804910;
        }
    }
    match current_block {
        8418170951678595278 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa94) as *mut uint16) = 15 as uint16;
            v1 = *(game.g_rom.offset(
                (((0x9bc1c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc1c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16
                        as *mut uint8)
                        .offset(
                            (::core::mem::size_of::<uint16>() as usize)
                                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                                .wrapping_sub(1 as usize) as isize,
                        ) as isize,
                ) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) = v1;
        }
        _ => {}
    }
    let mut v2: uint16 = (2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int) as uint16;
    if abs16(*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
        as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int
        == 256 as ::core::ffi::c_int
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) as ::core::ffi::c_int
            - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 16 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) =
            (v3 as ::core::ffi::c_int + 32 as ::core::ffi::c_int) as uint16;
    }
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x9bc2c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc2c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v2 as isize) as int8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x9bc2c2 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc2c2 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    as int8 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x9bc302 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc302 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v2 as isize) as int8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x9bc302 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc302 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset((v2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                    as int8 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16);
    GrappleBeamFunc_BE98(game);
}
pub unsafe fn GrappleBeamFunc_BE98(game: &mut Game) {
    let mut v0: int16 = 0;
    let mut v1: int16 = 0;
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) as ::core::ffi::c_int)
        as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v0 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v0 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
    }
    v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) as ::core::ffi::c_int)
        as int16;
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) as ::core::ffi::c_int)
        as int16 as ::core::ffi::c_int)
        < 0 as ::core::ffi::c_int
    {
        if v1 as ::core::ffi::c_int + 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                    + 12 as ::core::ffi::c_int) as uint16;
        }
    } else if v1 as ::core::ffi::c_int - 13 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - 12 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GrappleBeamFunc_BEEB(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0x9bc122 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc122 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) = (*(game.g_rom.offset(
        (((0x9bc14a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc14a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v0 as isize)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0x9bc136 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc136 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v0 as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) = (*(game.g_rom.offset(
        (((0x9bc15e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc15e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v0 as isize)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn GrappleBeamFunc_BF1B(game: &mut Game) {
    let mut r22: uint16 = (*(game.g_rom.offset(
        (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut SamusPoseParams)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize))
    .y_offset_to_gfx as uint16;
    let mut v0: uint16 = (2 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_49_FaceL_Moonwalk as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_4A_FaceR_Moonwalk as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc122 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc122 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc14a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc14a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc136 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc136 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc15e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc15e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
    } else {
        let mut v2: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc172 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc172 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc19a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc19a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc186 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc186 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc1ae as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc1ae as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn UpdateGrappleBeamTilesAndIncrFlameCtr(game: &mut Game) {
    UpdateGrappleBeamTiles(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) as ::core::ffi::c_int
        - 120 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh29 = *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16);
        *fresh29 = (*fresh29).wrapping_add(1);
    }
}
pub unsafe fn UpdateGrappleBeamTiles(game: &mut Game) {
    let ref mut fresh25 = *((&raw mut game.g_ram as *mut uint8).offset(0xd3e) as *mut uint16);
    *fresh25 = (*fresh25).wrapping_sub(1);
    if *fresh25 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd3e) as *mut uint16) = 5 as uint16;
        let ref mut fresh26 = *((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16);
        *fresh26 = (*fresh26 as ::core::ffi::c_int + 512 as ::core::ffi::c_int) as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16)
            as ::core::ffi::c_int
            - *(game.g_rom.offset(
                (((0x9bc344 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc344 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16) as ::core::ffi::c_int) as int16
            as ::core::ffi::c_int
            >= 0 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bc342 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc342 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16);
        }
    }
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 32 as uint16;
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16);
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh27 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh27 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = -(102 as ::core::ffi::c_int) as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 25088 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let mut v1: uint16 = (*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        & 0xfe as ::core::ffi::c_int) as uint16;
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 128 as uint16;
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = *(game.g_rom.offset(
        (((0x9bc346 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc346 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    v0 = (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    let fresh28 = v0;
    v0 = v0.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh28 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = -(102 as ::core::ffi::c_int) as uint8;
    (*gVramWriteEntry(v0 as ::core::ffi::c_int, game)).size = 25104 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        (v0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn HandleGrappleBeamFlare(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 16 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 3 as uint16;
        }
        let ref mut fresh20 = *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16);
        *fresh20 = (*fresh20).wrapping_sub(1);
        if *fresh20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            let ref mut fresh21 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16);
            *fresh21 = (*fresh21).wrapping_add(1);
            let mut v0: uint16 = *fresh21;
            if *(game.g_rom.offset(
                (((0x90c487 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x90c487 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint8)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) as isize,
                ) as ::core::ffi::c_int
                == 254 as ::core::ffi::c_int
            {
                let ref mut fresh22 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16);
                *fresh22 = (*fresh22 as ::core::ffi::c_int
                    - *(game.g_rom.offset(
                        (((0x90c487 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x90c487 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint8)
                        .offset(
                            (*((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16)
                                as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int) as uint16
                                as isize,
                        ) as ::core::ffi::c_int) as uint16;
                v0 = *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16);
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x90c487 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x90c487 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(v0 as isize) as uint16;
        }
        let mut r22: uint16 = 0;
        if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            == 4 as ::core::ffi::c_int
        {
            r22 = (*((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x93a22b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x93a22b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(0) as ::core::ffi::c_int) as uint16;
        } else {
            r22 = (*((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16)
                as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x93a225 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x93a225 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset(0) as ::core::ffi::c_int) as uint16;
        }
        let mut r20: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        if !(*((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16)
            as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int)
        {
            DrawBeamGrappleSpritemap(r22, r20, r18, game);
        }
    }
}
pub unsafe fn GrappleBeamHandler(game: &mut Game) {
    let mut r18: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) != 0 {
        let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16);
        *fresh8 = (*fresh8).wrapping_sub(1);
    }
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16);
    *fresh9 = (*fresh9 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint16;
    CancelGrappleBeamIfIncompatiblePose(game);
    CallGrappleBeamFunc(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) as ::core::ffi::c_int
            | 0x9b0000 as ::core::ffi::c_int) as uint32,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) as ::core::ffi::c_int
        != 0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) as ::core::ffi::c_int
            - (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa74) as *mut uint16) as ::core::ffi::c_int
            & 4 as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0x196e) as *mut uint16)
            as ::core::ffi::c_int
            != 0
        && {
            r18 = Samus_GetBottom_R18(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
        }
        && *((&raw mut game.g_ram as *mut uint8).offset(0x195e) as *mut uint16)
            as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16);
        *fresh10 = (*fresh10 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
    } else {
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16);
        *fresh11 = (*fresh11 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint16;
    };
}
pub unsafe fn GrappleBeamFunc_Inactive(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0xe00) as *mut uint16)
                as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        GrappleBeamFunc_FireGoToCancel(game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd8) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcda) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcde) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xce0) as *mut uint16) = 0 as uint16;
        Samus_LoadSuitPalette(game);
    }
}
pub unsafe fn GrappleBeamFunc_FireGoToCancel(game: &mut Game) {
    let mut r22: uint16 = 0;
    let mut v0: ::core::ffi::c_int = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_F0_FaceR_Draygon_Move as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_BE_FaceL_Draygon_Move as ::core::ffi::c_int
    {
        v0 = CheckBannedDraygonGrappleDirs(
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16),
            game,
        ) as ::core::ffi::c_int;
        r22 = 6 as uint16;
    } else {
        r22 = (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize))
        .y_offset_to_gfx as uint16;
        v0 = (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize))
        .direction_shots_fired as ::core::ffi::c_int;
        if v0 & 0xf0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) = v0 as uint16;
    let mut v1: ::core::ffi::c_int = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd22) as *mut uint16) = *(game.g_rom.offset(
        (((0x9bc0db as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc0db as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd24) as *mut uint16) = *(game.g_rom.offset(
        (((0x9bc0ef as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc0ef as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) = *(game.g_rom.offset(
        (((0x9bc104 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc104 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) = 10 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_49_FaceL_Moonwalk as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_4A_FaceR_Moonwalk as ::core::ffi::c_int
        || *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
            != 1 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd02) as *mut uint16) =
            *(game.g_rom.offset(
                (((0x9bc122 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc122 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd04) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc136 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc136 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc122 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc122 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc14a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc14a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc136 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc136 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc15e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc15e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd02) as *mut uint16) =
            *(game.g_rom.offset(
                (((0x9bc172 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc172 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd04) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc186 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc186 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc172 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc172 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc19a as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc19a as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc186 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc186 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16) =
            (*(game.g_rom.offset(
                (((0x9bc1ae as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x9bc1ae as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int
                - r22 as ::core::ffi::c_int) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd06) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd0a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd0e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd10) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd12) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd14) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 12 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd28) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd2a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd2c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd3a) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd3c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd3e) as *mut uint16) = 5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16) = *(game.g_rom.offset(
        (((0x9bc342 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x9bc342 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd90) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd38) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    GrappleFunc_AF87(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb86 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 0 as uint16;
    LoadProjectilePalette(2 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(223) =
        32657 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
        (0x9bc703 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    QueueSfx1_Max1(5 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc0) as *mut uint16) = 0 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa58) as *mut uint16) as ::core::ffi::c_int
        == 0x90946e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa58) as *mut uint16) =
            (0x90a337 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CheckBannedDraygonGrappleDirs(mut a: uint16, game: &mut Game) -> uint16 {
    if a as ::core::ffi::c_int == kPose_BE_FaceL_Draygon_Move as ::core::ffi::c_int {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
            & kButton_Left as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & kButton_Down as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return 6 as uint16;
            }
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & kButton_Up as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return 8 as uint16;
            }
        }
        return 7 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        & kButton_Right as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 2 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        & kButton_Down as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return 3 as uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        & kButton_Up as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        return 2 as uint16;
    }
    return 1 as uint16;
}
pub fn ClearCarry_12() -> uint8 {
    return 0 as uint8;
}
pub unsafe fn GrappleBeamFunc_Firing(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return;
    }
    let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
    *fresh16 = (*fresh16 as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) as ::core::ffi::c_int
        - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        return;
    }
    let mut pair: PairU16 = GrappleBeam_CollDetect_Enemy(game);
    let mut v1: ::core::ffi::c_int = ProcessEnemyGrappleBeamColl(pair.k, pair.j, game);
    if v1 >= 0 as ::core::ffi::c_int {
        if v1 != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    } else {
        let mut v2: uint8 = BlockCollGrappleBeam(game);
        if v2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return;
        }
        if v2 as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
            return;
        }
    }
    QueueSfx1_Max6(6 as uint16, game);
    HandleConnectingGrapple(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) =
        -(8 as ::core::ffi::c_int) as uint16;
    let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0xa64) as *mut uint16);
    *fresh17 = (*fresh17 as ::core::ffi::c_int | 1 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn GrappleBeamFunc_ConnectedLockedInPlace(game: &mut Game) {
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && (GrappleBeam_CollDetect_Enemy(game).k as ::core::ffi::c_int != 0
            || CheckIfGrappleIsConnectedToBlock(game) as ::core::ffi::c_int != 0))
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc856 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GrappleBeamFunc_Connected_Swinging(game: &mut Game) {
    let mut current_block: u64;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        GrappleBeamFunc_BB64(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) != 0 {
            BlockFunc_AC31(game);
        }
        GrappleBeamFunc_BC1F(game);
        GrappleBeamFunc_BCFF(game);
        GrappleBeamFunc_BD44(game);
        HandleMovementAndCollForSamusGrapple(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
            && HandleSpecialGrappleBeamAngles(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                != 0
        {
            return;
        }
        if GrappleBeam_CollDetect_Enemy(game).k != 0 {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16);
            *fresh13 = (*fresh13 as ::core::ffi::c_int | 0x8000 as ::core::ffi::c_int) as uint16;
            current_block = 11650488183268122163;
        } else if CheckIfGrappleIsConnectedToBlock(game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        {
            current_block = 4881625112085436895;
        } else {
            current_block = 11650488183268122163;
        }
        match current_block {
            4881625112085436895 => {}
            _ => {
                BlockFunc_AC11(game);
                GrappleBeamFunc_BD95(game);
                return;
            }
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) as ::core::ffi::c_int
            != 0x8000 as ::core::ffi::c_int
    {
        PropelSamusFromGrappleSwing(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bcb8b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa58) as *mut uint16) =
            (0x90946e as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn GrappleBeamFunc_Wallgrab(game: &mut Game) {
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x9b2) as *mut uint16) as ::core::ffi::c_int
        & *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
        && CheckIfGrappleIsConnectedToBlock(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
            != 0)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 30 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc832 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GrappleBeamFunc_C832(game: &mut Game) {
    let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16);
    *fresh12 = (*fresh12).wrapping_sub(1);
    if *fresh12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if Samus_GrappleWallJumpCheck((16 as int16 as int32) << 16 as ::core::ffi::c_int, game) != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                (0x9bc9ce as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
            (0x9bc8c5 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn GrappleBeamFunc_Cancel(game: &mut Game) {
    QueueSfx1_Max15(7 as uint16, game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as ::core::ffi::c_int
        == kMovementType_16_Grappling as ::core::ffi::c_int
    {
        Samus_Pose_CancelGrapple(game);
    } else {
        CallSomeSamusCode(0x1c as uint16, game);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0) as *mut uint16) = 0 as uint16;
    LoadProjectilePalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
        (0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Samus_PostGrappleCollisionDetect(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GrappleBeam_Func2(game: &mut Game) {
    let mut current_block: u64;
    QueueSfx1_Max15(7 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_B2_FaceR_Grapple_Air as ::core::ffi::c_int
    {
        current_block = 16041378612104114408;
    } else {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_B3_FaceL_Grapple_Air as ::core::ffi::c_int
        {
            current_block = 13151459208701579741;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
            as ::core::ffi::c_int
            - 17 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if (*(game.g_rom.offset(
                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut SamusPoseParams)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize,
                ))
            .direction_shots_fired as ::core::ffi::c_int
                & 0xf0 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
                    == 4 as ::core::ffi::c_int
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                        kPose_28_FaceL_Crouch as uint16;
                } else {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                        kPose_27_FaceR_Crouch as uint16;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                    *(game.g_rom.offset(
                        (((0x9bc9c4 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                            << 15 as ::core::ffi::c_int
                            | 0x9bc9c4 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                            & 0x3fffff as ::core::ffi::c_int) as isize,
                    ) as *const uint8 as *mut uint8)
                        .offset(
                            *(&raw mut (*(game.g_rom.offset(
                                (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                    << 15 as ::core::ffi::c_int
                                    | 0x91b629 as ::core::ffi::c_int
                                        & 0x7fff as ::core::ffi::c_int)
                                    & 0x3fffff as ::core::ffi::c_int)
                                    as isize,
                            ) as *const uint8
                                as *mut SamusPoseParams)
                                .offset(0))
                            .direction_shots_fired)
                                .offset(
                                    (8 as ::core::ffi::c_int
                                        * *((&raw mut game.g_ram as *mut uint8).offset(0xa1c)
                                            as *mut uint16)
                                            as ::core::ffi::c_int)
                                        as isize,
                                ) as isize,
                        ) as uint16;
            }
            current_block = 8968942556654489080;
        } else if (*(game.g_rom.offset(
            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut SamusPoseParams)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize))
        .direction_shots_fired as ::core::ffi::c_int
            & 0xf0 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
                != 4 as ::core::ffi::c_int
            {
                current_block = 16041378612104114408;
            } else {
                current_block = 13151459208701579741;
            }
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                *(game.g_rom.offset(
                    (((0x9bc9ba as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x9bc9ba as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint8)
                    .offset(
                        *(&raw mut (*(game.g_rom.offset(
                            (((0x91b629 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                                << 15 as ::core::ffi::c_int
                                | 0x91b629 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                                & 0x3fffff as ::core::ffi::c_int)
                                as isize,
                        ) as *const uint8
                            as *mut SamusPoseParams)
                            .offset(0))
                        .direction_shots_fired)
                            .offset(
                                (8 as ::core::ffi::c_int
                                    * *((&raw mut game.g_ram as *mut uint8).offset(0xa1c)
                                        as *mut uint16)
                                        as ::core::ffi::c_int)
                                    as isize,
                            ) as isize,
                    ) as uint16;
            current_block = 8968942556654489080;
        }
        match current_block {
            8968942556654489080 => {}
            16041378612104114408 => {}
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                    kPose_02_FaceL_Normal as uint16;
                current_block = 8968942556654489080;
            }
        }
    }
    match current_block {
        16041378612104114408 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
                kPose_01_FaceR_Normal as uint16;
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa32) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc6) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0) as *mut uint16) = 0 as uint16;
    LoadProjectilePalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
        (0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Samus_PostGrappleCollisionDetect(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GrappleBeamFunc_C9CE(game: &mut Game) {
    QueueSfx1_Max15(7 as uint16, game);
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
        == 8 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
            kPose_84_FaceL_Walljump as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
            kPose_83_FaceR_Walljump as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa32) as *mut uint16) = 6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdce) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb22) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2a) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb38) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0) as *mut uint16) = 0 as uint16;
    LoadProjectilePalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
        (0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Samus_PostGrappleCollisionDetect(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn PropelSamusFromGrappleSwing(game: &mut Game) {
    let mut v1: int16 = 0;
    let mut v2: int16 = 0;
    let mut v0: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        v0 = (2 as ::core::ffi::c_int
            * *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        v2 = kSinCosTable8bit_Sext[(*((&raw mut game.g_ram as *mut uint8).offset(0xcfa)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as usize];
        if v2 as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16,
                Multiply16x16(v2 as uint16, v0, game),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) = 2 as uint16;
        } else {
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16,
                Multiply16x16(-(v2 as ::core::ffi::c_int) as uint16, v0, game),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) = 1 as uint16;
        }
    } else {
        v0 = (-(2 as ::core::ffi::c_int)
            * *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)
                as ::core::ffi::c_int) as uint16;
        v1 = kSinCosTable8bit_Sext[(*((&raw mut game.g_ram as *mut uint8).offset(0xcfa)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as usize];
        if (v1 as ::core::ffi::c_int) < 0 as ::core::ffi::c_int {
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16,
                Multiply16x16(-(v1 as ::core::ffi::c_int) as uint16, v0, game),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) = 2 as uint16;
        } else {
            SetHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xb2e) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xb2c) as *mut uint16,
                Multiply16x16(v1 as uint16, v0, game),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xb36) as *mut uint16) = 1 as uint16;
        }
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xb4a) as *mut uint16) = 2 as uint16;
    let mut r18: uint16 =
        (3 as ::core::ffi::c_int * (v0 as ::core::ffi::c_int >> 9 as ::core::ffi::c_int)) as uint16;
    r18 = (64 as ::core::ffi::c_int - r18 as ::core::ffi::c_int) as uint16;
    let mut v3: uint16 = abs16(
        kSinCosTable8bit_Sext[((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)
            as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int) as uint8
            as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int) as usize] as uint16,
    );
    SetHiLo(
        (&raw mut game.g_ram as *mut uint8).offset(0xb46) as *mut uint16,
        (&raw mut game.g_ram as *mut uint8).offset(0xb48) as *mut uint16,
        Multiply16x16(v3, v0, game),
    );
}
pub unsafe fn GrappleBeamFunc_ReleaseFromSwing(game: &mut Game) {
    QueueSfx1_Max15(7 as uint16, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
            kPose_52_FaceL_Jump_NoAim_MoveF as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa2c) as *mut uint16) =
            kPose_51_FaceR_Jump_NoAim_MoveF as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xa32) as *mut uint16) = 7 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd1e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd20) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd34) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa9e) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd0) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd6) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd8) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcda) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcdc) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcde) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xce0) as *mut uint16) = 0 as uint16;
    LoadProjectilePalette(
        *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16),
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
        (0x9bc4f0 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa5c) as *mut uint16) =
        (0x90eb52 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int) as uint16;
    Samus_PostGrappleCollisionDetect(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9d2) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa04) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn CallGrappleBeamFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10206052 => {
            GrappleBeamFunc_BB64(game);
            return;
        }
        10206239 => {
            GrappleBeamFunc_BC1F(game);
            return;
        }
        10206463 => {
            GrappleBeamFunc_BCFF(game);
            return;
        }
        10206532 => {
            GrappleBeamFunc_BD44(game);
            return;
        }
        10206613 => {
            GrappleBeamFunc_BD95(game);
            return;
        }
        10206872 => {
            GrappleBeamFunc_BE98(game);
            return;
        }
        10206955 => {
            GrappleBeamFunc_BEEB(game);
            return;
        }
        10207003 => {
            GrappleBeamFunc_BF1B(game);
            return;
        }
        10208496 => {
            GrappleBeamFunc_Inactive(game);
            return;
        }
        10208542 => {
            GrappleBeamFunc_FireGoToCancel(game);
            return;
        }
        10209027 => {
            GrappleBeamFunc_Firing(game);
            return;
        }
        10209150 => {
            GrappleBeamFunc_ConnectedLockedInPlace(game);
            return;
        }
        10209181 => {
            GrappleBeamFunc_Connected_Swinging(game);
            return;
        }
        10209300 => {
            GrappleBeamFunc_Wallgrab(game);
            return;
        }
        10209330 => {
            GrappleBeamFunc_C832(game);
            return;
        }
        10209366 => {
            GrappleBeamFunc_Cancel(game);
            return;
        }
        10209742 => {
            GrappleBeamFunc_C9CE(game);
            return;
        }
        10210187 => {
            GrappleBeamFunc_ReleaseFromSwing(game);
            return;
        }
        10209477 => {
            GrappleBeam_Func2(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
