//! Backend-free composition of the actual animation and palette cores.
#![forbid(unsafe_code)]
#![allow(dead_code)]
mod animation;
mod composition_checks;
mod enemy_checks;
mod enemy_host;
mod game;
mod palette;
mod player;
mod player_checks;
mod room_host;
mod session_host;
use game::room_mechanisms as room;
#[path = "../../rom.rs"]
mod rom;
#[path = "../../verify_probe.rs"]
mod verify_probe;
mod types {
    // These scalar aliases match types.rs:641; constants are actual source.
    pub(crate) type Buttons = u32;
    pub(crate) type Consts_60 = u32;
    pub(crate) type Consts_82 = u32;
    pub(crate) type Consts_84 = u32;
    pub(crate) type Consts_86 = u32;
    pub(crate) type Consts_87 = u32;
    pub(crate) type Consts_8D = u32;
    pub(crate) type Consts_8F = u32;
    pub(crate) type Consts_90 = u32;
    pub(crate) type Consts_91 = u32;
    pub(crate) type Consts_9B = u32;
    pub(crate) type Consts_A0 = u32;
    pub(crate) type Consts_A4 = u32;
    pub(crate) type Consts_A6 = u32;
    pub(crate) type Consts_A9 = u32;
    pub(crate) type Consts_B4 = u32;
    pub(crate) type EnemyProps = u32;
    pub(crate) type GameState = u32;
    pub(crate) type LoadingGameState = u32;
    pub(crate) type ProjectileType = u32;
    pub(crate) type SamusMovementType = u32;
    pub(crate) type SamusPose = u32;

