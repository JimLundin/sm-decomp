//! Shaktool, Maridia's sand-digging saw creature: a chain of seven pieces,
//! a saw hand at each end, four arm pieces and a head (bank $AA:D911-$AA:DF5C).
//!
//! Piece 0, the primary piece, is initially the right saw hand; piece 6, the
//! final piece, the left one. The pieces occupy consecutive enemy slots, and
//! every piece names its primary piece's slot.
use crate::actors::boss_lease::BossLease;
use crate::actors::layouts::part_view;
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;
use crate::projectile_geometry::SINE_COSINE;
use crate::ram::{WorkRamAccess, word};

/// A lease on a game with Shaktool's effects.
pub trait Lease: BossLease<Game: Ai> {}
impl<T: BossLease<Game: Ai> + ?Sized> Lease for T {}

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
    use crate::ram::word_arrays::ENEMY_DRAWING_QUEUE;
    use core::mem::offset_of;
    const fn previous(field: usize) -> usize {
        (0x0f78 - PIECE) as usize + field
    }
    assert!(ENEMY_DRAWING_QUEUE + 2 * PREVIOUS_X_POS == previous(offset_of!(Piece, base.x_pos)));
    assert!(
        ENEMY_DRAWING_QUEUE + 2 * PREVIOUS_X_SUBPOS == previous(offset_of!(Piece, base.x_subpos))
    );
    assert!(ENEMY_DRAWING_QUEUE + 2 * PREVIOUS_Y_POS == previous(offset_of!(Piece, base.y_pos)));
    assert!(
        ENEMY_DRAWING_QUEUE + 2 * PREVIOUS_Y_SUBPOS == previous(offset_of!(Piece, base.y_subpos))
    );
};

/// The seven piece slots, from the primary piece's slot `primary`.
pub fn segments(primary: u16) -> impl Iterator<Item = i32> {
    (0..PIECES).map(move |piece| i32::from(primary) + piece * PIECE)
}

/// Move Shaktool piece 1px ($AA:D956): move piece `k` one pixel along angle
/// `a`. Was `Shaktool_D956`.
pub fn move_piece_one_pixel<L: Lease>(
    k: u16,
    jp: EnemyScript,
    a: u16,
    game: &mut L,
) -> EnemyScript {
    let piece = i32::from(k);
    let angle = a as u8 as usize;
    session_slot!(
        game,
        Piece[piece].base(x_pos, x_subpos) += ((SINE_COSINE[angle + 64] as i32) << 8) as u32
    );
    session_slot!(
        game,
        Piece[piece].base(y_pos, y_subpos) += ((SINE_COSINE[angle] as i32) << 8) as u32
    );
    jp
}

/// Shaktool pieces movement options = `a` ($AA:DB0E). Was `Shaktool_DB0E`.
pub fn set_movement_options<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = session_slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        session_slot!(game, Piece[piece].movement_options = a);
    }
}

/// Shaktool pieces facing angle = `a` ($AA:DB27). Was `Shaktool_DB27`.
pub fn set_facing_angles<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = session_slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        session_slot!(game, Piece[piece].facing_angle = a);
    }
}

/// Shaktool pieces neighbour angle = `a` ($AA:DB40). Was `Shaktool_DB40`.
pub fn set_neighbour_angles<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = session_slot!(game, Piece[k as i32].primary_piece);
    for piece in segments(primary) {
        session_slot!(game, Piece[piece].neighbour_angle = a);
    }
}

/// Flip Shaktool ($AA:DB59): flip the head, swap and reverse the neighbour
/// angles of the pieces at mirrored places, swap the saw hands' positions and
/// centre the arm pieces and head. Was `Shaktool_DB59`.
pub fn flip<L: Lease>(k: u16, game: &mut L) {
    let primary: u16 = session_slot!(game, Piece[k as i32].primary_piece);
    let head = i32::from(primary) + HEAD;
    session_slot!(
        game,
        Piece[head].direction = session_slot!(game, Piece[head].direction) ^ FLIPPED
    );
    for near_offset in [PIECE, 2 * PIECE, HEAD] {
        let far = i32::from(primary) + 7 * PIECE - near_offset;
        let far_angle: u16 = session_slot!(game, Piece[far].neighbour_angle);
        let near = i32::from(primary) + near_offset;
        session_slot!(
            game,
            Piece[far].neighbour_angle =
                (session_slot!(game, Piece[near].neighbour_angle) ^ HALF_TURN) & WHOLE_ANGLE
        );
        session_slot!(
            game,
            Piece[near].neighbour_angle = (far_angle ^ HALF_TURN) & WHOLE_ANGLE
        );
    }
    let last = i32::from(primary) + FINAL_PIECE;
    let first = i32::from(primary);
    macro_rules! swap_saw_hands {
        ($($field:ident),+) => {
            $(
                let value: u16 = session_slot!(game, Piece[last].base.$field);
                session_slot!(game, Piece[last].base.$field = session_slot!(game, Piece[first].base.$field));
                session_slot!(game, Piece[first].base.$field = value);
            )+
        };
    }
    swap_saw_hands!(x_subpos, y_subpos, x_pos, y_pos);
    for piece in segments(primary).skip(1).take(5) {
        session_slot!(game, Piece[piece].base.x_subpos = HALF_PIXEL);
    }
    for piece in segments(primary).skip(1).take(5) {
        session_slot!(game, Piece[piece].base.y_subpos = HALF_PIXEL);
    }
}

