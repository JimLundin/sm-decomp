pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_90::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{CheckBossBitForCurArea, Multiply16x16};
use crate::game::sm_84::{SpawnPLM, SpawnPLMWithCollision};
use crate::game::sm_90::KillProjectile;
use crate::game::sm_a0::CalculateAngleFromXY;
use crate::game_state::Game;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
use crate::snes::snes::snes_read;

pub use crate::types::Consts_84;
pub use crate::types::Consts_90;
pub use crate::types::Consts_94;
pub use crate::types::GameState;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, DoorDef, Func_V,
    OamEnt, SamusPose, VoidP,
};
pub const addr_word_94B197: Consts_94 = 45463;
pub const addr_word_94B193: Consts_94 = 45459;
pub const addr_word_94B18F: Consts_94 = 45455;
pub const addr_word_94B18B: Consts_94 = 45451;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CollInfo {
    pub ci_r18_r20: int32,
    pub ci_r24: uint16,
    pub ci_r26: uint16,
    pub ci_r28: uint16,
    pub ci_r30: uint16,
    pub ci_r32: uint16,
    pub ci_r38: uint16,
    pub ci_r40: uint16,
}
pub type Func_CollInfo_U8 = unsafe fn(*mut CollInfo, &mut Game) -> uint8;
pub use crate::types::Pair_Bool_Amt;
/// Stack-owned sample state for checking Samus's edges after releasing grapple.
struct PostGrappleCollisionSample {
    remaining_span: u16,
    initial_span: u16,
    edge_position: u16,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_8F(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0x8f0000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn RomPtr_94(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0x940000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn gOamEnt(mut v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    return (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt;
}
#[inline]
unsafe fn get_DoorDef(mut a: uint16, game: &mut Game) -> *mut DoorDef {
    return RomPtr(((0x830000u32) | ((a) as u32)), game) as *mut DoorDef;
}
pub const fnDoorTransitionFunction_HandleElevator: ::core::ffi::c_int = 0x82e17d;
pub const fnGrappleInstr_Goto: uint32 = 9744628;
pub const fnGrappleBeam_Func2: ::core::ffi::c_int = 0x9bc8c5;

static kAlignPos_Tab1: [uint8; 512] = [
    16, 16, 16, 16, 16, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 15, 14, 13, 12, 11, 10,
    9, 9, 10, 11, 12, 13, 14, 15, 16, 16, 14, 12, 10, 8, 6, 4, 2, 2, 4, 6, 8, 10, 12, 14, 16, 16,
    16, 16, 16, 16, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 12, 12, 8, 8, 8, 8, 4, 4, 4, 4,
    0, 0, 0, 0, 14, 14, 12, 12, 10, 10, 8, 8, 6, 6, 4, 4, 2, 2, 0, 0, 16, 16, 16, 16, 16, 16, 16,
    16, 16, 16, 16, 16, 16, 16, 16, 16, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 16, 16,
    16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 15, 15, 15, 14, 14, 14, 13, 13, 13, 12,
    12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 9, 8, 8, 8, 7, 7, 7, 6, 6, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3,
    2, 2, 2, 1, 1, 1, 20, 20, 20, 20, 20, 20, 20, 20, 16, 14, 12, 10, 8, 6, 4, 2, 16, 14, 12, 10,
    8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 15, 12, 9, 6,
    3, 20, 20, 20, 20, 20, 20, 14, 11, 8, 5, 2, 0, 0, 0, 0, 0, 16, 13, 10, 7, 4, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];
static kAlignYPos_Tab0: [uint8; 512] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 0, 0, 0, 0, 0,
    0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 15, 14, 13, 12, 11, 10,
    9, 9, 10, 11, 12, 13, 14, 15, 16, 16, 14, 12, 10, 8, 6, 4, 2, 2, 4, 6, 8, 10, 12, 14, 16, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 12, 12, 8, 8, 8, 8, 4, 4, 4, 4, 0, 0, 0, 0,
    14, 14, 12, 12, 10, 10, 8, 8, 6, 6, 4, 4, 2, 2, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
    16, 16, 16, 16, 16, 16, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 16, 16, 16, 16, 15,
    14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 16, 16, 15, 15, 14, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8, 7, 7, 6, 6,
    5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 16, 16, 16, 15, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 12, 11,
    11, 11, 10, 10, 10, 9, 9, 9, 8, 8, 8, 7, 7, 7, 6, 6, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 1,
    1, 1, 20, 20, 20, 20, 20, 20, 20, 20, 16, 14, 12, 10, 8, 6, 4, 2, 16, 14, 12, 10, 8, 6, 4, 2,
    0, 0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 15, 12, 9, 6, 3, 20, 20,
    20, 20, 20, 20, 14, 11, 8, 5, 2, 0, 0, 0, 0, 0, 16, 13, 10, 7, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
static kTab948E54: [uint8; 20] = [
    0, 0, 0x80, 0x80, 0, 0x80, 0, 0x80, 0, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
];
static kBlockColl_Horiz_Slope_NonSquare_Tab: [uint16; 64] = [
    0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0,
    0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0x1000, 0xb0, 0x1000, 0xb0, 0, 0x100, 0, 0x100,
    0x1000, 0xc0, 0, 0x100, 0x1000, 0xc0, 0x1000, 0xc0, 0x800, 0xd8, 0x800, 0xd8, 0x600, 0xf0,
    0x600, 0xf0, 0x600, 0xf0, 0x4000, 0x80, 0x4000, 0x80, 0x6000, 0x50, 0x6000, 0x50, 0x6000, 0x50,
];
fn PostGrappleColl_Horiz_Slope_NonSquare(
    sample: &PostGrappleCollisionSample,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    let negative = game.samus_collision_direction() & 1 != 0;
    let x = game.samus_x_pos();
    let block = game.cur_block_index();
    let width = game.room_width_in_blocks() as u8;
    let column = if width == 0 {
        block
    } else {
        block % u16::from(width)
    };
    if x >> 4 != column {
        return 0xffff;
    }
    // k selects the slope bytes even when the live collision index differs.
    let bts = game.g_ram[0x16402 + usize::from(k)];
    if (bts & 0x40 != 0) == negative {
        return PostGrappleColl_Horiz_Solid(sample, game);
    }
    let y = game.samus_y_pos();
    let y = if bts & 0x80 != 0 { y ^ 15 } else { y };
    let index = usize::from(bts & 0x1f) * 16 + usize::from(y & 15);
    let boundary = i16::from(kAlignPos_Tab1[index] & 0x1f);
    let difference = boundary - (sample.edge_position & 15) as i16 - 1;
    if negative {
        if difference <= 0 {
            !(difference as u16)
        } else {
            0xffff
        }
    } else if difference >= 0 {
        difference as u16
    } else {
        0xffff
    }
}
fn PostGrappleColl_Vert_Slope_NonSquare(
    sample: &PostGrappleCollisionSample,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    let negative = game.samus_collision_direction() & 1 != 0;
    let block = game.cur_block_index();
    let width = game.room_width_in_blocks() as u8;
    let column = if width == 0 {
        block
    } else {
        block % u16::from(width)
    };
    let x = game.samus_x_pos();
    if x >> 4 != column {
        return 0xffff;
    }
    let edge = sample.edge_position & 15;
    // The passed index selects slope bytes independently of column alignment.
    let bts = game.g_ram[0x16402 + usize::from(k)];
    if (bts & 0x80 != 0) == negative {
        return PostGrappleColl_Vert_Solid(sample, game);
    }
    let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
    let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
    let boundary = i16::from(kAlignYPos_Tab0[index] & 0x1f);
    let difference = boundary - edge as i16 - 1;
    if negative {
        if difference <= 0 {
            !(difference as u16)
        } else {
            0xffff
        }
    } else if difference >= 0 {
        difference as u16
    } else {
        0xffff
    }
}
fn PostGrappleColl_Horiz_Slope_Square(
    sample: &PostGrappleCollisionSample,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    let bts = game.g_ram[0x16402 + usize::from(k)];
    let index =
        u16::from(bts & 0x1f) * 4 + (u16::from(bts >> 6) ^ ((sample.edge_position & 8) >> 3));
    if !square_slope_occupied(
        index,
        sample.remaining_span,
        sample.initial_span,
        game.samus_y_pos(),
        game.samus_y_radius(),
        2,
    ) {
        return 0xffff;
    }
    let distance = sample.edge_position & 7;
    if game.samus_collision_direction() & 1 != 0 {
        distance
    } else {
        distance ^ 7
    }
}
fn PostGrappleColl_Vertical_Slope_Square(
    sample: &PostGrappleCollisionSample,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    let bts = game.g_ram[0x16402 + usize::from(k)];
    let index =
        u16::from(bts & 0x1f) * 4 + (u16::from(bts >> 6) ^ ((sample.edge_position & 8) >> 2));
    if !square_slope_occupied(
        index,
        sample.remaining_span,
        sample.initial_span,
        game.samus_x_pos(),
        game.samus_x_radius(),
        1,
    ) {
        return 0xffff;
    }
    let distance = sample.edge_position & 7;
    if game.samus_collision_direction() & 1 != 0 {
        distance
    } else {
        distance ^ 7
    }
}
fn PostGrappleColl_Horiz_Slope(sample: &PostGrappleCollisionSample, game: &mut Game) -> uint16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Horiz_Slope_Square(sample, block, game)
    } else {
        PostGrappleColl_Horiz_Slope_NonSquare(sample, block, game)
    }
}
fn PostGrappleColl_Horiz_Solid(sample: &PostGrappleCollisionSample, game: &mut Game) -> uint16 {
    sample.edge_position & 0xf
}
fn PostGrappleColl_Vert_Slope(sample: &PostGrappleCollisionSample, game: &mut Game) -> uint16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Vertical_Slope_Square(sample, block, game)
    } else {
        PostGrappleColl_Vert_Slope_NonSquare(sample, block, game)
    }
}
fn PostGrappleColl_Vert_Solid(sample: &PostGrappleCollisionSample, game: &mut Game) -> uint16 {
    sample.edge_position & 0xf
}
fn PostGrappleColl_Horiz(
    sample: &PostGrappleCollisionSample,
    k: uint16,
    game: &mut Game,
) -> uint16 {
    game.set_cur_block_index(k >> 1);
    match game.current_level_block() >> 12 {
        1 => PostGrappleColl_Horiz_Slope(sample, game),
        8..=15 => PostGrappleColl_Horiz_Solid(sample, game),
        _ => 0xffff,
    }
}
fn PostGrappleColl_Vert(sample: &PostGrappleCollisionSample, k: uint16, game: &mut Game) -> uint16 {
    game.set_cur_block_index(k >> 1);
    match game.current_level_block() >> 12 {
        1 => PostGrappleColl_Vert_Slope(sample, game),
        8..=15 => PostGrappleColl_Vert_Solid(sample, game),
        _ => 0xffff,
    }
}
fn post_grapple_traverse(horizontal: bool, positive: bool, game: &mut Game) {
    let (direction, distance_offset) = match (horizontal, positive) {
        (true, true) => (1, 0xe04),   // distance_to_eject_samus_left
        (true, false) => (0, 0xe06),  // distance_to_eject_samus_right
        (false, true) => (3, 0xe08),  // distance_to_eject_samus_up
        (false, false) => (2, 0xe0a), // distance_to_eject_samus_down
    };
    game.set_samus_collision_direction(direction);
    game.g_ram[distance_offset..distance_offset + 2].fill(0);
    let span = if horizontal {
        Samus_GetYposSpan(game)
    } else {
        Samus_GetXposSpan(game)
    };
    let (position, radius) = if horizontal {
        (game.samus_x_pos(), game.samus_x_radius())
    } else {
        (game.samus_y_pos(), game.samus_y_radius())
    };
    let edge = if positive {
        position.wrapping_add(radius).wrapping_sub(1)
    } else {
        position.wrapping_sub(radius)
    };
    let row = if horizontal {
        game.samus_y_pos().wrapping_sub(game.samus_y_radius()) >> 4
    } else {
        edge >> 4
    };
    let column = if horizontal {
        edge >> 4
    } else {
        game.samus_x_pos().wrapping_sub(game.samus_x_radius()) >> 4
    };
    // The initial row multiply uses only the low bytes of row and room width.
    let row_start = u16::from(row as u8) * u16::from(game.room_width_in_blocks() as u8);
    game.set_cur_block_index(row_start.wrapping_add(column));
    let mut block_offset = game.cur_block_index().wrapping_mul(2);
    let mut context = PostGrappleCollisionSample {
        remaining_span: span,
        initial_span: span,
        edge_position: edge,
    };
    loop {
        let result = if horizontal {
            PostGrappleColl_Horiz(&context, block_offset, game)
        } else {
            PostGrappleColl_Vert(&context, block_offset, game)
        } as i16;
        if result >= 0 {
            let distance = (result as u16).wrapping_add(1);
            let maximum =
                u16::from_le_bytes([game.g_ram[distance_offset], game.g_ram[distance_offset + 1]]);
            if distance >= maximum {
                game.g_ram[distance_offset..distance_offset + 2]
                    .copy_from_slice(&distance.to_le_bytes());
            }
        }
        // Horizontal samples use the full live width after the collision callback.
        let step = if horizontal {
            game.room_width_in_blocks().wrapping_mul(2)
        } else {
            2
        };
        block_offset = block_offset.wrapping_add(step);
        context.remaining_span = context.remaining_span.wrapping_sub(1);
        if context.remaining_span & 0x8000 != 0 {
            break;
        }
    }
}
pub fn PostGrappleCollisionDetect_X(game: &mut Game) {
    post_grapple_traverse(true, true, game);
    post_grapple_traverse(true, false, game);
}
pub fn PostGrappleCollisionDetect_Y(game: &mut Game) {
    post_grapple_traverse(false, true, game);
    post_grapple_traverse(false, false, game);
}
unsafe fn BlockColl_Horiz_Slope_NonSquare(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_slope_bts();
    if bts & 0x80 != 0 {
        return 0;
    }
    let y_speed = u32::from_le_bytes([
        game.g_ram[0xb2c],
        game.g_ram[0xb2d],
        game.g_ram[0xb2e],
        game.g_ram[0xb2f],
    ]);
    if y_speed != 0 {
        return 0;
    }
    let displacement = (*ci).ci_r18_r20;
    // Keep the middle sixteen bits of the arithmetic right shift.
    let shifted = (displacement >> 8) as u16;
    let factor = u32::from(kBlockColl_Horiz_Slope_NonSquare_Tab[usize::from(bts & 0x1f) * 2 + 1]);
    let multiplicand = if displacement < 0 {
        shifted.wrapping_neg()
    } else {
        shifted
    };
    game.g_ram[0x5e9..0x5eb].copy_from_slice(&multiplicand.to_le_bytes());
    game.g_ram[0x5eb..0x5ed].copy_from_slice(&(factor as u16).to_le_bytes());
    (*ci).ci_r18_r20 = if displacement >= 0 {
        (u32::from(shifted) * factor) as i32
    } else {
        ((u32::from(shifted.wrapping_neg()) * factor) as i32).wrapping_neg()
    };
    0
}
unsafe fn BlockColl_Vert_Slope_NonSquare(ci: *mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    let direction = game.samus_collision_direction();
    let upward = direction & 1 != 0;
    let x_position = game.samus_x_pos();
    // The SNES division register receives only the low byte of room width.
    let width = game.room_width_in_blocks() as u8;
    let block = game.cur_block_index();
    let column = if width == 0 {
        block
    } else {
        block % u16::from(width)
    };
    if x_position >> 4 != column {
        return 0;
    }
    let bts = game.g_ram[0x16402 + usize::from(k)];
    if (bts & 0x80 != 0) == upward {
        return 0;
    }
    let x_offset = if bts & 0x40 != 0 {
        x_position ^ 15
    } else {
        x_position
    };
    let index = usize::from(bts & 0x1f) * 16 + usize::from(x_offset & 15);
    let height = kAlignYPos_Tab0[index] & 0x1f;
    let radius = game.samus_y_radius();
    let y_offset = if upward {
        radius.wrapping_add((*ci).ci_r24).wrapping_sub(1) & 15
    } else {
        ((*ci).ci_r24.wrapping_sub(radius) & 15) ^ 15
    };
    let adjustment = (height as i16) - (y_offset as i16) - 1;
    if adjustment > 0 {
        return 0;
    }
    let displacement = (((*ci).ci_r18_r20 >> 16) as i16)
        .wrapping_add(adjustment)
        .max(0);
    (*ci).ci_r18_r20 = i32::from(displacement) << 16;
    1
}
pub unsafe fn Samus_AlignYPosSlope(game: &mut Game) {
    let movement_flags = u16::from_le_bytes([game.g_ram[0xa46], game.g_ram[0xa47]]);
    if movement_flags & 2 == 0 {
        return;
    }
    let x = game.samus_x_pos();
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    CalculateBlockAt(x, radius.wrapping_add(y).wrapping_sub(1), 0, 0, game);
    if game.current_level_block() & 0xf000 == 0x1000 && game.current_block_bts() & 0x1f >= 5 {
        let radius = game.samus_y_radius();
        let y = game.samus_y_pos();
        let offset = radius.wrapping_add(y).wrapping_sub(1) & 15;
        let bts = game.current_block_bts();
        if bts & 0x80 == 0 {
            let x = game.samus_x_pos();
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(kAlignYPos_Tab0[index] & 0x1f) - offset as i16 - 1;
            if adjustment < 0 {
                game.g_ram[0xafa..0xafc]
                    .copy_from_slice(&y.wrapping_add(adjustment as u16).to_le_bytes());
                game.set_samus_pos_adjusted_by_slope_flag(1);
            }
        }
    }
    // Re-read the position after the bottom-edge alignment before looking up the top block.
    let x = game.samus_x_pos();
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    CalculateBlockAt(x, y.wrapping_sub(radius), 0, 0, game);
    if game.current_level_block() & 0xf000 == 0x1000 && game.current_block_bts() & 0x1f >= 5 {
        let y = game.samus_y_pos();
        let radius = game.samus_y_radius();
        let offset = (y.wrapping_sub(radius) & 15) ^ 15;
        let bts = game.current_block_bts();
        if bts & 0x80 != 0 {
            let x = game.samus_x_pos();
            let x = if bts & 0x40 != 0 { x ^ 15 } else { x };
            let index = usize::from(bts & 0x1f) * 16 + usize::from(x & 15);
            let adjustment = i16::from(kAlignYPos_Tab0[index] & 0x1f) - offset as i16 - 1;
            if adjustment <= 0 {
                game.g_ram[0xafa..0xafc]
                    .copy_from_slice(&y.wrapping_sub(adjustment as u16).to_le_bytes());
                game.set_samus_pos_adjusted_by_slope_flag(1);
            }
        }
    }
}
fn square_slope_occupied(
    index: u16,
    first: u16,
    last: u16,
    position: u16,
    radius: u16,
    other: u16,
) -> bool {
    if first == 0 && radius.wrapping_add(position).wrapping_sub(1) & 8 == 0 {
        return kTab948E54[usize::from(index)] != 0;
    }
    if first != 0 && first == last && position.wrapping_sub(radius) & 8 != 0 {
        return kTab948E54[usize::from(index ^ other)] != 0;
    }
    kTab948E54[usize::from(index)] != 0 || kTab948E54[usize::from(index ^ other)] != 0
}
fn square_slope_delta(edge: u16, radius: u16, position: u16, negative: bool) -> i32 {
    // Truncate to signed sixteen bits before clamping at the eight-pixel boundary.
    let delta = if negative {
        radius
            .wrapping_add(edge | 7)
            .wrapping_add(1)
            .wrapping_sub(position) as i16
    } else {
        (edge & 0xfff8).wrapping_sub(radius).wrapping_sub(position) as i16
    };
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
unsafe fn BlockColl_Horiz_Slope_Square(
    ci: *mut CollInfo,
    slope: uint16,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    let orientation = u16::from(game.g_ram[0x16402 + usize::from(k)] >> 6);
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ (((*ci).ci_r32 & 8) >> 3));
    let perpendicular_position = game.samus_y_pos();
    let perpendicular_radius = game.samus_y_radius();
    if !square_slope_occupied(
        index,
        (*ci).ci_r26,
        (*ci).ci_r28,
        perpendicular_position,
        perpendicular_radius,
        2,
    ) {
        return 0;
    }
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_x_radius();
    let position = game.samus_x_pos();
    (*ci).ci_r18_r20 = square_slope_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_x_subpos(subposition);
    1
}
unsafe fn BlockColl_Vert_Slope_Square(
    ci: *mut CollInfo,
    slope: uint16,
    k: uint16,
    game: &mut Game,
) -> uint8 {
    let orientation = u16::from(game.g_ram[0x16402 + usize::from(k)] >> 6);
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ (((*ci).ci_r32 & 8) >> 2));
    let perpendicular_position = game.samus_x_pos();
    let perpendicular_radius = game.samus_x_radius();
    if !square_slope_occupied(
        index,
        (*ci).ci_r26,
        (*ci).ci_r28,
        perpendicular_position,
        perpendicular_radius,
        1,
    ) {
        return 0;
    }
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_y_radius();
    let position = game.samus_y_pos();
    (*ci).ci_r18_r20 = square_slope_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_y_subpos(subposition);
    if !negative {
        game.set_samus_pos_adjusted_by_slope_flag(1);
    }
    1
}
fn apply_spike_block_damage(amount: u16, game: &mut Game) {
    let invincibility = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
    if invincibility != 0 {
        return;
    }
    game.g_ram[0x18a8..0x18aa].copy_from_slice(&60u16.to_le_bytes());
    game.g_ram[0x18aa..0x18ac].copy_from_slice(&10u16.to_le_bytes());
    let damage = u16::from_le_bytes([game.g_ram[0xa50], game.g_ram[0xa51]]);
    game.g_ram[0xa50..0xa52].copy_from_slice(&damage.wrapping_add(amount).to_le_bytes());
    // Spike blocks read only the facing byte, unlike the spike-air reaction.
    let knockback = u16::from((game.g_ram[0xa1e] ^ 12) & 8 != 0);
    game.g_ram[0xa54..0xa56].copy_from_slice(&knockback.to_le_bytes());
}
fn ClearCarry_4(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
// The boundary difference wraps to signed sixteen bits before the direction clamp.
fn solid_collision_delta(edge: u16, radius: u16, position: u16, negative: bool) -> i32 {
    let delta = if negative {
        radius
            .wrapping_add(edge | 15)
            .wrapping_add(1)
            .wrapping_sub(position) as i16
    } else {
        (edge & 0xfff0).wrapping_sub(radius).wrapping_sub(position) as i16
    };
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
// Keep the context raw: PLM callbacks can also access its active displacement.
unsafe fn BlockColl_Horiz_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_x_radius();
    let position = game.samus_x_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_x_subpos(subposition);
    1
}
unsafe fn BlockColl_Vert_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_y_radius();
    let position = game.samus_y_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_y_subpos(subposition);
    1
}
unsafe fn BlockColl_Horiz_Slope(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let slope = u16::from(game.current_block_bts() & 0x1f);
    if slope < 5 {
        return BlockColl_Horiz_Slope_Square(ci, slope, game.cur_block_index(), game);
    }
    game.set_current_slope_bts(u16::from(game.current_block_bts()));
    BlockColl_Horiz_Slope_NonSquare(ci, game)
}
unsafe fn BlockColl_Vert_Slope(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let slope = u16::from(game.current_block_bts() & 0x1f);
    if slope < 5 {
        return BlockColl_Vert_Slope_Square(ci, slope, game.cur_block_index(), game);
    }
    // The ROM's vertical dispatch stores the full LDA long,X word, including
    // the neighboring BTS byte. Long indexing can cross the WRAM bank edge.
    let address = 0x7f6402 + u32::from(game.cur_block_index());
    let low = snes_read(game.g_snes, address, game);
    let high = snes_read(game.g_snes, address + 1, game);
    game.set_current_slope_bts(u16::from_le_bytes([low, high]));
    BlockColl_Vert_Slope_NonSquare(ci, game.cur_block_index(), game)
}
fn ClearCarry_5(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
fn BlockColl_Vert_SpikeAir(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
unsafe fn react_to_spike_block(game: &mut Game) {
    let bts = game.current_block_bts();
    // Preserve the former sixteen-entry table's panic before solid collision.
    assert!(bts < 16, "spike-block BTS index out of bounds: {bts}");
    match bts {
        0 => {
            let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
            // Check the boss before reading the live invincibility timer.
            if area != 3 || CheckBossBitForCurArea(1, game) & 1 != 0 {
                apply_spike_block_damage(60, game);
            }
        }
        1 | 3 => apply_spike_block_damage(16, game),
        _ => {}
    }
}
unsafe fn BlockColl_Horiz_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    react_to_spike_block(game);
    BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
}
unsafe fn BlockColl_Vert_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    react_to_spike_block(game);
    BlockColl_Vert_SolidShootGrappleBlock(ci, game)
}
// High-bit BTS values select an area-specific table in the live ROM.
unsafe fn special_collision_plm(bts: u8, area_tables: u32, game: &mut Game) -> u16 {
    if bts & 0x80 != 0 {
        let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
        let table = game.read_rom_word(area_tables, usize::from(area) * 2);
        if table & 0x8000 == 0 {
            // Retain the legacy invalid-window diagnostic and g_fail write.
            RomPtr_94(table, game);
        }
        // The legacy mapper ignores bit 15 after diagnosing an invalid window.
        game.read_rom_word(0x948000 | u32::from(table), usize::from(bts & 0x7f) * 2)
    } else {
        game.read_rom_word(0x949139, usize::from(bts) * 2)
    }
}
unsafe fn BlockColl_Horiz_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492d9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    if bts & 0x80 != 0 {
        result & 1
    } else {
        0
    }
}
unsafe fn BlockColl_Vert_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockColl_Horiz_SpecialAir(ci, game)
}
unsafe fn BlockColl_Horiz_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492e9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        // Setup can adjust displacement or Samus RAM; collision reads remain live.
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Vert_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492e9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Horiz_BombableAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 == 0 {
        let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
        SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    }
    0
}
unsafe fn BlockColl_Vert_BombableAir(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockColl_Horiz_BombableAir(ci, game)
}
unsafe fn BlockColl_Horiz_BombBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        return BlockColl_Horiz_SolidShootGrappleBlock(ci, game);
    }
    let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        // Setup may adjust the displacement and Samus RAM before collision.
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Vert_BombBlock(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        return BlockColl_Vert_SolidShootGrappleBlock(ci, game);
    }
    let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
