//! Checked trail state/content adapter and native OAM output.
//! ROM capture, Mode7 and native diagnostics remain explicit unsafe dependencies.
use crate::game::projectile_trail_emission::{self as emission, Context, Half, Owner, Point, Write};
use crate::game::projectile_trail_playback::{self as playback, Definitions, Instruction, Presentation};
use crate::game::projectile_playback_ram::trail_frame;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as a};
use crate::rom::RomRecord;
#[derive(Clone, Copy)]
struct Side {
    timer: usize,
    script: usize,
    tile: usize,
    x: usize,
    y: usize,
}
const SIDES: [Side; 2] = [
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

impl Owner for &mut Game {
    type Cursor = u16;
    type Sprite = u16;
    fn remaining(&self, pair: usize, half: Half) -> u16 {
        word!(self,SIDES[half as usize].timer+2*pair;)
    }
    fn cursor(&self, pair: usize, half: Half) -> u16 {
        word!(self,SIDES[half as usize].script+2*pair;)
    }
    fn y(&self, pair: usize, half: Half) -> u16 {
        word!(self,SIDES[half as usize].y+2*pair;)
    }
    fn write(&mut self, pair: usize, half: Half, value: Write<u16, u16>) {
        let fields = SIDES[half as usize];
        let (field, value) = match value {
            Write::Timer(v) => (fields.timer, v),
            Write::Cursor(v) => (fields.script, v),
            Write::Tile(v) => (fields.tile, v),
            Write::X(v) => (fields.x, v),
            Write::Y(v) => (fields.y, v),
        };
        word!(self,field+2*pair;=value);
    }
}
#[forbid(unsafe_code)]
pub(crate) fn style(kind: u16) -> Option<u16> {
    if kind & 0xf00 == 0 {
        Some(kind & 63)
    } else {
        let class = kind.to_ne_bytes()[1] & 15;
        (class < 3).then_some(u16::from(class) + 31)
    }
}
struct Offsets(RomRecord);
impl emission::Offsets for Offsets {
    fn left_y(&self) -> i8 {
        self.0.byte(1) as i8
    }
    fn left_x(&self) -> i8 {
        self.0.byte(0) as i8
    }
    fn right_y(&self) -> i8 {
        self.0.byte(3) as i8
    }
    fn right_x(&self) -> i8 {
        self.0.byte(2) as i8
    }
}
struct Ram {
    frozen: bool,
}
impl<'a> Context<&'a mut Game> for Ram {
    type Style = u16;
    type Marker = u16;
    type Offsets = Offsets;
    fn style(&self, game: &&mut Game, projectile: u16) -> Option<u16> {
        style(word!(game.projectile_type[projectile >> 1]))
    }
    fn script(&mut self, game: &'a mut Game, style: u16, half: Half) -> (&'a mut Game, u16) {
        let address = match half {
            Half::Left => 0x90b5bb,
            Half::Right => 0x90b609,
        };
        let cursor = crate::rtl::rom_table_word!(game, address, style as isize);
        (game, cursor)
    }
    fn selected_projectile(&self, game: &&mut Game) -> u16 {
        word!(game.projectile_index)
    }
    fn marker(&mut self, game: &'a mut Game, projectile: u16) -> (&'a mut Game, u16) {
        let marker = trail_frame(projectile, game);
        (game, marker)
    }
    fn origin(&mut self, game: &'a mut Game, projectile: u16) -> (&'a mut Game, Point) {
        let index = usize::from(projectile >> 1);
        let point = if word!(game.ceres_status) & 0x8000 == 0 {
            Point {
                x: word!(game.projectile_x_pos[index]),
                y: word!(game.projectile_y_pos[index]),
            }
        } else {
            let point = unsafe { crate::game::sm_8b::CalcExplosion_Mode7(projectile, game) };
            Point {
                x: word!(game.layer1_x_pos).wrapping_add(point.x),
                y: word!(game.layer1_y_pos).wrapping_add(point.y),
            }
        };
        (game, point)
    }
    fn offsets(
        &mut self,
        game: &'a mut Game,
        projectile: u16,
        marker: u16,
    ) -> (&'a mut Game, Offsets) {
        let index = usize::from(projectile >> 1);
        let kind = word!(game.projectile_type[index]);
        let table = if kind & 0x20 != 0 {
            0x9ba4e3
        } else if kind & 0x10 != 0 {
            0x9ba4cb
        } else {
            0x9ba4b3
        };
        let first = crate::rtl::rom_table_word!(game, table, (kind & 15) as isize)
            .wrapping_add(2 * (word!(game.projectile_dir[index]) & 15));
        let second = unsafe { crate::rtl::rom_bank_record::<0x9b>(first, game) }
            .word(0)
            .wrapping_add(marker.wrapping_mul(4));
        let data = unsafe { crate::rtl::rom_bank_record::<0x9b>(second, game) };
        (game, Offsets(data))
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
impl<'a> Definitions<&'a mut Game> for Ram {
    type Cursor = ProgramCounter;
    fn start(&mut self, cursor: u16, half: Half) -> ProgramCounter {
        ProgramCounter::new(cursor, half)
    }
    fn instruction(
        &mut self,
        game: &'a mut Game,
        cursor: ProgramCounter,
    ) -> (&'a mut Game, Instruction<ProgramCounter>) {
        let value = unsafe { crate::rtl::rom_bank_record::<0x90>(cursor.address(), game) }.word(0);
        (game, cursor.instruction(value))
    }
    fn frame(&mut self, game: &'a mut Game, cursor: ProgramCounter) -> (&'a mut Game, u16, u16) {
        let sprite = unsafe { crate::rtl::rom_bank_record::<0x90>(cursor.address(), game) }.word(2);
        (game, sprite, cursor.following())
    }
    fn unknown(&mut self, game: &'a mut Game) -> &'a mut Game {
        unsafe { crate::rtl::Unreachable(game) };
        game
    }
}
impl<'a> Presentation<&'a mut Game> for Ram {
    fn draw(&mut self, game: &'a mut Game, pair: usize, half: Half) -> &'a mut Game {
        draw(pair, half as usize, self.frozen, game);
        game
    }
}
pub fn emit(projectile: u16, game: &mut Game) {
    emission::emit(game, projectile, &mut Ram { frozen: false });
}
pub fn place(projectile: u16, pair: u16, game: &mut Game) {
    emission::place(
        game,
        projectile,
        usize::from(pair >> 1),
        &mut Ram { frozen: false },
    );
}
pub fn update(game: &mut Game) {
    crate::verify_probe::hit("trails.update");
    let frozen = word!(game.time_is_frozen_flag) != 0;
    playback::present(game, frozen, &mut Ram { frozen });
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