/// Set Shaktool piece neighbour angle delta due to block collision ($AA:DC07):
/// a quarter of the whole-angle difference between the facing and neighbour
/// angles, in the direction of travel. Was `Shaktool_DC07`.
pub fn set_collision_angle_delta<L: Lease>(k: u16, game: &mut L) {
    let piece = i32::from(k);
    let delta: u16 = if session_slot!(game, Piece[piece].movement_options) & ANTICLOCKWISE != 0 {
        session_slot!(game, Piece[piece].neighbour_angle)
            .wrapping_sub(session_slot!(game, Piece[piece].facing_angle))
    } else {
        session_slot!(game, Piece[piece].facing_angle)
            .wrapping_sub(session_slot!(game, Piece[piece].neighbour_angle))
    };
    session_slot!(
        game,
        Piece[piece].neighbour_angle_delta = 4u16.wrapping_mul(delta.to_ne_bytes()[1] as u16)
    );
}

/// The previous piece's whole and subpixel position words, high word first.
fn previous_position<L: Lease>(queue: usize, high: usize, low: usize, game: &mut L) -> u32 {
    let high = word!(game.ENEMY_DRAWING_QUEUE[queue + high]);
    let low = word!(game.ENEMY_DRAWING_QUEUE[queue + low]);
    (u32::from(high) << 16) | u32::from(low)
}

/// Position Shaktool piece relative to previous piece ($AA:DC2A): place piece
/// `k` at its neighbour angle from the previous piece. Was `Shaktool_DC2A`.
pub fn position_relative_to_previous<L: Lease>(k: u16, game: &mut L) {
    let piece = i32::from(k);
    let angle = session_slot!(byte game, Piece[piece].neighbour_angle[1]) as isize;
    let queue = usize::from(k) >> 1;
    session_slot!(
        game,
        Piece[piece].base(x_pos, x_subpos) =
            (((crate::shaktool_tables::sine(game, angle) as i16 as i32) << 8i32) as u32)
                .wrapping_add(previous_position(
                    queue,
                    PREVIOUS_X_POS,
                    PREVIOUS_X_SUBPOS,
                    game
                ))
    );
    session_slot!(
        game,
        Piece[piece].base(y_pos, y_subpos) =
            (((crate::shaktool_tables::negative_cosine(game, angle) as i16 as i32) << 8i32) as u32)
                .wrapping_add(previous_position(
                    queue,
                    PREVIOUS_Y_POS,
                    PREVIOUS_Y_SUBPOS,
                    game
                ))
    );
}

/// Set Shaktool pieces neighbour angle and angle delta for curling ($AA:DC6F):
/// every piece's neighbour angle becomes piece `k`'s facing angle, with the
/// curling angle deltas. Was `Shaktool_DC6F`.
pub fn set_curling_angles<L: Lease>(k: u16, game: &mut L) {
    let piece = i32::from(k);
    set_neighbour_angles(k, session_slot!(game, Piece[piece].facing_angle), game);
    let primary = session_slot!(game, Piece[piece].primary_piece);
    for (index, piece) in segments(primary).enumerate() {
        session_slot!(
            game,
            Piece[piece].neighbour_angle_delta = CURL_ANGLE_DELTAS[index]
        );
    }
}

/// Bank $AA Shaktool functions, stored in `function`.
pub mod function {
    pub const NOTHING: u16 = 0xdcaa;
    pub const ARM_PIECE: u16 = 0xdcac;
    pub const HEAD: u16 = 0xdcd7;
    pub const FINAL_PIECE: u16 = 0xdd25;
}

