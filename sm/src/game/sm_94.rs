pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_90::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{CheckBossBitForCurArea, Multiply16x16};
use crate::game::sm_84::{SpawnPLM, SpawnPLMWithCollision};
use crate::game::sm_90::KillProjectile;
use crate::game::sm_a0::CalculateAngleFromXY;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
use crate::snes::snes::snes_read;

pub use crate::types::Consts_84;
pub use crate::types::Consts_90;
pub use crate::types::Consts_94;
pub use crate::types::GameState;
pub use crate::types::{DoorDef, Func_V, OamEnt, SamusPose, VoidP};
pub const addr_word_94B197: Consts_94 = 45463;
pub const addr_word_94B193: Consts_94 = 45459;
pub const addr_word_94B18F: Consts_94 = 45455;
pub const addr_word_94B18B: Consts_94 = 45451;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CollInfo {
    pub ci_r18_r20: i32,
    pub ci_r24: u16,
    pub ci_r26: u16,
    pub ci_r28: u16,
    pub ci_r30: u16,
    pub ci_r32: u16,
    pub ci_r38: u16,
    pub ci_r40: u16,
}
pub type Func_CollInfo_U8 = unsafe fn(*mut CollInfo, &mut Game) -> u8;
pub use crate::types::Pair_Bool_Amt;
/// Stack-owned sample state for checking Samus's edges after releasing grapple.
struct PostGrappleCollisionSample {
    remaining_span: u16,
    initial_span: u16,
    edge_position: u16,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
#[inline]
unsafe fn RomPtr_8F(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0x8f0000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_94(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0x940000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gOamEnt(v: i32, game: &mut Game) -> *mut OamEnt {
    (&raw mut game.g_ram as *mut u8).offset((0x370 as i32 + v) as isize) as *mut u8 as *mut OamEnt
}
#[inline]
unsafe fn get_DoorDef(a: u16, game: &mut Game) -> *mut DoorDef {
    RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef
}
pub const fnDoorTransitionFunction_HandleElevator: i32 = 0x82e17d;
pub const fnGrappleInstr_Goto: u32 = 9744628;
pub const fnGrappleBeam_Func2: i32 = 0x9bc8c5;

static kAlignPos_Tab1: [u8; 512] = [
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
static kAlignYPos_Tab0: [u8; 512] = [
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
static kTab948E54: [u8; 20] = [
    0, 0, 0x80, 0x80, 0, 0x80, 0, 0x80, 0, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
];
static kBlockColl_Horiz_Slope_NonSquare_Tab: [u16; 64] = [
    0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0,
    0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0x1000, 0xb0, 0x1000, 0xb0, 0, 0x100, 0, 0x100,
    0x1000, 0xc0, 0, 0x100, 0x1000, 0xc0, 0x1000, 0xc0, 0x800, 0xd8, 0x800, 0xd8, 0x600, 0xf0,
    0x600, 0xf0, 0x600, 0xf0, 0x4000, 0x80, 0x4000, 0x80, 0x6000, 0x50, 0x6000, 0x50, 0x6000, 0x50,
];
fn PostGrappleColl_Horiz_Slope_NonSquare(
    sample: &PostGrappleCollisionSample,
    k: u16,
    game: &mut Game,
) -> u16 {
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
    k: u16,
    game: &mut Game,
) -> u16 {
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
    k: u16,
    game: &mut Game,
) -> u16 {
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
    k: u16,
    game: &mut Game,
) -> u16 {
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
fn PostGrappleColl_Horiz_Slope(sample: &PostGrappleCollisionSample, game: &mut Game) -> u16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Horiz_Slope_Square(sample, block, game)
    } else {
        PostGrappleColl_Horiz_Slope_NonSquare(sample, block, game)
    }
}
fn PostGrappleColl_Horiz_Solid(sample: &PostGrappleCollisionSample, game: &mut Game) -> u16 {
    sample.edge_position & 0xf
}
fn PostGrappleColl_Vert_Slope(sample: &PostGrappleCollisionSample, game: &mut Game) -> u16 {
    let slope = game.current_block_bts() & 0x1f;
    let block = game.cur_block_index();
    if slope < 5 {
        PostGrappleColl_Vertical_Slope_Square(sample, block, game)
    } else {
        PostGrappleColl_Vert_Slope_NonSquare(sample, block, game)
    }
}
fn PostGrappleColl_Vert_Solid(sample: &PostGrappleCollisionSample, game: &mut Game) -> u16 {
    sample.edge_position & 0xf
}
fn PostGrappleColl_Horiz(sample: &PostGrappleCollisionSample, k: u16, game: &mut Game) -> u16 {
    game.set_cur_block_index(k >> 1);
    match game.current_level_block() >> 12 {
        1 => PostGrappleColl_Horiz_Slope(sample, game),
        8..=15 => PostGrappleColl_Horiz_Solid(sample, game),
        _ => 0xffff,
    }
}
fn PostGrappleColl_Vert(sample: &PostGrappleCollisionSample, k: u16, game: &mut Game) -> u16 {
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
unsafe fn BlockColl_Horiz_Slope_NonSquare(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Vert_Slope_NonSquare(ci: *mut CollInfo, k: u16, game: &mut Game) -> u8 {
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
    slope: u16,
    k: u16,
    game: &mut Game,
) -> u8 {
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
    slope: u16,
    k: u16,
    game: &mut Game,
) -> u8 {
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
fn ClearCarry_4(_ci: *mut CollInfo, _game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Horiz_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_x_radius();
    let position = game.samus_x_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_x_subpos(subposition);
    1
}
unsafe fn BlockColl_Vert_SolidShootGrappleBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let negative = (*ci).ci_r18_r20 < 0;
    let radius = game.samus_y_radius();
    let position = game.samus_y_pos();
    (*ci).ci_r18_r20 = solid_collision_delta((*ci).ci_r32, radius, position, negative);
    let subposition = if negative { 0u16 } else { 0xffff };
    game.set_samus_y_subpos(subposition);
    1
}
unsafe fn BlockColl_Horiz_Slope(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let slope = u16::from(game.current_block_bts() & 0x1f);
    if slope < 5 {
        return BlockColl_Horiz_Slope_Square(ci, slope, game.cur_block_index(), game);
    }
    game.set_current_slope_bts(u16::from(game.current_block_bts()));
    BlockColl_Horiz_Slope_NonSquare(ci, game)
}
unsafe fn BlockColl_Vert_Slope(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
fn ClearCarry_5(_ci: *mut CollInfo, _game: &mut Game) -> u8 {
    0
}
fn BlockColl_Vert_SpikeAir(_ci: *mut CollInfo, _game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Horiz_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    react_to_spike_block(game);
    BlockColl_Horiz_SolidShootGrappleBlock(ci, game)
}
unsafe fn BlockColl_Vert_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Horiz_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492d9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    if bts & 0x80 != 0 {
        result & 1
    } else {
        0
    }
}
unsafe fn BlockColl_Vert_SpecialAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    BlockColl_Horiz_SpecialAir(ci, game)
}
unsafe fn BlockColl_Horiz_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Vert_SpecialBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    let header = special_collision_plm(bts, 0x9492e9, game);
    let result = SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game) & 1;
    if result != 0 {
        BlockColl_Vert_SolidShootGrappleBlock(ci, game)
    } else {
        result
    }
}
unsafe fn BlockColl_Horiz_BombableAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    if bts & 0x80 == 0 {
        let header = game.read_rom_word(0x94936b, usize::from(bts) * 2);
        SpawnPLMWithCollision(header, &raw mut (*ci).ci_r18_r20, game);
    }
    0
}
unsafe fn BlockColl_Vert_BombableAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    BlockColl_Horiz_BombableAir(ci, game)
}
unsafe fn BlockColl_Horiz_BombBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn BlockColl_Vert_BombBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
pub unsafe fn BlockColl_Horiz_Door(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
pub unsafe fn BlockColl_Vert_Door(ci: *mut CollInfo, game: &mut Game) -> u8 {
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
fn BlockReact_HorizExt(_ci: *mut CollInfo, game: &mut Game) -> u8 {
    let offset = i16::from(game.current_block_bts() as i8);
    if offset == 0 {
        return 0;
    }
    game.set_cur_block_index(game.cur_block_index().wrapping_add(offset as u16));
    0xff
}
fn BlockReact_VertExt(_ci: *mut CollInfo, game: &mut Game) -> u8 {
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
fn Samus_GetYposSpan(game: &mut Game) -> u16 {
    let position = game.samus_y_pos();
    let radius = game.samus_y_radius();
    samus_block_span(position, radius)
}
fn Samus_GetXposSpan(game: &mut Game) -> u16 {
    let position = game.samus_x_pos();
    let radius = game.samus_x_radius();
    samus_block_span(position, radius)
}
unsafe fn BlockColl_Horiz_CheckColl(ci: &mut CollInfo, k: u16, game: &mut Game) -> u8 {
    game.set_cur_block_index(k >> 1);
    // Keep the raw context alive across retries and nested PLM setup calls.
    let context = ci as *mut CollInfo;
    game.react_to_block(|block_type, game| match block_type {
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
    })
}
unsafe fn BlockColl_Vert_CheckColl(ci: &mut CollInfo, k: u16, game: &mut Game) -> u8 {
    game.set_cur_block_index(k >> 1);
    // Keep the same raw context valid across every extension-block retry.
    let context = ci as *mut CollInfo;
    game.react_to_block(|block_type, game| match block_type {
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
    })
}
unsafe fn BlockColl_Handle_Horiz(amt: i32, game: &mut Game) -> Pair_Bool_Amt {
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
unsafe fn BlockColl_Handle_Vert_LeftToRight(amt: i32, game: &mut Game) -> Pair_Bool_Amt {
    handle_vertical_collision(amt, false, game)
}
unsafe fn BlockColl_Handle_Vert_RightToLeft(amt: i32, game: &mut Game) -> Pair_Bool_Amt {
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
    game.set_samus_collision_direction(direction | 15);
    game.set_flag_samus_in_quicksand(0);
}
fn finish_pose_collision(pair: Pair_Bool_Amt, game: &mut Game) -> i32 {
    game.set_samus_collision_flag(u16::from(pair.flag));
    pair.amt.wrapping_abs()
}
pub unsafe fn WallJumpBlockCollDetect(amt: i32, game: &mut Game) -> i32 {
    prepare_pose_collision(game);
    let pair = BlockColl_Handle_Horiz(amt, game);
    finish_pose_collision(pair, game)
}
unsafe fn CollDetectDueToPoseChange_SingleBlock(amt: i32, game: &mut Game) -> i32 {
    prepare_pose_collision(game);
    let frame = u16::from_le_bytes([game.g_ram[0x5b6], game.g_ram[0x5b7]]);
    let pair = if frame & 1 == 0 {
        BlockColl_Handle_Vert_LeftToRight(amt, game)
    } else {
        BlockColl_Handle_Vert_RightToLeft(amt, game)
    };
    finish_pose_collision(pair, game)
}
pub unsafe fn Samus_CollDetectChangedPose(amt: i32, game: &mut Game) -> i32 {
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
pub unsafe fn Samus_MoveRight_NoSolidColl(mut amt: i32, game: &mut Game) -> i32 {
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
pub unsafe fn Samus_MoveDown_NoSolidColl(mut amt: i32, game: &mut Game) -> i32 {
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
fn BlockInsideReact_Slope(_ci: *mut CollInfo, _game: &mut Game) -> u8 {
    0
}
fn BlockInsideReact_ShootableAir(_ci: *mut CollInfo, game: &mut Game) -> u8 {
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
fn BlockInsideReact_SpikeAir(_ci: *mut CollInfo, game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    // Keep the former sixteen-entry table's rejection of invalid BTS values.
    assert!(bts < 16, "spike-air BTS index out of bounds: {bts}");
    if bts == 2 {
        BlockInsideReact_SpikeAir_BTS2(game);
    }
    0
}
fn BlockInsideReact_Special_(_ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn BlockInsideReact_SpecialAir(_ci: *mut CollInfo, game: &mut Game) -> u8 {
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
unsafe fn dispatch_block_inside(block_type: u16, ci: *mut CollInfo, game: &mut Game) -> u8 {
    match block_type {
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
    game.react_to_block(|block_type, game| {
        dispatch_block_inside(block_type, &raw mut context, game)
    });
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
fn CalculateBlockAt(x: u16, y: u16, x_offset: u16, y_offset: u16, game: &mut Game) {
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
static kBlockFunc_9C73_Tab0: [u8; 36] = [
    3, 20, 0, 10, 40, 1, 5, 30, 0, 10, 40, 1, 20, 80, 0, 20, 80, 1, 20, 80, 0, 20, 80, 1, 30, 100,
    0, 30, 100, 1, 30, 100, 0, 30, 100, 1,
];
static kBlockFunc_9C73_Tab1: [i8; 18] = [
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
unsafe fn BlockFunc_9C73(v0: u16, game: &mut Game) -> u16 {
    let v1: i32 = v0 as i32 >> 1 as i32;
    if (ram_word!(game, projectile_type[v1]) as i32 & 0xf00 as i32) != 0i32 {
        kBlockFunc_9C73_Tab1[(2 as i32
            * (*(((&raw mut game.g_ram as *mut u8).offset(0xc18) as *mut u16).offset(v1 as isize)
                as *mut u16 as *mut u8)
                .offset(
                    (::core::mem::size_of::<u16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                ) as i32
                & 0xf as i32)
            + 1 as i32) as usize] as u16
    } else {
        let r18: i32 = ram_word!(game, projectile_type[v1]) as i32 & 0xf as i32;
        kBlockFunc_9C73_Tab0[(3 as i32 * r18 + 2 as i32) as usize] as u16
    }
}
pub unsafe fn BombOrPowerBomb_Func1(v0: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let temp_collision_DD2: u16 = BlockFunc_9C73(v0, game);
    let v1: i32 = v0 as i32 >> 1 as i32;
    v2 = ram_word!(game, projectile_x_pos[v1]) as i16;
    if v2 >= 0i16 {
        let r26: u16 = ram_word!(game, projectile_x_pos[v1]);
        if ((*(&raw mut v2 as *mut u8).offset(
            (::core::mem::size_of::<i16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i16)
            .wrapping_sub((ram_word!(game, room_width_in_scrolls)) as i16)
            < 0i16
        {
            v3 = ram_word!(game, projectile_y_pos[v1]) as i16;
            if v3 >= 0i16 {
                let R28: u16 = ram_word!(game, projectile_y_pos[v1]);
                if ((*(&raw mut v3 as *mut u8).offset(
                    (::core::mem::size_of::<i16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                )) as i16)
                    .wrapping_sub((ram_word!(game, room_height_in_scrolls)) as i16)
                    < 0i16
                {
                    set_ram_word!(game.cur_block_index = 0 as u16);
                    CalculateBlockAt(r26, R28, 0 as u16, 0 as u16, game);
                    let mut ci: CollInfo = CollInfo {
                        ci_r18_r20: 0,
                        ci_r24: 0 as u16,
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
unsafe fn BlockColl_GetBlockIndexAbove(k: u16, game: &mut Game) -> u16 {
    (k as i32
        - ram_word!(game, room_width_in_blocks) as i32
        - ram_word!(game, room_width_in_blocks) as i32) as u16
}
unsafe fn BlockColl_GetBlockIndexRight(k: u16, game: &mut Game) -> u16 {
    (ram_word!(game, room_width_in_blocks) as i32 * 2 as i32 + k as i32 + 1 as i32 + 1 as i32)
        as u16
}
unsafe fn BlockColl_GetBlockIndexLeft(k: u16) -> u16 {
    (k).wrapping_sub(4u16)
}
unsafe fn BlockColl_GetBlockIndexBelow(k: u16, game: &mut Game) -> u16 {
    (ram_word!(game, room_width_in_blocks) as i32 * 2 as i32 + k as i32 + 1 as i32 + 1 as i32)
        as u16
}
unsafe fn BlockColl_BombExplosion(ci: *mut CollInfo, k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    if ram_word!(game, projectile_variables[v1]) == 0 {
        v2 = ram_word!(game, projectile_type[v1]) as i16;
        if (v2 as i32 & 1 as i32) == 0i32 {
            set_ram_word!(game.projectile_type[v1] = ((v2) as u16) | (1u16));
            if ram_word!(game, cur_block_index) != 0xffffu16 {
                let v3: u16 = (2u16).wrapping_mul(ram_word!(game, cur_block_index));
                BlockBombedReact(
                    ci,
                    (2u16).wrapping_mul(ram_word!(game, cur_block_index)),
                    game,
                );
                let mut idx: u16 = BlockColl_GetBlockIndexAbove(v3, game);
                BlockBombedReact(ci, idx, game);
                let idx2: u16 = BlockColl_GetBlockIndexRight(idx, game);
                BlockBombedReact(ci, idx2, game);
                idx = BlockColl_GetBlockIndexLeft(idx2);
                BlockBombedReact(ci, idx, game);
                idx = BlockColl_GetBlockIndexBelow(idx, game);
                BlockBombedReact(ci, idx, game);
            }
        }
    }
}
unsafe fn sub_949D5D(ci: *mut CollInfo, game: &mut Game) -> u8 {
    if (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as i32
        & 0x1f as i32)
        < 5i32
    {
        1 as u8
    } else {
        BlockShotReactVert_Slope_NonSquare(ci, game)
    }
}
unsafe fn BlockBombedReact_Special(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let v0: u8 = *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize);
    if (v0 as i32 & 0x80 as i32) != 0i32 {
        let kBlockBombedReact_Region_Plm: *mut u16 = RomPtr_94(0x9e44 as u16, game) as *mut u16;
        let v2: *const u16 = RomPtr_94(
            *kBlockBombedReact_Region_Plm.offset(ram_word!(game, area_index) as isize),
            game,
        ) as *mut u16;
        SpawnPLM(*v2.offset((v0 as i32 & 0x7f as i32) as isize), game);
    } else {
        let kBlockBombedReact_Special_Plm: *mut u16 = RomPtr_94(0x9da4 as u16, game) as *mut u16;
        SpawnPLM(*kBlockBombedReact_Special_Plm.offset(v0 as isize), game);
    }
    1 as u8
}
unsafe fn BlockReact_ShootableAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let v0: u8 = *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize);
    if (v0 as i32 & 0x80 as i32) == 0i32 {
        let t: u16 = *(RomPtr_94(0x9ea6 as u16, game) as *mut u16).offset(v0 as isize);
        if t == 0xb974u16 {
            (*ci).ci_r38 = 0 as u16;
            (*ci).ci_r40 = 0xffff as u16;
        }
        SpawnPLM(t, game);
    }
    0 as u8
}
unsafe fn BlockReact_Shootable(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let v0: u8 = *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize);
    if (v0 as i32 & 0x80 as i32) != 0i32 {
        let kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm: *mut u16 =
            RomPtr_94(0x9fc6 as u16, game) as *mut u16;
        let v2: *mut u16 = RomPtr_94(
            *kBlockShotBombGrappleReaction_ShootableBlock_RegionPlm
                .offset(ram_word!(game, area_index) as isize),
            game,
        ) as *mut u16;
        SpawnPLM(*v2.offset((v0 as i32 & 0x7f as i32) as isize), game);
    } else {
        let t: u16 = *(RomPtr_94(0x9ea6 as u16, game) as *mut u16).offset(v0 as isize);
        if t == 0xb974u16 {
            (*ci).ci_r38 = 0 as u16;
            (*ci).ci_r40 = 0xffff as u16;
        }
        SpawnPLM(t, game);
    }
    1 as u8
}
unsafe fn BlockReact_BombableAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let kBlockShotBombGrappleReact_BombableBlockPlm: *mut u16 =
        RomPtr_94(0xa012 as u16, game) as *mut u16;
    let v0: u8 = *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize);
    if (v0 as i32 & 0x80 as i32) == 0i32 {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    0 as u8
}
unsafe fn BlockReact_BombableBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let kBlockShotBombGrappleReact_BombableBlockPlm: *mut u16 =
        RomPtr_94(0xa012 as u16, game) as *mut u16;
    let v0: u8 = *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize);
    if (v0 as i32 & 0x80 as i32) == 0i32 {
        SpawnPLM(
            *kBlockShotBombGrappleReact_BombableBlockPlm.offset(v0 as isize),
            game,
        );
    }
    1 as u8
}
unsafe fn BlockBombedReact(ci: *mut CollInfo, v0: u16, game: &mut Game) {
    set_ram_word!(game.cur_block_index = (v0 as i32 >> 1 as i32) as u16);
    game.react_to_block(|block_type, game| match block_type {
        0 | 1 | 2 | 3 | 6 => 0,
        4 => BlockReact_ShootableAir(ci, game),
        5 => BlockReact_HorizExt(ci, game),
        7 => BlockReact_BombableAir(ci, game),
        8 | 9 | 10 | 14 => 1,
        11 => BlockBombedReact_Special(ci, game),
        12 => BlockReact_Shootable(ci, game),
        13 => BlockReact_VertExt(ci, game),
        15 => BlockReact_BombableBlock(ci, game),
        _ => unreachable!(),
    });
}
unsafe fn BlockFunc_A06A(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v3: i16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    v2 = ram_word!(game, projectile_variables[v1]) as i16;
    if v2 != 0 {
        if v2 < 0i16 {
            set_ram_word!(game.projectile_variables[v1] = 0 as u16);
        }
    } else {
        let r18: u16 = *((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8)
            .offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as u16;
        let r20: u16 = (((3u16).wrapping_mul(
            (*((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        )) as i32
            >> 2 as i32) as u16;
        v3 = ((ram_word!(game, power_bomb_explosion_x_pos)) as i16).wrapping_sub(
            (*((&raw mut game.g_ram as *mut u8).offset(0xcea) as *mut u16 as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            )) as i16,
        );
        if v3 < 0i16 {
            v3 = 0 as i16;
        }
        let r22: u16 = (v3 as u16 as i32 >> 4 as i32) as u16;
        let mut v4: u16 = (((r18).wrapping_add(ram_word!(game, power_bomb_explosion_x_pos))) as i32
            >> 4 as i32) as u16;
        if v4 >= ram_word!(game, room_width_in_blocks) {
            v4 = (ram_word!(game, room_width_in_blocks)).wrapping_sub(1u16);
        }
        let r24: u16 = v4;
        let mut v5: u16 = (ram_word!(game, power_bomb_explosion_y_pos)).wrapping_sub(r20);
        if (v5 as i16) < 0i16 {
            v5 = 0 as u16;
        }
        let r26: u16 = (v5 as i32 >> 4 as i32) as u16;
        let mut v6: u16 = (((r20).wrapping_add(ram_word!(game, power_bomb_explosion_y_pos))) as i32
            >> 4 as i32) as u16;
        if v6 >= ram_word!(game, room_height_in_blocks) {
            v6 = (ram_word!(game, room_height_in_blocks)).wrapping_sub(1u16);
        }
        let R28: u16 = v6;
        let a: u16 = (r26 as i32 | (r22 as i32) << 8 as i32) as u16;
        BlockFunc_A0F4(a, r22, r24, r26, R28, game);
        BlockFunc_A11A(a, r24, r26, R28, game);
        BlockFunc_A0F4(
            (R28 as i32 | (r22 as i32) << 8 as i32) as u16,
            r22,
            r24,
            r26,
            R28,
            game,
        );
        BlockFunc_A11A(
            (r26 as i32 | (r24 as i32) << 8 as i32) as u16,
            r24,
            r26,
            R28,
            game,
        );
    };
}
unsafe fn BlockFunc_A0F4(mut a: u16, r22: u16, r24: u16, r26: u16, r28: u16, game: &mut Game) {
    let RegWord: u16 = Mult8x8(a as u8, ram_word!(game, room_width_in_blocks) as u8);
    let mut v3: u16 = (2 as i32
        * (RegWord as i32
            + *(&raw mut a as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as i32)) as u16;
    let mut v4: i16 = ((r24) as i16).wrapping_sub((r22) as i16);
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
        v3 = (v3).wrapping_add(2u16);
        v4 -= 1;
        if (v4 < 0i16) {
            break;
        }
    }
}
unsafe fn BlockFunc_A11A(mut a: u16, r24: u16, r26: u16, r28: u16, game: &mut Game) {
    let RegWord: u16 = Mult8x8(a as u8, ram_word!(game, room_width_in_blocks) as u8);
    let mut v3: u16 = (2 as i32
        * (RegWord as i32
            + *(&raw mut a as *mut u8).offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            ) as i32)) as u16;
    let mut v4: i16 = ((r28) as i16).wrapping_sub((r26) as i16);
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
        v3 = (v3 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2 as i32) as u16;
        v4 -= 1;
        if (v4 < 0i16) {
            break;
        }
    }
}
unsafe fn BlockShotReactHoriz_Slope(ci: *mut CollInfo, game: &mut Game) -> u8 {
    if (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as i32
        & 0x1f as i32)
        < 5i32
    {
        BlockShotReactVert_Slope_Square(
            ci,
            ((*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)) as u16)
                & (0x1fu16),
            ram_word!(game, cur_block_index),
            game,
        )
    } else {
        BlockShotReactVert_Slope_NonSquare(ci, game)
    }
}
unsafe fn BlockShotReactVert_Slope(ci: *mut CollInfo, game: &mut Game) -> u8 {
    if (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as i32
        & 0x1f as i32)
        < 5i32
    {
        BlockShotReactHoriz_Slope_Square(
            ci,
            ((*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)) as u16)
                & (0x1fu16),
            ram_word!(game, cur_block_index),
            game,
        )
    } else {
        BlockShotReactHoriz_Slope_NonSquare(ci, game)
    }
}
unsafe fn react_to_shot_block(
    ci: *mut CollInfo,
    k: u16,
    slope: Func_CollInfo_U8,
    game: &mut Game,
) -> u8 {
    if k >= ram_word!(game, room_size_in_blocks) {
        return 1 as u8;
    }
    set_ram_word!(game.cur_block_index = (k as i32 >> 1 as i32) as u16);
    let v1 = game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 6 => 0,
        1 => slope(ci, game),
        4 => BlockReact_ShootableAir(ci, game),
        5 => BlockReact_HorizExt(ci, game),
        7 => BlockReact_BombableAir(ci, game),
        8 | 9 | 10 | 11 | 14 => 1,
        12 => BlockReact_Shootable(ci, game),
        13 => BlockReact_VertExt(ci, game),
        15 => BlockReact_BombableBlock(ci, game),
        _ => unreachable!(),
    });
    if v1 != 0 {
        (*ci).ci_r40 = (*ci).ci_r40.wrapping_sub(1);
    }
    v1
}
unsafe fn BlockShotReactHoriz(ci: *mut CollInfo, k: u16, game: &mut Game) -> u8 {
    react_to_shot_block(ci, k, BlockShotReactHoriz_Slope, game)
}
unsafe fn BlockShotReactVert(ci: *mut CollInfo, k: u16, game: &mut Game) -> u8 {
    react_to_shot_block(ci, k, BlockShotReactVert_Slope, game)
}
unsafe fn BlockGetSomePos1(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1 as i32;
    let r38: u16 = (ram_word!(game, projectile_y_pos[v1]) as i32
        - ram_word!(game, projectile_y_radius[v1]) as i32
        & 0xfff0 as i32) as u16;
    ((ram_word!(game, projectile_y_radius[v1]) as i32
        + ram_word!(game, projectile_y_pos[v1]) as i32
        - 1 as i32
        - r38 as i32) as u16 as i32
        >> 4 as i32) as u16
}
unsafe fn BlockGetSomePos2(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1 as i32;
    let r38: u16 = (ram_word!(game, projectile_x_pos[v1]) as i32
        - ram_word!(game, projectile_x_radius[v1]) as i32
        & 0xfff0 as i32) as u16;
    ((ram_word!(game, projectile_x_radius[v1]) as i32
        + ram_word!(game, projectile_x_pos[v1]) as i32
        - 1 as i32
        - r38 as i32) as u16 as i32
        >> 4 as i32) as u16
}
pub unsafe fn BlockCollNoWaveBeamHoriz(k: u16, game: &mut Game) -> u8 {
    let mut v7: u16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_x_speed[v1]) as i16 as i32) << 8 as i32;
    let some_pos: u16 = BlockGetSomePos1(k, game);
    let prod: u16 = Mult8x8(
        (((ram_word!(game, projectile_y_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_y_radius[v1]))) as i32
            >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xb8c) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let r24: u16 = ram_word!(game, projectile_x_pos[v1]);
    if amt < 0i32 {
        v7 = (ram_word!(game, projectile_x_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_x_radius[v1]));
    } else {
        v7 = (ram_word!(game, projectile_x_radius[v1]) as i32
            + ram_word!(game, projectile_x_pos[v1]) as i32
            - 1 as i32) as u16;
    }
    let mut R28: u16 = v7;
    let mut v8: u16 = (2 as i32 * (prod as i32 + (v7 as i32 >> 4 as i32))) as u16;
    if some_pos as i32 - 16 as i32 & 0x8000 as i32 == 0
        || ((*(&raw mut R28 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i16)
            .wrapping_sub((ram_word!(game, room_width_in_scrolls)) as i16)
            >= 0i16
    {
        return 0 as u8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as u16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactHoriz(&raw mut ci, v8, game);
        v8 = (v8 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2 as i32) as u16;
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if ((ci.ci_r38 as i32 & 0x8000 as i32) != 0i32) {
            break;
        }
    }
    if (ci.ci_r40 as i32 & 0x8000 as i32) == 0i32 {
        return 0 as u8;
    }
    KillProjectile(k, game);
    1 as u8
}
pub unsafe fn BlockCollNoWaveBeamVert(k: u16, game: &mut Game) -> u8 {
    let mut v7: u16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_y_speed[v1]) as i16 as i32) << 8 as i32;
    let some_pos: u16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xba0) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let r24: u16 = ram_word!(game, projectile_y_pos[v1]);
    if amt < 0i32 {
        v7 = (ram_word!(game, projectile_y_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_y_radius[v1]));
    } else {
        v7 = (ram_word!(game, projectile_y_radius[v1]) as i32
            + ram_word!(game, projectile_y_pos[v1]) as i32
            - 1 as i32) as u16;
    }
    let mut R28: u16 = v7;
    let v8: u16 = (((ram_word!(game, projectile_x_pos[v1]))
        .wrapping_sub(ram_word!(game, projectile_x_radius[v1]))) as i32
        >> 4 as i32) as u16;
    let mut v9: u16 = (2 as i32
        * (Mult8x8(
            (v7 as i32 >> 4 as i32) as u8,
            ram_word!(game, room_width_in_blocks) as u8,
        ) as i32
            + v8 as i32)) as u16;
    if some_pos as i32 - 16 as i32 & 0x8000 as i32 == 0
        || ((*(&raw mut R28 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i16)
            .wrapping_sub((ram_word!(game, room_height_in_scrolls)) as i16)
            >= 0i16
    {
        return 0 as u8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as u16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    loop {
        BlockShotReactVert(&raw mut ci, v9, game);
        v9 = (v9).wrapping_add(2u16);
        ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
        if ((ci.ci_r38 as i32 & 0x8000 as i32) != 0i32) {
            break;
        }
    }
    if (ci.ci_r40 as i32 & 0x8000 as i32) == 0i32 {
        return 0 as u8;
    }
    KillProjectile(k, game);
    1 as u8
}
pub unsafe fn BlockCollWaveBeamHoriz(k: u16, game: &mut Game) -> u8 {
    let mut v7: u16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_x_speed[v1]) as i16 as i32) << 8 as i32;
    let some_pos: u16 = BlockGetSomePos1(k, game);
    let prod: u16 = Mult8x8(
        (((ram_word!(game, projectile_y_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_y_radius[v1]))) as i32
            >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xb8c) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let r24: u16 = ram_word!(game, projectile_x_pos[v1]);
    if amt < 0i32 {
        v7 = (ram_word!(game, projectile_x_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_x_radius[v1]));
    } else {
        v7 = (ram_word!(game, projectile_x_radius[v1]) as i32
            + ram_word!(game, projectile_x_pos[v1]) as i32
            - 1 as i32) as u16;
    }
    let mut R28: u16 = v7;
    let mut v8: u16 = (2 as i32 * (prod as i32 + (v7 as i32 >> 4 as i32))) as u16;
    if some_pos as i32 - 16 as i32 & 0x8000 as i32 == 0 {
        return 0 as u8;
    }
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as u16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    let v9: u8 = *(((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16).offset(v1 as isize)
        as *mut u16 as *mut u8)
        .offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        );
    if v9 as i32 & 0x80 as i32 == 0
        && ((v9) as i16).wrapping_sub((ram_word!(game, room_height_in_scrolls)) as i16) < 0i16
        && ((*(&raw mut R28 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i16)
            .wrapping_sub((ram_word!(game, room_width_in_scrolls)) as i16)
            < 0i16
    {
        loop {
            BlockShotReactHoriz(&raw mut ci, v8, game);
            v8 = (v8 as i32 + ram_word!(game, room_width_in_blocks) as i32 * 2 as i32) as u16;
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if ((ci.ci_r38 as i32 & 0x8000 as i32) != 0i32) {
                break;
            }
        }
    }
    0 as u8
}
pub unsafe fn BlockCollWaveBeamVert(k: u16, game: &mut Game) -> u8 {
    let mut v7: u16 = 0;
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_y_speed[v1]) as i16 as i32) << 8 as i32;
    let some_pos: u16 = BlockGetSomePos2(k, game);
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xba0) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let r24: u16 = ram_word!(game, projectile_y_pos[v1]);
    if amt < 0i32 {
        v7 = (ram_word!(game, projectile_y_pos[v1]))
            .wrapping_sub(ram_word!(game, projectile_y_radius[v1]));
    } else {
        v7 = (ram_word!(game, projectile_y_radius[v1]) as i32
            + ram_word!(game, projectile_y_pos[v1]) as i32
            - 1 as i32) as u16;
    }
    let mut R28: u16 = v7;
    let prod: u16 = Mult8x8(
        (v7 as i32 >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    let v8: u16 = (((ram_word!(game, projectile_x_pos[v1]))
        .wrapping_sub(ram_word!(game, projectile_x_radius[v1]))) as i32
        >> 4 as i32) as u16;
    let mut v9: u16 = (2 as i32 * (prod as i32 + v8 as i32)) as u16;
    if some_pos as i32 - 16 as i32 & 0x8000 as i32 == 0 {
        return 0 as u8;
    }
    let v10: u8 =
        *(((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16).offset(v1 as isize)
            as *mut u16 as *mut u8)
            .offset(
                (::core::mem::size_of::<u16>())
                    .wrapping_div(::core::mem::size_of::<u8>())
                    .wrapping_sub(1_usize) as isize,
            );
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: some_pos,
        ci_r28: R28,
        ci_r30: 0 as u16,
        ci_r32: 0,
        ci_r38: some_pos,
        ci_r40: some_pos,
    };
    if v10 as i32 & 0x80 as i32 == 0
        && ((v10) as i16).wrapping_sub((ram_word!(game, room_width_in_scrolls)) as i16) < 0i16
        && ((*(&raw mut R28 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        )) as i16)
            .wrapping_sub((ram_word!(game, room_height_in_scrolls)) as i16)
            < 0i16
    {
        loop {
            BlockShotReactVert(&raw mut ci, v9, game);
            v9 = (v9).wrapping_add(2u16);
            ci.ci_r38 = ci.ci_r38.wrapping_sub(1);
            if ((ci.ci_r38 as i32 & 0x8000 as i32) != 0i32) {
                break;
            }
        }
    }
    0 as u8
}
pub unsafe fn BlockCollMissileHoriz(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_x_speed[v1]) as i16 as i32) << 8 as i32;
    let prod: u16 = Mult8x8(
        (ram_word!(game, projectile_y_pos[v1]) as i32 >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xb8c) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let v3: u16 = ram_word!(game, projectile_bomb_x_subpos[v1]);
    let mut r24: u16 = ram_word!(game, projectile_x_pos[v1]);
    let r28: u16 = r24;
    let v7: u16 = (2 as i32
        * (prod as i32 + (ram_word!(game, projectile_x_pos[v1]) as i32 >> 4 as i32)))
        as u16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as u16,
        ci_r28: r28,
        ci_r30: 1 as u16,
        ci_r32: 0,
        ci_r38: 0 as u16,
        ci_r40: 0,
    };
    if ((*(&raw mut r24 as *mut u8).offset(
        (::core::mem::size_of::<u16>())
            .wrapping_div(::core::mem::size_of::<u8>())
            .wrapping_sub(1_usize) as isize,
    )) as i16)
        .wrapping_sub((ram_word!(game, room_width_in_scrolls)) as i16)
        >= 0i16
        || BlockShotReactHoriz(&raw mut ci, v7, game) == 0
    {
        return 0 as u8;
    }
    KillProjectile(k, game);
    1 as u8
}
pub unsafe fn BlockCollMissileVert(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1 as i32;
    let amt: i32 = (ram_word!(game, projectile_bomb_y_speed[v1]) as i16 as i32) << 8 as i32;
    AddToHiLo(
        ((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16).offset(v1 as isize)
            as *mut u16,
        ((&raw mut game.g_ram as *mut u8).offset(0xba0) as *mut u16).offset(v1 as isize)
            as *mut u16,
        amt as u32,
    );
    let mut r24: u16 = ram_word!(game, projectile_y_pos[v1]);
    let r28: u16 = r24;
    let prod: u16 = Mult8x8(
        (ram_word!(game, projectile_y_pos[v1]) as i32 >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    let v7: u16 = (ram_word!(game, projectile_x_pos[v1]) as i32 >> 4 as i32) as u16;
    let v8: u16 = (2 as i32 * (prod as i32 + v7 as i32)) as u16;
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: amt,
        ci_r24: r24,
        ci_r26: 0 as u16,
        ci_r28: r28,
        ci_r30: 1 as u16,
        ci_r32: 0,
        ci_r38: 0 as u16,
        ci_r40: 0,
    };
    if ((*(&raw mut r24 as *mut u8).offset(
        (::core::mem::size_of::<u16>())
            .wrapping_div(::core::mem::size_of::<u8>())
            .wrapping_sub(1_usize) as isize,
    )) as i16)
        .wrapping_sub((ram_word!(game, room_height_in_scrolls)) as i16)
        >= 0i16
        || BlockShotReactVert(&raw mut ci, v8, game) == 0
    {
        return 0 as u8;
    }
    KillProjectile(k, game);
    1 as u8
}
unsafe fn BlockShotReact_Slope_NonSquare(ci: *mut CollInfo, j: u16, k: u16, game: &mut Game) -> u8 {
    let v2: u16 = (ram_word!(game, projectile_x_pos[(j as i32 >> 1i32)]) as i32 & 0xf as i32
        ^ (if (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(k as isize) as i32
            & 0x40 as i32)
            != 0i32
        {
            0xf as i32
        } else {
            0 as i32
        })) as u16;
    let v4: u16 = (ram_word!(game, projectile_y_pos[(j as i32 >> 1i32)]) as i32 & 0xf as i32
        ^ (if (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(k as isize) as i32
            & 0x80 as i32)
            != 0i32
        {
            0xf as i32
        } else {
            0 as i32
        })) as u16;
    let v5: u16 = ((kAlignYPos_Tab0[(16 as i32
        * (*(&raw mut game.g_ram as *mut u8)
            .offset(0x16402)
            .offset(k as isize) as i32
            & 0x1f as i32)
        + (v2 as i32 & 0xf as i32)) as usize]) as u16)
        & (0x1fu16);
    if ((v5) as i16).wrapping_sub((v4) as i16) <= 0i16 {
        (*ci).ci_r38 = 0 as u16;
        (*ci).ci_r40 = 0 as u16;
        1 as u8
    } else {
        0 as u8
    }
}
unsafe fn BlockShotReactVert_Slope_NonSquare(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ram_word!(game, cur_block_index);
    let v1: u16 = ram_word!(game, projectile_index);
    let div: u16 = SnesDivide(
        ram_word!(game, cur_block_index),
        ram_word!(game, room_width_in_blocks) as u8,
    );
    if ram_word!(game, projectile_y_pos[(v1 as i32 >> 1i32)]) as i32 >> 4 as i32 == div as i32 {
        BlockShotReact_Slope_NonSquare(ci, v1, v0, game)
    } else {
        0 as u8
    }
}
unsafe fn BlockShotReactHoriz_Slope_NonSquare(ci: *mut CollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ram_word!(game, cur_block_index);
    let v1: u16 = ram_word!(game, projectile_index);
    let mod_0: u16 = SnesModulus(
        ram_word!(game, cur_block_index),
        ram_word!(game, room_width_in_blocks) as u8,
    );
    if ram_word!(game, projectile_x_pos[(v1 as i32 >> 1i32)]) as i32 >> 4 as i32 == mod_0 as i32 {
        BlockShotReact_Slope_NonSquare(ci, v1, v0, game)
    } else {
        0 as u8
    }
}
pub unsafe fn BlockCollSpreadBomb(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1 as i32;
    let r26: u16 = ram_word!(game, projectile_x_pos[v1]);
    let r28: u16 = ram_word!(game, projectile_y_pos[v1]);
    set_ram_word!(game.cur_block_index = 0 as u16);
    CalculateBlockAt(r26, r28, 0 as u16, 0 as u16, game);
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0,
        ci_r24: 0 as u16,
        ci_r26: r26,
        ci_r28: r28,
        ci_r30: 0 as u16,
        ci_r32: 0 as u16,
        ci_r38: 0,
        ci_r40: 0,
    };
    if ram_word!(game, projectile_variables[v1]) == 0 {
        BlockColl_BombExplosion(&raw mut ci, k, game);
        return 0 as u8;
    }
    if ram_word!(game, cur_block_index) == 0xffffu16 {
        return 1 as u8;
    }
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 6 | 7 => 0,
        1 => sub_949D5D(&raw mut ci, game),
        4 | 8 | 9 | 10 | 11 | 12 | 14 | 15 => 1,
        5 => BlockReact_HorizExt(&raw mut ci, game),
        13 => BlockReact_VertExt(&raw mut ci, game),
        _ => unreachable!(),
    })
}
unsafe fn BlockShotReactVert_Slope_Square(
    ci: *mut CollInfo,
    a: u16,
    k: u16,
    game: &mut Game,
) -> u8 {
    let temp_collision_DD4: u16 = (4u16).wrapping_mul(a);
    let temp_collision_DD6: u16 = (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as i32
        >> 6 as i32) as u16;
    let mut v2: u16 = (4 as i32 * a as i32
        + (temp_collision_DD6 as i32 ^ ((*ci).ci_r28 as i32 & 8 as i32) >> 3 as i32))
        as u16;
    let v3: i32 = ram_word!(game, projectile_index) as i32 >> 1 as i32;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            if (ram_word!(game, projectile_y_pos[v3]) as i32
                - ram_word!(game, projectile_y_radius[v3]) as i32
                & 8 as i32)
                != 0i32
                || kTab948E54[v2 as usize] == 0
            {
                let v4: u16 = (v2) ^ (2u16);
                if (*(((&raw mut game.g_ram as *mut u8).offset(0xbc8) as *mut u16)
                    .offset(v3 as isize) as *mut u16 as *mut u8)
                    .offset(0) as i32
                    + *(((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16)
                        .offset(v3 as isize) as *mut u16 as *mut u8)
                        .offset(0) as i32
                    - 1 as i32
                    & 8 as i32)
                    == 0i32
                    || kTab948E54[v4 as usize] == 0
                {
                    return 0 as u8;
                }
            }
            return 1 as u8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 == (*ci).ci_r26
                && (ram_word!(game, projectile_y_pos[v3]) as i32
                    - ram_word!(game, projectile_y_radius[v3]) as i32
                    & 8 as i32)
                    != 0i32
            {
                return (kTab948E54[(v2 as i32 ^ 2 as i32) as usize] != 0u8) as u8;
            }
        } else if (*(((&raw mut game.g_ram as *mut u8).offset(0xbc8) as *mut u16)
            .offset(v3 as isize) as *mut u16 as *mut u8)
            .offset(0) as i32
            + *(((&raw mut game.g_ram as *mut u8).offset(0xb78) as *mut u16).offset(v3 as isize)
                as *mut u16 as *mut u8)
                .offset(0) as i32
            - 1 as i32
            & 8 as i32)
            == 0i32
        {
            return (kTab948E54[v2 as usize] != 0u8) as u8;
        }
        if kTab948E54[v2 as usize] != 0u8 {
            return 1 as u8;
        }
        return (kTab948E54[(v2 as i32 ^ 2 as i32) as usize] != 0u8) as u8;
    }
    if (ram_word!(game, projectile_y_pos[v3]) as i32 & 8 as i32) != 0i32 {
        v2 = (v2) ^ (2u16);
    }
    (kTab948E54[v2 as usize] != 0u8) as u8
}
unsafe fn BlockShotReactHoriz_Slope_Square(
    ci: *mut CollInfo,
    a: u16,
    k: u16,
    game: &mut Game,
) -> u8 {
    let temp_collision_DD4: u16 = (4u16).wrapping_mul(a);
    let temp_collision_DD6: u16 = (*(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(k as isize) as i32
        >> 6 as i32) as u16;
    let mut v2: u16 = (4 as i32 * a as i32
        + (temp_collision_DD6 as i32 ^ ((*ci).ci_r28 as i32 & 8 as i32) >> 2 as i32))
        as u16;
    if (*ci).ci_r30 == 0 {
        if (*ci).ci_r26 == 0 {
            let v3: i32 = ram_word!(game, projectile_index) as i32 >> 1 as i32;
            if (ram_word!(game, projectile_x_pos[v3]) as i32
                - ram_word!(game, projectile_x_radius[v3]) as i32
                & 8 as i32)
                != 0i32
                || kTab948E54[v2 as usize] == 0u8
            {
                let v4: u16 = (v2) ^ (1u16);
                if (*(((&raw mut game.g_ram as *mut u8).offset(0xbb4) as *mut u16)
                    .offset(v3 as isize) as *mut u16 as *mut u8)
                    .offset(0) as i32
                    + *(((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16)
                        .offset(v3 as isize) as *mut u16 as *mut u8)
                        .offset(0) as i32
                    - 1 as i32
                    & 8 as i32)
                    == 0i32
                    || kTab948E54[v4 as usize] == 0u8
                {
                    return 0 as u8;
                }
            }
            return 1 as u8;
        }
        if (*ci).ci_r38 != 0 {
            if (*ci).ci_r38 == (*ci).ci_r26
                && (*((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16)
                    .offset((ram_word!(game, projectile_index) as i32 >> 1 as i32) as isize)
                    as i32
                    - *((&raw mut game.g_ram as *mut u8).offset(0xbb4) as *mut u16)
                        .offset((ram_word!(game, projectile_index) as i32 >> 1 as i32) as isize)
                        as i32
                    & 8 as i32)
                    != 0i32
            {
                return (kTab948E54[(v2 as i32 ^ 1 as i32) as usize] != 0u8) as u8;
            }
        } else if (*(((&raw mut game.g_ram as *mut u8).offset(0xbb4) as *mut u16)
            .offset((ram_word!(game, projectile_index) as i32 >> 1 as i32) as isize)
            as *mut u16 as *mut u8)
            .offset(0) as i32
            + *(((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16)
                .offset((ram_word!(game, projectile_index) as i32 >> 1 as i32) as isize)
                as *mut u16 as *mut u8)
                .offset(0) as i32
            - 1 as i32
            & 8 as i32)
            == 0i32
        {
            return (kTab948E54[v2 as usize] != 0u8) as u8;
        }
        if kTab948E54[v2 as usize] != 0u8 {
            return 1 as u8;
        }
        return (kTab948E54[(v2 as i32 ^ 1 as i32) as usize] != 0u8) as u8;
    }
    if (*((&raw mut game.g_ram as *mut u8).offset(0xb64) as *mut u16)
        .offset((ram_word!(game, projectile_index) as i32 >> 1 as i32) as isize) as i32
        & 8 as i32)
        != 0i32
    {
        v2 = (v2) ^ (1u16);
    }
    (kTab948E54[v2 as usize] != 0u8) as u8
}
unsafe fn BlockReactGrapple_GrappleBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    static kBlockReactGrapple_GrappleBlockPlm: [u16; 4] = [0xd0d8, 0xd0dc, 0xd0e0, 0xd0d8];
    update_ram_word!(
        game.grapple_beam_flags,
        |fresh0| (fresh0 as i32 & !(0x8000 as i32)) as u16
    );
    if *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as i32
        & 0x80 as i32
        != 0
    {
        0 as u8
    } else {
        SpawnPLM(
            kBlockReactGrapple_GrappleBlockPlm[*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)
                as usize],
            game,
        )
    }
}
unsafe fn BlockReactGrapple_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    static kBlockReactGrapple_SpikeBlockPlm: [u16; 16] = [
        0xd0e4, 0xd0e4, 0xd0e4, 0xd0e8, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
        0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
    ];
    if *(&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as i32
        & 0x80 as i32
        != 0
    {
        0 as u8
    } else {
        SpawnPLM(
            kBlockReactGrapple_SpikeBlockPlm[*(&raw mut game.g_ram as *mut u8)
                .offset(0x16402)
                .offset(ram_word!(game, cur_block_index) as isize)
                as usize],
            game,
        )
    }
}
pub unsafe fn BlockCollGrappleBeam(game: &mut Game) -> u8 {
    let grapple_vel_x: i32 =
        (ram_word!(game, grapple_beam_extension_x_velocity) as i16 as i32) << 6 as i32;
    let grapple_vel_y: i32 =
        (ram_word!(game, grapple_beam_extension_y_velocity) as i16 as i32) << 6 as i32;
    let mut result: u8 = 0 as u8;
    let mut i: i32 = 0 as i32;
    while i < 4i32 {
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xd10) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xd0e) as *mut u16,
            grapple_vel_x as u32,
        );
        AddToHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xd14) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xd12) as *mut u16,
            grapple_vel_y as u32,
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xd08) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xd06) as *mut u16,
            ((ram_word!(game, grapple_beam_end_x_offset) as u32) << 16 as i32
                | ram_word!(game, grapple_beam_end_x_suboffset) as u32)
                .wrapping_add(
                    (ram_word!(game, samus_x_pos) as u32) << 16 as i32
                        | ram_word!(game, samus_x_subpos) as u32,
                )
                .wrapping_add(
                    ((ram_word!(game, grapple_beam_origin_x_offset) as i32) << 16 as i32) as u32,
                ),
        );
        SetHiLo(
            (&raw mut game.g_ram as *mut u8).offset(0xd0c) as *mut u16,
            (&raw mut game.g_ram as *mut u8).offset(0xd0a) as *mut u16,
            ((ram_word!(game, grapple_beam_end_y_offset) as u32) << 16 as i32
                | ram_word!(game, grapple_beam_end_y_suboffset) as u32)
                .wrapping_add(
                    (ram_word!(game, samus_y_pos) as u32) << 16 as i32
                        | ram_word!(game, samus_y_subpos) as u32,
                )
                .wrapping_add(
                    ((ram_word!(game, grapple_beam_origin_y_offset) as i32) << 16 as i32) as u32,
                ),
        );
        result = BlockReactGrapple(game);
        if (result as i32 & 0x40 as i32) != 0i32 && (result as i32 & 1 as i32) != 0i32 {
            set_ram_word!(
                game.grapple_beam_end_x_pos = (ram_word!(game, grapple_beam_end_x_pos) as i32
                    & 0xfff0 as i32
                    | 8 as i32) as u16
            );
            set_ram_word!(
                game.grapple_beam_end_y_pos = (ram_word!(game, grapple_beam_end_y_pos) as i32
                    & 0xfff0 as i32
                    | 8 as i32) as u16
            );
            return result;
        }
        i += 1;
    }
    result
}
pub unsafe fn BlockReactGrapple(game: &mut Game) -> u8 {
    let mut v0: i16 = 0;

    let prod: u16 = Mult8x8(
        (ram_word!(game, grapple_beam_end_y_pos) as i32 >> 4 as i32) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    v0 = (ram_word!(game, grapple_beam_end_x_pos) as i32 >> 4 as i32) as i16;
    set_ram_word!(game.cur_block_index = (prod).wrapping_add((v0) as u16));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as i32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 6 => 0,
        1 | 8 | 9 | 11 => 1,
        4 => BlockReact_ShootableAir(&raw mut ci, game),
        5 => BlockReact_HorizExt(&raw mut ci, game),
        7 => BlockReact_BombableAir(&raw mut ci, game),
        10 => BlockReactGrapple_SpikeBlock(&raw mut ci, game),
        12 => BlockReact_Shootable(&raw mut ci, game),
        13 => BlockReact_VertExt(&raw mut ci, game),
        14 => BlockReactGrapple_GrappleBlock(&raw mut ci, game),
        15 => BlockReact_BombableBlock(&raw mut ci, game),
        _ => unreachable!(),
    })
}
pub static kSinCosTable8bit_Sext: [i16; 320] = [
    -(256 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(254 as i32) as i16,
    -(254 as i32) as i16,
    -(253 as i32) as i16,
    -(252 as i32) as i16,
    -(251 as i32) as i16,
    -(249 as i32) as i16,
    -(248 as i32) as i16,
    -(246 as i32) as i16,
    -(244 as i32) as i16,
    -(243 as i32) as i16,
    -(241 as i32) as i16,
    -(238 as i32) as i16,
    -(236 as i32) as i16,
    -(234 as i32) as i16,
    -(231 as i32) as i16,
    -(228 as i32) as i16,
    -(225 as i32) as i16,
    -(222 as i32) as i16,
    -(219 as i32) as i16,
    -(216 as i32) as i16,
    -(212 as i32) as i16,
    -(209 as i32) as i16,
    -(205 as i32) as i16,
    -(201 as i32) as i16,
    -(197 as i32) as i16,
    -(193 as i32) as i16,
    -(189 as i32) as i16,
    -(185 as i32) as i16,
    -(181 as i32) as i16,
    -(176 as i32) as i16,
    -(171 as i32) as i16,
    -(167 as i32) as i16,
    -(162 as i32) as i16,
    -(157 as i32) as i16,
    -(152 as i32) as i16,
    -(147 as i32) as i16,
    -(142 as i32) as i16,
    -(136 as i32) as i16,
    -(131 as i32) as i16,
    -(126 as i32) as i16,
    -(120 as i32) as i16,
    -(115 as i32) as i16,
    -(109 as i32) as i16,
    -(103 as i32) as i16,
    -(97 as i32) as i16,
    -(92 as i32) as i16,
    -(86 as i32) as i16,
    -(80 as i32) as i16,
    -(74 as i32) as i16,
    -(68 as i32) as i16,
    -(62 as i32) as i16,
    -(56 as i32) as i16,
    -(49 as i32) as i16,
    -(43 as i32) as i16,
    -(37 as i32) as i16,
    -(31 as i32) as i16,
    -(25 as i32) as i16,
    -(18 as i32) as i16,
    -(12 as i32) as i16,
    -(6 as i32) as i16,
    0 as i16,
    6 as i16,
    12 as i16,
    18 as i16,
    25 as i16,
    31 as i16,
    37 as i16,
    43 as i16,
    49 as i16,
    56 as i16,
    62 as i16,
    68 as i16,
    74 as i16,
    80 as i16,
    86 as i16,
    92 as i16,
    97 as i16,
    103 as i16,
    109 as i16,
    115 as i16,
    120 as i16,
    126 as i16,
    131 as i16,
    136 as i16,
    142 as i16,
    147 as i16,
    152 as i16,
    157 as i16,
    162 as i16,
    167 as i16,
    171 as i16,
    176 as i16,
    181 as i16,
    185 as i16,
    189 as i16,
    193 as i16,
    197 as i16,
    201 as i16,
    205 as i16,
    209 as i16,
    212 as i16,
    216 as i16,
    219 as i16,
    222 as i16,
    225 as i16,
    228 as i16,
    231 as i16,
    234 as i16,
    236 as i16,
    238 as i16,
    241 as i16,
    243 as i16,
    244 as i16,
    246 as i16,
    248 as i16,
    249 as i16,
    251 as i16,
    252 as i16,
    253 as i16,
    254 as i16,
    254 as i16,
    255 as i16,
    255 as i16,
    255 as i16,
    256 as i16,
    255 as i16,
    255 as i16,
    255 as i16,
    254 as i16,
    254 as i16,
    253 as i16,
    252 as i16,
    251 as i16,
    249 as i16,
    248 as i16,
    246 as i16,
    244 as i16,
    243 as i16,
    241 as i16,
    238 as i16,
    236 as i16,
    234 as i16,
    231 as i16,
    228 as i16,
    225 as i16,
    222 as i16,
    219 as i16,
    216 as i16,
    212 as i16,
    209 as i16,
    205 as i16,
    201 as i16,
    197 as i16,
    193 as i16,
    189 as i16,
    185 as i16,
    181 as i16,
    176 as i16,
    171 as i16,
    167 as i16,
    162 as i16,
    157 as i16,
    152 as i16,
    147 as i16,
    142 as i16,
    136 as i16,
    131 as i16,
    126 as i16,
    120 as i16,
    115 as i16,
    109 as i16,
    103 as i16,
    97 as i16,
    92 as i16,
    86 as i16,
    80 as i16,
    74 as i16,
    68 as i16,
    62 as i16,
    56 as i16,
    49 as i16,
    43 as i16,
    37 as i16,
    31 as i16,
    25 as i16,
    18 as i16,
    12 as i16,
    6 as i16,
    0 as i16,
    -(6 as i32) as i16,
    -(12 as i32) as i16,
    -(18 as i32) as i16,
    -(25 as i32) as i16,
    -(31 as i32) as i16,
    -(37 as i32) as i16,
    -(43 as i32) as i16,
    -(49 as i32) as i16,
    -(56 as i32) as i16,
    -(62 as i32) as i16,
    -(68 as i32) as i16,
    -(74 as i32) as i16,
    -(80 as i32) as i16,
    -(86 as i32) as i16,
    -(92 as i32) as i16,
    -(97 as i32) as i16,
    -(103 as i32) as i16,
    -(109 as i32) as i16,
    -(115 as i32) as i16,
    -(120 as i32) as i16,
    -(126 as i32) as i16,
    -(131 as i32) as i16,
    -(136 as i32) as i16,
    -(142 as i32) as i16,
    -(147 as i32) as i16,
    -(152 as i32) as i16,
    -(157 as i32) as i16,
    -(162 as i32) as i16,
    -(167 as i32) as i16,
    -(171 as i32) as i16,
    -(176 as i32) as i16,
    -(181 as i32) as i16,
    -(185 as i32) as i16,
    -(189 as i32) as i16,
    -(193 as i32) as i16,
    -(197 as i32) as i16,
    -(201 as i32) as i16,
    -(205 as i32) as i16,
    -(209 as i32) as i16,
    -(212 as i32) as i16,
    -(216 as i32) as i16,
    -(219 as i32) as i16,
    -(222 as i32) as i16,
    -(225 as i32) as i16,
    -(228 as i32) as i16,
    -(231 as i32) as i16,
    -(234 as i32) as i16,
    -(236 as i32) as i16,
    -(238 as i32) as i16,
    -(241 as i32) as i16,
    -(243 as i32) as i16,
    -(244 as i32) as i16,
    -(246 as i32) as i16,
    -(248 as i32) as i16,
    -(249 as i32) as i16,
    -(251 as i32) as i16,
    -(252 as i32) as i16,
    -(253 as i32) as i16,
    -(254 as i32) as i16,
    -(254 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(256 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(255 as i32) as i16,
    -(254 as i32) as i16,
    -(254 as i32) as i16,
    -(253 as i32) as i16,
    -(252 as i32) as i16,
    -(251 as i32) as i16,
    -(249 as i32) as i16,
    -(248 as i32) as i16,
    -(246 as i32) as i16,
    -(244 as i32) as i16,
    -(243 as i32) as i16,
    -(241 as i32) as i16,
    -(238 as i32) as i16,
    -(236 as i32) as i16,
    -(234 as i32) as i16,
    -(231 as i32) as i16,
    -(228 as i32) as i16,
    -(225 as i32) as i16,
    -(222 as i32) as i16,
    -(219 as i32) as i16,
    -(216 as i32) as i16,
    -(212 as i32) as i16,
    -(209 as i32) as i16,
    -(205 as i32) as i16,
    -(201 as i32) as i16,
    -(197 as i32) as i16,
    -(193 as i32) as i16,
    -(189 as i32) as i16,
    -(185 as i32) as i16,
    -(181 as i32) as i16,
    -(176 as i32) as i16,
    -(171 as i32) as i16,
    -(167 as i32) as i16,
    -(162 as i32) as i16,
    -(157 as i32) as i16,
    -(152 as i32) as i16,
    -(147 as i32) as i16,
    -(142 as i32) as i16,
    -(136 as i32) as i16,
    -(131 as i32) as i16,
    -(126 as i32) as i16,
    -(120 as i32) as i16,
    -(115 as i32) as i16,
    -(109 as i32) as i16,
    -(103 as i32) as i16,
    -(97 as i32) as i16,
    -(92 as i32) as i16,
    -(86 as i32) as i16,
    -(80 as i32) as i16,
    -(74 as i32) as i16,
    -(68 as i32) as i16,
    -(62 as i32) as i16,
    -(56 as i32) as i16,
    -(49 as i32) as i16,
    -(43 as i32) as i16,
    -(37 as i32) as i16,
    -(31 as i32) as i16,
    -(25 as i32) as i16,
    -(18 as i32) as i16,
    -(12 as i32) as i16,
    -(6 as i32) as i16,
];
unsafe fn BlockFunc_A957(tmpD82: u16, tmpD84: u16, game: &mut Game) {
    let mut v5: i16 = 0;
    let mut v7: i16 = 0;
    let v0: u16 = tmpD82;
    let mut v2: u16 = 0;
    let mut v3: u16 = 0;
    if (ram_word!(game, grapple_beam_flags) as i32 & 0x8000 as i32) == 0i32 {
        let v1: i32 = tmpD82 as i32 >> 1 as i32;
        if (kSinCosTable8bit_Sext[(v1 + 64 as i32) as usize] as i32 & 0x8000 as i32) != 0i32 {
            v2 = (ram_word!(game, grapple_beam_end_x_pos) as i32 & 0xfff0 as i32 | 7 as i32) as u16;
        } else {
            v2 = (ram_word!(game, grapple_beam_end_x_pos) as i32 & 0xfff0 as i32 | 8 as i32) as u16;
        }
        set_ram_word!(game.grapple_beam_end_x_pos = v2);
        if (kSinCosTable8bit_Sext[v1 as usize] as i32 & 0x8000 as i32) != 0i32 {
            v3 = (ram_word!(game, grapple_beam_end_y_pos) as i32 & 0xfff0 as i32 | 7 as i32) as u16;
        } else {
            v3 = (ram_word!(game, grapple_beam_end_y_pos) as i32 & 0xfff0 as i32 | 8 as i32) as u16;
        }
        set_ram_word!(game.grapple_beam_end_y_pos = v3);
    }
    let v4: i32 = v0 as i32 >> 1 as i32;
    v5 = kSinCosTable8bit_Sext[(v4 + 64 as i32) as usize];
    let mut v6: u16 = 0;
    let mut v8: u16 = 0;
    if v5 < 0i16 {
        if v5 == -256i16 {
            v6 = (ram_word!(game, grapple_beam_end_x_pos)).wrapping_sub(tmpD84);
        } else {
            let prod: u16 = Mult8x8(tmpD84 as u8, -(v5 as i8 as i32) as u8);
            v6 =
                (ram_word!(game, grapple_beam_end_x_pos) as i32 - (prod as i32 >> 8 as i32)) as u16;
        }
    } else if v5 == 256i16 {
        v6 = (tmpD84).wrapping_add(ram_word!(game, grapple_beam_end_x_pos));
    } else {
        let prod_0: u16 = Mult8x8(
            tmpD84 as u8,
            kSinCosTable8bit_Sext[(v4 + 64 as i32) as usize] as u8,
        );
        v6 = (ram_word!(game, grapple_beam_end_x_pos) as i32 + (prod_0 as i32 >> 8 as i32)) as u16;
    }
    set_ram_word!(game.grapple_beam_grapple_start_x = v6);
    set_ram_word!(game.grapple_beam_grapple_start_block_x = (v6 as i32 >> 4 as i32) as u8 as u16);
    v7 = kSinCosTable8bit_Sext[v4 as usize];
    if v7 < 0i16 {
        if v7 == -256i16 {
            v8 = (ram_word!(game, grapple_beam_end_y_pos)).wrapping_sub(tmpD84);
        } else {
            let prod_1: u16 = Mult8x8(tmpD84 as u8, -(v7 as i32) as u8);
            v8 = (ram_word!(game, grapple_beam_end_y_pos) as i32 - (prod_1 as i32 >> 8 as i32))
                as u16;
        }
    } else if v7 == 256i16 {
        v8 = (tmpD84).wrapping_add(ram_word!(game, grapple_beam_end_y_pos));
    } else {
        let prod_2: u16 = Mult8x8(tmpD84 as u8, kSinCosTable8bit_Sext[v4 as usize] as u8);
        v8 = (ram_word!(game, grapple_beam_end_y_pos) as i32 + (prod_2 as i32 >> 8 as i32)) as u16;
    }
    set_ram_word!(game.grapple_beam_grapple_start_y = v8);
    set_ram_word!(game.grapple_beam_grapple_start_block_y = (v8 as i32 >> 4 as i32) as u8 as u16);
}
unsafe fn BlockReact_AA64(game: &mut Game) -> u8 {
    let v0: u16 = ram_word!(game, grapple_beam_grapple_start_block_x);
    let RegWord: u16 = Mult8x8(
        ram_word!(game, grapple_beam_grapple_start_block_y) as u8,
        ram_word!(game, room_width_in_blocks) as u8,
    );
    set_ram_word!(game.cur_block_index = (RegWord).wrapping_add(v0));
    let mut ci: CollInfo = CollInfo {
        ci_r18_r20: 0 as i32,
        ci_r24: 0,
        ci_r26: 0,
        ci_r28: 0,
        ci_r30: 0,
        ci_r32: 0,
        ci_r38: 0,
        ci_r40: 0,
    };
    game.react_to_block(|block_type, game| match block_type {
        0 | 3 | 4 | 6 | 7 => 0,
        1 | 8 | 9 | 11 | 12 | 14 | 15 => 1,
        2 => BlockReact_AA64_SpikeAir(&raw mut ci, game),
        5 => BlockReact_HorizExt(&raw mut ci, game),
        10 => BlockReact_AA64_SpikeBlock(&raw mut ci, game),
        13 => BlockReact_VertExt(&raw mut ci, game),
        _ => unreachable!(),
    })
}
static g_word_94AAF7: [u16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
unsafe fn BlockReact_AA64_SpikeAir(ci: *mut CollInfo, game: &mut Game) -> u8 {
    if ram_word!(game, samus_invincibility_timer) != 0 {
        return 0 as u8;
    }
    let v0: i16 = *((&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as *mut u8
        as *mut u16) as i16;
    if v0 >= 0i16 {
        let v: u32 = ((g_word_94AAF7[v0 as usize] as i16 as i32) << 16 as i32) as u32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut u8).offset(0xa50) as *mut u16,
                (&raw mut game.g_ram as *mut u8).offset(0xa4e) as *mut u16,
                v,
            );
            set_ram_word!(game.samus_invincibility_timer = 60 as u16);
            set_ram_word!(game.samus_knockback_timer = 10 as u16);
        }
    }
    0 as u8
}
static g_word_94AB70: [u16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
unsafe fn BlockReact_AA64_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    if ram_word!(game, samus_invincibility_timer) != 0 {
        return 1 as u8;
    }
    let v0: i16 = *((&raw mut game.g_ram as *mut u8)
        .offset(0x16402)
        .offset(ram_word!(game, cur_block_index) as isize) as *mut u8
        as *mut u16) as i16;
    if v0 >= 0i16 {
        let v: u32 = ((g_word_94AB70[v0 as usize] as i16 as i32) << 16 as i32) as u32;
        if v != 0 {
            AddToHiLo(
                (&raw mut game.g_ram as *mut u8).offset(0xa50) as *mut u16,
                (&raw mut game.g_ram as *mut u8).offset(0xa4e) as *mut u16,
                v,
            );
            set_ram_word!(game.samus_invincibility_timer = 60 as u16);
            set_ram_word!(game.samus_knockback_timer = 10 as u16);
        }
    }
    1 as u8
}
unsafe fn BlockFunc_ABB0(game: &mut Game) -> u8 {
    BlockReact_AA64(game)
}
unsafe fn BlockFunc_ABE6(tmpD82: u16, game: &mut Game) -> u8 {
    let mut v0: u8 = 0;
    set_ram_word!(game.g_word_7E0D98 = 6 as u16);
    let mut grapple_beam_tmpD84: u16 = (ram_word!(game, grapple_beam_length)).wrapping_add(8u16);
    loop {
        BlockFunc_A957(tmpD82, grapple_beam_tmpD84, game);
        v0 = (BlockFunc_ABB0(game)) & (1u8);
        if v0 != 0 {
            break;
        }
        grapple_beam_tmpD84 = (grapple_beam_tmpD84).wrapping_add(8u16);
        if update_ram_word!(game.g_word_7E0D98, wrapping_sub(1), reread) == 0 {
            return 0 as u8;
        }
    }
    v0
}
pub unsafe fn BlockFunc_AC11(game: &mut Game) {
    let grapple_beam_tmpD82: u16 =
        (2u16).wrapping_mul((*(&raw mut game.g_ram as *mut u8).offset(0xcfb)) as u16);
    let grapple_beam_tmpD84: u16 = ram_word!(game, grapple_beam_length);
    BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
    set_ram_word!(
        game.x_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_grapple_start_x)
    );
    set_ram_word!(
        game.y_pos_of_start_of_grapple_beam = ram_word!(game, grapple_beam_grapple_start_y)
    );
}
pub unsafe fn BlockFunc_AC31(game: &mut Game) -> u8 {
    if ram_word!(game, grapple_beam_length_delta) == 0 {
        return 0 as u8;
    }
    let mut end: u16 = (ram_word!(game, grapple_beam_length_delta))
        .wrapping_add(ram_word!(game, grapple_beam_length));
    if (ram_word!(game, grapple_beam_length_delta) as i32 & 0x8000 as i32) != 0i32 {
        if end < 8u16 {
            set_ram_word!(game.grapple_beam_length_delta = 0 as u16);
            end = 8 as u16;
        }
        let increment: i32 = 8 as i32;
        let grapple_beam_tmpD82: u16 =
            (2u16).wrapping_mul((*(&raw mut game.g_ram as *mut u8).offset(0xcfb)) as u16);
        let mut v1: u16 = ram_word!(game, grapple_beam_length);
        while v1 != end {
            let grapple_beam_tmpD84: u16 = (increment + v1 as i32 - 1 as i32) as u16;
            BlockFunc_A957(grapple_beam_tmpD82, grapple_beam_tmpD84, game);
            if BlockReact_AA64(game) != 0 {
                set_ram_word!(game.grapple_beam_length = v1);
                return 1 as u8;
            }
            v1 = v1.wrapping_sub(1);
        }
        set_ram_word!(game.grapple_beam_length = end);
        0 as u8
    } else {
        if end >= 63u16 {
            set_ram_word!(game.grapple_beam_length_delta = 0 as u16);
            end = 63 as u16;
        }
        let increment_0: i32 = 56 as i32;
        let grapple_beam_tmpD82_0: u16 =
            (2u16).wrapping_mul((*(&raw mut game.g_ram as *mut u8).offset(0xcfb)) as u16);
        let mut v1_0: u16 = ram_word!(game, grapple_beam_length);
        while v1_0 != end {
            let grapple_beam_tmpD84_0: u16 = (increment_0 + v1_0 as i32 + 1 as i32) as u16;
            BlockFunc_A957(grapple_beam_tmpD82_0, grapple_beam_tmpD84_0, game);
            if BlockReact_AA64(game) != 0 {
                set_ram_word!(game.grapple_beam_length = v1_0);
                return 1 as u8;
            }
            v1_0 = v1_0.wrapping_add(1);
        }
        set_ram_word!(game.grapple_beam_length = end);
        0 as u8
    }
}
pub unsafe fn HandleMovementAndCollForSamusGrapple(game: &mut Game) -> u8 {
    let mut v2: u16 = 0;
    let mut v5: u16 = 0;
    let mut current_block: u64;
    let mut prod16: u16 = 0;
    let mut v4: i16 = 0;
    let mut v7: i16 = 0;
    let mut v3: u16 = 0;
    let mut v6: u16 = 0;
    let mut grapple_beam_tmpD86: u16 = 0;
    let mut grapple_beam_tmpD88: u16 = 0;
    let mut tmpD9C: u16 = 0;
    let mut v0: u16 = 256 as u16;
    if (ram_word!(game, grapple_beam_flags) as i32 & 1 as i32) != 0i32 {
        v0 = 160 as u16;
    }
    let v1: u16 =
        (ram_word!(game, grapple_beam_unkD2E)).wrapping_add(ram_word!(game, grapple_beam_unkD26));
    if ((ram_word!(game, grapple_beam_unkD2E)) as i16)
        .wrapping_add((ram_word!(game, grapple_beam_unkD26)) as i16)
        >= 0i16
    {
        prod16 = (Multiply16x16(v1, v0, game) >> 8 as i32) as u16;
        if prod16 == 0 {
            return 0 as u8;
        }
        tmpD9C = prod16;
        grapple_beam_tmpD88 = (2 as i32
            * (((ram_word!(game, grapple_beam_end_angle16)).wrapping_add(prod16)) as i32
                >> 8 as i32)) as u16;
        v2 = 0;
        v2 = (2u16).wrapping_mul((*(&raw mut game.g_ram as *mut u8).offset(0xcfb)) as u16);
        if v2 != grapple_beam_tmpD88 {
            loop {
                grapple_beam_tmpD86 = v2;
                let grapple_beam_tmpD82: u16 = (v2 as i32 + 2 as i32 & 0x1ff as i32) as u16;
                if BlockFunc_ABE6(grapple_beam_tmpD82, game) as i32 & 1 as i32 != 0 {
                    current_block = 7976072742316086414;
                    break;
                }
                v2 = (grapple_beam_tmpD86 as i32 + 2 as i32 & 0x1ff as i32) as u16;
                if v2 == grapple_beam_tmpD88 {
                    current_block = 16784897620381762466;
                    break;
                }
            }
            if current_block == 16784897620381762466 {
            } else {
                *(&raw mut v4 as *mut u8).offset(0) = 0 as u8;
                *(&raw mut v4 as *mut u8).offset(
                    (::core::mem::size_of::<i16>())
                        .wrapping_div(::core::mem::size_of::<u8>())
                        .wrapping_sub(1_usize) as isize,
                ) = (grapple_beam_tmpD86 as i32 >> 1 as i32) as u8;
                set_ram_word!(game.grapple_beam_end_angle16 = ((v4) as u16) | (0x80u16));
                set_ram_word!(game.grapple_beam_end_angles_mirror = ((v4) as u16) | (0x80u16));
                if ram_word!(game, g_word_7E0D98) != 6u16 && ram_word!(game, g_word_7E0D98) != 5u16
                    || ram_word!(game, grapple_beam_length) != 8u16
                {
                    current_block = 15145243721762997867;
                } else {
                    current_block = 4207414038393540328;
                }
            }
        } else {
            current_block = 16784897620381762466;
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                update_ram_word!(game.grapple_beam_end_angle16, wrapping_add(tmpD9C));
                set_ram_word!(
                    game.grapple_beam_end_angles_mirror = ram_word!(game, grapple_beam_end_angle16)
                );
                update_ram_word!(game.grapple_beam_unkD36, |fresh5| (fresh5 as i32
                    & !(0x8000 as i32))
                    as u16);
                if (update_ram_word!(game.grapple_beam_unkD30, wrapping_sub(1), reread) as i32
                    & 0x8000 as i32)
                    != 0i32
                {
                    set_ram_word!(game.grapple_beam_unkD30 = 0 as u16);
                }
                if (ram_word!(game, grapple_beam_unkD2E) as i32 & 0x8000 as i32) == 0i32 {
                    v3 = (ram_word!(game, grapple_beam_unkD2E)).wrapping_sub(6u16);
                    if ((ram_word!(game, grapple_beam_unkD2E)) as i16).wrapping_sub(6i16) >= 0i16 {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                } else {
                    v3 = (ram_word!(game, grapple_beam_unkD2E)).wrapping_add(6u16);
                    if ((ram_word!(game, grapple_beam_unkD2E)) as i16).wrapping_add(6i16) < 0i16 {
                        current_block = 3103951223134113261;
                    } else {
                        current_block = 18377268871191777778;
                    }
                }
                if current_block == 18377268871191777778 {
                    v3 = 0 as u16;
                }
                set_ram_word!(game.grapple_beam_unkD2E = v3);
                return 0 as u8;
            }
        }
    } else {
        prod16 = (Multiply16x16(-(v1 as i32) as u16, v0, game) >> 8 as i32) as u16;
        if prod16 == 0 {
            return 0 as u8;
        }
        tmpD9C = -(prod16 as i32) as u16;
        grapple_beam_tmpD88 = (2 as i32
            * (((ram_word!(game, grapple_beam_end_angle16)).wrapping_sub(prod16)) as i32
                >> 8 as i32)) as u16;
        v5 = 0;
        v5 = (2u16).wrapping_mul((*(&raw mut game.g_ram as *mut u8).offset(0xcfb)) as u16);
        if v5 == grapple_beam_tmpD88 {
            current_block = 5091811861639934416;
        } else {
            current_block = 11777552016271000781;
        }
        loop {
            if current_block == 11777552016271000781 {
                grapple_beam_tmpD86 = v5;
                let grapple_beam_tmpD82_0: u16 = (v5 as i32 - 2 as i32 & 0x1ff as i32) as u16;
                if BlockFunc_ABE6(grapple_beam_tmpD82_0, game) as i32 & 1 as i32 != 0 {
                    *(&raw mut v7 as *mut u8).offset(0) = 0 as u8;
                    *(&raw mut v7 as *mut u8).offset(
                        (::core::mem::size_of::<i16>())
                            .wrapping_div(::core::mem::size_of::<u8>())
                            .wrapping_sub(1_usize) as isize,
                    ) = (grapple_beam_tmpD86 as i32 >> 1 as i32) as u8;
                    set_ram_word!(game.grapple_beam_end_angle16 = ((v7) as u16) | (0x80u16));
                    set_ram_word!(game.grapple_beam_end_angles_mirror = ((v7) as u16) | (0x80u16));
                    if (ram_word!(game, g_word_7E0D98) == 6u16
                        || ram_word!(game, g_word_7E0D98) == 5u16)
                        && ram_word!(game, grapple_beam_length) == 8u16
                    {
                        current_block = 4207414038393540328;
                        break;
                    } else {
                        current_block = 15145243721762997867;
                        break;
                    }
                } else {
                    v5 = (grapple_beam_tmpD86 as i32 - 2 as i32 & 0x1ff as i32) as u16;
                    if v5 == grapple_beam_tmpD88 {
                        current_block = 5091811861639934416;
                    } else {
                        current_block = 11777552016271000781;
                    }
                }
            } else {
                update_ram_word!(game.grapple_beam_end_angle16, wrapping_add(tmpD9C));
                set_ram_word!(
                    game.grapple_beam_end_angles_mirror = ram_word!(game, grapple_beam_end_angle16)
                );
                update_ram_word!(game.grapple_beam_unkD36, |fresh8| (fresh8 as i32
                    & !(0x8000 as i32))
                    as u16);
                if (update_ram_word!(game.grapple_beam_unkD30, wrapping_sub(1), reread) as i32
                    & 0x8000 as i32)
                    != 0i32
                {
                    set_ram_word!(game.grapple_beam_unkD30 = 0 as u16);
                }
                if (ram_word!(game, grapple_beam_unkD2E) as i32 & 0x8000 as i32) == 0i32 {
                    current_block = 12997042908615822766;
                    break;
                } else {
                    current_block = 10399321362245223758;
                    break;
                }
            }
        }
        match current_block {
            15145243721762997867 => {}
            4207414038393540328 => {}
            _ => {
                if current_block == 10399321362245223758 {
                    v6 = (ram_word!(game, grapple_beam_unkD2E)).wrapping_add(6u16);
                    if ((ram_word!(game, grapple_beam_unkD2E)) as i16).wrapping_add(6i16) < 0i16 {
                        current_block = 9815132409580070441;
                    } else {
                        current_block = 13619784596304402172;
                    }
                } else {
                    v6 = (ram_word!(game, grapple_beam_unkD2E)).wrapping_sub(6u16);
                    if ((ram_word!(game, grapple_beam_unkD2E)) as i16).wrapping_sub(6i16) >= 0i16 {
                        current_block = 9815132409580070441;
                    } else {
                        current_block = 13619784596304402172;
                    }
                }
                if current_block == 13619784596304402172 {
                    v6 = 0 as u16;
                }
                set_ram_word!(game.grapple_beam_unkD2E = v6);
                return 0 as u8;
            }
        }
    }
    if current_block == 15145243721762997867 {
        set_ram_word!(game.grapple_beam_unkD30 = 16 as u16);
        set_ram_word!(
            game.grapple_beam_unkD26 =
                -(ram_word!(game, grapple_beam_unkD26) as i16 as i32 >> 1 as i32) as u16
        );
        set_ram_word!(
            game.grapple_beam_unkD2E =
                -(ram_word!(game, grapple_beam_unkD2E) as i16 as i32 >> 1 as i32) as u16
        );
        1 as u8
    } else {
        update_ram_word!(game.grapple_beam_unkD36, |fresh3| (fresh3) | (0x8000u16));
        set_ram_word!(game.grapple_beam_unkD26 = 0 as u16);
        set_ram_word!(game.grapple_beam_unkD2E = 0 as u16);
        1 as u8
    }
}
pub unsafe fn BlockFunc_AEE3(game: &mut Game) -> u8 {
    if ((ram_word!(game, grapple_beam_unkD26) as i32
        ^ ram_word!(game, grapple_beam_end_angle16) as i32)
        & 0x8000 as i32)
        != 0i32
    {
        set_ram_word!(game.grapple_beam_unkD38 = 0 as u16);
        1 as u8
    } else {
        if update_ram_word!(game.grapple_beam_unkD38, wrapping_add(1), reread) == 32u16 {
            set_ram_word!(game.grapple_beam_function = ((0x9bc8c5i32) as u16));
        }
        set_ram_word!(game.grapple_beam_unkD26 = 0 as u16);
        set_ram_word!(game.grapple_beam_unkD2E = 0 as u16);
        1 as u8
    }
}
pub unsafe fn GrappleFunc_AF87(game: &mut Game) {
    let mut i: i32 = 30 as i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1 as i32;
        set_ram_word!(game.grapple_segment_anim_instr_ptrs[v1] = addr_word_94B197 as u16);
        set_ram_word!(
            game.grapple_segment_anim_instr_timers[(v1 + 15i32)] = addr_word_94B193 as u16
        );
        set_ram_word!(
            game.grapple_segment_anim_instr_timers[(v1 + 14i32)] = addr_word_94B18F as u16
        );
        set_ram_word!(
            game.grapple_segment_anim_instr_timers[(v1 + 13i32)] = addr_word_94B18B as u16
        );
        set_ram_word!(game.grapple_segment_anim_instr_timers[v1] = 1 as u16);
        *(((&raw mut game.g_ram as *mut u8).offset(0xd40) as *mut u16 as *mut u8).offset(i as isize)
            as *mut u16) = 1 as u16;
        *(((&raw mut game.g_ram as *mut u8).offset(0xd3e) as *mut u16 as *mut u8).offset(i as isize)
            as *mut u16) = 1 as u16;
        *(((&raw mut game.g_ram as *mut u8).offset(0xd3c) as *mut u16 as *mut u8).offset(i as isize)
            as *mut u16) = 1 as u16;
        i -= 8 as i32;
    }
}
unsafe fn GrappleInstr_Goto(j: u16, game: &mut Game) -> u16 {
    *(RomPtr_94(j, game) as *mut u16)
}
pub unsafe fn CallGrappleInstr(ea: u32, j: u16, game: &mut Game) -> u16 {
    match ea {
        9744628 => GrappleInstr_Goto(j, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn HandleGrappleBeamGfx(game: &mut Game) {
    let v1: i32 = CalculateAngleFromXY(
        (ram_word!(game, grapple_beam_end_x_pos))
            .wrapping_sub(ram_word!(game, x_pos_of_start_of_grapple_beam_prevframe)),
        (ram_word!(game, grapple_beam_end_y_pos))
            .wrapping_sub(ram_word!(game, y_pos_of_start_of_grapple_beam_prevframe)),
        game,
    ) as i32;
    let r28_r26: u32 =
        ((kSinCosTable8bit_Sext[(v1 + 64 as i32) as usize] as i32) << 11 as i32) as u32;
    let r32_r30: u32 = ((kSinCosTable8bit_Sext[v1 as usize] as i32) << 11 as i32) as u32;
    let mut chr: u16 =
        ((*(&raw mut game.g_ram as *mut u8).offset(0xcfb) as i32 & 0x80 as i32) >> 1 as i32) as u16;
    chr = (chr as i32
        | 2 as i32
            * ((*(&raw mut game.g_ram as *mut u8).offset(0xcfb) as i32 ^ chr as i32) & 0x40 as i32
                ^ 0x40 as i32)) as u16;
    chr = ((chr as i32) << 8 as i32) as u16;
    let mut r20_r18: u32 = ((ram_word!(game, x_pos_of_start_of_grapple_beam_prevframe) as i32
        - ram_word!(game, layer1_x_pos) as i32
        - 4 as i32)
        << 16 as i32) as u32;
    let mut r24_r22: u32 = ((ram_word!(game, y_pos_of_start_of_grapple_beam_prevframe) as i32
        - ram_word!(game, layer1_y_pos) as i32
        - 4 as i32)
        << 16 as i32) as u32;
    if (ram_word!(game, grapple_beam_length) as i32 & 0x8000 as i32) != 0i32 {
        return;
    }
    let mut n: i32 =
        (ram_word!(game, grapple_beam_length) as i32 / 8 as i32 & 0xf as i32) - 1 as i32;
    let mut q: i32 = 15 as i32;
    loop {
        let mut i: i32 = 0;
        let ref mut fresh17 =
            *((&raw mut game.g_ram as *mut u8).offset(0xd42) as *mut u16).offset(q as isize);
        let fresh18 = *fresh17;
        *fresh17 = (*fresh17).wrapping_sub(1);
        if fresh18 == 1u16 {
            let mut v9: *const u16 = ::core::ptr::null::<u16>();
            i = ram_word!(game, grapple_segment_anim_instr_ptrs[q]) as i32;
            loop {
                v9 = RomPtr_94(i as u16, game) as *const u16;
                if (*v9 as i32 & 0x8000 as i32) == 0i32 {
                    break;
                }
                i = CallGrappleInstr(((*v9) as u32) | (0x940000u32), (i + 2 as i32) as u16, game)
                    as i32;
            }
            set_ram_word!(game.grapple_segment_anim_instr_timers[q] = *v9);
            set_ram_word!(game.grapple_segment_anim_instr_ptrs[q] = (i + 4 as i32) as u16);
        }
        if ((r24_r22 | r20_r18) & 0xff000000 as u32) != 0u32 {
            break;
        }
        let v12: u16 = *(RomPtr_94(
            (ram_word!(game, grapple_segment_anim_instr_ptrs[q])).wrapping_sub(2u16),
            game,
        ) as *mut u16);
        DrawGrappleOams(
            (r20_r18 >> 16 as i32) as u16,
            (r24_r22 >> 16 as i32) as u16,
            (chr) | (v12),
            game,
        );
        r20_r18 = r20_r18.wrapping_add(r28_r26);
        r24_r22 = r24_r22.wrapping_add(r32_r30);
        q -= 1;
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
    if ram_word!(game, samus_pose) as i32 == kPose_B2_FaceR_Grapple_Air as i32
        || ram_word!(game, samus_pose) as i32 == kPose_B3_FaceL_Grapple_Air as i32
    {
        DrawGrappleOams3(game);
    } else if (ram_word!(game, grapple_beam_end_y_pos) as i32
        - ram_word!(game, layer1_y_pos) as i32
        & 0xff00 as i32)
        == 0i32
    {
        DrawGrappleOams3(game);
    }
}
unsafe fn DrawGrappleOams(x_r20: u16, y_r24: u16, chr_r38: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, oam_next_ptr);
    let v2: *mut OamEnt = gOamEnt(ram_word!(game, oam_next_ptr) as i32, game);
    (*v2).xcoord = x_r20 as u8;
    (*v2).ycoord = y_r24 as u8;
    *(&raw mut (*v2).charnum as *mut u16) = chr_r38;
    set_ram_word!(game.oam_next_ptr = (v1).wrapping_add(4u16));
}
unsafe fn DrawGrappleOams3(game: &mut Game) {
    let idx: u16 = ram_word!(game, oam_next_ptr);
    let v2: *mut OamEnt = gOamEnt(idx as i32, game);
    let x: u16 = (ram_word!(game, grapple_beam_end_x_pos) as i32
        - ram_word!(game, layer1_x_pos) as i32
        - 4 as i32) as u16;
    (*v2).xcoord = x as u8;
    (*v2).ycoord = (ram_word!(game, grapple_beam_end_y_pos) as i32
        - ram_word!(game, layer1_y_pos) as i32
        - 4 as i32) as u8;
    *(&raw mut (*v2).charnum as *mut u16) = 14880 as u16;
    update_ram_word!(
        game.oam_ext[(idx as i32 >> 5 as i32)],
        |fresh19| (fresh19 as i32
            | ((x as i32 & 0x100 as i32) >> 8 as i32)
                << 2 as i32 * (idx as i32 >> 2 as i32 & 7 as i32)) as u16
    );
    set_ram_word!(game.oam_next_ptr = (idx).wrapping_add(4u16));
}
