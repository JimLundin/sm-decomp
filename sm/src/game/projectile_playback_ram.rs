//! Checked playback/trail access. ROM capture and native diagnostics still have
//! explicit unsafe dependencies; no animation state is copied across calls.
use crate::game::projectile_playback::{self as playback, Command, Definitions, Frame, Owner, Write};
use crate::game::projectile_trail_selection::{self as trail, Source, TrailDefinitions};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomRecord;

impl Owner for &mut Game {
    type Cursor = u16;
    type Sprite = u16;
    type Slot = u16;
    fn remaining(&self, slot: u16) -> u16 {
        word!(self.projectile_bomb_instruction_timers[slot >> 1])
    }
    fn cursor(&self, slot: u16) -> u16 {
        word!(self.projectile_bomb_instruction_ptr[slot >> 1])
    }
    fn write(&mut self, slot: u16, value: Write<u16, u16>) {
        let slot = usize::from(slot >> 1);
        match value {
            Write::Timer(v) => word!(self.projectile_bomb_instruction_timers[slot] = v),
            Write::Cursor(v) => word!(self.projectile_bomb_instruction_ptr[slot] = v),
            Write::Sprite(v) => word!(self.projectile_spritemap_pointers[slot] = v),
            Write::XRadius(v) => word!(self.projectile_x_radius[slot] = v),
            Write::YRadius(v) => word!(self.projectile_y_radius[slot] = v),
        }
    }
}
struct View {
    data: RomRecord,
    cursor: u16,
}
impl Frame<u16, u16> for View {
    fn command(&self) -> Command {
        match self.data.word(0) {
            0x822f => Command::Release,
            0x8239 => Command::Jump,
            duration if duration & 0x8000 == 0 => Command::Show { duration },
            _ => Command::Unknown,
        }
    }
    fn sprite(&self) -> u16 {
        self.data.word(2)
    }
    fn x_radius(&self) -> u16 {
        u16::from(self.data.byte(4))
    }
    fn y_radius(&self) -> u16 {
        u16::from(self.data.byte(5))
    }
    fn next(&self) -> u16 {
        self.cursor.wrapping_add(8)
    }
}
struct Ram;
impl<'a> Definitions<&'a mut Game> for Ram {
    type Frame = View;
    fn open(&mut self, game: &'a mut Game, cursor: u16) -> (&'a mut Game, View) {
        let data = unsafe { crate::rtl::rom_bank_record::<0x93>(cursor, game) };
        (game, View { data, cursor })
    }
    fn follow_jump(&mut self, game: &'a mut Game, cursor: u16) -> (&'a mut Game, Option<u16>) {
        let (game, frame) = self.open(game, cursor.wrapping_add(2));
        let target = frame.data.word(0);
        (game, (target != 0).then_some(target))
    }
    fn unknown(&mut self, game: &'a mut Game) -> (&'a mut Game, Option<u16>) {
        let result = unsafe { crate::rtl::Unreachable(game) };
        (game, result.then_some(1))
    }
}
impl<'a> TrailDefinitions<&'a mut Game> for Ram {
    type Marker = u16;
    fn sample_trail(
        &mut self,
        game: &'a mut Game,
        cursor: u16,
        source: Source,
    ) -> (&'a mut Game, u16) {
        let cursor = if source == Source::Upcoming {
            cursor
        } else {
            cursor.wrapping_sub(8)
        };
        let (game, frame) = self.open(game, cursor);
        (game, frame.data.word(6))
    }
}
pub(crate) fn advance(game: &mut Game) {
    let selected = word!(game.projectile_index);
    let (game, outcome) = playback::advance(game, selected, &mut Ram);
    if outcome == playback::Outcome::Release {
        crate::game::sm_90::ClearProjectile(selected, game);
    }
}
pub(crate) fn trail_frame(slot: u16, game: &mut Game) -> u16 {
    trail::sample(game, slot, &mut Ram).1
}
