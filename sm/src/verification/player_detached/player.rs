//! Prepared-domain host for the actual projectile core. Unconverted effects fail.
use super::game::player_projectiles::{
    access::{ShotLane, Store},
    host::{Drawing, Host, PaletteEffect, PlayerStep},
    state::{Inputs, Session},
};
use super::{Composed, Event, RomImage, RomRecord, colors};

impl Host for Composed {
    fn session(&self) -> &Session {
        &self.world.player
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.world.player
    }
    fn inputs(&self) -> &Inputs {
        &self.world.player_inputs
    }
    fn loaded_image(&self) -> Option<RomImage> {
        Some(self.image.clone())
    }
    fn native_image(&self) -> RomImage {
        self.image.clone()
    }
    fn record(&mut self, bank: u8, cursor: u16) -> RomRecord {
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
    fn player_step(&mut self, _: PlayerStep) {
        panic!("Samus simulation outside prepared scope")
    }
    fn palette(&mut self, effect: PaletteEffect) -> u8 {
        match effect {
            PaletteEffect::Spawn(definition) => {
                self.events.push(Event::PaletteSpawn(definition));
                colors::spawn(self, definition);
                0
            }
            _ => panic!("Samus palette operation outside prepared scope"),
        }
    }
    fn enable_hdma(&mut self) {
        panic!("HDMA outside prepared scope")
    }
    fn spawn_power_bomb(&mut self) {
        panic!("power-bomb backend outside prepared scope")
    }
    fn spawn_mechanism(&mut self, _: u16) {
        panic!("room mechanism outside prepared scope")
    }
    fn draw(&mut self, drawing: Drawing) {
        match drawing {
            Drawing::Projectile { slot, x, y } => self.events.push(Event::Shot(slot, x, y)),
            Drawing::Beam { sprite, x, y } => self.events.push(Event::Beam(sprite, x, y)),
            Drawing::CrashEchoes => panic!("crash echo rendering outside prepared scope"),
        }
    }
    fn draw_trail(&mut self, _: usize, _: bool, _: bool) {
        panic!("trail output outside prepared scope")
    }
    fn invalid_damage(&mut self) {
        panic!("invalid prepared projectile damage")
    }
    fn unknown(&mut self) -> bool {
        panic!("unknown prepared projectile operation")
    }
    fn room_level(&self, cell: u16) -> u16 {
        self.terrain[usize::from(cell)]
    }
    fn room_bts(&self, cell: u16) -> u8 {
        self.bts[usize::from(cell)]
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
        self.world.player.shots[usize::from(raw >> 1)]
            .variable
            .to_le_bytes()[usize::from(high)]
    }
    fn echo_x(&self, index: usize) -> u16 {
        self.world.player.echoes.x[index]
    }
    fn set_echo_x(&mut self, index: usize, value: u16) {
        self.world.player.echoes.x[index] = value;
    }
    fn set_echo_y(&mut self, index: usize, value: u16) {
        self.world.player.echoes.y[index] = value;
    }
    fn set_echo_index_native(&mut self, raw: u16, value: u16) {
        assert_eq!(raw, 0, "echo overlap outside prepared scope");
        self.world.player.echoes.index = u16::from_le_bytes(value.to_ne_bytes());
    }
}
