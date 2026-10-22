//! Checked RAM/content access for shared initialization. Native diagnostics and
//! ROM record capture retain their existing unsafe dependencies explicitly here.
use crate::game::projectile_initialization::{self as initialization, Field, Initialization};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
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
struct Ram {
    slot: usize,
}
impl<'a> initialization::Context<&'a mut Game> for Ram {
    type Weapon = (Initialization, u16);
    type Launch = RomRecord;
    type Cursor = u16;
    type Frame = RomRecord;
    fn weapon(
        &mut self,
        game: &'a mut Game,
        (kind, flags): Self::Weapon,
    ) -> (&'a mut Game, RomRecord) {
        let (address, selector) = table(kind, flags);
        let pointer = crate::rtl::rom_table_word!(game, address, selector);
        let record = unsafe { crate::rtl::rom_bank_record::<0x93>(pointer, game) };
        (game, record)
    }
    fn set(&mut self, game: &mut &'a mut Game, field: Field, value: u16) {
        match field {
            Field::Damage => word!(game.projectile_damage[self.slot] = value),
            Field::XRadius => word!(game.projectile_x_radius[self.slot] = value),
            Field::YRadius => word!(game.projectile_y_radius[self.slot] = value),
            Field::Timer => word!(game.projectile_bomb_instruction_timers[self.slot] = value),
        }
    }
    fn invalid_damage(&mut self, game: &'a mut Game) -> &'a mut Game {
        unsafe { crate::game::sm_80::InvalidInterrupt_Crash(game) };
        game
    }
    fn entry(
        &mut self,
        game: &'a mut Game,
        launch: &RomRecord,
        direction: Option<usize>,
    ) -> (&'a mut Game, u16) {
        let offset = direction.map_or(2, |direction| {
            [2, 4, 6, 8, 10, 12, 14, 16, 18, 20][direction]
        });
        (game, launch.word(offset))
    }
    fn set_cursor(&mut self, game: &mut &'a mut Game, cursor: u16) {
        word!(game.projectile_bomb_instruction_ptr[self.slot] = cursor);
    }
    fn frame(&mut self, game: &'a mut Game, cursor: u16) -> (&'a mut Game, RomRecord) {
        let record = unsafe { crate::rtl::rom_bank_record::<0x93>(cursor, game) };
        (game, record)
    }
    fn radius(
        &mut self,
        game: &'a mut Game,
        frame: &RomRecord,
        field: Field,
    ) -> (&'a mut Game, u16) {
        let offset = match field {
            Field::XRadius => 4,
            Field::YRadius => 5,
            _ => unreachable!(),
        };
        (game, u16::from(frame.byte(offset)))
    }
}
pub(crate) fn initialize(k: u16, kind: Initialization, game: &mut Game) {
    let slot = usize::from(k >> 1);
    let direction = matches!(kind, Initialization::Projectile | Initialization::Echo)
        .then(|| usize::from(word!(game.projectile_dir[slot]) & 15));
    let flags = word!(game.projectile_type[slot]);
    initialization::initialize(game, (kind, flags), kind, direction, &mut Ram { slot });
}
