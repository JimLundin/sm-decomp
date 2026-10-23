//! Exact animation encoding and exclusive legacy effects; not a second engine owner.
#![forbid(unsafe_code)]
use super::{
    content,
    execution::{Host, SoundChannel, StatueProjectile},
    native,
    state::{AnimatedTiles, Field, Slot, Upload},
};
use crate::{
    game::{
        constants::bank_86::{addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul},
        music::QueueMusic_Delayed8,
        palette_effects, progress_flags,
        sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6},
    },
    game_state::Game,
    ram_map::words as ram,
    rom::RomRecord,
};
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put_prefix(bytes: &mut [u8], at: usize, value: u16) {
    let [low, high] = value.to_le_bytes();
    bytes[at] = low;
    bytes[at + 1] = high;
}
fn address(field: Field, index: u16) -> usize {
    let base = match field {
        Field::Id => 0x1ef5,
        Field::Counter => 0x1f01,
        Field::Program => 0x1f0d,
        Field::Delay => 0x1f19,
        Field::Source => 0x1f25,
    };
    base + usize::from(index & !1)
}
fn decode(game: &Game) -> AnimatedTiles {
    let bytes = &game.g_ram;
    AnimatedTiles {
        flags: word(bytes, 0x1ef1),
        selected: word(bytes, 0x1ef3),
        last_instruction: word(bytes, 0x1f49),
        slots: core::array::from_fn(|i| {
            let at = 2 * i;
            Slot {
                id: word(bytes, 0x1ef5 + at),
                counter: word(bytes, 0x1f01 + at),
                program: word(bytes, 0x1f0d + at),
                delay: word(bytes, 0x1f19 + at),
                source: word(bytes, 0x1f25 + at),
                size: word(bytes, 0x1f31 + at),
                vram: word(bytes, 0x1f3d + at),
            }
        }),
    }
}
fn encode(state: &AnimatedTiles, game: &mut Game) {
    let bytes = &mut game.g_ram;
    put(bytes, 0x1ef1, state.flags);
    put(bytes, 0x1ef3, state.selected);
    put(bytes, 0x1f49, state.last_instruction);
    for (index, slot) in state.slots.iter().enumerate() {
        for (base, value) in [
            (0x1ef5, slot.id),
            (0x1f01, slot.counter),
            (0x1f0d, slot.program),
            (0x1f19, slot.delay),
            (0x1f25, slot.source),
            (0x1f31, slot.size),
            (0x1f3d, slot.vram),
        ] {
            put(bytes, base + index * 2, value);
        }
    }
}
pub(super) struct Runtime<'g> {
    state: Option<AnimatedTiles>,
    game: &'g mut Game,
}
impl<'g> Runtime<'g> {
    pub(super) fn new(game: &'g mut Game) -> Self {
        Self {
            state: Some(decode(game)),
            game,
        }
    }
    fn foreign<R>(&mut self, call: impl FnOnce(&mut Game) -> R) -> R {
        let state = self.state.take().expect("animation foreign interval");
        encode(&state, self.game);
        drop(state);
        let result = call(self.game);
        self.state = Some(decode(self.game));
        result
    }
}
impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            encode(state, self.game);
        }
    }
}
impl Host for Runtime<'_> {
    type Record = RomRecord;
    fn tiles(&self) -> &AnimatedTiles {
        self.state.as_ref().expect("animation foreign interval")
    }
    fn tiles_mut(&mut self) -> &mut AnimatedTiles {
        self.state.as_mut().expect("animation foreign interval")
    }
    fn record(&mut self, address: u16) -> RomRecord {
        if address & 0x8000 == 0 {
            self.foreign(|game| native::low_record(address, game))
        } else {
            content::record(
                self.game
                    .loaded_rom
                    .as_ref()
                    .expect("ROM must be loaded")
                    .clone(),
                address,
            )
        }
    }
    fn music(&mut self, track: u16) {
        self.foreign(|game| QueueMusic_Delayed8(track, game));
    }
    fn sound(&mut self, channel: SoundChannel, sound: u16) {
        self.foreign(|game| match channel {
            SoundChannel::One => QueueSfx1_Max6(sound, game),
            SoundChannel::Two => QueueSfx2_Max6(sound, game),
            SoundChannel::Three => QueueSfx3_Max6(sound, game),
        });
    }
    fn check_boss(&mut self, mask: u16) -> bool {
        self.foreign(|game| progress_flags::CheckBossBitForCurArea(mask, game) & 1 != 0)
    }
    fn set_boss(&mut self, mask: u16) {
        self.foreign(|game| progress_flags::SetBossBitForCurArea(mask, game));
    }
    fn check_event(&mut self, event: u16) -> bool {
        self.foreign(|game| progress_flags::CheckEventHappened(event, game) & 1 != 0)
    }
    fn set_event(&mut self, event: u16) {
        self.foreign(|game| progress_flags::SetEventHappened(event, game));
    }
    fn boss_at(&mut self, area: u8) -> u8 {
        self.foreign(|game| game.g_ram[0xd828 + usize::from(area)])
    }
    fn statue(&mut self) -> u16 {
        self.foreign(|game| word(&game.g_ram, ram::tourian_entrance_statue_animstate))
    }
    fn set_statue(&mut self, value: u16) {
        self.foreign(|game| {
            put_prefix(
                &mut game.g_ram,
                ram::tourian_entrance_statue_animstate,
                value,
            )
        });
    }
    fn projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        let definition = match kind {
            StatueProjectile::EyeGlow => addr_kEproj_TourianStatueEyeGlow,
            StatueProjectile::Soul => addr_kEproj_TourianStatueSoul,
        };
        self.foreign(|game| native::projectile(definition as u16, parameter, game));
    }
    fn palette_color(&mut self, index: usize, value: u16) {
        self.foreign(|game| put_prefix(&mut game.g_ram, 0xc000 + 2 * index, value));
    }
    fn target_color(&mut self, index: usize, value: u16) {
        self.foreign(|game| put_prefix(&mut game.g_ram, 0xc200 + 2 * index, value));
    }
    fn spawn_palette(&mut self, definition: u16) {
        self.foreign(|game| palette_effects::spawn(definition, game));
    }
    fn upload(&mut self, request: Upload) {
        self.foreign(|game| native::upload(request, game));
    }
    fn unreachable(&mut self) -> u16 {
        self.foreign(native::unreachable)
    }
    fn outside_field(&mut self, field: Field, index: u16) -> u16 {
        self.foreign(|game| word(&game.g_ram, address(field, index)))
    }
    fn put_outside_field(&mut self, field: Field, index: u16, value: u16) {
        self.foreign(|game| put_prefix(&mut game.g_ram, address(field, index), value));
    }
    fn put_outside_counter_byte(&mut self, index: u16, value: u8) {
        self.foreign(|game| game.g_ram[0x1f01 + usize::from(index)] = value);
    }
    fn probe(&mut self, name: &'static str) {
        crate::verify_probe::hit(name);
    }
}

#[path = "../../verification/animated_runtime_checks.rs"]
mod guard_checks;
pub(super) fn check_guard(game: &mut Game) -> bool {
    guard_checks::run(game)
}
