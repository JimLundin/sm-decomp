//! Shaktool, Maridia's sand-digging saw creature: a chain of seven pieces,
//! a saw hand at each end, four arm pieces and a head (bank $AA:D911-$AA:DF5C).
//!
//! Piece 0, the primary piece, is initially the right saw hand; piece 6, the
//! final piece, the left one. The pieces occupy consecutive enemy slots, and
//! every piece names its primary piece's slot.
use crate::actors::boss_lease::BossState;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::actors::state::Actor;
use crate::enemy_script::EnemyScript;
use crate::geometry::Position;
use crate::projectile_geometry::SINE_COSINE;
use crate::ram::word;

/// What the Shaktool pieces need beyond a boss's state: the bank $AA
/// functions they do not run themselves, and the angle between two actors.
pub trait Lease: BossState + SlotState {
    /// Run the bank $AA function at `address` for the current piece.
    fn run_function(&mut self, address: u32);
    /// The angle from actor `from` to actor `to`.
    fn angle_between(&mut self, from: u16, to: u16) -> u16;
}

slot_view! {
    /// A piece's view of its slot.
    Piece, PieceEdit {
        record: [
            facing_angle = var_a, neighbour_angle = var_b, neighbour_angle_delta = var_c,
            direction = var_d, primary_piece = var_e, function = var_f,
            movement_options = parameter_1, piece_index = parameter_2,
        ],
        extended: [],
    }
}

/// Bytes between consecutive piece slots.
const PIECE: u16 = 64;
const PIECES: u16 = 7;
const HEAD: u16 = 3 * PIECE;
const FINAL_PIECE: u16 = 6 * PIECE;

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
    use crate::actors::layouts::EnemyLayout104 as Piece;
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
pub fn segments(primary: u16) -> impl Iterator<Item = u16> {
    (0..PIECES).map(move |piece| primary + piece * PIECE)
}

/// Move Shaktool piece 1px ($AA:D956): move piece `k` one pixel along angle
/// `a`. Was `Shaktool_D956`.
pub fn move_piece_one_pixel<L: Lease>(
    k: u16,
    jp: EnemyScript,
    a: u16,
    game: &mut L,
) -> EnemyScript {
    let piece = k;
    let angle = a as u8 as usize;
    let amount: u32 = ((SINE_COSINE[angle + 64] as i32) << 8) as u32;
    PieceEdit::of(game, piece)
        .actor()
        .x
        .advance(amount.cast_signed());
    let amount: u32 = ((SINE_COSINE[angle] as i32) << 8) as u32;
    PieceEdit::of(game, piece)
        .actor()
        .y
        .advance(amount.cast_signed());
    jp
}

/// Shaktool pieces movement options = `a` ($AA:DB0E). Was `Shaktool_DB0E`.
pub fn set_movement_options<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = Piece::of(game, k).primary_piece();
    for piece in segments(primary) {
        *PieceEdit::of(game, piece).movement_options() = a;
    }
}

/// Shaktool pieces facing angle = `a` ($AA:DB27). Was `Shaktool_DB27`.
pub fn set_facing_angles<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = Piece::of(game, k).primary_piece();
    for piece in segments(primary) {
        *PieceEdit::of(game, piece).facing_angle() = a;
    }
}

/// Shaktool pieces neighbour angle = `a` ($AA:DB40). Was `Shaktool_DB40`.
pub fn set_neighbour_angles<L: Lease>(k: u16, a: u16, game: &mut L) {
    let primary = Piece::of(game, k).primary_piece();
    for piece in segments(primary) {
        *PieceEdit::of(game, piece).neighbour_angle() = a;
    }
}

/// Flip Shaktool ($AA:DB59): flip the head, swap and reverse the neighbour
/// angles of the pieces at mirrored places, swap the saw hands' positions and
/// centre the arm pieces and head. Was `Shaktool_DB59`.
pub fn flip<L: Lease>(k: u16, game: &mut L) {
    crate::probe::hit("shaktool.flip");
    let primary: u16 = Piece::of(game, k).primary_piece();
    let head = primary + HEAD;
    *PieceEdit::of(game, head).direction() ^= FLIPPED;
    for near_offset in [PIECE, 2 * PIECE, HEAD] {
        let far = primary + 7 * PIECE - near_offset;
        let far_angle: u16 = Piece::of(game, far).neighbour_angle();
        let near = primary + near_offset;
        let near_angle = Piece::of(game, near).neighbour_angle();
        *PieceEdit::of(game, far).neighbour_angle() = (near_angle ^ HALF_TURN) & WHOLE_ANGLE;
        *PieceEdit::of(game, near).neighbour_angle() = (far_angle ^ HALF_TURN) & WHOLE_ANGLE;
    }
    let last = primary + FINAL_PIECE;
    let first = primary;
    // The saw hands swap subpixels first, then pixels.
    let words: [fn(&mut Actor) -> &mut u16; 4] = [
        |actor| &mut actor.x.fraction,
        |actor| &mut actor.y.fraction,
        |actor| &mut actor.x.whole,
        |actor| &mut actor.y.whole,
    ];
    for word in words {
        let value = *word(PieceEdit::of(game, last).actor());
        let other = *word(PieceEdit::of(game, first).actor());
        *word(PieceEdit::of(game, last).actor()) = other;
        *word(PieceEdit::of(game, first).actor()) = value;
    }
    for piece in segments(primary).skip(1).take(5) {
        PieceEdit::of(game, piece).actor().x.fraction = HALF_PIXEL;
    }
    for piece in segments(primary).skip(1).take(5) {
        PieceEdit::of(game, piece).actor().y.fraction = HALF_PIXEL;
    }
}

