//! Shaktool, Maridia's sand-digging saw creature: a chain of seven pieces,
//! a saw hand at each end, four arm pieces and a head (bank $AA:D911-$AA:DF5C).
//!
//! Piece 0, the primary piece, is initially the right saw hand; piece 6, the
//! final piece, the left one. The pieces occupy consecutive enemy slots, and
//! every piece names its primary piece's slot.
#![forbid(unsafe_code)]
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use crate::projectile_geometry::SINE_COSINE;
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess, slot, word};

/// What Shaktool needs beyond work RAM: its trigonometry tables.
pub trait Host: WorkRamAccess + CompatibilityRam {
    /// The sine at `angle` in Shaktool's table at $AA:E0BD.
    fn shaktool_sine(&self, angle: isize) -> u16;
    /// The negative cosine at `angle` in Shaktool's table at $AA:E03D.
    fn shaktool_negative_cosine(&self, angle: isize) -> u16;
}

part_view! {
    /// A piece's view of its slot.
    Piece: EnemyLayout8 {
        words: [
            facing_angle, neighbour_angle, neighbour_angle_delta,
            // The head's direction and flip flag, or a piece's curl angle.
            direction, primary_piece, function, movement_options, piece_index,
        ],
    }
}

/// Bytes between consecutive piece slots.
const PIECE: i32 = 64;
const PIECES: i32 = 7;
const HEAD: i32 = 3 * PIECE;
const FINAL_PIECE: i32 = 6 * PIECE;

/// `direction` bit 15 flips the head.
const FLIPPED: u16 = 0x8000;
/// `movement_options` bit 15: the chain moves anticlockwise.
const ANTICLOCKWISE: u16 = 0x8000;
/// Half a turn, and the whole-angle byte of an angle word.
const HALF_TURN: u16 = 0x8000;
const WHOLE_ANGLE: u16 = 0xff00;
/// Flipping centres the moved pieces in their pixels.
const HALF_PIXEL: u16 = 0x8000;

/// $AA:DEE9: each piece's neighbour angle delta when curling.
const CURL_ANGLE_DELTAS: [u16; 7] = [0, 0x20, 0x60, 0xc0, 0x140, 0x1a0, 0x1e0];

// The original reads the previous piece's position through the drawing queue
// (`enemy_drawing_queue`, $7E:0E84): from piece `k`, these word indices past
// `k / 2` reach the previous piece's x, x subpixel, y and y subpixel.
const PREVIOUS_X_POS: usize = 91;
const PREVIOUS_X_SUBPOS: usize = 92;
const PREVIOUS_Y_POS: usize = 93;
const PREVIOUS_Y_SUBPOS: usize = 94;
const _: () = {
    use crate::ram::word_arrays::enemy_drawing_queue;
    use core::mem::offset_of;
    const fn previous(field: usize) -> usize {
        (0x0f78 - PIECE) as usize + field
    }
    assert!(enemy_drawing_queue + 2 * PREVIOUS_X_POS == previous(offset_of!(Piece, base.x_pos)));
    assert!(
        enemy_drawing_queue + 2 * PREVIOUS_X_SUBPOS == previous(offset_of!(Piece, base.x_subpos))
    );
    assert!(enemy_drawing_queue + 2 * PREVIOUS_Y_POS == previous(offset_of!(Piece, base.y_pos)));
    assert!(
        enemy_drawing_queue + 2 * PREVIOUS_Y_SUBPOS == previous(offset_of!(Piece, base.y_subpos))
    );
};

/// The seven piece slots, from the primary piece's slot `primary`.
pub fn segments(primary: u16) -> impl Iterator<Item = i32> {
    (0..PIECES).map(move |piece| i32::from(primary) + piece * PIECE)
}

/// Move Shaktool piece 1px ($AA:D956): move piece `k` one pixel along angle
/// `a`. Was `Shaktool_D956`.
pub fn move_piece_one_pixel(k: u16, jp: EnemyScript, a: u16, game: &mut impl Host) -> EnemyScript {
    let piece = i32::from(k);
    let angle = a as u8 as usize;
    slot!(
        game,
        Piece[piece].base(x_pos, x_subpos) += ((SINE_COSINE[angle + 64] as i32) << 8) as u32
    );
    slot!(
        game,
        Piece[piece].base(y_pos, y_subpos) += ((SINE_COSINE[angle] as i32) << 8) as u32
    );
    jp
}

/// Shaktool pieces movement options = `a` ($AA:DB0E). Was `Shaktool_DB0E`.
pub fn set_movement_options(k: u16, a: u16, game: &mut impl Host) {
    let primary = slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        slot!(game, Piece[piece].movement_options = a);
    }
}

/// Shaktool pieces facing angle = `a` ($AA:DB27). Was `Shaktool_DB27`.
pub fn set_facing_angles(k: u16, a: u16, game: &mut impl Host) {
    let primary = slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        slot!(game, Piece[piece].facing_angle = a);
    }
}