pub unsafe fn BlockColl_Horiz_Door(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    game.g_ram[0x99c..0x99e].copy_from_slice(&0xe17du16.to_le_bytes());
    let bts = game.current_block_bts();
    let door_list = u16::from_le_bytes([game.g_ram[0x7b5], game.g_ram[0x7b6]]);
    let entry = door_list.wrapping_add(u16::from(bts & 0x7f) * 2);
    if entry & 0x8000 == 0 {
        RomPtr_8F(entry, game);
    }
    let door = game.read_rom_word(0x8f8000 | u32::from(entry), 0);
    if door & 0x8000 == 0 {
        get_DoorDef(door, game);
    }
    let destination = game.read_rom_word(0x838000 | u32::from(door), 0);
    if destination & 0x8000 == 0 {
        // Preserve the original pose comparison, even though its threshold is a state value.
        let pose = u16::from_le_bytes([game.g_ram[0xa1c], game.g_ram[0xa1d]]);
        if pose < 9 {
            game.g_ram[0xe16..0xe18].copy_from_slice(&1u16.to_le_bytes());
        }
        BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
    } else {
        game.g_ram[0x78d..0x78f].copy_from_slice(&door.to_le_bytes());
        game.g_ram[0x998..0x99a].copy_from_slice(&9u16.to_le_bytes());
        0
    }
}
pub unsafe fn BlockColl_Vert_Door(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    game.g_ram[0x99c..0x99e].copy_from_slice(&0xe17du16.to_le_bytes());
    let bts = game.current_block_bts();
    let door_list = u16::from_le_bytes([game.g_ram[0x7b5], game.g_ram[0x7b6]]);
    let entry = door_list.wrapping_add(u16::from(bts & 0x7f) * 2);
    if entry & 0x8000 == 0 {
        RomPtr_8F(entry, game);
    }
    let door = game.read_rom_word(0x8f8000 | u32::from(entry), 0);
    if door & 0x8000 == 0 {
        get_DoorDef(door, game);
    }
    let destination = game.read_rom_word(0x838000 | u32::from(door), 0);
    if destination & 0x8000 == 0 {
        // Preserve the original pose comparison, even though its threshold is a state value.
        let pose = u16::from_le_bytes([game.g_ram[0xa1c], game.g_ram[0xa1d]]);
        if pose < 9 {
            game.g_ram[0xe16..0xe18].copy_from_slice(&1u16.to_le_bytes());
        }
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        game.g_ram[0x78d..0x78f].copy_from_slice(&door.to_le_bytes());
        game.g_ram[0x998..0x99a].copy_from_slice(&9u16.to_le_bytes());
        0
    }
}
fn BlockReact_HorizExt(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let offset = i16::from(game.current_block_bts() as i8);
    if offset == 0 {
        return 0;
    }
    game.set_cur_block_index(game.cur_block_index().wrapping_add(offset as u16));
    0xff
}
fn BlockReact_VertExt(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let offset = i16::from(game.current_block_bts() as i8);
    if offset == 0 {
        return 0;
    }
    let block_offset = (offset as u16).wrapping_mul(game.room_width_in_blocks());
    game.set_cur_block_index(game.cur_block_index().wrapping_add(block_offset));
    0xff
}
fn samus_block_span(position: u16, radius: u16) -> u16 {
    let first_block_edge = position.wrapping_sub(radius) & 0xfff0;
    radius
        .wrapping_add(position)
        .wrapping_sub(1)
        .wrapping_sub(first_block_edge)
        >> 4
}
fn Samus_GetYposSpan(game: &mut Game) -> uint16 {
    let position = game.samus_y_pos();
    let radius = game.samus_y_radius();
    samus_block_span(position, radius)
}
fn Samus_GetXposSpan(game: &mut Game) -> uint16 {
    let position = game.samus_x_pos();
    let radius = game.samus_x_radius();
    samus_block_span(position, radius)
}
unsafe fn BlockColl_Horiz_CheckColl(ci: &mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    game.set_cur_block_index(k >> 1);
    // Keep the raw context alive across retries and nested PLM setup calls.
    let context = ci as *mut CollInfo;
    let result = loop {
        let result = match game.current_level_block() >> 12 {
            0 | 4 | 6 => ClearCarry_4(context, game),
            1 => BlockColl_Horiz_Slope(context, game),
            2 => ClearCarry_5(context, game),
            3 => BlockColl_Horiz_SpecialAir(context, game),
            5 => BlockReact_HorizExt(context, game),
            7 => BlockColl_Horiz_BombableAir(context, game),
            8 | 12 | 14 => BlockColl_Horiz_SolidShootGrappleBlock(context, game),
            9 => BlockColl_Horiz_Door(context, game),
            10 => BlockColl_Horiz_SpikeBlock(context, game),
            11 => BlockColl_Horiz_SpecialBlock(context, game),
            13 => BlockReact_VertExt(context, game),
            15 => BlockColl_Horiz_BombBlock(context, game),
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            break result;
        }
    };
    result
}
unsafe fn BlockColl_Vert_CheckColl(ci: &mut CollInfo, k: uint16, game: &mut Game) -> uint8 {
    game.set_cur_block_index(k >> 1);
    // Keep the same raw context valid across every extension-block retry.
    let context = ci as *mut CollInfo;
    let result = loop {
        let result = match game.current_level_block() >> 12 {
            0 | 4 | 6 => ClearCarry_4(context, game),
            1 => BlockColl_Vert_Slope(context, game),
            2 => BlockColl_Vert_SpikeAir(context, game),
            3 => BlockColl_Vert_SpecialAir(context, game),
            5 => BlockReact_HorizExt(context, game),
            7 => BlockColl_Vert_BombableAir(context, game),
            8 | 12 | 14 => BlockColl_Vert_SolidShootGrappleBlock(context, game),
            9 => BlockColl_Vert_Door(context, game),
            10 => BlockColl_Vert_SpikeBlock(context, game),
            11 => BlockColl_Vert_SpecialBlock(context, game),
            13 => BlockReact_VertExt(context, game),
            15 => BlockColl_Vert_BombBlock(context, game),
            _ => unreachable!(),
        };
        if result & 0x80 == 0 {
            break result;
        }
    };
    result
}
unsafe fn BlockColl_Handle_Horiz(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    let span = Samus_GetYposSpan(game);
    let y = game.samus_y_pos();
    let y_radius = game.samus_y_radius();
    let row = (y.wrapping_sub(y_radius) >> 4) as u8;
    let width = game.room_width_in_blocks() as u8;
    let row_start = u16::from(row) * u16::from(width);
    let x = game.samus_x_pos();
    let fraction = game.samus_x_subpos();
    let target =
        (((u32::from(x) << 16) | u32::from(fraction)).wrapping_add(amt as u32) >> 16) as u16;
    let radius = game.samus_x_radius();
    let edge = if amt >= 0 {
        radius.wrapping_add(target).wrapping_sub(1)
    } else {
        target.wrapping_sub(radius)
    };
    let mut block_offset = row_start.wrapping_add(edge >> 4).wrapping_mul(2);
    let mut context = CollInfo {
        ci_r18_r20: amt,
        ci_r24: target,
        ci_r26: span,
        ci_r28: span,
        ci_r30: 0,
        ci_r32: edge,
        ci_r38: 0,
        ci_r40: 0,
    };
    while BlockColl_Horiz_CheckColl(&mut context, block_offset, game) & 1 == 0 {
        // Callbacks can change the room width and remaining row count.
        block_offset = block_offset.wrapping_add(game.room_width_in_blocks().wrapping_mul(2));
        context.ci_r26 = context.ci_r26.wrapping_sub(1);
        if context.ci_r26 & 0x8000 != 0 {
            return Pair_Bool_Amt {
                flag: false,
                amt: context.ci_r18_r20,
            };
        }
    }
    Pair_Bool_Amt {
        flag: true,
        amt: context.ci_r18_r20,
    }
}
unsafe fn BlockColl_Handle_Vert_LeftToRight(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    handle_vertical_collision(amt, false, game)
}
unsafe fn BlockColl_Handle_Vert_RightToLeft(amt: int32, game: &mut Game) -> Pair_Bool_Amt {
    handle_vertical_collision(amt, true, game)
}
unsafe fn handle_vertical_collision(
    amt: i32,
    right_to_left: bool,
    game: &mut Game,
) -> Pair_Bool_Amt {
    let span = Samus_GetXposSpan(game);
    let y = game.samus_y_pos();
    let fraction = game.samus_y_subpos();
    let target =
        (((u32::from(y) << 16) | u32::from(fraction)).wrapping_add(amt as u32) >> 16) as u16;
    let radius = game.samus_y_radius();
    let edge = if amt >= 0 {
        radius.wrapping_add(target).wrapping_sub(1)
    } else {
        target.wrapping_sub(radius)
    };
    let row = (edge >> 4) as u8;
    let width = game.room_width_in_blocks() as u8;
    let row_start = u16::from(row) * u16::from(width);
    let x = game.samus_x_pos();
    let radius = game.samus_x_radius();
    let column = if right_to_left {
        radius.wrapping_add(x).wrapping_sub(1) >> 4
    } else {
        x.wrapping_sub(radius) >> 4
    };
    game.set_cur_block_index(row_start.wrapping_add(column));
    let mut context = CollInfo {
        ci_r18_r20: amt,
        ci_r24: target,
        ci_r26: if right_to_left { 0 } else { span },
        ci_r28: span,
        ci_r30: 0,
        ci_r32: edge,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut block_offset = game.cur_block_index().wrapping_mul(2);
    while BlockColl_Vert_CheckColl(&mut context, block_offset, game) & 1 == 0 {
        if right_to_left {
            context.ci_r26 = context.ci_r26.wrapping_add(1);
            if context.ci_r28 < context.ci_r26 {
                return Pair_Bool_Amt {
                    flag: false,
                    amt: context.ci_r18_r20,
                };
            }
            block_offset = block_offset.wrapping_sub(2);
        } else {
            context.ci_r26 = context.ci_r26.wrapping_sub(1);
            if context.ci_r26 & 0x8000 != 0 {
                return Pair_Bool_Amt {
                    flag: false,
                    amt: context.ci_r18_r20,
                };
            }
            block_offset = block_offset.wrapping_add(2);
        }
    }
    Pair_Bool_Amt {
        flag: true,
        amt: context.ci_r18_r20,
    }
}
fn prepare_pose_collision(game: &mut Game) {
    let direction = game.samus_collision_direction();
    game.set_samus_collision_direction((direction | 15));
    game.set_flag_samus_in_quicksand(0);
}
fn finish_pose_collision(pair: Pair_Bool_Amt, game: &mut Game) -> i32 {
    game.set_samus_collision_flag(u16::from(pair.flag));
    pair.amt.wrapping_abs()
}
pub unsafe fn WallJumpBlockCollDetect(amt: int32, game: &mut Game) -> int32 {
    prepare_pose_collision(game);
    let pair = BlockColl_Handle_Horiz(amt, game);
    finish_pose_collision(pair, game)
}
unsafe fn CollDetectDueToPoseChange_SingleBlock(amt: int32, game: &mut Game) -> int32 {
    prepare_pose_collision(game);
    let frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
    let pair = if frame & 1 == 0 {
        BlockColl_Handle_Vert_LeftToRight(amt, game)
    } else {
        BlockColl_Handle_Vert_RightToLeft(amt, game)
    };
    finish_pose_collision(pair, game)
}
pub unsafe fn Samus_CollDetectChangedPose(amt: int32, game: &mut Game) -> int32 {
    let integer = (amt >> 16) as i16;
    if integer.wrapping_abs() as u16 & 0xfff8 == 0 {
        return CollDetectDueToPoseChange_SingleBlock(amt, game);
    }
    let probe = i32::from(((integer as u16 & 0xfff0) | 8) as i16) << 16;
    let adjusted = CollDetectDueToPoseChange_SingleBlock(probe, game);
    let collided = game.samus_collision_flag();
    if collided != 0 {
        return adjusted;
    }
    CollDetectDueToPoseChange_SingleBlock(amt, game)
}
fn add_samus_position(position: &mut [u8], amount: i32) {
    // RAM stores the integer word first, then the fractional word.
    let integer = u16::from_le_bytes([position[0], position[1]]);
    let fraction = u16::from_le_bytes([position[2], position[3]]);
    let result = ((u32::from(integer) << 16) | u32::from(fraction)).wrapping_add(amount as u32);
    position[2..4].copy_from_slice(&(result as u16).to_le_bytes());
    position[0..2].copy_from_slice(&((result >> 16) as u16).to_le_bytes());
}
pub unsafe fn Samus_MoveRight_NoSolidColl(mut amt: int32, game: &mut Game) -> int32 {
    let collided = if amt != 0 {
        game.set_flag_samus_in_quicksand(0);
        let pair = BlockColl_Handle_Horiz(amt, game);
        amt = pair.amt;
        pair.flag
    } else {
        false
    };
    game.set_samus_collision_flag(u16::from(collided));
    add_samus_position(&mut game.g_ram[0xaf6..0xafa], amt);
    amt
}
pub unsafe fn Samus_MoveDown_NoSolidColl(mut amt: int32, game: &mut Game) -> int32 {
    if amt != 0 {
        game.set_samus_pos_adjusted_by_slope_flag(0);
        game.set_flag_samus_in_quicksand(0);
        let frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
        let pair = if frame & 1 == 0 {
            BlockColl_Handle_Vert_LeftToRight(amt, game)
        } else {
            BlockColl_Handle_Vert_RightToLeft(amt, game)
        };
        amt = pair.amt;
        if pair.flag {
            game.set_samus_collision_flag(1);
            add_samus_position(&mut game.g_ram[0xafa..0xafe], amt);
            return amt;
        }
    }
    let slope_collision = game.flag_samus_in_quicksand();
    game.set_samus_collision_flag(u16::from(slope_collision != 0));
    add_samus_position(&mut game.g_ram[0xafa..0xafe], amt);
    amt
}
fn BlockInsideReact_Slope(_ci: *mut CollInfo, _game: &mut Game) -> uint8 {
    0
}
fn BlockInsideReact_ShootableAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockInsideReact_SpecialAir_Default(game);
    0
}
fn BlockInsideReact_SpikeAir_BTS2(game: &mut Game) {
    let contact_damage = u16::from_le_bytes([game.g_ram[0xa6e], game.g_ram[0xa6f]]);
    if contact_damage == 0 {
        let invincibility = u16::from_le_bytes([game.g_ram[0x18a8], game.g_ram[0x18a9]]);
        if invincibility == 0 {
            game.g_ram[0x18a8..0x18aa].copy_from_slice(&60u16.to_le_bytes());
            game.g_ram[0x18aa..0x18ac].copy_from_slice(&10u16.to_le_bytes());
            let damage = u16::from_le_bytes([game.g_ram[0xa50], game.g_ram[0xa51]]);
            game.g_ram[0xa50..0xa52].copy_from_slice(&damage.wrapping_add(16).to_le_bytes());
            let facing = u16::from_le_bytes([game.g_ram[0xa1e], game.g_ram[0xa1f]]);
            let knockback = u16::from((facing ^ 12) & 8 != 0);
            game.g_ram[0xa54..0xa56].copy_from_slice(&knockback.to_le_bytes());
        }
    }
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpikeAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    // Keep the former sixteen-entry table's rejection of invalid BTS values.
    assert!(bts < 16, "spike-air BTS index out of bounds: {bts}");
    if bts == 2 {
        BlockInsideReact_SpikeAir_BTS2(game);
    }
    0
}
fn BlockInsideReact_Special_(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    BlockInsideReact_SpecialAir_Default(game);
    0
}
fn BlockInsideReact_SpecialAir_Default(game: &mut Game) {
    let speed_table = (addr_kSamusSpeedTable_Normal_X as u16).wrapping_add(12);
    game.g_ram[0xa6c..0xa6e].copy_from_slice(&speed_table.to_le_bytes());
}
unsafe fn special_air_movement_enabled(game: &mut Game) -> bool {
    let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
    if area == 3 && CheckBossBitForCurArea(1, game) & 1 == 0 {
        return false;
    }
    let speed = u16::from_le_bytes([game.g_ram[0xb2e], game.g_ram[0xb2f]]);
    speed == 0
}
fn set_special_air_displacement(displacement: u16, game: &mut Game) {
    game.g_ram[0xb56..0xb58].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb58..0xb5a].copy_from_slice(&displacement.to_le_bytes());
}
unsafe fn BlockInsideReact_SpecialAir_8(game: &mut Game) {
    if special_air_movement_enabled(game) {
        set_special_air_displacement(2, game);
    }
    BlockInsideReact_SpecialAir_Default(game);
}
unsafe fn BlockInsideReact_SpecialAir_9(game: &mut Game) {
    if special_air_movement_enabled(game) {
        set_special_air_displacement(0xfffe, game);
    }
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpecialAir_10(game: &mut Game) {
    set_special_air_displacement(2, game);
    BlockInsideReact_SpecialAir_Default(game);
}
fn BlockInsideReact_SpecialAir_11(game: &mut Game) {
    set_special_air_displacement(0xfffe, game);
    BlockInsideReact_SpecialAir_Default(game);
}
unsafe fn BlockInsideReact_SpecialAir_70(game: &mut Game) {
    let sample = game.inside_block_reaction_samus_point();
    if sample == 1 {
        SpawnPLM(addr_kPlmHeader_B6FF as u16, game);
    }
}
const SPECIAL_AIR_PLM_TABLES: [u16; 8] = [
    0x9a06, 0x9a26, 0x9a46, 0x9a66, 0x9a86, 0x9aa6, 0x9ac6, 0x9ae6,
];
unsafe fn BlockInsideReact_SpecialAir(_ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let bts = game.current_block_bts();
    if bts & 0x80 != 0 {
        let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
        let table = SPECIAL_AIR_PLM_TABLES[usize::from(area)];
        let plm = game.read_rom_word(0x940000 | u32::from(table), usize::from(bts & 0x7f) * 2);
        SpawnPLM(plm, game);
    } else {
        // Preserve the former eighty-entry dispatch table's invalid-BTS panic.
        assert!(bts < 80, "special-air BTS index out of bounds: {bts}");
        match bts {
            8 => BlockInsideReact_SpecialAir_8(game),
            9 => BlockInsideReact_SpecialAir_9(game),
            10 => BlockInsideReact_SpecialAir_10(game),
            11 => BlockInsideReact_SpecialAir_11(game),
            70 => BlockInsideReact_SpecialAir_70(game),
            _ => BlockInsideReact_SpecialAir_Default(game),
        }
    }
    0
}
unsafe fn dispatch_block_inside(ci: *mut CollInfo, game: &mut Game) -> uint8 {
    match game.current_level_block() >> 12 {
        0 | 4 | 6 | 7 => BlockInsideReact_ShootableAir(ci, game),
        1 => BlockInsideReact_Slope(ci, game),
        2 => BlockInsideReact_SpikeAir(ci, game),
        3 => BlockInsideReact_SpecialAir(ci, game),
        5 => BlockReact_HorizExt(ci, game),
        13 => BlockReact_VertExt(ci, game),
        _ => BlockInsideReact_Special_(ci, game),
    }
}
unsafe fn react_inside_sample(x: u16, y: u16, game: &mut Game) {
    CalculateBlockAt(x, y, 0, 0, game);
    let mut context = CollInfo {
        ci_r18_r20: 0,
        ci_r24: 0,
        ci_r26: x,
        ci_r28: y,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    // Keep the same context alive across retries and callback-adjusted block indexes.
    while dispatch_block_inside(&raw mut context, game) & 0x80 != 0 {}
}
pub unsafe fn BlockInsideDetection(game: &mut Game) {
    game.g_ram[0xb4c..0xb4e].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb56..0xb58].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb58..0xb5a].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb5a..0xb5c].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb5c..0xb5e].copy_from_slice(&0u16.to_le_bytes());
    game.set_inside_block_reaction_samus_point(0);
    let mut r26 = game.samus_x_pos();
    let radius = game.samus_y_radius();
    let y = game.samus_y_pos();
    let samus_bottom_boundary_position = radius.wrapping_add(y).wrapping_sub(1);
    let mut r28 = samus_bottom_boundary_position;
    react_inside_sample(r26, r28, game);
    game.set_inside_block_reaction_samus_point(1);
    let y = game.samus_y_pos();
    if (samus_bottom_boundary_position ^ y) & 0xfff0 != 0 {
        r26 = game.samus_x_pos();
        r28 = y;
        react_inside_sample(r26, r28, game);
    }
    game.set_inside_block_reaction_samus_point(2);
    let y = game.samus_y_pos();
    let radius = game.samus_y_radius();
    let top = y.wrapping_sub(radius);
    let changed_row_bits = (samus_bottom_boundary_position ^ top) & 0xfff0;
    // Preserve the original XOR test; this is not a simple center/top comparison.
    if changed_row_bits != 0 && (y ^ changed_row_bits) & 0xfff0 != 0 {
        r26 = game.samus_x_pos();
        r28 = top;
        react_inside_sample(r26, r28, game);
    }
}
fn CalculateBlockAt(x: uint16, y: uint16, x_offset: uint16, y_offset: uint16, game: &mut Game) {
    let x = x.wrapping_add(x_offset);
    let y = y.wrapping_add(y_offset);
    // The original signed tests accept only wrapped coordinates in 0..4096.
    let block = if x < 4096 && y < 4096 {
        let width = u16::from(game.room_width_in_blocks() as u8);
        (x >> 4) + (y >> 4) * width
    } else {
        0xffff
    };
    game.set_cur_block_index(block);
}
static kBlockFunc_9C73_Tab0: [uint8; 36] = [
    3, 20, 0, 10, 40, 1, 5, 30, 0, 10, 40, 1, 20, 80, 0, 20, 80, 1, 20, 80, 0, 20, 80, 1, 30, 100,
    0, 30, 100, 1, 30, 100, 0, 30, 100, 1,
];
static kBlockFunc_9C73_Tab1: [int8; 18] = [
    0,
    0,
    10,
    0,
    50,
    0,
    50,
    3,
    0,
    -(1),
    10,
    2,
    0,
    -(1),
    0,
    -(1),
    0,
    -(1),
];
unsafe fn BlockFunc_9C73(mut v0: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int)
        != 0i32
    {
        return kBlockFunc_9C73_Tab1[(2 as ::core::ffi::c_int
            * (*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                .offset(v1 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
            + 1 as ::core::ffi::c_int) as usize] as uint16;
    } else {
        let mut r18: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xc18)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int;
        return kBlockFunc_9C73_Tab0
            [(3 as ::core::ffi::c_int * r18 + 2 as ::core::ffi::c_int) as usize]
            as uint16;
    };
}
pub unsafe fn BombOrPowerBomb_Func1(mut v0: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut temp_collision_DD2: uint16 = BlockFunc_9C73(v0, game);
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize)
        as int16;
    if v2 >= 0i16 {
        let mut r26: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize);
        if ((*(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16)) as i16),
            )
            < 0i16
        {
            v3 = *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize) as int16;
            if v3 >= 0i16 {
                let mut R28: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xb78)
                    as *mut uint16)
                    .offset(v1 as isize);
                if ((*(&raw mut v3 as *mut uint8).offset(
                    (::core::mem::size_of::<int16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                )) as i16)
                    .wrapping_sub(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16))
                            as i16),
                    )
                    < 0i16
                {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
                        0 as uint16;
                    CalculateBlockAt(r26, R28, 0 as uint16, 0 as uint16, game);
                    let mut ci: CollInfo = CollInfo {
                        ci_r18_r20: 0,
                        ci_r24: 0 as uint16,
                        ci_r26: r26,
                        ci_r28: R28,
                        ci_r30: 0,
                        ci_r32: 0,
                        ci_r38: 0,
                        ci_r40: 0,
                    };
                    if temp_collision_DD2 == 2u16 {
                        BlockColl_BombExplosion(&raw mut ci, v0, game);
                    } else {
                        BlockFunc_A06A(v0, game);
                    }
                }
            }
        }
    }
}
unsafe fn BlockColl_GetBlockIndexAbove(mut k: uint16, game: &mut Game) -> uint16 {
    return (k as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int)
        as uint16;
}
unsafe fn BlockColl_GetBlockIndexRight(mut k: uint16, game: &mut Game) -> uint16 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + k as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_GetBlockIndexLeft(mut k: uint16) -> uint16 {
    return ((k).wrapping_sub((4u16)));
}
unsafe fn BlockColl_GetBlockIndexBelow(mut k: uint16, game: &mut Game) -> uint16 {
    return (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
        as ::core::ffi::c_int
        * 2 as ::core::ffi::c_int
        + k as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockColl_BombExplosion(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16).offset(v1 as isize) == 0
    {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
            as int16;
        if (v2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                .offset(v1 as isize) = (((v2) as u16) | (1u16));
            if *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) != 0xffffu16 {
                let mut v3: uint16 = ((2u16).wrapping_mul(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16)),
                ));
                BlockBombedReact(
                    ci,
                    ((2u16).wrapping_mul(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16)),
                    )),
                    game,
                );
                let mut idx: uint16 = BlockColl_GetBlockIndexAbove(v3, game);
                BlockBombedReact(ci, idx, game);
                let mut idx2: uint16 = BlockColl_GetBlockIndexRight(idx, game);
                BlockBombedReact(ci, idx2, game);
                idx = BlockColl_GetBlockIndexLeft(idx2);
                BlockBombedReact(ci, idx, game);
                idx = BlockColl_GetBlockIndexBelow(idx, game);
                BlockBombedReact(ci, idx, game);
            }
        }
    }
}
fn ClearCarry_9(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
fn SetCarry_0(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn sub_949D5D(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5i32
    {
        return 1 as uint8;
    } else {
        return BlockShotReactVert_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockBombedReact_Special(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize);
    if (v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32 {
        let mut kBlockBombedReact_Region_Plm: *mut uint16 =
            RomPtr_94(0x9e44 as uint16_t, game) as *mut uint16;
        let mut v2: *const uint16 = RomPtr_94(
            *kBlockBombedReact_Region_Plm.offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
            ),
            game,
        ) as *mut uint16;
        SpawnPLM(
            *v2.offset((v0 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as isize),
            game,
        );
    } else {
        let mut kBlockBombedReact_Special_Plm: *mut uint16 =
            RomPtr_94(0x9da4 as uint16_t, game) as *mut uint16;
        SpawnPLM(*kBlockBombedReact_Special_Plm.offset(v0 as isize), game);
    }
    return 1 as uint8;
}
unsafe fn BlockReact_ShootableAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize);
    if (v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32 {
        let mut t: uint16 =
            *(RomPtr_94(0x9ea6 as uint16_t, game) as *mut uint16).offset(v0 as isize);
        if t == 0xb974u16 {
            (*ci).ci_r38 = 0 as uint16;
            (*ci).ci_r40 = 0xffff as uint16;
        }
        SpawnPLM(t, game);
    }
    return 0 as uint8;
}
unsafe fn BlockReact_Shootable(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize);
    if (v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) != 0i32 {
        let mut kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm: *mut uint16 =
            RomPtr_94(0x9fc6 as uint16_t, game) as *mut uint16;
        let mut v2: *mut uint16 = RomPtr_94(
            *kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm.offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize,
            ),
            game,
        ) as *mut uint16;
        SpawnPLM(
            *v2.offset((v0 as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as isize),
            game,
        );
    } else {
        let mut t: uint16 =
            *(RomPtr_94(0x9ea6 as uint16_t, game) as *mut uint16).offset(v0 as isize);
        if t == 0xb974u16 {
            (*ci).ci_r38 = 0 as uint16;
            (*ci).ci_r40 = 0xffff as uint16;
        }
        SpawnPLM(t, game);
    }
    return 1 as uint8;
}
unsafe fn BlockReact_BombableAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut kBlockShotBombGrappleReact_BombableBlockPlm: *mut uint16 =
        RomPtr_94(0xa012 as uint16_t, game) as *mut uint16;
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize);
    if (v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32 {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    return 0 as uint8;
}
unsafe fn BlockReact_BombableBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut kBlockShotBombGrappleReact_BombableBlockPlm: *mut uint16 =
        RomPtr_94(0xa012 as uint16_t, game) as *mut uint16;
    let mut v0: uint8 = *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize);
    if (v0 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int) == 0i32 {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    return 1 as uint8;
}
static kBlockBombedReact: [Option<Func_CollInfo_U8>; 16] = unsafe {
    [
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockBombedReact_Special as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
    ]
};
unsafe fn BlockBombedReact(mut ci: *mut CollInfo, mut v0: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        (v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    while kBlockBombedReact[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int)
        >> 12 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(ci, game) as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {}
}
unsafe fn BlockFunc_A06A(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v3: int16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16).offset(v1 as isize)
        as int16;
    if v2 != 0 {
        if v2 < 0i16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16)
                .offset(v1 as isize) = 0 as uint16;
        }
    } else {
        let mut r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as uint16;
        let mut r20: uint16 = (((3u16).wrapping_mul(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                )) as u16),
        )) as ::core::ffi::c_int
            >> 2 as ::core::ffi::c_int) as uint16;
        v3 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0xcea) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    )) as i16),
            ));
        if v3 < 0i16 {
            v3 = 0 as int16;
        }
        let mut r22: uint16 =
            (v3 as uint16 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
        let mut v4: uint16 = (((r18)
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16))))
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        if v4 >= *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) {
            v4 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16))
                .wrapping_sub((1u16)));
        }
        let mut r24: uint16 = v4;
        let mut v5: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xce4)
            as *mut uint16))
            .wrapping_sub((r20)));
        if (v5 as int16) < 0i16 {
            v5 = 0 as uint16;
        }
        let mut r26: uint16 = (v5 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16;
        let mut v6: uint16 = (((r20)
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16))))
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint16;
        if v6 >= *((&raw mut game.g_ram as *mut uint8).offset(0x7a7) as *mut uint16) {
            v6 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a7) as *mut uint16))
                .wrapping_sub((1u16)));
        }
        let mut R28: uint16 = v6;
        let mut a: uint16 = (r26 as ::core::ffi::c_int
            | (r22 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint16;
        BlockFunc_A0F4(a, r22, r24, r26, R28, game);
        BlockFunc_A11A(a, r24, r26, R28, game);
        BlockFunc_A0F4(
            (R28 as ::core::ffi::c_int | (r22 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16,
            r22,
            r24,
            r26,
            R28,
            game,
        );
        BlockFunc_A11A(
            (r26 as ::core::ffi::c_int | (r24 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16,
            r24,
            r26,
            R28,
            game,
        );
    };
}
unsafe fn BlockFunc_A0F4(
    mut a: uint16,
    mut r22: uint16,
    mut r24: uint16,
    mut r26: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut RegWord: uint16 = Mult8x8(
        a as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * (RegWord as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int)) as uint16;
    let mut v4: int16 = (((r24) as i16).wrapping_sub(((r22) as i16)));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: r24,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        BlockBombedReact(&raw mut ci, v3, game);
        v3 = ((v3).wrapping_add((2u16)));
        v4 -= 1;
        if !(v4 >= 0i16) {
            break;
        }
    }
}
unsafe fn BlockFunc_A11A(
    mut a: uint16,
    mut r24: uint16,
    mut r26: uint16,
    mut r28: uint16,
    game: &mut Game,
) {
    let mut RegWord: uint16 = Mult8x8(
        a as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    let mut v3: uint16 = (2 as ::core::ffi::c_int
        * (RegWord as ::core::ffi::c_int
            + *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int)) as uint16;
    let mut v4: int16 = (((r28) as i16).wrapping_sub(((r26) as i16)));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: r24,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        BlockBombedReact(&raw mut ci, v3, game);
        v3 = (v3 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        v4 -= 1;
        if !(v4 >= 0i16) {
            break;
        }
    }
}
unsafe fn BlockShotReactHoriz_Slope(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5i32
    {
        return BlockShotReactVert_Slope_Square(
            ci,
            (((*(&raw mut game.g_ram as *mut uint8).offset(0x16402).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize,
            )) as u16)
                & (0x1fu16)),
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16),
            game,
        );
    } else {
        return BlockShotReactVert_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockShotReactVert_Slope(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        < 5i32
    {
        return BlockShotReactHoriz_Slope_Square(
            ci,
            (((*(&raw mut game.g_ram as *mut uint8).offset(0x16402).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize,
            )) as u16)
                & (0x1fu16)),
            *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16),
            game,
        );
    } else {
        return BlockShotReactHoriz_Slope_NonSquare(ci, game);
    };
}
unsafe fn BlockShotReactHoriz(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockShotReactHoriz: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockShotReactHoriz_Slope as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: uint8 = 0;
    if k >= *((&raw mut game.g_ram as *mut uint8).offset(0x7b9) as *mut uint16) {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        v1 = kBlockShotReactHoriz[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(ci, game);
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    if v1 != 0 {
        (*ci).ci_r40 = (*ci).ci_r40.wrapping_sub(1);
    }
    return v1;
}
unsafe fn BlockShotReactVert(mut ci: *mut CollInfo, mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockShotReactVert: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockShotReactVert_Slope as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: uint8 = 0;
    if k >= *((&raw mut game.g_ram as *mut uint8).offset(0x7b9) as *mut uint16) {
        return 1 as uint8;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        (k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    loop {
        v1 = kBlockShotReactVert[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(ci, game);
        if !(v1 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    if v1 != 0 {
        (*ci).ci_r40 = (*ci).ci_r40.wrapping_sub(1);
    }
    return v1;
}
unsafe fn BlockGetSomePos1(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r38: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    return ((*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r38 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
}
unsafe fn BlockGetSomePos2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r38: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
        & 0xfff0 as ::core::ffi::c_int) as uint16;
    return ((*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r38 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BlockCollNoWaveBeamHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbdc) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos1(k, game);
    let mut prod: uint16 = Mult8x8(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize))
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                    .offset(v1 as isize)),
            )) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize);
    if amt < 0i32 {
        v7 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
            .offset(v1 as isize))
        .wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                .offset(v1 as isize)),
        ));
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16)) as i16),
            )
            >= 0i16
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactHoriz(&raw mut ci, v8, game);
        v8 = (v8 as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int) as uint16;
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if !((ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    if (ci.ci_r40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollNoWaveBeamVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbf0) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize);
    if amt < 0i32 {
        v7 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
            .offset(v1 as isize))
        .wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                .offset(v1 as isize)),
        ));
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize))
    .wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(v1 as isize)),
    )) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v9: uint16 = (2 as ::core::ffi::c_int
        * (Mult8x8(
            (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
            *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
        ) as ::core::ffi::c_int
            + v8 as ::core::ffi::c_int)) as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        || ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as i16),
            )
            >= 0i16
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactVert(&raw mut ci, v9, game);
        v9 = ((v9).wrapping_add((2u16)));
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if !((ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    if (ci.ci_r40 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollWaveBeamHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbdc) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos1(k, game);
    let mut prod: uint16 = Mult8x8(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize))
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                    .offset(v1 as isize)),
            )) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize);
    if amt < 0i32 {
        v7 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
            .offset(v1 as isize))
        .wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                .offset(v1 as isize)),
        ));
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int)))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        return 0 as uint8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    let mut v9: uint8 = *(((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
    if v9 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
        && ((v9) as i16).wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as i16),
        ) < 0i16
        && ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16)) as i16),
            )
            < 0i16
    {
        loop {
            BlockShotReactHoriz(&raw mut ci, v8, game);
            v8 = (v8 as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16)
                    as ::core::ffi::c_int
                    * 2 as ::core::ffi::c_int) as uint16;
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if !((ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32) {
                break;
            }
        }
    }
    return 0 as uint8;
}
pub unsafe fn BlockCollWaveBeamVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v7: uint16 = 0;
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbf0) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut some_pos: uint16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize);
    if amt < 0i32 {
        v7 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
            .offset(v1 as isize))
        .wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                .offset(v1 as isize)),
        ));
    } else {
        v7 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as uint16;
    }
    let mut R28: uint16 = v7;
    let mut prod: uint16 = Mult8x8(
        (v7 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    let mut v8: uint16 = (((*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize))
    .wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(v1 as isize)),
    )) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v9: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v8 as ::core::ffi::c_int))
        as uint16;
    if some_pos as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
    {
        return 0 as uint8;
    }
    let mut v10: uint8 = *(((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize) as *mut uint16 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as uint16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    if v10 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0
        && ((v10) as i16).wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16)) as i16),
        ) < 0i16
        && ((*(&raw mut R28 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i16)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as i16),
            )
            < 0i16
    {
        loop {
            BlockShotReactVert(&raw mut ci, v9, game);
            v9 = ((v9).wrapping_add((2u16)));
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if !((ci.ci_r38 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32) {
                break;
            }
        }
    }
    return 0 as uint8;
}
pub unsafe fn BlockCollMissileHoriz(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbdc) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xb8c) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut v3: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb8c) as *mut uint16).offset(v1 as isize);
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize);
    let mut r28: uint16 = r24;
    let mut v7: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                >> 4 as ::core::ffi::c_int))) as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as uint16,
        ci_r28: r28,
        ci_r30: 1 as uint16,
        ci_r32: 0,
        ci_r38: 0 as uint16,
        ci_r40: 0,
    };
    if ((*(&raw mut r24 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    )) as i16)
        .wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7a9) as *mut uint16)) as i16),
        )
        >= 0i16
        || BlockShotReactHoriz(&raw mut ci, v7, game) == 0
    {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
pub unsafe fn BlockCollMissileVert(mut k: uint16, game: &mut Game) -> uint8 {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut amt: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xbf0) as *mut uint16)
        .offset(v1 as isize) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        ((&raw mut game.g_ram as *mut uint8).offset(0xba0) as *mut uint16).offset(v1 as isize)
            as *mut uint16,
        amt as uint32,
    );
    let mut r24: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize);
    let mut r28: uint16 = r24;
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize)
            as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    let mut v7: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint16;
    let mut v8: uint16 = (2 as ::core::ffi::c_int
        * (prod as ::core::ffi::c_int + v7 as ::core::ffi::c_int))
        as uint16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as uint16,
        ci_r28: r28,
        ci_r30: 1 as uint16,
        ci_r32: 0,
        ci_r38: 0 as uint16,
        ci_r40: 0,
    };
    if ((*(&raw mut r24 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    )) as i16)
        .wrapping_sub(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x7ab) as *mut uint16)) as i16),
        )
        >= 0i16
        || BlockShotReactVert(&raw mut ci, v8, game) == 0
    {
        return 0 as uint8;
    }
    KillProjectile(k, game);
    return 1 as uint8;
}
unsafe fn BlockShotReact_Slope_NonSquare(
    mut ci: *mut CollInfo,
    mut j: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        ^ (if (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x40 as ::core::ffi::c_int)
            != 0i32
        {
            0xf as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as uint16;
    let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int
        ^ (if (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int)
            != 0i32
        {
            0xf as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as uint16;
    let mut v5: uint16 = (((kAlignYPos_Tab0[(16 as ::core::ffi::c_int
        * (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(k as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
        + (v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int))
        as usize]) as u16)
        & (0x1fu16));
    if ((v5) as i16).wrapping_sub(((v4) as i16)) <= 0i16 {
        (*ci).ci_r38 = 0 as uint16;
        (*ci).ci_r40 = 0 as uint16;
        return 1 as uint8;
    } else {
        return 0 as uint8;
    };
}
unsafe fn BlockShotReactVert_Slope_NonSquare(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16);
    let mut div: uint16 = SnesDivide(
        *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == div as ::core::ffi::c_int
    {
        return BlockShotReact_Slope_NonSquare(ci, v1, v0, game);
    } else {
        return 0 as uint8;
    };
}
unsafe fn BlockShotReactHoriz_Slope_NonSquare(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16);
    let mut mod_0: uint16 = SnesModulus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int
        == mod_0 as ::core::ffi::c_int
    {
        return BlockShotReact_Slope_NonSquare(ci, v1, v0, game);
    } else {
        return 0 as uint8;
    };
}
pub unsafe fn BlockCollSpreadBomb(mut k: uint16, game: &mut Game) -> uint8 {
    static kBlockCollSpreadBomb: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(sub_949D5D as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_9 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_0 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r26: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v1 as isize);
    let mut r28: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v1 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) = 0 as uint16;
    CalculateBlockAt(r26, r28, 0 as uint16, 0 as uint16, game);
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: 0 as uint16,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0 as uint16,
        ci_r32: 0 as uint16,
        ci_r38: 0,
        ci_r40: 0,
    };
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc7c) as *mut uint16).offset(v1 as isize) == 0
    {
        BlockColl_BombExplosion(&raw mut ci, k, game);
        return 0 as uint8;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) == 0xffffu16 {
        return 1 as uint8;
    }
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockCollSpreadBomb[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
unsafe fn BlockShotReactVert_Slope_Square(
    mut ci: *mut CollInfo,
    mut a: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut temp_collision_DD4: uint16 = ((4u16).wrapping_mul((a)));
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*ci).ci_r28 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 3 as ::core::ffi::c_int)) as uint16;
    let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xdde)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                != 0i32
                || kTab948E54[v2 as usize] == 0
            {
                let mut v4: uint16 = ((v2) ^ (2u16));
                if (*(((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16 as *mut uint8)
                    .offset(0) as ::core::ffi::c_int
                    + *(((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int)
                    == 0i32
                    || kTab948E54[v4 as usize] == 0
                {
                    return 0 as uint8;
                }
            }
            return 1 as uint8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 == (*ci).ci_r26
                && (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v3 as isize) as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int)
                    != 0i32
            {
                return (kTab948E54[(v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize]
                    != 0u8) as uint8;
            }
        } else if (*(((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
            .offset(v3 as isize) as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v3 as isize) as *mut uint16 as *mut uint8)
                .offset(0) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int)
            == 0i32
        {
            return (kTab948E54[v2 as usize] != 0u8) as uint8;
        }
        if kTab948E54[v2 as usize] != 0u8 {
            return 1 as uint8;
        }
        return (kTab948E54[(v2 as ::core::ffi::c_int ^ 2 as ::core::ffi::c_int) as usize] != 0u8)
            as uint8;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v3 as isize)
        as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int)
        != 0i32
    {
        v2 = ((v2) ^ (2u16));
    }
    return (kTab948E54[v2 as usize] != 0u8) as uint8;
}
unsafe fn BlockShotReactHoriz_Slope_Square(
    mut ci: *mut CollInfo,
    mut a: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint8 {
    let mut temp_collision_DD4: uint16 = ((4u16).wrapping_mul((a)));
    let mut temp_collision_DD6: uint16 = (*(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(k as isize) as ::core::ffi::c_int
        >> 6 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = (4 as ::core::ffi::c_int * a as ::core::ffi::c_int
        + (temp_collision_DD6 as ::core::ffi::c_int
            ^ ((*ci).ci_r28 as ::core::ffi::c_int & 8 as ::core::ffi::c_int)
                >> 2 as ::core::ffi::c_int)) as uint16;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            let mut v3: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xdde)
                as *mut uint16) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            if (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v3 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                & 8 as ::core::ffi::c_int)
                != 0i32
                || kTab948E54[v2 as usize] == 0u8
            {
                let mut v4: uint16 = ((v2) ^ (1u16));
                if (*(((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v3 as isize) as *mut uint16 as *mut uint8)
                    .offset(0) as ::core::ffi::c_int
                    + *(((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                        .offset(v3 as isize) as *mut uint16 as *mut uint8)
                        .offset(0) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int)
                    == 0i32
                    || kTab948E54[v4 as usize] == 0u8
                {
                    return 0 as uint8;
                }
            }
            return 1 as uint8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 == (*ci).ci_r26
                && (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16)
                            as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                    & 8 as ::core::ffi::c_int)
                    != 0i32
            {
                return (kTab948E54[(v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize]
                    != 0u8) as uint8;
            }
        } else if (*(((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as *mut uint16 as *mut uint8)
            .offset(0) as ::core::ffi::c_int
            + *(((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as *mut uint16 as *mut uint8)
                .offset(0) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
            & 8 as ::core::ffi::c_int)
            == 0i32
        {
            return (kTab948E54[v2 as usize] != 0u8) as uint8;
        }
        if kTab948E54[v2 as usize] != 0u8 {
            return 1 as uint8;
        }
        return (kTab948E54[(v2 as ::core::ffi::c_int ^ 1 as ::core::ffi::c_int) as usize] != 0u8)
            as uint8;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xdde) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 8 as ::core::ffi::c_int)
        != 0i32
    {
        v2 = ((v2) ^ (1u16));
    }
    return (kTab948E54[v2 as usize] != 0u8) as uint8;
}
fn ClearCarryZero(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
fn SetCarryClearOvf(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn BlockReactGrapple_GrappleBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    static kBlockReactGrapple_GrappleBlockPlm: [uint16; 4] = [0xd0d8, 0xd0dc, 0xd0e0, 0xd0d8];
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16);
    *fresh0 = (*fresh0 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    } else {
        return SpawnPLM(
            kBlockReactGrapple_GrappleBlockPlm[*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize,
                ) as usize],
            game,
        );
    };
}
unsafe fn BlockReactGrapple_SpikeBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    static kBlockReactGrapple_SpikeBlockPlm: [uint16; 16] = [
        0xd0e4, 0xd0e4, 0xd0e4, 0xd0e8, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
        0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
    ];
    if *(&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        return 0 as uint8;
    } else {
        return SpawnPLM(
            kBlockReactGrapple_SpikeBlockPlm[*(&raw mut game.g_ram as *mut uint8)
                .offset(0x16402)
                .offset(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize,
                ) as usize],
            game,
        );
    };
}
pub unsafe fn BlockCollGrappleBeam(game: &mut Game) -> uint8 {
    let mut grapple_vel_x: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd22)
        as *mut uint16) as int16 as int32)
        << 6 as ::core::ffi::c_int;
    let mut grapple_vel_y: int32 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd24)
        as *mut uint16) as int16 as int32)
        << 6 as ::core::ffi::c_int;
    let mut result: uint8 = 0 as uint8;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4i32 {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd10) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd0e) as *mut uint16,
            grapple_vel_x as uint32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd14) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd12) as *mut uint16,
            grapple_vel_y as uint32,
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd06) as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xd10) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xd0e) as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16)
                            as uint32,
                )
                .wrapping_add(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xd02) as *mut uint16)
                        as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int) as uint32,
                ),
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xd0a) as *mut uint16,
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xd14) as *mut uint16) as uint32)
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xd12) as *mut uint16) as uint32)
                .wrapping_add(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as uint32)
                        << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16)
                            as uint32,
                )
                .wrapping_add(
                    ((*((&raw mut game.g_ram as *mut uint8).offset(0xd04) as *mut uint16)
                        as ::core::ffi::c_int)
                        << 16 as ::core::ffi::c_int) as uint32,
                ),
        );
        result = BlockReactGrapple(game);
        if (result as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int) != 0i32
            && (result as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xfff0 as ::core::ffi::c_int
                    | 8 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) =
                (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xfff0 as ::core::ffi::c_int
                    | 8 as ::core::ffi::c_int) as uint16;
            return result;
        }
        i += 1;
    }
    return result;
}
pub unsafe fn BlockReactGrapple(game: &mut Game) -> uint8 {
    let mut v0: int16 = 0;
    static kBlockReactGrapple: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_ShootableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarryZero as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReactGrapple_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarryClearOvf as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_Shootable as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReactGrapple_GrappleBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_BombableBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut prod: uint16 = Mult8x8(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    v0 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as int16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        ((prod).wrapping_add(((v0) as u16)));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockReactGrapple[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
pub static kSinCosTable8bit_Sext: [int16; 320] = [
    -(256 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
    0 as int16,
    6 as int16,
    12 as int16,
    18 as int16,
    25 as int16,
    31 as int16,
    37 as int16,
    43 as int16,
    49 as int16,
    56 as int16,
    62 as int16,
    68 as int16,
    74 as int16,
    80 as int16,
    86 as int16,
    92 as int16,
    97 as int16,
    103 as int16,
    109 as int16,
    115 as int16,
    120 as int16,
    126 as int16,
    131 as int16,
    136 as int16,
    142 as int16,
    147 as int16,
    152 as int16,
    157 as int16,
    162 as int16,
    167 as int16,
    171 as int16,
    176 as int16,
    181 as int16,
    185 as int16,
    189 as int16,
    193 as int16,
    197 as int16,
    201 as int16,
    205 as int16,
    209 as int16,
    212 as int16,
    216 as int16,
    219 as int16,
    222 as int16,
    225 as int16,
    228 as int16,
    231 as int16,
    234 as int16,
    236 as int16,
    238 as int16,
    241 as int16,
    243 as int16,
    244 as int16,
    246 as int16,
    248 as int16,
    249 as int16,
    251 as int16,
    252 as int16,
    253 as int16,
    254 as int16,
    254 as int16,
    255 as int16,
    255 as int16,
    255 as int16,
    256 as int16,
    255 as int16,
    255 as int16,
    255 as int16,
    254 as int16,
    254 as int16,
    253 as int16,
    252 as int16,
    251 as int16,
    249 as int16,
    248 as int16,
    246 as int16,
    244 as int16,
    243 as int16,
    241 as int16,
    238 as int16,
    236 as int16,
    234 as int16,
    231 as int16,
    228 as int16,
    225 as int16,
    222 as int16,
    219 as int16,
    216 as int16,
    212 as int16,
    209 as int16,
    205 as int16,
    201 as int16,
    197 as int16,
    193 as int16,
    189 as int16,
    185 as int16,
    181 as int16,
    176 as int16,
    171 as int16,
    167 as int16,
    162 as int16,
    157 as int16,
    152 as int16,
    147 as int16,
    142 as int16,
    136 as int16,
    131 as int16,
    126 as int16,
    120 as int16,
    115 as int16,
    109 as int16,
    103 as int16,
    97 as int16,
    92 as int16,
    86 as int16,
    80 as int16,
    74 as int16,
    68 as int16,
    62 as int16,
    56 as int16,
    49 as int16,
    43 as int16,
    37 as int16,
    31 as int16,
    25 as int16,
    18 as int16,
    12 as int16,
    6 as int16,
    0 as int16,
    -(6 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(256 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(255 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(254 as ::core::ffi::c_int) as int16,
    -(253 as ::core::ffi::c_int) as int16,
    -(252 as ::core::ffi::c_int) as int16,
    -(251 as ::core::ffi::c_int) as int16,
    -(249 as ::core::ffi::c_int) as int16,
    -(248 as ::core::ffi::c_int) as int16,
    -(246 as ::core::ffi::c_int) as int16,
    -(244 as ::core::ffi::c_int) as int16,
    -(243 as ::core::ffi::c_int) as int16,
    -(241 as ::core::ffi::c_int) as int16,
    -(238 as ::core::ffi::c_int) as int16,
    -(236 as ::core::ffi::c_int) as int16,
    -(234 as ::core::ffi::c_int) as int16,
    -(231 as ::core::ffi::c_int) as int16,
    -(228 as ::core::ffi::c_int) as int16,
    -(225 as ::core::ffi::c_int) as int16,
    -(222 as ::core::ffi::c_int) as int16,
    -(219 as ::core::ffi::c_int) as int16,
    -(216 as ::core::ffi::c_int) as int16,
    -(212 as ::core::ffi::c_int) as int16,
    -(209 as ::core::ffi::c_int) as int16,
    -(205 as ::core::ffi::c_int) as int16,
    -(201 as ::core::ffi::c_int) as int16,
    -(197 as ::core::ffi::c_int) as int16,
    -(193 as ::core::ffi::c_int) as int16,
    -(189 as ::core::ffi::c_int) as int16,
    -(185 as ::core::ffi::c_int) as int16,
    -(181 as ::core::ffi::c_int) as int16,
    -(176 as ::core::ffi::c_int) as int16,
    -(171 as ::core::ffi::c_int) as int16,
    -(167 as ::core::ffi::c_int) as int16,
    -(162 as ::core::ffi::c_int) as int16,
    -(157 as ::core::ffi::c_int) as int16,
    -(152 as ::core::ffi::c_int) as int16,
    -(147 as ::core::ffi::c_int) as int16,
    -(142 as ::core::ffi::c_int) as int16,
    -(136 as ::core::ffi::c_int) as int16,
    -(131 as ::core::ffi::c_int) as int16,
    -(126 as ::core::ffi::c_int) as int16,
    -(120 as ::core::ffi::c_int) as int16,
    -(115 as ::core::ffi::c_int) as int16,
    -(109 as ::core::ffi::c_int) as int16,
    -(103 as ::core::ffi::c_int) as int16,
    -(97 as ::core::ffi::c_int) as int16,
    -(92 as ::core::ffi::c_int) as int16,
    -(86 as ::core::ffi::c_int) as int16,
    -(80 as ::core::ffi::c_int) as int16,
    -(74 as ::core::ffi::c_int) as int16,
    -(68 as ::core::ffi::c_int) as int16,
    -(62 as ::core::ffi::c_int) as int16,
    -(56 as ::core::ffi::c_int) as int16,
    -(49 as ::core::ffi::c_int) as int16,
    -(43 as ::core::ffi::c_int) as int16,
    -(37 as ::core::ffi::c_int) as int16,
    -(31 as ::core::ffi::c_int) as int16,
    -(25 as ::core::ffi::c_int) as int16,
    -(18 as ::core::ffi::c_int) as int16,
    -(12 as ::core::ffi::c_int) as int16,
    -(6 as ::core::ffi::c_int) as int16,
];
unsafe fn BlockFunc_A957(mut tmpD82: uint16, mut tmpD84: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut v7: int16 = 0;
    let mut v0: uint16 = tmpD82;
    let mut v2: uint16 = 0;
    let mut v3: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        let mut v1: ::core::ffi::c_int = tmpD82 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 7 as ::core::ffi::c_int) as uint16;
        } else {
            v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16) = v2;
        if (kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 7 as ::core::ffi::c_int) as uint16;
        } else {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                as ::core::ffi::c_int
                & 0xfff0 as ::core::ffi::c_int
                | 8 as ::core::ffi::c_int) as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16) = v3;
    }
    let mut v4: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    v5 = kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize];
    let mut v6: uint16 = 0;
    let mut v8: uint16 = 0;
    if v5 < 0i16 {
        if v5 == -256i16 {
            v6 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16))
                .wrapping_sub((tmpD84)));
        } else {
            let mut prod: uint16 = Mult8x8(
                tmpD84 as uint8,
                -(v5 as int8 as ::core::ffi::c_int) as uint8,
            );
            v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
                as ::core::ffi::c_int
                - (prod as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
                as uint16;
        }
    } else if v5 == 256i16 {
        v6 = ((tmpD84)
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16))));
    } else {
        let mut prod_0: uint16 = Mult8x8(
            tmpD84 as uint8,
            kSinCosTable8bit_Sext[(v4 + 64 as ::core::ffi::c_int) as usize] as uint8,
        );
        v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
            as ::core::ffi::c_int
            + (prod_0 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd90) as *mut uint16) = v6;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd94) as *mut uint16) =
        (v6 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8 as uint16;
    v7 = kSinCosTable8bit_Sext[v4 as usize];
    if v7 < 0i16 {
        if v7 == -256i16 {
            v8 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16))
                .wrapping_sub((tmpD84)));
        } else {
            let mut prod_1: uint16 = Mult8x8(tmpD84 as uint8, -(v7 as ::core::ffi::c_int) as uint8);
            v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
                as ::core::ffi::c_int
                - (prod_1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
                as uint16;
        }
    } else if v7 == 256i16 {
        v8 = ((tmpD84)
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16))));
    } else {
        let mut prod_2: uint16 =
            Mult8x8(tmpD84 as uint8, kSinCosTable8bit_Sext[v4 as usize] as uint8);
        v8 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
            as ::core::ffi::c_int
            + (prod_2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)) as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xd92) as *mut uint16) = v8;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd96) as *mut uint16) =
        (v8 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8 as uint16;
}
fn ClearCarry_10(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 0 as uint8;
}
fn SetCarry_3(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    return 1 as uint8;
}
unsafe fn BlockReact_AA64(game: &mut Game) -> uint8 {
    static kBlockReactLut_AB90_0: [Option<Func_CollInfo_U8>; 16] = unsafe {
        [
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_AA64_SpikeAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_AA64_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
            Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        ]
    };
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd94) as *mut uint16);
    let mut RegWord: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xd96) as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        ((RegWord).wrapping_add((v0)));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    let mut rv: uint8 = 0;
    loop {
        rv = kBlockReactLut_AB90_0[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
static g_word_94AAF7: [uint16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
unsafe fn BlockReact_AA64_SpikeAir(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) != 0 {
        return 0 as uint8;
    }
    let mut v0: int16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as int16;
    if v0 >= 0i16 {
        let mut v: uint32 = ((g_word_94AAF7[v0 as usize] as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xa4e) as *mut uint16,
                v,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 60 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 10 as uint16;
        }
    }
    return 0 as uint8;
}
static g_word_94AB70: [uint16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
unsafe fn BlockReact_AA64_SpikeBlock(mut ci: *mut CollInfo, game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) != 0 {
        return 1 as uint8;
    }
    let mut v0: int16 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0x16402)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as int16;
    if v0 >= 0i16 {
        let mut v: uint32 = ((g_word_94AB70[v0 as usize] as int16 as ::core::ffi::c_int)
            << 16 as ::core::ffi::c_int) as uint32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16,
                (&raw mut game.g_ram as *mut uint8).offset(0xa4e) as *mut uint16,
                v,
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 60 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 10 as uint16;
        }
    }
    return 1 as uint8;
}
static kBlockReactLut_AB90: [Option<Func_CollInfo_U8>; 16] = unsafe {
    [
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_AA64_SpikeAir as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_HorizExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(ClearCarry_10 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_AA64_SpikeBlock as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(BlockReact_VertExt as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
        Some(SetCarry_3 as unsafe fn(*mut CollInfo, &mut Game) -> uint8),
    ]
};
unsafe fn BlockFunc_ABB0(game: &mut Game) -> uint8 {
    let mut rv: uint8 = 0;
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xd94) as *mut uint16);
    let mut RegWord: uint16 = Mult8x8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xd96) as *mut uint16) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) =
        ((RegWord).wrapping_add((v0)));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as int32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    loop {
        rv = kBlockReactLut_AB90[((*((&raw mut game.g_ram as *mut uint8).offset(0x10002)
            as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16) as isize)
            as ::core::ffi::c_int
            & 0xf000 as ::core::ffi::c_int)
            >> 12 as ::core::ffi::c_int) as usize]
            .expect("non-null function pointer")(&raw mut ci, game);
        if !(rv as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0) {
            break;
        }
    }
    return rv;
}
unsafe fn BlockFunc_ABE6(mut tmpD82: uint16, game: &mut Game) -> uint8 {
    let mut v0: uint8 = 0;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16) = 6 as uint16;
    let mut grapple_beam_tmpD84: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xcfe)
        as *mut uint16))
        .wrapping_add((8u16)));
    loop {
        BlockFunc_A957(tmpD82, grapple_beam_tmpD84, game);
        v0 = ((BlockFunc_ABB0(game)) & (1u8));
        if v0 != 0 {
            break;
        }
        grapple_beam_tmpD84 = ((grapple_beam_tmpD84).wrapping_add((8u16)));
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16);
        *fresh10 = (*fresh10).wrapping_sub(1);
        if *fresh10 == 0 {
            return 0 as uint8;
        }
    }
    return v0;
}
pub unsafe fn BlockFunc_AC11(game: &mut Game) {
    let mut grapple_beam_tmpD82: uint16 =
        ((2u16).wrapping_mul(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)) as u16)));
    let mut grapple_beam_tmpD84: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
    BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd16) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd90) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xd18) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xd92) as *mut uint16);
}
pub unsafe fn BlockFunc_AC31(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) == 0 {
        return 0 as uint8;
    }
    let mut end: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16))
        .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16))));
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        if end < 8u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 0 as uint16;
            end = 8 as uint16;
        }
        let mut increment: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        let mut grapple_beam_tmpD82: uint16 =
            ((2u16).wrapping_mul(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)) as u16)));
        let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
        while v1 != end {
            let mut grapple_beam_tmpD84: uint16 =
                (increment + v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
            BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
            if BlockReact_AA64(game) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = v1;
                return 1 as uint8;
            }
            v1 = v1.wrapping_sub(1);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = end;
        return 0 as uint8;
    } else {
        if end >= 63u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd00) as *mut uint16) = 0 as uint16;
            end = 63 as uint16;
        }
        let mut increment_0: ::core::ffi::c_int = 56 as ::core::ffi::c_int;
        let mut grapple_beam_tmpD82_0: uint16 =
            ((2u16).wrapping_mul(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)) as u16)));
        let mut v1_0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16);
        while v1_0 != end {
            let mut grapple_beam_tmpD84_0: uint16 =
                (increment_0 + v1_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16;
            BlockFunc_A957(grapple_beam_tmpD82_0, grapple_beam_tmpD84_0, game);
            if BlockReact_AA64(game) != 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = v1_0;
                return 1 as uint8;
            }
            v1_0 = v1_0.wrapping_add(1);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) = end;
        return 0 as uint8;
    };
}
pub unsafe fn HandleMovementAndCollForSamusGrapple(game: &mut Game) -> uint8 {
    let mut v2: uint16 = 0;
    let mut v5: uint16 = 0;
    let mut current_block: u64;
    let mut prod16: uint16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v3: uint16 = 0;
    let mut v6: uint16 = 0;
    let mut grapple_beam_tmpD86: uint16 = 0;
    let mut grapple_beam_tmpD88: uint16 = 0;
    let mut tmpD9C: uint16 = 0;
    let mut v0: uint16 = 256 as uint16;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcf4) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        v0 = 160 as uint16;
    }
    let mut v1: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
        .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16))));
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16)) as i16).wrapping_add(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16)) as i16),
    ) >= 0i16
    {
        prod16 = (Multiply16x16(v1, v0, game) >> 8 as ::core::ffi::c_int) as uint16;
        if prod16 == 0 {
            return 0 as uint8;
        }
        tmpD9C = prod16;
        grapple_beam_tmpD88 = (2 as ::core::ffi::c_int
            * (((*((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16))
                .wrapping_add((prod16))) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        v2 = 0;
        v2 = ((2u16).wrapping_mul(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)) as u16)));
        if v2 != grapple_beam_tmpD88 {
            loop {
                grapple_beam_tmpD86 = v2;
                let mut grapple_beam_tmpD82: uint16 =
                    (v2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                        & 0x1ff as ::core::ffi::c_int) as uint16;
                if BlockFunc_ABE6(grapple_beam_tmpD82, game) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0
                {
                    current_block = 7976072742316086414;
                    break;
                }
                v2 = (grapple_beam_tmpD86 as ::core::ffi::c_int + 2 as ::core::ffi::c_int
                    & 0x1ff as ::core::ffi::c_int) as uint16;
                if v2 == grapple_beam_tmpD88 {
                    current_block = 16784897620381762466;
                    break;
                }
            }
            match current_block {
                16784897620381762466 => {}
                _ => {
                    *(&raw mut v4 as *mut uint8).offset(0) = 0 as uint8;
                    *(&raw mut v4 as *mut uint8).offset(
                        (::core::mem::size_of::<int16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1 as usize) as isize,
                    ) = (grapple_beam_tmpD86 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        as uint8;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) =
                        (((v4) as u16) | (0x80u16));
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) =
                        (((v4) as u16) | (0x80u16));
                    if *((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16) != 6u16
                        && *((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16)
                            != 5u16
                        || *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16)
                            != 8u16
                    {
                        current_block = 15145243721762997867;
                    } else {
                        current_block = 4207414038393540328;
                    }
                }
            }
        } else {
            current_block = 16784897620381762466;
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                let ref mut fresh4 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16);
                *fresh4 = ((*fresh4).wrapping_add((tmpD9C)));
                *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16);
                let ref mut fresh5 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16);
                *fresh5 =
                    (*fresh5 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
                let ref mut fresh6 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16);
                *fresh6 = (*fresh6).wrapping_sub(1);
                if (*fresh6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16) =
                        0 as uint16;
                }
                if (*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int)
                    == 0i32
                {
                    v3 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                        .wrapping_sub((6u16)));
                    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                        as i16)
                        .wrapping_sub((6i16))
                        >= 0i16
                    {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                } else {
                    v3 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                        .wrapping_add((6u16)));
                    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                        as i16)
                        .wrapping_add((6i16))
                        < 0i16
                    {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                }
                match current_block {
                    18377268871191777778 => {
                        v3 = 0 as uint16;
                    }
                    _ => {}
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = v3;
                return 0 as uint8;
            }
        }
    } else {
        prod16 = (Multiply16x16(-(v1 as ::core::ffi::c_int) as uint16, v0, game)
            >> 8 as ::core::ffi::c_int) as uint16;
        if prod16 == 0 {
            return 0 as uint8;
        }
        tmpD9C = -(prod16 as ::core::ffi::c_int) as uint16;
        grapple_beam_tmpD88 = (2 as ::core::ffi::c_int
            * (((*((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16))
                .wrapping_sub((prod16))) as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        v5 = 0;
        v5 = ((2u16).wrapping_mul(((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)) as u16)));
        if v5 == grapple_beam_tmpD88 {
            current_block = 5091811861639934416;
        } else {
            current_block = 11777552016271000781;
        }
        loop {
            match current_block {
                11777552016271000781 => {
                    grapple_beam_tmpD86 = v5;
                    let mut grapple_beam_tmpD82_0: uint16 =
                        (v5 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 0x1ff as ::core::ffi::c_int) as uint16;
                    if BlockFunc_ABE6(grapple_beam_tmpD82_0, game) as ::core::ffi::c_int
                        & 1 as ::core::ffi::c_int
                        != 0
                    {
                        *(&raw mut v7 as *mut uint8).offset(0) = 0 as uint8;
                        *(&raw mut v7 as *mut uint8).offset(
                            (::core::mem::size_of::<int16>())
                                .wrapping_div(::core::mem::size_of::<uint8>())
                                .wrapping_sub(1 as usize) as isize,
                        ) = (grapple_beam_tmpD86 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            as uint8;
                        *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16) =
                            (((v7) as u16) | (0x80u16));
                        *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) =
                            (((v7) as u16) | (0x80u16));
                        if (*((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16)
                            == 6u16
                            || *((&raw mut game.g_ram as *mut uint8).offset(0xd98) as *mut uint16)
                                == 5u16)
                            && *((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16)
                                == 8u16
                        {
                            current_block = 4207414038393540328;
                            break;
                        } else {
                            current_block = 15145243721762997867;
                            break;
                        }
                    } else {
                        v5 = (grapple_beam_tmpD86 as ::core::ffi::c_int - 2 as ::core::ffi::c_int
                            & 0x1ff as ::core::ffi::c_int) as uint16;
                        if v5 == grapple_beam_tmpD88 {
                            current_block = 5091811861639934416;
                        } else {
                            current_block = 11777552016271000781;
                        }
                    }
                }
                _ => {
                    let ref mut fresh7 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16);
                    *fresh7 = ((*fresh7).wrapping_add((tmpD9C)));
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcfc) as *mut uint16) =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16);
                    let ref mut fresh8 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16);
                    *fresh8 =
                        (*fresh8 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
                    let ref mut fresh9 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16);
                    *fresh9 = (*fresh9).wrapping_sub(1);
                    if (*fresh9 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16) =
                            0 as uint16;
                    }
                    if (*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int)
                        == 0i32
                    {
                        current_block = 12997042908615822766;
                        break;
                    } else {
                        current_block = 10399321362245223758;
                        break;
                    }
                }
            }
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                match current_block {
                    10399321362245223758 => {
                        v6 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e)
                            as *mut uint16))
                            .wrapping_add((6u16)));
                        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                            as i16)
                            .wrapping_add((6i16))
                            < 0i16
                        {
                            current_block = 9815132409580070441;
                        } else {
                            current_block = 13619784596304402172;
                        }
                    }
                    _ => {
                        v6 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e)
                            as *mut uint16))
                            .wrapping_sub((6u16)));
                        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16))
                            as i16)
                            .wrapping_sub((6i16))
                            >= 0i16
                        {
                            current_block = 9815132409580070441;
                        } else {
                            current_block = 13619784596304402172;
                        }
                    }
                }
                match current_block {
                    13619784596304402172 => {
                        v6 = 0 as uint16;
                    }
                    _ => {}
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = v6;
                return 0 as uint8;
            }
        }
    }
    match current_block {
        15145243721762997867 => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd30) as *mut uint16) = 16 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) =
                -(*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as int16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) =
                -(*((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) as int16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as uint16;
            return 1 as uint8;
        }
        _ => {
            let ref mut fresh3 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd36) as *mut uint16);
            *fresh3 = ((*fresh3) | (0x8000u16));
            *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = 0 as uint16;
            return 1 as uint8;
        }
    };
}
pub unsafe fn BlockFunc_AEE3(game: &mut Game) -> uint8 {
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) as ::core::ffi::c_int
        ^ *((&raw mut game.g_ram as *mut uint8).offset(0xcfa) as *mut uint16)
            as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xd38) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    } else {
        let ref mut fresh22 = *((&raw mut game.g_ram as *mut uint8).offset(0xd38) as *mut uint16);
        *fresh22 = (*fresh22).wrapping_add(1);
        if *fresh22 == 32u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xd32) as *mut uint16) =
                (((0x9bc8c5i32) as u16) & (0xffffu16));
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xd26) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd2e) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn GrappleFunc_AF87(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd62) as *mut uint16).offset(v1 as isize) =
            addr_word_94B197 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16)
            .offset((v1 + 15 as ::core::ffi::c_int) as isize) = addr_word_94B193 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16)
            .offset((v1 + 14 as ::core::ffi::c_int) as isize) = addr_word_94B18F as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16)
            .offset((v1 + 13 as ::core::ffi::c_int) as isize) = addr_word_94B18B as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd40) as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd3e) as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0xd3c) as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16) = 1 as uint16;
        i -= 8 as ::core::ffi::c_int;
    }
}
unsafe fn GrappleInstr_Goto(mut j: uint16, game: &mut Game) -> uint16 {
    return *(RomPtr_94(j, game) as *mut uint16);
}
pub unsafe fn CallGrappleInstr(mut ea: uint32, mut j: uint16, game: &mut Game) -> uint16 {
    match ea {
        9744628 => return GrappleInstr_Goto(j, game),
        _ => return Unreachable(game) as uint16,
    };
}
pub unsafe fn HandleGrappleBeamGfx(game: &mut Game) {
    let mut v1: ::core::ffi::c_int = CalculateAngleFromXY(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16))
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16)))),
        ((*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16))
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16)))),
        game,
    ) as ::core::ffi::c_int;
    let mut r28_r26: uint32 = ((kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int)
        << 11 as ::core::ffi::c_int) as uint32;
    let mut r32_r30: uint32 = ((kSinCosTable8bit_Sext[v1 as usize] as ::core::ffi::c_int)
        << 11 as ::core::ffi::c_int) as uint32;
    let mut chr: uint16 = ((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb)
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int) as uint16;
    chr = (chr as ::core::ffi::c_int
        | 2 as ::core::ffi::c_int
            * ((*(&raw mut game.g_ram as *mut uint8).offset(0xcfb) as ::core::ffi::c_int
                ^ chr as ::core::ffi::c_int)
                & 0x40 as ::core::ffi::c_int
                ^ 0x40 as ::core::ffi::c_int)) as uint16;
    chr = ((chr as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16;
    let mut r20_r18: uint32 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd1a) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
            as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32;
    let mut r24_r22: uint32 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xd1c) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
            as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int)
        << 16 as ::core::ffi::c_int) as uint32;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xcfe) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        return;
    }
    let mut n: ::core::ffi::c_int = (*((&raw mut game.g_ram as *mut uint8).offset(0xcfe)
        as *mut uint16) as ::core::ffi::c_int
        / 8 as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        - 1 as ::core::ffi::c_int;
    let mut q: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
    loop {
        let mut i: ::core::ffi::c_int = 0;
        let ref mut fresh17 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16).offset(q as isize);
        let fresh18 = *fresh17;
        *fresh17 = (*fresh17).wrapping_sub(1);
        if fresh18 == 1u16 {
            let mut v9: *const uint16 = ::core::ptr::null::<uint16>();
            i = *((&raw mut game.g_ram as *mut uint8).offset(0xd62) as *mut uint16)
                .offset(q as isize) as ::core::ffi::c_int;
            loop {
                v9 = RomPtr_94(i as uint16_t, game) as *const uint16;
                if (*v9 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                    break;
                }
                i = CallGrappleInstr(
                    (((*v9) as u32) | (0x940000u32)),
                    (i + 2 as ::core::ffi::c_int) as uint16,
                    game,
                ) as ::core::ffi::c_int;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xd42) as *mut uint16)
                .offset(q as isize) = *v9;
            *((&raw mut game.g_ram as *mut uint8).offset(0xd62) as *mut uint16)
                .offset(q as isize) = (i + 4 as ::core::ffi::c_int) as uint16;
        }
        if ((r24_r22 | r20_r18) & 0xff000000 as uint32) != 0u32 {
            break;
        }
        let mut v12: uint16 = *(RomPtr_94(
            ((*((&raw mut game.g_ram as *mut uint8).offset(0xd62) as *mut uint16)
                .offset(q as isize))
            .wrapping_sub((2u16))),
            game,
        ) as *mut uint16);
        DrawGrappleOams(
            (r20_r18 >> 16 as ::core::ffi::c_int) as uint16,
            (r24_r22 >> 16 as ::core::ffi::c_int) as uint16,
            ((chr) | (v12)),
            game,
        );
        r20_r18 = r20_r18.wrapping_add(r28_r26);
        r24_r22 = r24_r22.wrapping_add(r32_r30);
        q -= 1;
        n -= 1;
        if !(n >= 0i32) {
            break;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_B2_FaceR_Grapple_Air as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_B3_FaceL_Grapple_Air as ::core::ffi::c_int
    {
        DrawGrappleOams3(game);
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        == 0i32
    {
        DrawGrappleOams3(game);
    }
}
unsafe fn DrawGrappleOams(
    mut x_r20: uint16,
    mut y_r24: uint16,
    mut chr_r38: uint16,
    game: &mut Game,
) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16);
    let mut v2: *mut OamEnt = gOamEnt(
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).xcoord = x_r20 as uint8;
    (*v2).ycoord = y_r24 as uint8;
    *(&raw mut (*v2).charnum as *mut uint16) = chr_r38;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) =
        ((v1).wrapping_add((4u16)));
}
unsafe fn DrawGrappleOams3(game: &mut Game) {
    let mut idx: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16);
    let mut v2: *mut OamEnt = gOamEnt(idx as ::core::ffi::c_int, game);
    let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xd08) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
    (*v2).xcoord = x as uint8;
    (*v2).ycoord = (*((&raw mut game.g_ram as *mut uint8).offset(0xd0c) as *mut uint16)
        as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16) as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint8;
    *(&raw mut (*v2).charnum as *mut uint16) = 14880 as uint16;
    let ref mut fresh19 = *((&raw mut game.g_ram as *mut uint8).offset(0x570) as *mut uint16)
        .offset((idx as ::core::ffi::c_int >> 5 as ::core::ffi::c_int) as isize);
    *fresh19 = (*fresh19 as ::core::ffi::c_int
        | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int)
            << 2 as ::core::ffi::c_int
                * (idx as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 7 as ::core::ffi::c_int))
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) =
        ((idx).wrapping_add((4u16)));
}
