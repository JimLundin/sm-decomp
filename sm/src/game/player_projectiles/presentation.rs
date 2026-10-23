//! Projectile/trail selection stays owned; actual native OAM consumers are explicit effects.
use super::{Context, Transaction};
use crate::game::projectile_presentation::{self as presentation, Pass};
use crate::game::projectile_presentation_ram as native;
use crate::game::projectile_trail_emission::{self as emission, Half, Owner, Point, Write};
use crate::game::projectile_trail_playback::{
    self as playback, Definitions, Instruction, Presentation,
};
use crate::game::projectile_trails::{ProgramCounter, SIDES, style};
use crate::rom::RomRecord;
impl Owner for Transaction<'_> {
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
pub(super) struct Offsets(RomRecord);
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
impl<'a> emission::Context<Transaction<'a>> for Context {
    type Style = u16;
    type Marker = u16;
    type Offsets = Offsets;
    fn style(&self, game: &Transaction<'a>, projectile: u16) -> Option<u16> {
        style(word!(game.projectile_type[projectile >> 1]))
    }
    fn script(&mut self, game: Transaction<'a>, style: u16, half: Half) -> (Transaction<'a>, u16) {
        let address = match half {
            Half::Left => 0x90b5bb,
            Half::Right => 0x90b609,
        };
        let cursor = game
            .rom()
            .table_word(crate::rtl::lorom_offset(address) as usize, style as isize);
        (game, cursor)
    }
    fn selected_projectile(&self, game: &Transaction<'a>) -> u16 {
        word!(game.projectile_index)
    }
    fn marker(&mut self, game: Transaction<'a>, projectile: u16) -> (Transaction<'a>, u16) {
        super::animation::trail_frame(projectile, game)
    }
    fn origin(&mut self, game: Transaction<'a>, projectile: u16) -> (Transaction<'a>, Point) {
        let index = usize::from(projectile >> 1);
        let point = if word!(game.ceres_status) & 0x8000 == 0 {
            Point {
                x: word!(game.projectile_x_pos[index]),
                y: word!(game.projectile_y_pos[index]),
            }
        } else {
            let (x, y) = project(projectile, &game);
            Point {
                x: word!(game.layer1_x_pos).wrapping_add(x),
                y: word!(game.layer1_y_pos).wrapping_add(y),
            }
        };
        (game, point)
    }
    fn offsets(
        &mut self,
        game: Transaction<'a>,
        projectile: u16,
        marker: u16,
    ) -> (Transaction<'a>, Offsets) {
        let index = usize::from(projectile >> 1);
        let kind = word!(game.projectile_type[index]);
        let table = if kind & 0x20 != 0 {
            0x9ba4e3
        } else if kind & 0x10 != 0 {
            0x9ba4cb
        } else {
            0x9ba4b3
        };
        let first = game
            .rom()
            .table_word(
                crate::rtl::lorom_offset(table) as usize,
                (kind & 15) as isize,
            )
            .wrapping_add(2 * (word!(game.projectile_dir[index]) & 15));
        let (game, first) = Context::record(game, 0x9b, first);
        let second = first.word(0).wrapping_add(marker.wrapping_mul(4));
        let (game, data) = Context::record(game, 0x9b, second);
        (game, Offsets(data))
    }
}
impl<'a> Definitions<Transaction<'a>> for Context {
    type Cursor = ProgramCounter;
    fn start(&mut self, cursor: u16, half: Half) -> ProgramCounter {
        ProgramCounter::new(cursor, half)
    }
    fn instruction(
        &mut self,
        game: Transaction<'a>,
        cursor: ProgramCounter,
    ) -> (Transaction<'a>, Instruction<ProgramCounter>) {
        let (game, record) = Context::record(game, 0x90, cursor.address());
        let value = record.word(0);
        (game, cursor.instruction(value))
    }
    fn frame(
        &mut self,
        game: Transaction<'a>,
        cursor: ProgramCounter,
    ) -> (Transaction<'a>, u16, u16) {
        let (game, record) = Context::record(game, 0x90, cursor.address());
        let sprite = record.word(2);
        (game, sprite, cursor.following())
    }
    fn unknown(&mut self, game: Transaction<'a>) -> Transaction<'a> {
        game.foreign(|g| unsafe { crate::rtl::Unreachable(g) }).0
    }
}
impl<'a> Presentation<Transaction<'a>> for Context {
    fn draw(&mut self, game: Transaction<'a>, pair: usize, half: Half) -> Transaction<'a> {
        game.foreign(|g| crate::game::projectile_trails::draw(pair, half as usize, self.frozen, g))
            .0
    }
}
impl presentation::Owner for Transaction<'_> {
    type Cursor = u16;
    type Sprite = u16;
    type Kind = u16;
    fn contains(&self, _: u16) -> bool {
        true
    }
    fn cursor(&self, slot: u16) -> u16 {
        word!(self.projectile_bomb_instruction_ptr[slot >> 1])
    }
    fn kind(&self, slot: u16) -> u16 {
        word!(self.projectile_type[slot >> 1])
    }
    fn phase(&self, slot: u16) -> u16 {
        word!(self.projectile_variables[slot >> 1])
    }
    fn sprite(&self, slot: u16) -> u16 {
        word!(self.projectile_spritemap_pointers[slot >> 1])
    }
    fn x(&self, slot: u16) -> u16 {
        word!(self.projectile_x_pos[slot >> 1])
    }
    fn y(&self, slot: u16) -> u16 {
        word!(self.projectile_y_pos[slot >> 1])
    }
    fn camera_x(&self) -> u16 {
        word!(self.layer1_x_pos)
    }
    fn camera_y(&self) -> u16 {
        word!(self.layer1_y_pos)
    }
    fn frame(&self) -> u16 {
        word!(self.nmi_frame_counter_word)
    }
    fn projected(&self) -> bool {
        word!(self.ceres_status) & 0x8000 != 0
    }
    fn selected(&self) -> u16 {
        word!(self.projectile_index)
    }
    fn select(&mut self, slot: u16) {
        word!(self.projectile_index = slot);
    }
}

pub fn project(slot: u16, game: &Transaction<'_>) -> (u16, u16) {
    let index = usize::from(slot >> 1);
    let dx = word!(game.projectile_x_pos[index]).wrapping_sub(word!(game.reg_M7X));
    let dy = word!(game.reg_M7Y).wrapping_sub(word!(game.projectile_y_pos[index]));
    let (x, y) = presentation::project(
        (dx, dy),
        (
            word!(game.reg_M7A),
            word!(game.reg_M7B),
            word!(game.reg_M7C),
        ),
    );
    (
        x.wrapping_add(word!(game.reg_M7X))
            .wrapping_sub(word!(game.layer1_x_pos)),
        word!(game.reg_M7Y)
            .wrapping_sub(y)
            .wrapping_sub(word!(game.layer1_y_pos)),
    )
}

impl<'a> presentation::Context<Transaction<'a>> for Context {
    fn project(&mut self, game: Transaction<'a>, slot: u16) -> (Transaction<'a>, (u16, u16)) {
        let point = project(slot, &game);
        (game, point)
    }
    fn draw(&mut self, game: Transaction<'a>, slot: u16, position: (u16, u16)) -> Transaction<'a> {
        crate::verify_probe::hit(if word!(game.projectile_type[slot >> 1]) & 0xf00 >= 0x300 {
            "player_projectiles.draw.explosion"
        } else {
            "player_projectiles.draw.projectile"
        });
        game.foreign(|g| native::projectile(slot, position, g, &mut native::Native))
            .0
    }
    fn finish(&mut self, game: Transaction<'a>) -> Transaction<'a> {
        let (game, ()) = game.foreign(|g| native::crash_echoes(g, &mut native::Native));
        update_trails(game)
    }
    fn resume(&mut self, _: Transaction<'a>, _: Pass, _: u16, _: (u16, u16)) -> Transaction<'a> {
        unreachable!("RAM covers every projectile selector")
    }
}
pub(super) fn emit(projectile: u16, game: Transaction<'_>) -> Transaction<'_> {
    emission::emit(game, projectile, &mut Context::default())
}
pub(super) fn update_trails(game: Transaction<'_>) -> Transaction<'_> {
    crate::verify_probe::hit("trails.update");
    let frozen = word!(game.time_is_frozen_flag) != 0;
    playback::present(
        game,
        frozen,
        &mut Context {
            frozen,
            ..Context::default()
        },
    )
}
pub(super) fn draw_projectiles(game: Transaction<'_>) -> Transaction<'_> {
    presentation::present(game, Pass::Ordinary, &mut Context::default())
}
pub(super) fn draw_explosions(game: Transaction<'_>) -> Transaction<'_> {
    presentation::present(game, Pass::Explosions, &mut Context::default())
}