/// Shaktool pieces neighbour angle = `a` ($AA:DB40). Was `Shaktool_DB40`.
pub fn set_neighbour_angles(k: u16, a: u16, game: &mut impl Host) {
    let primary = slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        slot!(game, Piece[piece].neighbour_angle = a);
    }
}

/// Flip Shaktool ($AA:DB59): flip the head, swap and reverse the neighbour
/// angles of the pieces at mirrored places, swap the saw hands' positions and
/// centre the arm pieces and head. Was `Shaktool_DB59`.
pub fn flip(k: u16, game: &mut impl Host) {
    let primary: u16 = slot!(game, Piece[k as i32].primary_piece);
    let head = i32::from(primary) + HEAD;
    slot!(
        game,
        Piece[head].direction = slot!(game, Piece[head].direction) ^ FLIPPED
    );
    for near_offset in [PIECE, 2 * PIECE, HEAD] {
        let far = i32::from(primary) + 7 * PIECE - near_offset;
        let far_angle: u16 = slot!(game, Piece[far].neighbour_angle);
        let near = i32::from(primary) + near_offset;
        slot!(
            game,
            Piece[far].neighbour_angle =
                (slot!(game, Piece[near].neighbour_angle) ^ HALF_TURN) & WHOLE_ANGLE
        );
        slot!(
            game,
            Piece[near].neighbour_angle = (far_angle ^ HALF_TURN) & WHOLE_ANGLE
        );
    }
    let last = i32::from(primary) + FINAL_PIECE;
    let first = i32::from(primary);
    macro_rules! swap_saw_hands {
        ($($field:ident),+) => {
            $(
                let value: u16 = slot!(game, Piece[last].base.$field);
                slot!(game, Piece[last].base.$field = slot!(game, Piece[first].base.$field));
                slot!(game, Piece[first].base.$field = value);
            )+
        };
    }
    swap_saw_hands!(x_subpos, y_subpos, x_pos, y_pos);
    for piece in segments(primary).skip(1).take(5) {
        slot!(game, Piece[piece].base.x_subpos = HALF_PIXEL);
    }
    for piece in segments(primary).skip(1).take(5) {
        slot!(game, Piece[piece].base.y_subpos = HALF_PIXEL);
    }
}

/// Set Shaktool piece neighbour angle delta due to block collision ($AA:DC07):
/// a quarter of the whole-angle difference between the facing and neighbour
/// angles, in the direction of travel. Was `Shaktool_DC07`.
pub fn set_collision_angle_delta(k: u16, game: &mut impl Host) {
    let piece = i32::from(k);
    let delta: u16 = if slot!(game, Piece[piece].movement_options) & ANTICLOCKWISE != 0 {
        slot!(game, Piece[piece].neighbour_angle)
            .wrapping_sub(slot!(game, Piece[piece].facing_angle))
    } else {
        slot!(game, Piece[piece].facing_angle)
            .wrapping_sub(slot!(game, Piece[piece].neighbour_angle))
    };
    slot!(
        game,
        Piece[piece].neighbour_angle_delta = 4u16.wrapping_mul(delta.to_ne_bytes()[1] as u16)
    );
}

/// The previous piece's whole and subpixel position words, high word first.
fn previous_position(queue: usize, high: usize, low: usize, game: &mut impl Host) -> u32 {
    let high = word!(game.enemy_drawing_queue[queue + high]);
    let low = word!(game.enemy_drawing_queue[queue + low]);
    (u32::from(high) << 16) | u32::from(low)
}

/// Position Shaktool piece relative to previous piece ($AA:DC2A): place piece
/// `k` at its neighbour angle from the previous piece. Was `Shaktool_DC2A`.
pub fn position_relative_to_previous(k: u16, game: &mut impl Host) {
    let piece = i32::from(k);
    let angle = slot!(byte game, Piece[piece].neighbour_angle[1]) as isize;
    let queue = usize::from(k) >> 1;
    slot!(
        game,
        Piece[piece].base(x_pos, x_subpos) =
            (((game.shaktool_sine(angle) as i16 as i32) << 8i32) as u32).wrapping_add(
                previous_position(queue, PREVIOUS_X_POS, PREVIOUS_X_SUBPOS, game)
            )
    );
    slot!(
        game,
        Piece[piece].base(y_pos, y_subpos) =
            (((game.shaktool_negative_cosine(angle) as i16 as i32) << 8i32) as u32).wrapping_add(
                previous_position(queue, PREVIOUS_Y_POS, PREVIOUS_Y_SUBPOS, game)
            )
    );
}

/// Set Shaktool pieces neighbour angle and angle delta for curling ($AA:DC6F):
/// every piece's neighbour angle becomes piece `k`'s facing angle, with the
/// curling angle deltas. Was `Shaktool_DC6F`.
pub fn set_curling_angles(k: u16, game: &mut impl Host) {
    let piece = i32::from(k);
    set_neighbour_angles(k, slot!(game, Piece[piece].facing_angle), game);
    let primary = slot!(game, Piece[piece].primary_piece);
    for (index, piece) in segments(primary).enumerate() {
        slot!(
            game,
            Piece[piece].neighbour_angle_delta = CURL_ANGLE_DELTAS[index]
        );
    }
}