const BANK: u8 = 0xaa;
/// $AA:DEDB: each piece's function.
const PIECE_FUNCTIONS: u16 = 0xdedb;
/// `movement_options` bit 14: the chain has curled and turns as one; bit 13:
/// it flipped at the last block, so the next block reverses it instead.
const CURLED: u16 = 0x4000;
const FLIPPED_AT_BLOCK: u16 = 0x2000;
/// A curl ends when the curl angle passes F000h.
const CURL_END: u16 = 0xf000;
/// A curl restarts its angle at 78h whole steps.
const CURL_RESTART: u16 = 0x7800;
/// A quarter turn, between the final piece's facing and the primary piece.
const QUARTER_TURN: u16 = 0x4000;
/// Shot dead, every piece's properties become just bit 9, deleted.
const DEAD_PROPERTIES: u16 = 0x200;

/// What the Shaktool pieces' AI needs beyond the shared actor operations
/// and tables: its function dispatch.
pub trait Ai: BossLease<Game = Self> + WorkRamAccess {
    /// Run the bank $AA function at `address` for the current piece.
    fn run_function(&mut self, address: u32);
    /// The angle from actor `from` to actor `to`.
    fn angle_between(&mut self, from: u16, to: u16) -> u16;
}

/// Main AI and hurt AI ($AA:DCA3): the piece's function. Was `Shaktool_Hurt`.
pub fn main_ai<L: Lease>(game: &mut L) {
    let piece = i32::from(game.current_enemy());
    let address = (u32::from(BANK) << 16) | u32::from(session_slot!(game, Piece[piece].function));
    game.foreign(|game| game.run_function(address));
}

/// Initialisation AI ($AA:DE43): piece `piece_index` from the piece table,
/// placed from the previous piece. Was `Shaktool_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let piece = i32::from(game.current_enemy());
    session_slot!(game, Piece[piece].base.instruction_timer = 1);
    session_slot!(game, Piece[piece].base.timer = 0);
    session_slot!(game, Piece[piece].facing_angle = 0);
    session_slot!(game, Piece[piece].direction = 0);
    let offset: u16 = session_slot!(game, Piece[piece].piece_index);
    let data = crate::shaktool_tables::piece(game, (offset as i32 >> 1) as isize);
    session_slot!(
        game,
        Piece[piece].base.properties =
            (session_slot!(game, Piece[piece].base.properties)) | data.properties
    );
    session_slot!(
        game,
        Piece[piece].primary_piece = (game.current_enemy()).wrapping_sub(data.ram_offset)
    );
    session_slot!(
        game,
        Piece[piece].function =
            crate::cartridge::native_record(BANK, PIECE_FUNCTIONS, game).word(offset as usize)
    );
    // The curl angle deltas are read at the byte offset, as the original did.
    let at = offset as usize;
    let curl = u16::from_ne_bytes([
        CURL_ANGLE_DELTAS[at / 2].to_ne_bytes()[at % 2],
        CURL_ANGLE_DELTAS[at.div_ceil(2)].to_ne_bytes()[(at + 1) % 2],
    ]);
    session_slot!(
        game,
        Piece[piece].neighbour_angle_delta = curl.wrapping_sub(data.zero)
    );
    session_slot!(game, Piece[piece].neighbour_angle = data.neighbour_angle);
    session_slot!(
        game,
        Piece[piece].base.current_instruction = data.instruction_list
    );
    session_slot!(game, Piece[piece].base.layer = data.layer);
    if offset != 0 {
        position_relative_to_previous(game.current_enemy(), game);
    }
}

/// Enemy touch ($AA:DF2F). Was `Shaktool_Touch`.
pub fn touch<L: Lease>(game: &mut L) {
    game.contact_reaction(crate::actors::host::Death::Finalize);
}

/// Enemy shot ($AA:DF34): shot dead, every piece is deleted. Was
/// `Shaktool_Shot`.
pub fn shot<L: Lease>(game: &mut L) {
    game.shot_reaction(crate::actors::host::Shot::Finalize);
    let piece = i32::from(game.current_enemy());
    if session_slot!(game, Piece[piece].base.health) == 0 {
        let primary: u16 = session_slot!(game, Piece[piece].primary_piece);
        for segment in segments(primary) {
            session_slot!(game, Piece[segment].base.properties = DEAD_PROPERTIES);
        }
    }
}

