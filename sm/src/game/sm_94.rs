use crate::game::actors::api as actor_api;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_90::*;
pub use crate::game::prelude::*;
use crate::game::projectile_block_reactions as reactions;
use crate::game::projectile_room::Axis as ShotAxis;

use crate::game::room_mechanisms::spawn_collision;

pub use crate::types::Consts_94;

crate::game::constants::typed_constants! { Consts_94;
    addr_word_94B197 = 45463, addr_word_94B193 = 45459,
    addr_word_94B18F = 45455, addr_word_94B18B = 45451,
}

pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    fnDoorTransitionFunction_HandleElevator = 0x82e17d,
    fnGrappleBeam_Func2 = 0x9bc8c5,
}
const RomPtr_94: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x94>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
}

pub(super) use crate::game::projectile_geometry::SLOPE_HEIGHTS as kAlignYPos_Tab0;
pub(super) use crate::game::projectile_geometry::SLOPE_QUADRANTS as kTab948E54;
pub(super) static kBlockColl_Horiz_Slope_NonSquare_Tab: [u16; 64] = [
    0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0,
    0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0x1000, 0xb0, 0x1000, 0xb0, 0, 0x100, 0, 0x100,
    0x1000, 0xc0, 0, 0x100, 0x1000, 0xc0, 0x1000, 0xc0, 0x800, 0xd8, 0x800, 0xd8, 0x600, 0xf0,
    0x600, 0xf0, 0x600, 0xf0, 0x4000, 0x80, 0x4000, 0x80, 0x6000, 0x50, 0x6000, 0x50, 0x6000, 0x50,
];
pub(super) use crate::game::projectile_geometry::square_slope_occupied;

pub(crate) fn BlockReact_HorizExt(game: &mut Game) -> u8 {
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
pub(crate) fn BlockReact_VertExt(game: &mut Game) -> u8 {
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

fn BlockInsideReact_Slope(_game: &mut Game) -> u8 {
    0
}
fn BlockInsideReact_ShootableAir(game: &mut Game) -> u8 {
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
fn BlockInsideReact_SpikeAir(game: &mut Game) -> u8 {
    let bts = game.current_block_bts();
    // Keep the former sixteen-entry table's rejection of invalid BTS values.
    assert!(bts < 16, "spike-air BTS index out of bounds: {bts}");
    if bts == 2 {
        BlockInsideReact_SpikeAir_BTS2(game);
    }
    0
}
fn BlockInsideReact_Special_(game: &mut Game) -> u8 {
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
unsafe fn BlockInsideReact_SpecialAir(game: &mut Game) -> u8 {
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
unsafe fn dispatch_block_inside(block_type: u16, game: &mut Game) -> u8 {
    unsafe {
        match block_type {
            0 | 4 | 6 | 7 => BlockInsideReact_ShootableAir(game),
            1 => BlockInsideReact_Slope(game),
            2 => BlockInsideReact_SpikeAir(game),
            3 => BlockInsideReact_SpecialAir(game),
            5 => BlockReact_HorizExt(game),
            13 => BlockReact_VertExt(game),
            _ => BlockInsideReact_Special_(game),
        }
    }
}
unsafe fn react_inside_sample(x: u16, y: u16, game: &mut Game) {
    unsafe {
        CalculateBlockAt(x, y, 0, 0, game);
        game.react_to_block(|block_type, game| dispatch_block_inside(block_type, game));
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
unsafe fn BlockReactGrapple_GrappleBlock(game: &mut Game) -> u8 {
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
unsafe fn BlockReactGrapple_SpikeBlock(game: &mut Game) -> u8 {
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
                    game.grapple_beam_end_x_pos = word!(game.grapple_beam_end_x_pos) & 0xfff0 | 8
                );
                word!(
                    game.grapple_beam_end_y_pos = word!(game.grapple_beam_end_y_pos) & 0xfff0 | 8
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
        game.react_to_block(|block_type, game| match block_type {
            0 | 2 | 3 | 6 => 0,
            1 | 8 | 9 | 11 => 1,
            kind @ (4 | 7 | 12 | 15) => {
                let solid = matches!(kind, 12 | 15);
                if let Some(mechanism) = reactions::mechanism(matches!(kind, 7 | 15), solid, game) {
                    reactions::Definitions::spawn(game, mechanism.header);
                }
                u8::from(solid)
            }
            5 => BlockReact_HorizExt(game),
            10 => BlockReactGrapple_SpikeBlock(game),
            13 => BlockReact_VertExt(game),
            14 => BlockReactGrapple_GrappleBlock(game),
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
        do_until! { n < 0 ;
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
