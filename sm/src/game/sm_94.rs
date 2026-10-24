use crate::game::actors::api as actor_api;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_90::*;
pub use crate::game::prelude::*;
use crate::game::projectile_room::Axis as ShotAxis;

use crate::game::gameplay_flag_runtime::CheckBossBitForCurArea;

use crate::game::room_mechanisms::spawn_collision;

use crate::snes::snes::snes_read;

pub use crate::types::Consts_94;

crate::game::constants::typed_constants! { Consts_94;
    addr_word_94B197 = 45463, addr_word_94B193 = 45459,
    addr_word_94B18F = 45455, addr_word_94B18B = 45451,
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct CollInfo {
        pub ci_r18_r20: i32;
        pub ci_r24, ci_r26, ci_r28, ci_r30, ci_r32, ci_r38, ci_r40: u16;
    }
}

/// Stack-owned sample state for checking Samus's edges after releasing grapple.
struct PostGrappleCollisionSample {
    remaining_span: u16,
    initial_span: u16,
    edge_position: u16,
}
pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnDoorTransitionFunction_HandleElevator = 0x82e17d,
    fnGrappleBeam_Func2 = 0x9bc8c5,
}
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
const RomPtr_94: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x94>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
}
#[inline]
unsafe fn get_DoorDef(a: u16, game: &mut Game) -> *mut DoorDef {
    unsafe { RomPtr((0x830000u32) | ((a) as u32), game) as *mut DoorDef }
}

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
pub(super) use crate::game::projectile_geometry::SLOPE_HEIGHTS as kAlignYPos_Tab0;
pub(super) use crate::game::projectile_geometry::SLOPE_QUADRANTS as kTab948E54;
pub(super) static kBlockColl_Horiz_Slope_NonSquare_Tab: [u16; 64] = [
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

macro_rules! check_grapple_square_slope_span {
    (
        $game:ident, $block_index:ident, $sample:ident, $orientation_shift:literal,
        $position:ident, $radius:ident, $sample_mask:literal, $distance:ident $(,)?
    ) => {
        let bts = $game.g_ram[0x16402 + usize::from($block_index)];
        let index = u16::from(bts & 0x1f) * 4
            + (u16::from(bts >> 6) ^ (($sample.edge_position & 8) >> $orientation_shift));
        if !square_slope_occupied(
            index,
            $sample.remaining_span,
            $sample.initial_span,
            $game.$position(),
            $game.$radius(),
            $sample_mask,
        ) {
            return 0xffff;
        }
        let $distance = $sample.edge_position & 7;
    };
}

fn PostGrappleColl_Horiz_Slope_Square(
    sample: &PostGrappleCollisionSample,
    k: u16,
    game: &mut Game,
) -> u16 {
    check_grapple_square_slope_span!(game, k, sample, 3, samus_y_pos, samus_y_radius, 2, distance);
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
    check_grapple_square_slope_span!(game, k, sample, 2, samus_x_pos, samus_x_radius, 1, distance);
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
        crate::game::samus_room_contact::span(game.samus_y_pos(), game.samus_y_radius())
    } else {
        crate::game::samus_room_contact::span(game.samus_x_pos(), game.samus_x_radius())
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
    do_until! { context.remaining_span & 0x8000 != 0;
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
pub(super) use crate::game::projectile_geometry::{
    square_slope_occupied, square_slope_single_block,
};

pub(crate) fn BlockReact_HorizExt(_ci: *mut CollInfo, game: &mut Game) -> u8 {
    if let Some(cell) = crate::game::projectile_block_reactions::redirect(
        ShotAxis::X,
        i16::from(game.current_block_bts() as i8),
        || game.room_width_in_blocks(),
        || game.cur_block_index(),
    ) {
        game.set_cur_block_index(cell);
        0xff
    } else {
        0
    }
}
pub(crate) fn BlockReact_VertExt(_ci: *mut CollInfo, game: &mut Game) -> u8 {
    if let Some(cell) = crate::game::projectile_block_reactions::redirect(
        ShotAxis::Y,
        i16::from(game.current_block_bts() as i8),
        || game.room_width_in_blocks(),
        || game.cur_block_index(),
    ) {
        game.set_cur_block_index(cell);
        0xff
    } else {
        0
    }
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
fn set_special_air_displacement(displacement: u16, game: &mut Game) {
    game.g_ram[0xb56..0xb58].copy_from_slice(&0u16.to_le_bytes());
    game.g_ram[0xb58..0xb5a].copy_from_slice(&displacement.to_le_bytes());
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
    unsafe {
        let sample = game.inside_block_reaction_samus_point();
        if sample == 1 {
            spawn_collision(addr_kPlmHeader_B6FF as u16, game);
        }
    }
}
const SPECIAL_AIR_PLM_TABLES: [u16; 8] = [
    0x9a06, 0x9a26, 0x9a46, 0x9a66, 0x9a86, 0x9aa6, 0x9ac6, 0x9ae6,
];
unsafe fn BlockInsideReact_SpecialAir(_ci: *mut CollInfo, game: &mut Game) -> u8 {
    unsafe {
        let bts = game.current_block_bts();
        if bts & 0x80 != 0 {
            let area = u16::from_le_bytes([game.g_ram[0x79f], game.g_ram[0x7a0]]);
            let table = SPECIAL_AIR_PLM_TABLES[usize::from(area)];
            let plm = game.read_rom_word(0x940000 | u32::from(table), usize::from(bts & 0x7f) * 2);
            spawn_collision(plm, game);
        } else {
            // Preserve the former eighty-entry dispatch table's invalid-BTS panic.
            assert!(bts < 80, "special-air BTS index out of bounds: {bts}");
            match bts {
                8 => crate::game::room_mechanisms::carry(
                    crate::game::room_mechanisms::Direction::Right,
                    game,
                ),
                9 => crate::game::room_mechanisms::carry(
                    crate::game::room_mechanisms::Direction::Left,
                    game,
                ),
                10 => BlockInsideReact_SpecialAir_10(game),
                11 => BlockInsideReact_SpecialAir_11(game),
                70 => BlockInsideReact_SpecialAir_70(game),
                _ => BlockInsideReact_SpecialAir_Default(game),
            }
        }
        0
    }
}
unsafe fn dispatch_block_inside(block_type: u16, ci: *mut CollInfo, game: &mut Game) -> u8 {
    unsafe {
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
}
unsafe fn react_inside_sample(x: u16, y: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn BlockInsideDetection(game: &mut Game) {
    unsafe {
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
unsafe fn BlockReactGrapple_GrappleBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    unsafe {
        static kBlockReactGrapple_GrappleBlockPlm: [u16; 4] = [0xd0d8, 0xd0dc, 0xd0e0, 0xd0d8];
        word!(game.grapple_beam_flags &= !(0x8000));
        if *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x80 != 0 {
            0
        } else {
            spawn_collision(
                kBlockReactGrapple_GrappleBlockPlm
                    [*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as usize],
                game,
            )
        }
    }
}
unsafe fn BlockReactGrapple_SpikeBlock(ci: *mut CollInfo, game: &mut Game) -> u8 {
    unsafe {
        static kBlockReactGrapple_SpikeBlockPlm: [u16; 16] = [
            0xd0e4, 0xd0e4, 0xd0e4, 0xd0e8, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
            0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4, 0xd0e4,
        ];
        if *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) & 0x80 != 0 {
            0
        } else {
            spawn_collision(
                kBlockReactGrapple_SpikeBlockPlm
                    [*ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as usize],
                game,
            )
        }
    }
}
pub unsafe fn BlockCollGrappleBeam(game: &mut Game) -> u8 {
    unsafe {
        let grapple_vel_x: i32 = (word!(game.grapple_beam_extension_x_velocity) as i16 as i32) << 6;
        let grapple_vel_y: i32 = (word!(game.grapple_beam_extension_y_velocity) as i16 as i32) << 6;
        let mut result: u8 = 0;
        let mut i: i32 = 0;
        while i < 4 {
            words!(
                game.grapple_beam_end_x_offset,
                grapple_beam_end_x_suboffset += grapple_vel_x as u32
            );
            words!(
                game.grapple_beam_end_y_offset,
                grapple_beam_end_y_suboffset += grapple_vel_y as u32
            );
            words!(
                game.grapple_beam_end_x_pos,
                grapple_beam_end_x_subpos =
                    (words!(game.grapple_beam_end_x_offset, grapple_beam_end_x_suboffset))
                        .wrapping_add(words!(game.samus_x_pos, samus_x_subpos),)
                        .wrapping_add(
                            ((word!(game.grapple_beam_origin_x_offset) as i32) << 16) as u32
                        )
            );
            words!(
                game.grapple_beam_end_y_pos,
                grapple_beam_end_y_subpos =
                    (words!(game.grapple_beam_end_y_offset, grapple_beam_end_y_suboffset))
                        .wrapping_add(words!(game.samus_y_pos, samus_y_subpos),)
                        .wrapping_add(
                            ((word!(game.grapple_beam_origin_y_offset) as i32) << 16) as u32
                        )
            );
            result = BlockReactGrapple(game);
            if (result & 0x40) != 0 && (result & 1) != 0 {
                word!(
                    game.grapple_beam_end_x_pos = (word!(game.grapple_beam_end_x_pos) & 0xfff0 | 8)
                );
                word!(
                    game.grapple_beam_end_y_pos = (word!(game.grapple_beam_end_y_pos) & 0xfff0 | 8)
                );
                return result;
            }
            i += 1;
        }
        result
    }
}
pub unsafe fn BlockReactGrapple(game: &mut Game) -> u8 {
    unsafe {
        let prod: u16 = Mult8x8(
            (word!(game.grapple_beam_end_y_pos) >> 4) as u8,
            word!(game.room_width_in_blocks) as u8,
        );
        let mut v0: i16 = (word!(game.grapple_beam_end_x_pos) >> 4) as i16;
        word!(game.cur_block_index = (prod).wrapping_add((v0) as u16));
        let mut ci: CollInfo = CollInfo {
            ci_r18_r20: 0,
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
            4 => crate::game::projectile_block_reactions_ram::trigger(&mut ci, false, false, game),
            5 => BlockReact_HorizExt(&raw mut ci, game),
            7 => crate::game::projectile_block_reactions_ram::trigger(&mut ci, true, false, game),
            10 => BlockReactGrapple_SpikeBlock(&raw mut ci, game),
            12 => crate::game::projectile_block_reactions_ram::trigger(&mut ci, false, true, game),
            13 => BlockReact_VertExt(&raw mut ci, game),
            14 => BlockReactGrapple_GrappleBlock(&raw mut ci, game),
            15 => crate::game::projectile_block_reactions_ram::trigger(&mut ci, true, true, game),
            _ => unreachable!(),
        })
    }
}
pub use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
// Read from byte offset 128 + byte_offset; retain the amplitude's low byte.
// The two bytes must be within the sine table. Odd byte offsets are allowed.
pub(crate) unsafe fn scale_sine_word(byte_offset: u16, amplitude: u16) -> u16 {
    unsafe {
        let table = (&raw const kSinCosTable8bit_Sext as *const i16).offset(64) as *const u8;
        let sample = u16::from_le_bytes([
            *table.offset(byte_offset as i32 as isize),
            *table.offset(byte_offset as i32 as isize).offset(1),
        ]);
        ((u32::from(sample) * u32::from(amplitude as u8)) >> 8) as u16
    }
}

pub unsafe fn GrappleFunc_AF87(game: &mut Game) {
    word_transfers!(game, (-8), 4;
        le(grapple_segment_anim_instr_ptrs, 15) <- value(addr_word_94B197 as u16),
        le(grapple_segment_anim_instr_timers, 30) <- value(addr_word_94B193 as u16),
        le(grapple_segment_anim_instr_timers, 29) <- value(addr_word_94B18F as u16),
        le(grapple_segment_anim_instr_timers, 28) <- value(addr_word_94B18B as u16),
        le(grapple_segment_anim_instr_timers, 15) <- value(1),
        native(0xd5e) <- value(1),
        native(0xd5c) <- value(1),
        native(0xd5a) <- value(1),
    );
}
unsafe fn GrappleInstr_Goto(j: u16, game: &mut Game) -> u16 {
    unsafe { *(RomPtr_94(j, game) as *mut u16) }
}
pub unsafe fn CallGrappleInstr(ea: u32, j: u16, game: &mut Game) -> u16 {
    unsafe {
        match ea {
            9744628 => GrappleInstr_Goto(j, game),
            _ => Unreachable(game) as u16,
        }
    }
}
pub unsafe fn HandleGrappleBeamGfx(game: &mut Game) {
    unsafe {
        let v1: i32 = actor_api::angle_from_delta(
            (word!(game.grapple_beam_end_x_pos))
                .wrapping_sub(word!(game.x_pos_of_start_of_grapple_beam_prevframe)),
            (word!(game.grapple_beam_end_y_pos))
                .wrapping_sub(word!(game.y_pos_of_start_of_grapple_beam_prevframe)),
        ) as i32;
        let r28_r26: u32 = ((kSinCosTable8bit_Sext[(v1 + 64) as usize] as i32) << 11) as u32;
        let r32_r30: u32 = ((kSinCosTable8bit_Sext[v1 as usize] as i32) << 11) as u32;
        let mut chr: u16 = ((*ram_ptr!(game; 0xcfb) as i32 & 0x80) >> 1) as u16;
        chr =
            (chr as i32 | 2 * ((*ram_ptr!(game; 0xcfb) as i32 ^ chr as i32) & 0x40 ^ 0x40)) as u16;
        chr = ((chr as i32) << 8) as u16;
        let mut r20_r18: u32 = ((word!(game.x_pos_of_start_of_grapple_beam_prevframe) as i32
            - word!(game.layer1_x_pos) as i32
            - 4i32)
            << 16) as u32;
        let mut r24_r22: u32 = ((word!(game.y_pos_of_start_of_grapple_beam_prevframe) as i32
            - word!(game.layer1_y_pos) as i32
            - 4i32)
            << 16) as u32;
        if (word!(game.grapple_beam_length) & 0x8000) != 0 {
            return;
        }
        let mut n: i32 = (word!(game.grapple_beam_length) as i32 / 8 & 0xf) - 1i32;
        let mut q: i32 = 15;
        do_until! { (n < 0);
            crate::update_place! { unaligned_word fresh17 [fresh18] = &raw mut (*(ram_ptr!(game, u16; 0xd42)).offset(q as isize)) => (u16::from_le(fresh17.read_unaligned())).wrapping_sub(1); }
            if fresh18 == 1 {
                let mut v9 = null::<u16>();
                let mut i: i32 = word!(game.grapple_segment_anim_instr_ptrs[q]) as i32;
                loop {
                    v9 = RomPtr_94(i as u16, game) as *const u16;
                    if (*v9 & 0x8000) == 0 {
                        break;
                    }
                    i = CallGrappleInstr(((*v9) as u32) | (0x940000u32), (i + 2) as u16, game) as i32;
                }
                word!(game.grapple_segment_anim_instr_timers[q] = *v9);
                word!(game.grapple_segment_anim_instr_ptrs[q] = (i + 4) as u16);
            }
            if ((r24_r22 | r20_r18) & 0xff000000) != 0 {
                break;
            }
            let v12: u16 = *(RomPtr_94(
                (word!(game.grapple_segment_anim_instr_ptrs[q])).wrapping_sub(2),
                game,
            ) as *mut u16);
            DrawGrappleOams(
                (r20_r18 >> 16) as u16,
                (r24_r22 >> 16) as u16,
                (chr) | (v12),
                game,
            );
            r20_r18 = r20_r18.wrapping_add(r28_r26);
            r24_r22 = r24_r22.wrapping_add(r32_r30);
            q -= 1;
            n -= 1;
        }
        if word!(game.samus_pose) as i32 == kPose_B2_FaceR_Grapple_Air as i32
            || word!(game.samus_pose) as i32 == kPose_B3_FaceL_Grapple_Air as i32
        {
            DrawGrappleOams3(game);
        } else if (word!(game.grapple_beam_end_y_pos) as i32 - word!(game.layer1_y_pos) as i32
            & 0xff00)
            == 0
        {
            DrawGrappleOams3(game);
        }
    }
}
unsafe fn DrawGrappleOams(x_r20: u16, y_r24: u16, chr_r38: u16, game: &mut Game) {
    unsafe {
        let v1: u16 = word!(game.oam_next_ptr);
        let v2 = gOamEnt(word!(game.oam_next_ptr) as i32, game);
        (*v2).xcoord = x_r20 as u8;
        (*v2).ycoord = y_r24 as u8;
        *(&raw mut (*v2).charnum as *mut u16) = chr_r38;
        word!(game.oam_next_ptr = (v1).wrapping_add(4));
    }
}
unsafe fn DrawGrappleOams3(game: &mut Game) {
    unsafe {
        let idx: u16 = word!(game.oam_next_ptr);
        let v2 = gOamEnt(idx as i32, game);
        let x: u16 = (word!(game.grapple_beam_end_x_pos) as i32
            - word!(game.layer1_x_pos) as i32
            - 4i32) as u16;
        (*v2).xcoord = x as u8;
        (*v2).ycoord = (word!(game.grapple_beam_end_y_pos) as i32
            - word!(game.layer1_y_pos) as i32
            - 4i32) as u8;
        *(&raw mut (*v2).charnum as *mut u16) = 14880;
        word!(
            bits_i32,
            game.oam_ext[(idx >> 5)] |= ((x as i32 & 0x100) >> 8) << 2 * (idx as i32 >> 2 & 7)
        );
        word!(game.oam_next_ptr = (idx).wrapping_add(4));
    }
}