/// Instruction - reset Shaktool functions ($AA:D9BA). Was `Shaktool_Instr_1`.
pub fn reset_functions<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let primary: u16 = session_slot!(game, Piece[k as i32].primary_piece);
    for (index, segment) in segments(primary).enumerate() {
        let function = crate::shaktool_tables::piece(game, index as isize).function;
        session_slot!(game, Piece[segment].function = function);
    }
    jp
}

/// The head's direction, which the unused projectile-relative moves follow.
fn head_direction<L: Lease>(k: u16, game: &mut L) -> u16 {
    let head = session_slot!(game, Piece[k as i32].primary_piece) as i32 + HEAD;
    session_slot!(game, Piece[head].direction)
}

/// The facing angle's whole byte over the neighbour angle's low byte.
fn facing_whole_angle<L: Lease>(k: u16, game: &mut L) -> u16 {
    let piece = k as i32;
    u16::from_ne_bytes([
        session_slot!(byte game, Piece[piece].facing_angle[1]),
        session_slot!(byte game, Piece[piece].facing_angle[2]),
    ])
}

/// Unused. Instruction - lower enemy 1px away from enemy projectile
/// ($AA:D931). Was `Shaktool_Instr_2`.
pub fn lower_away_from_projectile<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let angle = head_direction(k, game) ^ 0x80;
    move_piece_one_pixel(k, jp, angle, game)
}

/// Unused. Instruction - raise enemy 1px towards enemy projectile ($AA:D93F).
/// Was `Shaktool_Instr_3`.
pub fn raise_towards_projectile<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let angle = head_direction(k, game);
    move_piece_one_pixel(k, jp, angle, game)
}

/// Instruction - lower enemy 1px ($AA:D94A). Was `Shaktool_Instr_4`.
pub fn lower<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let angle = facing_whole_angle(k, game) ^ 0x80;
    move_piece_one_pixel(k, jp, angle, game)
}

/// Instruction - raise enemy 1px ($AA:D953). Was `Shaktool_Instr_5`.
pub fn raise<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let angle = facing_whole_angle(k, game);
    move_piece_one_pixel(k, jp, angle, game)
}

/// Shaktool function - arm piece ($AA:DCAC): follow the previous piece and
/// turn the neighbour angle, by a whole step once curled. Was `Shaktool_DCAC`.
pub fn arm_piece<L: Lease>(k: u16, game: &mut L) {
    position_relative_to_previous(k, game);
    let piece = i32::from(k);
    let delta: u16 = if (session_slot!(game, Piece[piece].movement_options) & CURLED) != 0 {
        session_slot!(
            game,
            Piece[piece].facing_angle =
                (session_slot!(game, Piece[piece].facing_angle)).wrapping_add(256)
        );
        256
    } else {
        session_slot!(game, Piece[piece].neighbour_angle_delta)
    };
    session_slot!(
        game,
        Piece[piece].neighbour_angle = (session_slot!(game, Piece[piece].neighbour_angle) as i32
            + if session_slot!(game, Piece[piece].movement_options) & ANTICLOCKWISE != 0 {
                -(delta as i32)
            } else {
                delta as i32
            }) as u16
    );
}

/// Shaktool function - head ($AA:DCD7): an arm piece that aims between its
/// neighbours. Was `Shaktool_DCD7`.
pub fn head<L: Lease>(k: u16, game: &mut L) {
    arm_piece(k, game);
    let piece = i32::from(k);
    let behind: u16 = (session_slot!(game, Piece[piece].neighbour_angle)) ^ HALF_TURN;
    let mut aim: i16 = (behind as i32
        + (((session_slot!(game, Piece[piece + PIECE].neighbour_angle)).wrapping_sub(behind))
            as i32
            >> 1)) as i16;
    if (session_slot!(game, Piece[piece].direction) & FLIPPED) != 0 {
        aim = {
            let mut bytes = aim.to_ne_bytes();
            bytes[1] ^= 0x80;
            i16::from_ne_bytes(bytes)
        };
    }
    // The aim rounded to one of eight directions.
    let direction: u16 = ((aim.to_ne_bytes()[1] as i32 + 8) & 0xe0) as u16;
    session_slot!(byte game, Piece[piece].direction[0] = direction as u8);
    session_slot!(
        game,
        Piece[piece].base.current_instruction =
            crate::shaktool_tables::head_direction(game, ((direction >> 4) >> 1) as isize)
                .instruction_list
    );
    session_slot!(game, Piece[piece].base.instruction_timer = 1);
}