/// Set Shaktool piece neighbour angle delta due to block collision ($AA:DC07):
/// a quarter of the whole-angle difference between the facing and neighbour
/// angles, in the direction of travel. Was `Shaktool_DC07`.
pub fn set_collision_angle_delta<L: Lease>(k: u16, game: &mut L) {
    let piece = k;
    let delta: u16 = if Piece::of(game, piece).movement_options() & ANTICLOCKWISE != 0 {
        Piece::of(game, piece)
            .neighbour_angle()
            .wrapping_sub(Piece::of(game, piece).facing_angle())
    } else {
        Piece::of(game, piece)
            .facing_angle()
            .wrapping_sub(Piece::of(game, piece).neighbour_angle())
    };
    *PieceEdit::of(game, piece).neighbour_angle_delta() =
        4u16.wrapping_mul(delta.to_ne_bytes()[1] as u16);
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
    let piece = k;
    let angle = isize::from(Piece::of(game, piece).neighbour_angle().to_le_bytes()[1]);
    let queue = usize::from(k) >> 1;
    let x =
        (((crate::shaktool_tables::sine(game, angle) as i16 as i32) << 8i32) as u32).wrapping_add(
            previous_position(queue, PREVIOUS_X_POS, PREVIOUS_X_SUBPOS, game),
        );
    set_position(game, piece, |actor| &mut actor.x, x);
    let y = (((crate::shaktool_tables::negative_cosine(game, angle) as i16 as i32) << 8i32) as u32)
        .wrapping_add(previous_position(
            queue,
            PREVIOUS_Y_POS,
            PREVIOUS_Y_SUBPOS,
            game,
        ));
    set_position(game, piece, |actor| &mut actor.y, y);
}

/// Store a 16.16 `position` to piece `k`'s axis, the subpixel first.
fn set_position<L: Lease>(
    game: &mut L,
    k: u16,
    axis: fn(&mut Actor) -> &mut Position,
    position: u32,
) {
    let mut piece = PieceEdit::of(game, k);
    let axis = axis(piece.actor());
    axis.fraction = position as u16;
    axis.whole = (position >> 16) as u16;
}

