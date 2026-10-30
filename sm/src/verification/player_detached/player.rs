//! Prepared-domain host for the actual projectile core. Unconverted effects fail.
use super::game::player_projectiles::{
    access::{ShotLane, Store},
    host::{Drawing, Host, PaletteEffect, PlayerStep},
};
use super::{Composed, Event, RomImage, RomRecord, colors};

impl Host for Composed {
    fn loaded_image(&self) -> Option<RomImage> {
        Some(self.image.clone())
    }
    fn native_image(&self) -> RomImage {
        self.image.clone()
    }
    fn record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        if let Some(record) = self.on_player_record.and_then(|hook| hook(self, bank, cursor)) {
            return record;
        }
        assert!(cursor & 0x8000 != 0, "unprepared low-window content");
        RomRecord {
            image: self.image.clone(),
            base: usize::from(bank & 0x7f) * 0x8000 + usize::from(cursor & 0x7fff),
        }
    }
    fn native_record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        <Self as Host>::record(self, bank, cursor)
    }
    fn sound(&mut self, bank: u8, limit: u8, sound: u16) {
        self.events.push(Event::PlayerSound(bank, limit, sound));
    }
    fn player_step(&mut self, step: PlayerStep) {
        match step {
            PlayerStep::SuitIndex => {
                super::game::samus_palette::update_suit_index(&mut self.world.player)
            }
            _ => panic!("Samus simulation outside prepared scope"),
        }
    }
    fn palette(&mut self, effect: PaletteEffect) -> u8 {
        match effect {
            PaletteEffect::Spawn(definition) => {
                self.events.push(Event::PaletteSpawn(definition));
                colors::spawn(self, definition);
                0
            }
            PaletteEffect::Suit => {
                super::game::samus_palette::load_suit(
                    self,
                    super::game::visual_state::PaletteBuffer::Current,
                );
                0
            }
            PaletteEffect::SuitTarget => {
                super::game::samus_palette::load_suit(
                    self,
                    super::game::visual_state::PaletteBuffer::Target,
                );
                0
            }
        }
    }
    fn enable_hdma(&mut self) {
        panic!("HDMA outside prepared scope")
    }
    fn spawn_power_bomb(&mut self) {
        panic!("power-bomb backend outside prepared scope")
    }
    fn spawn_mechanism(&mut self, header: u16) {
        if let Some(hook) = self.on_player_mechanism {
            self.events.push(Event::PlayerMechanism(header));
            return hook(self, header);
        }
        panic!("room mechanism outside prepared scope")
    }
    fn draw(&mut self, drawing: Drawing) {
        match drawing {
            Drawing::Projectile { slot, x, y } => self.events.push(Event::Shot(slot, x, y)),
            Drawing::Beam { sprite, x, y } => self.events.push(Event::Beam(sprite, x, y)),
            _ => panic!("drawing outside prepared scope"),
        }
    }
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        if let Some(hook) = self.on_player_trail {
            return hook(self, pair, right, frozen);
        }
        panic!("trail output outside prepared scope")
    }
    fn invalid_damage(&mut self) {
        panic!("invalid prepared projectile damage")
    }
    fn unknown(&mut self) -> bool {
        if let Some(hook) = self.on_player_unknown { return hook(self); }
        panic!("unknown prepared projectile operation")
    }
    fn room_level(&self, cell: u16) -> u16 {
        self.world.player.room.terrain.cells()[usize::from(cell)].level
    }
    fn room_bts(&self, cell: u16) -> u8 {
        self.world.player.room.terrain.cells()[usize::from(cell)].behavior
    }
    fn alias_shot_read(&self, _: ShotLane, _: u16) -> u16 {
        panic!("shot alias outside prepared scope")
    }
    fn alias_shot_write(&mut self, _: ShotLane, _: u16, _: u16, _: Store) {
        panic!("shot alias outside prepared scope")
    }
    fn variable_byte(&self, raw: u16, high: bool) -> u8 {
        assert!(
            raw & 1 == 0 && raw < 20,
            "variable alias outside prepared scope"
        );
        self.world.player.projectiles.shots[usize::from(raw >> 1)]
            .variable
            .to_le_bytes()[usize::from(high)]
    }
    fn set_echo_x(&mut self, index: usize, value: u16) {
        self.world.player.projectiles.echoes.x[index] = value;
    }
    fn set_echo_y(&mut self, index: usize, value: u16) {
        self.world.player.projectiles.echoes.y[index] = value;
    }
    fn set_echo_index_native(&mut self, raw: u16, value: u16) {
        assert_eq!(raw, 0, "echo overlap outside prepared scope");
        self.world.player.projectiles.echoes.index = u16::from_le_bytes(value.to_ne_bytes());
    }
}
