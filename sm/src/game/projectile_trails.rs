//! Checked trail state/content adapter and native OAM output.
//! ROM capture and native diagnostics remain explicit unsafe dependencies.
use crate::game::projectile_trail_emission::Half;
use crate::game::projectile_trail_playback::Instruction;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as a};
#[derive(Clone, Copy)]
pub(crate) struct Side {
    pub(crate) timer: usize,
    pub(crate) script: usize,
    pub(crate) tile: usize,
    pub(crate) x: usize,
    pub(crate) y: usize,
}
pub(crate) const SIDES: [Side; 2] = [
    Side {
        timer: a::projectiletrail_left_instr_timer,
        script: a::projectiletrail_left_instr_list_ptr,
        tile: a::projectiletrail_left_tile_and_attribs,
        x: a::projectiletrail_left_x_pos,
        y: a::projectiletrail_left_y_pos,
    },
    Side {
        timer: a::projectiletrail_right_instr_timer,
        script: a::projectiletrail_right_instr_list_ptr,
        tile: a::projectiletrail_right_tile_and_attribs,
        x: a::projectiletrail_right_x_pos,
        y: a::projectiletrail_right_y_pos,
    },
];

#[forbid(unsafe_code)]
pub(crate) fn style(kind: u16) -> Option<u16> {
    if kind & 0xf00 == 0 {
        Some(kind & 63)
    } else {
        let class = kind.to_ne_bytes()[1] & 15;
        (class < 3).then_some(u16::from(class) + 31)
    }
}
/// Original encoded cursor arithmetic: left keeps a wide counter, right wraps.
#[derive(Clone, Copy)]
pub(crate) struct ProgramCounter {
    value: i32,
    half: Half,
}
#[forbid(unsafe_code)]
impl ProgramCounter {
    pub(crate) fn new(cursor: u16, half: Half) -> Self {
        Self {
            value: i32::from(cursor),
            half,
        }
    }
    pub(crate) fn address(self) -> u16 {
        self.value as u16
    }
    pub(crate) fn following(self) -> u16 {
        (self.value + 4) as u16
    }
    pub(crate) fn instruction(self, value: u16) -> Instruction<Self> {
        if value & 0x8000 == 0 {
            return Instruction::Frame { duration: value };
        }
        let next = Self {
            value: match self.half {
                Half::Left => self.value + 2,
                Half::Right => i32::from((self.value as u16).wrapping_add(2)),
            },
            ..self
        };
        match value {
            0xb525 => Instruction::Move {
                target: Half::Left,
                delta_y: 1,
                next,
            },
            0xb587 => Instruction::Move {
                target: Half::Right,
                delta_y: 1,
                next,
            },
            0xb5b3 => Instruction::Move {
                target: Half::Left,
                delta_y: -1,
                next,
            },
            _ => Instruction::Unknown { next },
        }
    }
}
#[forbid(unsafe_code)]
fn native_word(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
}

#[forbid(unsafe_code)]
pub(crate) fn draw(pair: usize, side: usize, frozen: bool, game: &mut Game) {
    let saved = word!(game.oam_next_ptr);
    if (saved as i16).wrapping_sub(512) >= 0 {
        return;
    }
    let fields = SIDES[side];
    if frozen && word!(game, fields.timer + 2 * pair;) == 0 {
        return;
    }
    crate::verify_probe::hit(match (frozen, side) {
        (true, 0) => "trails.draw.frozen_left",
        (true, _) => "trails.draw.frozen_right",
        (false, 0) => "trails.draw.active_left",
        (false, _) => "trails.draw.active_right",
    });
    let x = word!(game, fields.x + 2 * pair;).wrapping_sub(word!(game.layer1_x_pos));
    if x & 0xff00 == 0 {
        let entry = 0x370 + usize::from(word!(game.oam_next_ptr));
        native_word(game, entry, x);
        // The first write can alias the live Y, camera, tile or cursor fields.
        let y = word!(game, fields.y + 2 * pair;).wrapping_sub(word!(game.layer1_y_pos));
        if y & 0xff00 == 0 {
            native_word(game, entry + 1, y);
            let tile = word!(game, fields.tile + 2 * pair;);
            native_word(game, entry + 2, tile);
            word!(game.oam_next_ptr = saved.wrapping_add(4));
        }
    }
}
