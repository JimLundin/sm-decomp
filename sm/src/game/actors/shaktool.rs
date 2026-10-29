//! Shaktool's seven pieces: shared settings, flipping, curling and placement.
#![forbid(unsafe_code)]
use crate::game::enemy_layouts::EnemyLayout8 as Shaktool;
use crate::game::enemy_script::EnemyScript;
use crate::game::prelude::*;
use crate::game::projectile_geometry::SINE_COSINE;
use crate::game::shaktool_tables;
use crate::ram_map::slot;

/// Neighbour angle deltas for curling, one per piece. Was `g_word_AADEE9`.
const CURL_ANGLE_DELTAS: [u16; 7] = [0, 0x20, 0x60, 0xc0, 0x140, 0x1a0, 0x1e0];

/// The seven piece slots, 64 bytes apart from the first piece's slot `base`.
pub(crate) fn segments(base: u16) -> impl Iterator<Item = i32> {
    [0, 64, 128, 192, 256, 320, 384]
        .into_iter()
        .map(move |offset| i32::from(base) + offset)
}

/// Move piece `k` one pixel along angle `a`. Was `Shaktool_D956`.
pub(crate) fn move_piece_one_pixel(
    k: u16,
    jp: EnemyScript,
    a: u16,
    game: &mut Game,
) -> EnemyScript {
    let e = i32::from(k);
    let angle = a as u8 as usize;
    slot!(
        game,
        Shaktool[e].base(x_pos, x_subpos) += ((SINE_COSINE[angle + 64] as i32) << 8) as u32
    );
    slot!(
        game,
        Shaktool[e].base(y_pos, y_subpos) += ((SINE_COSINE[angle] as i32) << 8) as u32
    );
    jp
}

/// Every piece's movement options = `a`. Was `Shaktool_DB0E`.
pub(crate) fn set_movement_options(k: u16, a: u16, game: &mut Game) {
    let base = slot!(game, Shaktool[k as i32].var_E);
    for segment in segments(base) {
        slot!(game, Shaktool[segment].parameter_1 = a);
    }
}

/// Every piece's facing angle = `a`. Was `Shaktool_DB27`.
pub(crate) fn set_facing_angles(k: u16, a: u16, game: &mut Game) {
    let base = slot!(game, Shaktool[k as i32].var_E);
    for segment in segments(base) {
        slot!(game, Shaktool[segment].var_A = a);
    }
}

/// Every piece's neighbour angle = `a`. Was `Shaktool_DB40`.
pub(crate) fn set_neighbour_angles(k: u16, a: u16, game: &mut Game) {
    let base = slot!(game, Shaktool[k as i32].var_E);
    for segment in segments(base) {
        slot!(game, Shaktool[segment].var_B = a);
    }
}

/// Reverse the chain: mirror neighbour angles end for end and swap the end
/// pieces' positions. Was `Shaktool_DB59`.
pub(crate) fn flip(k: u16, game: &mut Game) {
    let base: u16 = slot!(game, Shaktool[k as i32].var_E);
    let middle = i32::from(base) + 192;
    slot!(
        game,
        Shaktool[middle].var_D = slot!(game, Shaktool[middle].var_D) ^ 0x8000
    );
    for near_offset in [64, 128, 192] {
        let far = i32::from(base) + 448 - near_offset;
        let far_value: u16 = slot!(game, Shaktool[far].var_B);
        let near = i32::from(base) + near_offset;
        slot!(
            game,
            Shaktool[far].var_B = (slot!(game, Shaktool[near].var_B) ^ 0x8000) & 0xff00
        );
        slot!(game, Shaktool[near].var_B = (far_value ^ 0x8000) & 0xff00);
    }
    let far = i32::from(base) + 384;
    let near = i32::from(base);
    macro_rules! swap_endpoints {
        ($($field:ident),+) => {
            $(
                let value: u16 = slot!(game, Shaktool[far].base.$field);
                slot!(game, Shaktool[far].base.$field = slot!(game, Shaktool[near].base.$field));
                slot!(game, Shaktool[near].base.$field = value);
            )+
        };
    }
    swap_endpoints!(x_subpos, y_subpos, x_pos, y_pos);
    for segment in segments(base).skip(1).take(5) {
        slot!(game, Shaktool[segment].base.x_subpos = 0x8000);
    }
    for segment in segments(base).skip(1).take(5) {
        slot!(game, Shaktool[segment].base.y_subpos = 0x8000);
    }
}

/// Piece `k`'s neighbour angle delta after a block collision. Was `Shaktool_DC07`.
pub(crate) fn set_collision_angle_delta(k: u16, game: &mut Game) {
    let e = i32::from(k);
    let delta: u16 = if slot!(game, Shaktool[e].parameter_1) & 0x8000 != 0 {
        slot!(game, Shaktool[e].var_B).wrapping_sub(slot!(game, Shaktool[e].var_A))
    } else {
        slot!(game, Shaktool[e].var_A).wrapping_sub(slot!(game, Shaktool[e].var_B))
    };
    slot!(
        game,
        Shaktool[e].var_C = 4u16.wrapping_mul(delta.to_ne_bytes()[1] as u16)
    );
}

/// Place piece `k` at its neighbour angle from the previous piece, whose
/// position sits in the drawing queue. Was `Shaktool_DC2A`.
pub(crate) fn position_relative_to_previous(k: u16, game: &mut Game) {
    let e = i32::from(k);
    let angle = slot!(byte game, Shaktool[e].var_B[1]) as isize;
    let previous = i32::from(k) >> 1;
    slot!(
        game,
        Shaktool[e].base(x_pos, x_subpos) = (((shaktool_tables::sine(game, angle) as i16 as i32)
            << 8i32) as u32)
            .wrapping_add(words!((
                word!(game.enemy_drawing_queue[(previous + 91)]),
                word!(game.enemy_drawing_queue[(previous + 92)])
            )),)
    );
    slot!(
        game,
        Shaktool[e].base(y_pos, y_subpos) =
            (((shaktool_tables::negative_cosine(game, angle) as i16 as i32) << 8i32) as u32)
                .wrapping_add(words!((
                    word!(game.enemy_drawing_queue[(previous + 93)]),
                    word!(game.enemy_drawing_queue[(previous + 94)])
                )),)
    );
}

/// Every piece's neighbour angle = piece `k`'s facing angle, with the curling
/// angle deltas. Was `Shaktool_DC6F`.
pub(crate) fn set_curling_angles(k: u16, game: &mut Game) {
    let e = i32::from(k);
    set_neighbour_angles(k, slot!(game, Shaktool[e].var_A), game);
    let base = slot!(game, Shaktool[e].var_E);
    for (index, segment) in segments(base).enumerate() {
        slot!(game, Shaktool[segment].var_C = CURL_ANGLE_DELTAS[index]);
    }
}