    // Exact numeric Hitbox layout from types.rs, for shared contact_rules offsets.
    #[repr(C, packed)]
    pub(crate) struct Hitbox {
        pub left: u16,
        pub top: u16,
        pub right: u16,
        pub bottom: u16,
        pub func_ptr: u16,
        pub func_ptrA: u16,
    }
    pub(crate) struct LongPtr {
        pub addr: u16,
        pub bank: u8,
    }
}
use animation::{
    execution as animated,
    state::{AnimatedTiles, Field, Upload},
};
use palette::execution as colors;
use rom::{RomImage, RomRecord};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Progression {
    events: [u8; 8],
    bosses: [u8; 8],
    area: usize,
    scratch: u16,
}
impl Progression {
    fn event_byte(&mut self, index: usize) -> &mut u8 {
        match index {
            0..=7 => &mut self.events[index],
            8..=15 => &mut self.bosses[index - 8],
            _ => panic!("event outside prepared progression"),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct World {
    player: game::session::Session,
    player_inputs: game::external_inputs::ExternalInputs,
    animation: AnimatedTiles,
    palette: palette::state::Session,
    target_colors: [u16; 256],
    progression: Progression,
    statue: u16,
}
impl Default for World {
    fn default() -> Self {
        Self {
            animation: AnimatedTiles::default(),
            player: Default::default(),
            player_inputs: Default::default(),
            palette: palette::state::Session::default(),
            target_colors: [0; 256],
            progression: Progression::default(),
            statue: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    EnemySound(u8, u8, u16),
    EnemyRandom,
    EnemyGrapple,
    RoomSound(u8, u8, u16),
    RoomMessage(u16, u16),
    RoomVisual(u16),
    RoomMusic(u16),
    RoomControl(u16, u16, u16),
    PlayerSound(u8, u8, u16),
    Shot(u16, u16, u16),
    Beam(u16, u16, u16),
    PaletteSpawn(u16),
    Color(usize, u16),
    Target(usize, u16),
    Music(u16),
    Sound(u8, u16),
    Projectile(animated::StatueProjectile, u16),
    Upload(Upload),
}
struct Composed {
    world: World,
    image: RomImage,
    events: Vec<Event>,
    fail_upload: bool,
    late_message: Option<u8>,
    message_yields: std::collections::VecDeque<u8>,
    on_upload: Option<fn(&mut World)>,
    on_enemy_random: Option<fn(&mut Composed)>,
    on_enemy_sound: Option<fn(&mut Composed)>,
    expected_drop_award: Option<u16>,
    on_room_sound: Option<fn(&mut Composed)>,
    enemy_random: std::collections::VecDeque<u16>,
}
impl Composed {
    fn new() -> Self {
        Self {
            world: World::default(),
            image: RomImage::zeroed(0x100000),
            events: Vec::new(),
            fail_upload: false,
            late_message: None,
            message_yields: Default::default(),
            on_upload: None,
            on_enemy_random: None,
            on_enemy_sound: None,
            expected_drop_award: None,
            on_room_sound: None,
            enemy_random: Default::default(),
        }
    }
    fn byte(&self, bank: u8, cursor: u16, byte: u8) {
        let base = usize::from(bank & 0x7f) * 0x8000;
        self.image.write(base + usize::from(cursor & 0x7fff), byte);
    }
    fn word(&self, bank: u8, cursor: u16, value: u16) {
        let base = usize::from(bank & 0x7f) * 0x8000 + usize::from(cursor & 0x7fff);
        for (offset, byte) in value.to_ne_bytes().into_iter().enumerate() {
            self.image.write(base + offset, byte);
        }
    }
    fn tick(&mut self) {
        animated::advance(self);
        colors::advance(self);
        animated::upload_pending(self);
    }
    fn refresh_progression_input(&mut self) {
        let p = &self.world.progression;
        let next = p.bosses.get(p.area + 1).copied().unwrap_or(0);
        self.world.palette.inputs.miniboss_dead =
            u16::from_ne_bytes([p.bosses[p.area], next]) & 2 != 0;
    }
}
impl animated::Host for Composed {
    type Record = RomRecord;
    fn tiles(&self) -> &AnimatedTiles {
        &self.world.animation
    }
    fn tiles_mut(&mut self) -> &mut AnimatedTiles {
        &mut self.world.animation
    }
    fn record(&mut self, address: u16) -> RomRecord {
        animation::content::record(self.image.clone(), address)
    }
    fn music(&mut self, track: u16) {
        self.events.push(Event::Music(track));
    }
    fn sound(&mut self, channel: animated::SoundChannel, sound: u16) {
        self.events.push(Event::Sound(
            match channel {
                animated::SoundChannel::One => 1,
                animated::SoundChannel::Two => 2,
                animated::SoundChannel::Three => 3,
            },
            sound,
        ));
    }
    fn check_boss(&mut self, mask: u16) -> bool {
        let p = &mut self.world.progression;
        p.scratch = (p.scratch & 0xff00) | u16::from(mask as u8);
        p.bosses[p.area] & mask as u8 != 0
    }
    fn set_boss(&mut self, mask: u16) {
        let p = &mut self.world.progression;
        p.scratch = (p.scratch & 0xff00) | u16::from(mask as u8);
        p.bosses[p.area] |= mask as u8;
        self.refresh_progression_input();
    }
    fn check_event(&mut self, event: u16) -> bool {
        let p = &mut self.world.progression;
        p.scratch = 1 << (event & 7);
        let mask = p.scratch as u8;
        *p.event_byte(usize::from(event >> 3)) & mask != 0
    }
    fn set_event(&mut self, event: u16) {
        let p = &mut self.world.progression;
        p.scratch = 1 << (event & 7);
        let mask = p.scratch as u8;
        *p.event_byte(usize::from(event >> 3)) |= mask;
        self.refresh_progression_input();
    }
    fn boss_at(&mut self, area: u8) -> u8 {
        self.world.progression.bosses[usize::from(area)]
    }
    fn statue(&mut self) -> u16 {
        self.world.statue
    }
    fn set_statue(&mut self, value: u16) {
        self.world.statue = value;
    }
    fn projectile(&mut self, kind: animated::StatueProjectile, parameter: u16) {
        let _ = (kind, parameter);
        panic!("statue enemy projectiles outside prepared scope");
    }
    fn palette_color(&mut self, index: usize, value: u16) {
        self.world.palette.colors[index] = value;
        self.events.push(Event::Color(index, value));
    }
    fn target_color(&mut self, index: usize, value: u16) {
        self.world.target_colors[index] = value;
        self.events.push(Event::Target(index, value));
    }
    fn spawn_palette(&mut self, definition: u16) {
        self.events.push(Event::PaletteSpawn(definition));
        // Invoke the actual adopted palette core synchronously on THIS host.
        colors::spawn(self, definition);
    }
    fn upload(&mut self, request: Upload) {
        self.events.push(Event::Upload(request));
        if let Some(update) = self.on_upload.take() {
            update(&mut self.world);
        }
        assert!(!self.fail_upload, "intentional upload failure");
    }
    fn unreachable(&mut self) -> u16 {
        panic!("invalid animation instruction")
    }
    fn outside_field(&mut self, _: Field, _: u16) -> u16 {
        panic!("compatibility slot read")
    }
    fn put_outside_field(&mut self, _: Field, _: u16, _: u16) {
        panic!("compatibility slot write")
    }
    fn put_outside_counter_byte(&mut self, _: u16, _: u8) {
        panic!("compatibility counter byte")
    }
}
impl colors::Host for Composed {
    type Record = RomRecord;
    fn session(&self) -> &palette::state::Session {
        &self.world.palette
    }
    fn session_mut(&mut self) -> &mut palette::state::Session {
        &mut self.world.palette
    }
    fn record(&mut self, address: u16) -> RomRecord {
        palette::content::record(self.image.clone(), address)
    }
    fn music(&mut self, track: u16) {
        self.events.push(Event::Music(track));
    }
    fn sound(&mut self, channel: colors::SoundChannel, sound: u16) {
        self.events.push(Event::Sound(
            match channel {
                colors::SoundChannel::One => 1,
                colors::SoundChannel::Two => 2,
                colors::SoundChannel::Three => 3,
            },
            sound,
        ));
    }
    fn color_written(&mut self, index: u16, value: u16) {
        self.events.push(Event::Color(usize::from(index), value));
    }
    fn unreachable(&mut self) {
        panic!("invalid palette command")
    }
    fn outside_field(&mut self, _: palette::state::Field, _: u16) -> u16 {
        panic!("compatibility palette slot")
    }
    fn put_outside_field(&mut self, _: palette::state::Field, _: u16, _: u16) {
        panic!("compatibility palette slot")
    }
    fn put_outside_counter_byte(&mut self, _: u16, _: u8) {
        panic!("compatibility palette counter")
    }
    fn outside_flag(&mut self, _: u16) -> u16 {
        panic!("compatibility palette flag")
    }
    fn outside_color(&mut self, _: u16, _: u16) {
        panic!("compatibility palette color")
    }
}
mod checks;
fn main() {
    checks::run();
    composition_checks::run();
    enemy_checks::run();
    println!("player/animation/palette composed core checks passed");
}
