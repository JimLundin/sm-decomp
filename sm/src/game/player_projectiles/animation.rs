//! Direct shared initialization/playback bindings, for every native selector.
use super::{Context, Transaction};
use crate::game::projectile_initialization::{self as initialization, Field, Initialization};
use crate::game::projectile_playback::{self as playback, Command, Definitions, Frame, Owner, Write};
use crate::game::projectile_trail_selection::{self as trail, Source, TrailDefinitions};
use crate::rom::RomRecord;
#[forbid(unsafe_code)]
pub(crate) fn table(kind: Initialization, flags: u16) -> (u32, isize) {
    match kind {
        Initialization::SuperMissile => (0x93842b, ((flags >> 8) & 15) as isize),
        Initialization::Bomb => (0x9383f1, ((flags >> 8) & 15) as isize),
        Initialization::SpecialAttack => (0x938413, (flags & 15) as isize),
        Initialization::Echo => (0x938403, isize::from(flags.to_le_bytes()[0]) - 34),
        Initialization::Projectile if flags & 0xf00 != 0 => {
            (0x9383f1, isize::from(flags.to_ne_bytes()[1] & 15))
        }
        Initialization::Projectile => (
            if flags & 0x10 != 0 {
                0x9383d9
            } else {
                0x9383c1
            },
            (flags & 15) as isize,
        ),
    }
}
impl initialization::Launch for RomRecord {
    fn damage(&self) -> u16 {
        self.word(0)
    }
}
impl<'a> initialization::Context<Transaction<'a>> for Context {
    type Weapon = (Initialization, u16);
    type Launch = RomRecord;
    type Cursor = u16;
    type Frame = RomRecord;
    fn weapon(
        &mut self,
        game: Transaction<'a>,
        (kind, flags): Self::Weapon,
    ) -> (Transaction<'a>, RomRecord) {
        let (address, selector) = table(kind, flags);
        let pointer = game
            .rom()
            .table_word(crate::rtl::lorom_offset(address) as usize, selector);
        Context::record(game, 0x93, pointer)
    }
    fn set(&mut self, game: &mut Transaction<'a>, field: Field, value: u16) {
        match field {
            Field::Damage => word!(game.projectile_damage[self.slot] = value),
            Field::XRadius => word!(game.projectile_x_radius[self.slot] = value),
            Field::YRadius => word!(game.projectile_y_radius[self.slot] = value),
            Field::Timer => word!(game.projectile_bomb_instruction_timers[self.slot] = value),
        }
    }
    fn invalid_damage(&mut self, game: Transaction<'a>) -> Transaction<'a> {
        game.foreign(|g| unsafe { crate::game::sm_80::InvalidInterrupt_Crash(g) })
            .0
    }
    fn entry(
        &mut self,
        game: Transaction<'a>,
        launch: &RomRecord,
        direction: Option<usize>,
    ) -> (Transaction<'a>, u16) {
        let offset = direction.map_or(2, |direction| {
            [2, 4, 6, 8, 10, 12, 14, 16, 18, 20][direction]
        });
        (game, launch.word(offset))
    }
    fn set_cursor(&mut self, game: &mut Transaction<'a>, cursor: u16) {
        word!(game.projectile_bomb_instruction_ptr[self.slot] = cursor);
    }
    fn frame(&mut self, game: Transaction<'a>, cursor: u16) -> (Transaction<'a>, RomRecord) {
        Context::record(game, 0x93, cursor)
    }
    fn radius(
        &mut self,
        game: Transaction<'a>,
        frame: &RomRecord,
        field: Field,
    ) -> (Transaction<'a>, u16) {
        let offset = match field {
            Field::XRadius => 4,
            Field::YRadius => 5,
            _ => unreachable!(),
        };
        (game, u16::from(frame.byte(offset)))
    }
}
pub(super) fn initialize<'g>(
    k: u16,
    kind: Initialization,
    game: Transaction<'g>,
) -> Transaction<'g> {
    let slot = usize::from(k >> 1);
    let direction = matches!(kind, Initialization::Projectile | Initialization::Echo)
        .then(|| usize::from(word!(game.projectile_dir[slot]) & 15));
    let flags = word!(game.projectile_type[slot]);
    initialization::initialize(
        game,
        (kind, flags),
        kind,
        direction,
        &mut Context {
            slot,
            ..Context::default()
        },
    )
}

impl Owner for Transaction<'_> {
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
            Write::Cursor(v) => {
                word!(self.projectile_bomb_instruction_ptr[slot] = v);
                crate::verify_probe::hit("player_projectiles.playback.frame");
            }
            Write::Sprite(v) => word!(self.projectile_spritemap_pointers[slot] = v),
            Write::XRadius(v) => word!(self.projectile_x_radius[slot] = v),
            Write::YRadius(v) => word!(self.projectile_y_radius[slot] = v),
        }
    }
}
pub(super) struct View {
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
impl<'a> Definitions<Transaction<'a>> for Context {
    type Frame = View;
    fn open(&mut self, game: Transaction<'a>, cursor: u16) -> (Transaction<'a>, View) {
        let (game, data) = Context::record(game, 0x93, cursor);
        (game, View { data, cursor })
    }
    fn follow_jump(
        &mut self,
        game: Transaction<'a>,
        cursor: u16,
    ) -> (Transaction<'a>, Option<u16>) {
        let (game, frame) = self.open(game, cursor.wrapping_add(2));
        let target = frame.data.word(0);
        (game, (target != 0).then_some(target))
    }
    fn unknown(&mut self, game: Transaction<'a>) -> (Transaction<'a>, Option<u16>) {
        let (game, result) = game.foreign(|g| unsafe { crate::rtl::Unreachable(g) });
        (game, result.then_some(1))
    }
}
impl<'a> TrailDefinitions<Transaction<'a>> for Context {
    type Marker = u16;
    fn sample_trail(
        &mut self,
        game: Transaction<'a>,
        cursor: u16,
        source: Source,
    ) -> (Transaction<'a>, u16) {
        let cursor = if source == Source::Upcoming {
            cursor
        } else {
            cursor.wrapping_sub(8)
        };
        let (game, frame) = self.open(game, cursor);
        (game, frame.data.word(6))
    }
}

pub(super) fn advance(mut owner: Transaction<'_>) -> Transaction<'_> {
    let selected = word!(owner.projectile_index);
    let (next, outcome) = playback::advance(owner, selected, &mut Context::default());
    owner = next;
    if outcome == playback::Outcome::Release {
        super::motion::clear(selected, owner)
    } else {
        owner
    }
}
pub(super) fn trail_frame(slot: u16, owner: Transaction<'_>) -> (Transaction<'_>, u16) {
    trail::sample(owner, slot, &mut Context::default())
}