/// Set Shaktool pieces neighbour angle and angle delta for curling ($AA:DC6F):
/// every piece's neighbour angle becomes piece `k`'s facing angle, with the
/// curling angle deltas. Was `Shaktool_DC6F`.
pub fn set_curling_angles<L: Lease>(k: u16, game: &mut L) {
    let piece = k;
    set_neighbour_angles(k, Piece::of(game, piece).facing_angle(), game);
    let primary = Piece::of(game, piece).primary_piece();
    for (index, piece) in segments(primary).enumerate() {
        *PieceEdit::of(game, piece).neighbour_angle_delta() = CURL_ANGLE_DELTAS[index];
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

/// Run one of Shaktool's script instructions, keyed by address; `k` is the
/// piece.
pub fn instruction<L: Lease>(address: u32, k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    match address {
        11196721 => lower_away_from_projectile(k, jp, game),
        11196735 => raise_towards_projectile(k, jp, game),
        11196746 => lower(k, jp, game),
        11196755 => raise(k, jp, game),
        11196858 => reset_functions(k, jp, game),
        _ => unreachable!("unrecognized Shaktool instruction"),
    }
}

/// Main AI and hurt AI ($AA:DCA3): the piece's function. Was `Shaktool_Hurt`.
pub fn main_ai<L: Lease>(game: &mut L) {
    crate::probe::hit("shaktool.main_ai");
    let k = game.current_enemy();
    match Piece::of(game, k).function() {
        function::NOTHING => {}
        function::ARM_PIECE => arm_piece(k, game),
        function::HEAD => head(k, game),
        function::FINAL_PIECE => final_piece(k, game),
        other => game.run_function((u32::from(BANK) << 16) | u32::from(other)),
    }
}

/// Initialisation AI ($AA:DE43): piece `piece_index` from the piece table,
/// placed from the previous piece. Was `Shaktool_Init`.
pub fn initialize<L: Lease>(game: &mut L) {
    let piece = game.current_enemy();
    let offset = {
        let mut part = PieceEdit::of(game, piece);
        let actor = part.actor();
        actor.instruction_delay = 1;
        actor.timer = 0;
        *part.facing_angle() = 0;
        *part.direction() = 0;
        *part.piece_index()
    };
    let data = crate::shaktool_tables::piece(game, (offset as i32 >> 1) as isize);
    PieceEdit::of(game, piece).actor().properties |= data.properties;
    let primary = game.current_enemy().wrapping_sub(data.ram_offset);
    *PieceEdit::of(game, piece).primary_piece() = primary;
    let function =
        crate::cartridge::native_record(BANK, PIECE_FUNCTIONS, game).word(offset as usize);
    *PieceEdit::of(game, piece).function() = function;
    // The curl angle deltas are read at the byte offset, as the original did.
    let at = offset as usize;
    let curl = u16::from_ne_bytes([
        CURL_ANGLE_DELTAS[at / 2].to_ne_bytes()[at % 2],
        CURL_ANGLE_DELTAS[at.div_ceil(2)].to_ne_bytes()[(at + 1) % 2],
    ]);
    {
        let mut part = PieceEdit::of(game, piece);
        *part.neighbour_angle_delta() = curl.wrapping_sub(data.zero);
        *part.neighbour_angle() = data.neighbour_angle;
        let actor = part.actor();
        actor.instruction = data.instruction_list;
        actor.layer = data.layer;
    }
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
    let piece = game.current_enemy();
    if Piece::of(game, piece).actor().health == 0 {
        let primary: u16 = Piece::of(game, piece).primary_piece();
        for segment in segments(primary) {
            PieceEdit::of(game, segment).actor().properties = DEAD_PROPERTIES;
        }
    }
}

/// Instruction - reset Shaktool functions ($AA:D9BA). Was `Shaktool_Instr_1`.
pub fn reset_functions<L: Lease>(k: u16, jp: EnemyScript, game: &mut L) -> EnemyScript {
    let primary: u16 = Piece::of(game, k).primary_piece();
    for (index, segment) in segments(primary).enumerate() {
        let function = crate::shaktool_tables::piece(game, index as isize).function;
        *PieceEdit::of(game, segment).function() = function;
    }
    jp
}

/// The head's direction, which the unused projectile-relative moves follow.
fn head_direction<L: Lease>(k: u16, game: &mut L) -> u16 {
    let head = Piece::of(game, k).primary_piece() + HEAD;
    Piece::of(game, head).direction()
}

/// The facing angle's whole byte over the neighbour angle's low byte.
fn facing_whole_angle<L: Lease>(k: u16, game: &mut L) -> u16 {
    let piece = Piece::of(game, k);
    u16::from_ne_bytes([
        piece.facing_angle().to_le_bytes()[1],
        piece.neighbour_angle().to_le_bytes()[0],
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
    let piece = k;
    let delta: u16 = if (Piece::of(game, piece).movement_options() & CURLED) != 0 {
        let mut part = PieceEdit::of(game, piece);
        let facing = part.facing_angle();
        *facing = facing.wrapping_add(256);
        256
    } else {
        Piece::of(game, piece).neighbour_angle_delta()
    };
    let mut piece = PieceEdit::of(game, piece);
    let turn = if *piece.movement_options() & ANTICLOCKWISE != 0 {
        -(delta as i32)
    } else {
        delta as i32
    };
    let angle = piece.neighbour_angle();
    *angle = (*angle as i32 + turn) as u16;
}

/// Shaktool function - head ($AA:DCD7): an arm piece that aims between its
/// neighbours. Was `Shaktool_DCD7`.
pub fn head<L: Lease>(k: u16, game: &mut L) {
    arm_piece(k, game);
    let piece = k;
    let behind: u16 = (Piece::of(game, piece).neighbour_angle()) ^ HALF_TURN;
    let mut aim: i16 = (behind as i32
        + (((Piece::of(game, piece + PIECE).neighbour_angle()).wrapping_sub(behind)) as i32 >> 1))
        as i16;
    if (Piece::of(game, piece).direction() & FLIPPED) != 0 {
        aim = {
            let mut bytes = aim.to_ne_bytes();
            bytes[1] ^= 0x80;
            i16::from_ne_bytes(bytes)
        };
    }
    // The aim rounded to one of eight directions.
    let direction: u16 = ((aim.to_ne_bytes()[1] as i32 + 8) & 0xe0) as u16;
    {
        let mut head = PieceEdit::of(game, piece);
        let rounded = head.direction();
        *rounded = (*rounded & 0xff00) | (direction & 0xff);
    }
    let list = crate::shaktool_tables::head_direction(game, ((direction >> 4) >> 1) as isize)
        .instruction_list;
    let mut head = PieceEdit::of(game, piece);
    let actor = head.actor();
    actor.instruction = list;
    actor.instruction_delay = 1;
}

/// Shaktool function - final piece ($AA:DD25): moves as an arm piece; on a
/// block it flips once, then reverses, and head-bobs; else it curls. Was
/// `Shaktool_DD25`.
pub fn final_piece<L: Lease>(mut k: u16, game: &mut L) {
    let piece = k;
    let y: u16 = Piece::of(game, piece).actor().y.whole;
    let x: u16 = Piece::of(game, piece).actor().x.whole;
    arm_piece(k, game);
    let target_y: u16 = Piece::of(game, piece).actor().y.whole;
    let target_x: u16 = Piece::of(game, piece).actor().x.whole;
    {
        let mut part = PieceEdit::of(game, piece);
        part.actor().x.whole = x;
        part.actor().y.whole = y;
    }
    if game.move_horizontal(
        k,
        (((target_x as i16).wrapping_sub(x as i16)) as i32) << 16,
        crate::actors::host::Slopes::Ignore,
    ) || {
        PieceEdit::of(game, piece).actor().y.whole = y;
        game.move_vertical(k, (((target_y as i16).wrapping_sub(y as i16)) as i32) << 16)
    } {
        if (Piece::of(game, piece).movement_options() & FLIPPED_AT_BLOCK) != 0 {
            set_movement_options(
                k,
                (Piece::of(game, piece).movement_options() ^ ANTICLOCKWISE) & 0x8fff,
                game,
            );
        } else {
            {
                let mut part = PieceEdit::of(game, piece);
                part.actor().x.whole = x;
                part.actor().y.whole = y;
            }
            flip(k, game);
            set_movement_options(
                game.current_enemy(),
                (Piece::of(game, piece).movement_options()) | FLIPPED_AT_BLOCK,
                game,
            );
            set_movement_options(
                game.current_enemy(),
                (Piece::of(game, piece).movement_options()) & !CURLED,
                game,
            );
        }
        *PieceEdit::of(game, piece).direction() = 0;
        let primary: u16 = Piece::of(game, piece).primary_piece();
        let angle: u16 = game.angle_between(k, primary);
        let whole = ((angle as i32) << 8) as i16 as u16;
        let facing: u16 = if (Piece::of(game, piece).movement_options() & ANTICLOCKWISE) != 0 {
            whole.wrapping_sub(QUARTER_TURN)
        } else {
            whole.wrapping_add(QUARTER_TURN)
        };
        set_facing_angles(k, facing, game);
        for index in (0..PIECES).rev() {
            set_collision_angle_delta(k, game);
            let bobbing = k;
            *PieceEdit::of(game, bobbing).function() = function::NOTHING;
            let list = crate::shaktool_tables::piece(game, index as isize).head_bob;
            let mut part = PieceEdit::of(game, bobbing);
            let actor = part.actor();
            actor.instruction = list;
            actor.instruction_delay = 1;
            k = k.wrapping_sub(PIECE);
        }
    } else {
        {
            let mut part = PieceEdit::of(game, piece);
            part.actor().x.whole = target_x;
            part.actor().y.whole = target_y;
        }
        if (Piece::of(game, piece).movement_options() & CURLED) != 0 {
            let mut part = PieceEdit::of(game, piece);
            let facing = part.facing_angle();
            *facing = facing.wrapping_add(256);
        } else {
            if ((Piece::of(game, piece).facing_angle() ^ Piece::of(game, piece).neighbour_angle())
                & WHOLE_ANGLE)
                == 0
            {
                set_curling_angles(k, game);
                *PieceEdit::of(game, piece).direction() = CURL_RESTART;
                set_movement_options(
                    k,
                    (Piece::of(game, piece).movement_options()) & !FLIPPED_AT_BLOCK,
                    game,
                );
                let mut part = PieceEdit::of(game, piece);
                let curl = part.direction();
                *curl = ((curl.to_le_bytes()[1] as i32) << 8) as u16;
            }
            let curl: u16 = (Piece::of(game, piece).direction())
                .wrapping_add(Piece::of(game, piece).neighbour_angle_delta());
            *PieceEdit::of(game, piece).direction() = curl;
            if curl >= CURL_END {
                set_movement_options(
                    k,
                    (Piece::of(game, piece).movement_options()) | CURLED,
                    game,
                );
            }
        }
    }
}
