//! Backend-free composition of the actual animation and palette cores.
#![forbid(unsafe_code)]
#![allow(dead_code)]
mod animation;
mod palette;
#[path = "../../rom.rs"]
mod rom;
mod types {
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
#[derive(Clone, Debug, PartialEq, Eq)]
struct World {
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
            palette: palette::state::Session::default(),
            target_colors: [0; 256],
            progression: Progression::default(),
            statue: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
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
    on_upload: Option<fn(&mut World)>,
}
impl Composed {
    fn new() -> Self {
        Self {
            world: World::default(),
            image: RomImage::zeroed(0x70000),
            events: Vec::new(),
            fail_upload: false,
            on_upload: None,
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
        self.events.push(Event::Projectile(kind, parameter));
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
        0x2345
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
    println!("animated/palette composed core checks passed");
}