/// Shaktool function - final piece ($AA:DD25): moves as an arm piece; on a
/// block it flips once, then reverses, and head-bobs; else it curls. Was
/// `Shaktool_DD25`.
pub fn final_piece<L: Lease>(mut k: u16, game: &mut L) {
    let piece = i32::from(k);
    let y: u16 = session_slot!(game, Piece[piece].base.y_pos);
    let x: u16 = session_slot!(game, Piece[piece].base.x_pos);
    arm_piece(k, game);
    let target_y: u16 = session_slot!(game, Piece[piece].base.y_pos);
    let target_x: u16 = session_slot!(game, Piece[piece].base.x_pos);
    session_slot!(game, Piece[piece].base.x_pos = x);
    session_slot!(game, Piece[piece].base.y_pos = y);
    if game.move_horizontal(
        k,
        (((target_x as i16).wrapping_sub(x as i16)) as i32) << 16,
        crate::actors::host::Slopes::Ignore,
    ) || {
        session_slot!(game, Piece[piece].base.y_pos = y);
        game.move_vertical(k, (((target_y as i16).wrapping_sub(y as i16)) as i32) << 16)
    } {
        if (session_slot!(game, Piece[piece].movement_options) & FLIPPED_AT_BLOCK) != 0 {
            set_movement_options(
                k,
                (session_slot!(game, Piece[piece].movement_options) ^ ANTICLOCKWISE) & 0x8fff,
                game,
            );
        } else {
            session_slot!(game, Piece[piece].base.x_pos = x);
            session_slot!(game, Piece[piece].base.y_pos = y);
            flip(k, game);
            set_movement_options(
                game.current_enemy(),
                (session_slot!(game, Piece[piece].movement_options)) | FLIPPED_AT_BLOCK,
                game,
            );
            set_movement_options(
                game.current_enemy(),
                (session_slot!(game, Piece[piece].movement_options)) & !CURLED,
                game,
            );
        }
        session_slot!(game, Piece[piece].direction = 0);
        let primary: u16 = session_slot!(game, Piece[piece].primary_piece);
        let angle: u16 = game.foreign(|game| game.angle_between(k, primary));
        let whole = ((angle as i32) << 8) as i16 as u16;
        let facing: u16 =
            if (session_slot!(game, Piece[piece].movement_options) & ANTICLOCKWISE) != 0 {
                whole.wrapping_sub(QUARTER_TURN)
            } else {
                whole.wrapping_add(QUARTER_TURN)
            };
        set_facing_angles(k, facing, game);
        for index in (0..PIECES).rev() {
            set_collision_angle_delta(k, game);
            let bobbing = i32::from(k);
            session_slot!(game, Piece[bobbing].function = function::NOTHING);
            session_slot!(
                game,
                Piece[bobbing].base.current_instruction =
                    crate::shaktool_tables::piece(game, index as isize).head_bob
            );
            session_slot!(game, Piece[bobbing].base.instruction_timer = 1);
            k = k.wrapping_sub(PIECE as u16);
        }
    } else {
        session_slot!(game, Piece[piece].base.x_pos = target_x);
        session_slot!(game, Piece[piece].base.y_pos = target_y);
        if (session_slot!(game, Piece[piece].movement_options) & CURLED) != 0 {
            session_slot!(
                game,
                Piece[piece].facing_angle =
                    (session_slot!(game, Piece[piece].facing_angle)).wrapping_add(256)
            );
        } else {
            if ((session_slot!(game, Piece[piece].facing_angle)
                ^ session_slot!(game, Piece[piece].neighbour_angle))
                & WHOLE_ANGLE)
                == 0
            {
                set_curling_angles(k, game);
                session_slot!(game, Piece[piece].direction = CURL_RESTART);
                set_movement_options(
                    k,
                    (session_slot!(game, Piece[piece].movement_options)) & !FLIPPED_AT_BLOCK,
                    game,
                );
                session_slot!(
                    game,
                    Piece[piece].direction =
                        ((session_slot!(byte game, Piece[piece].direction[1]) as i32) << 8) as u16
                );
            }
            let curl: u16 = (session_slot!(game, Piece[piece].direction))
                .wrapping_add(session_slot!(game, Piece[piece].neighbour_angle_delta));
            session_slot!(game, Piece[piece].direction = curl);
            if curl >= CURL_END {
                set_movement_options(
                    k,
                    (session_slot!(game, Piece[piece].movement_options)) | CURLED,
                    game,
                );
            }
        }
    }
}
