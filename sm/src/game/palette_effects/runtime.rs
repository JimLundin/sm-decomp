//! Compatibility edge: exact byte encoding and exclusive legacy authority transfers.
#![forbid(unsafe_code)]
use super::{
    content, diagnostics,
    execution::{Host, SoundChannel},
    state::{Field, Inputs, PaletteEffects, PeriodicDamage, Session, Slot},
};
use crate::{
    game::{
        music::QueueMusic_Delayed8,
        sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6},
    },
    game_state::Game,
    ram_map::words as ram,
    rom::RomRecord,
};

fn address(field: Field) -> usize {
    match field {
        Field::Id => 0x1e7d,
        Field::Color => 0x1e8d,
        Field::PreAction => 0x1ead,
        Field::Program => 0x1ebd,
        Field::Delay => 0x1ecd,
        Field::Counter => 0x1edd,
    }
}
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn native(bytes: &[u8], at: usize) -> u16 {
    u16::from_ne_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn decode(game: &Game) -> Session {
    let bytes = &game.g_ram;
    Session {
        effects: PaletteEffects {
            slots: core::array::from_fn(|index| {
                let at = index * 2;
                Slot {
                    id: word(bytes, 0x1e7d + at),
                    color: word(bytes, 0x1e8d + at),
                    pre_action: word(bytes, 0x1ead + at),
                    program: word(bytes, 0x1ebd + at),
                    delay: word(bytes, 0x1ecd + at),
                    counter: word(bytes, 0x1edd + at),
                }
            }),
            flags: word(bytes, ram::flag_for_palette_fx_objects),
            selected: word(bytes, ram::palettefx_index),
            heat: word(bytes, ram::samus_in_heat_palfx_index),
            previous_heat: word(bytes, ram::samus_in_heat_palettefx_prev_index),
        },
        colors: core::array::from_fn(|index| word(bytes, 0xc000 + index * 2)),
        inputs: Inputs {
            cinematic: word(bytes, ram::cinematic_function),
            enemy_health: native(
                bytes,
                0xf78 + core::mem::offset_of!(crate::types::EnemyData, health),
            ),
            equipped_items: word(bytes, ram::equipped_items),
            frame: word(bytes, ram::nmi_frame_counter_word),
            health: word(bytes, ram::samus_health),
            player_y: word(bytes, ram::samus_y_pos),
            miniboss_dead: native(bytes, 0xd828 + usize::from(word(bytes, ram::area_index))) & 2
                != 0,
        },
        damage: PeriodicDamage {
            fraction: native(bytes, ram::samus_periodic_subdamage),
            amount: native(bytes, ram::samus_periodic_damage),
        },
    }
}
fn encode(session: &Session, game: &mut Game) {
    let bytes = &mut game.g_ram;
    for (index, slot) in session.effects.slots.iter().enumerate() {
        for field in [
            Field::Id,
            Field::Color,
            Field::PreAction,
            Field::Program,
            Field::Delay,
            Field::Counter,
        ] {
            put(bytes, address(field) + index * 2, slot.field(field));
        }
    }
    for (at, value) in [
        (ram::flag_for_palette_fx_objects, session.effects.flags),
        (ram::palettefx_index, session.effects.selected),
        (ram::samus_in_heat_palfx_index, session.effects.heat),
        (
            ram::samus_in_heat_palettefx_prev_index,
            session.effects.previous_heat,
        ),
    ] {
        put(bytes, at, value);
    }
    for (index, color) in session.colors.iter().copied().enumerate() {
        put(bytes, 0xc000 + index * 2, color);
    }
    // Inputs are observations, never written back. Only the damage pair is mutable.
    bytes[ram::samus_periodic_subdamage..ram::samus_periodic_subdamage + 2]
        .copy_from_slice(&session.damage.fraction.to_ne_bytes());
    bytes[ram::samus_periodic_damage..ram::samus_periodic_damage + 2]
        .copy_from_slice(&session.damage.amount.to_ne_bytes());
}

pub(super) struct Runtime<'g> {
    session: Option<Session>,
    game: &'g mut Game,
}
impl<'g> Runtime<'g> {
    pub(super) fn new(game: &'g mut Game) -> Self {
        Self {
            session: Some(decode(game)),
            game,
        }
    }
    fn foreign<R>(&mut self, call: impl FnOnce(&mut Game) -> R) -> R {
        let session = self.session.take().expect("palette foreign interval");
        encode(&session, self.game);
        drop(session);
        // There is no model to republish if the callback unwinds.
        let result = call(self.game);
        self.session = Some(decode(self.game));
        result
    }
}
impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        // A checked core read may panic after valid writes. Preserve that prefix.
        if let Some(session) = &self.session {
            encode(session, self.game);
        }
    }
}
impl Host for Runtime<'_> {
    type Record = RomRecord;
    fn session(&self) -> &Session {
        self.session.as_ref().expect("palette foreign interval")
    }
    fn session_mut(&mut self) -> &mut Session {
        self.session.as_mut().expect("palette foreign interval")
    }
    fn record(&mut self, address: u16) -> RomRecord {
        if address & 0x8000 == 0 {
            self.foreign(|game| diagnostics::low_record(address, game))
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
    fn sound(&mut self, channel: SoundChannel, sound: u16) {
        self.foreign(|game| match channel {
            SoundChannel::One => QueueSfx1_Max6(sound, game),
            SoundChannel::Two => QueueSfx2_Max6(sound, game),
            SoundChannel::Three => QueueSfx3_Max6(sound, game),
        });
    }
    fn music(&mut self, track: u16) {
        self.foreign(|game| QueueMusic_Delayed8(track, game));
    }
    fn unreachable(&mut self) {
        self.foreign(diagnostics::unreachable);
    }
    fn outside_field(&mut self, field: Field, index: u16) -> u16 {
        self.foreign(|game| word(&game.g_ram, address(field) + usize::from(index & !1)))
    }
    fn put_outside_field(&mut self, field: Field, index: u16, value: u16) {
        self.foreign(|game| {
            put(
                &mut game.g_ram,
                address(field) + usize::from(index & !1),
                value,
            )
        });
    }
    fn put_outside_counter_byte(&mut self, index: u16, value: u8) {
        self.foreign(|game| game.g_ram[address(Field::Counter) + usize::from(index)] = value);
    }
    fn outside_flag(&mut self, index: u16) -> u16 {
        self.foreign(|game| {
            native(
                &game.g_ram,
                ram::flag_for_palette_fx_objects + usize::from(index),
            )
        })
    }
    fn outside_color(&mut self, displacement: u16, value: u16) {
        self.foreign(|game| {
            put(
                &mut game.g_ram,
                0xc000 + usize::from(displacement & !1),
                value,
            )
        });
    }
    fn probe(&mut self, name: &'static str) {
        crate::verify_probe::hit(name);
    }
}

#[path = "../../verification/palette_runtime_checks.rs"]
mod guard_checks;
pub(super) fn check_guard(game: &mut Game) -> bool {
    guard_checks::run(game)
}
