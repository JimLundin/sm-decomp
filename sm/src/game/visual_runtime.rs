//! Visual adapters on the one aggregate Session, with explicit legacy effects.
use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{PaletteBuffer, Upload},
    },
    game_state::Game,
    rom::RomRecord,
};
impl visual_host::Record for RomRecord {
    fn byte(&self, displacement: usize) -> u8 {
        RomRecord::byte(self, displacement)
    }
    fn native_word(&self, displacement: usize) -> u16 {
        RomRecord::word(self, displacement)
    }
    fn bytes<const N: usize>(&self, displacement: usize) -> [u8; N] {
        let start = self
            .base
            .checked_add(displacement)
            .expect("ROM copy offset overflow");
        let end = start.checked_add(N).expect("ROM copy extent overflow");
        assert!(end <= self.image.len(), "ROM copy source extent");
        core::array::from_fn(|index| self.image.read(start + index))
    }
}
impl visual_host::Content for Runtime<'_> {
    type Record = RomRecord;
    fn record(&mut self, source: Source, bank: u8, address: u16) -> RomRecord {
        match source {
            Source::Loaded => player_projectiles::host::Host::record(self, bank, address),
            Source::Native => player_projectiles::host::Host::native_record(self, bank, address),
        }
    }
    fn atmospheric_record(&mut self, address: u16) -> RomRecord {
        let address = 0x900000 | u32::from(address);
        if address & 0x8000 == 0 {
            self.foreign(|game| {
                crate::game::session_runtime::invalid_window(game, address);
                #[cfg(sm_verify_dll)]
                crate::room_contact_start_observer::diagnostic(address, game);
            });
        }
        player_projectiles::content::captured(self.native_image(), address)
    }
    fn table(&self, source: Source, address: u32) -> RomRecord {
        let image = match source {
            Source::Loaded => self.loaded_image().expect("ROM must be loaded"),
            Source::Native => self.native_image(),
        };
        player_projectiles::content::captured(image, address)
    }
}
fn palette_address(member: PaletteMember, handle: u16) -> usize {
    let base = match member {
        PaletteMember::Definition => 0x1e7d,
        PaletteMember::ColorOffset => 0x1e8d,
        PaletteMember::PreAction => 0x1ead,
        PaletteMember::Program => 0x1ebd,
        PaletteMember::Delay => 0x1ecd,
        PaletteMember::Counter => 0x1edd,
    };
    base + usize::from(handle & !1)
}
fn animation_address(member: AnimationMember, handle: u16) -> usize {
    let base = match member {
        AnimationMember::Definition => 0x1ef5,
        AnimationMember::Counter => 0x1f01,
        AnimationMember::Program => 0x1f0d,
        AnimationMember::Delay => 0x1f19,
        AnimationMember::Source => 0x1f25,
    };
    base + usize::from(handle & !1)
}
impl visual_host::Aliases for Runtime<'_> {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16 {
        self.word(palette_address(member, handle))
    }
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16) {
        self.write_slice_le(palette_address(member, handle), value);
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1edd + usize::from(handle), value);
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        self.native_word(0x1e79 + usize::from(displacement))
    }
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16 {
        self.word(animation_address(member, handle))
    }
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16) {
        self.write_word_le(value, animation_address(member, handle));
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1f01 + usize::from(handle), value);
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        self.write_slice_le(0xc000 + usize::from(displacement & !1), value);
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        let base: usize = match buffer {
            PaletteBuffer::Current => 0xc000,
            PaletteBuffer::Target => 0xc200,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("palette index overflow"))
            .expect("palette address overflow");
        self.write_word_le(value, at);
    }
}
impl visual_host::Effects for Runtime<'_> {
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        self.foreign(|game| crate::game::sm_81::DrawSamusSpritemap(index, x, y, game));
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        use visual_host::{Content, Record};
        let frame_offset =
            2 * u16::from(self.session().atmospheric.slots[slot].frame_and_kind as u8);
        let saved = self.word(0x590);
        let x = self.session().atmospheric.slots[slot]
            .x
            .wrapping_sub(self.word(0x911))
            .wrapping_sub(4);
        if x as i16 >= 0
            && self.session().atmospheric.slots[slot]
                .x
                .wrapping_sub(self.word(0x911))
                .wrapping_sub(260)
                & 0x8000
                != 0
        {
            let entry = 0x370 + usize::from(self.word(0x590));
            self.set_byte(entry, x as u8);
            // X may alias the y coordinate, camera, cursor or another owned field.
            let y = self.session().atmospheric.slots[slot]
                .y
                .wrapping_sub(self.word(0x915))
                .wrapping_sub(4);
            if y as i16 >= 0
                && self.session().atmospheric.slots[slot]
                    .y
                    .wrapping_sub(self.word(0x915))
                    .wrapping_sub(260)
                    & 0x8000
                    != 0
            {
                self.set_byte(entry + 1, y as u8);
                let pointer = self
                    .table(Source::Loaded, 0x908bff)
                    .native_word(2 * usize::from(kind));
                let tile = self
                    .atmospheric_record(pointer.wrapping_add(frame_offset))
                    .native_word(0);
                self.write_slice_native(entry + 2, tile);
                self.write_word_le(saved.wrapping_add(4), 0x590);
                crate::game::session_access::SessionAccess::probe(self, "atmosphere.tile.written");
            }
        }
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        player_projectiles::host::Host::sound(self, channel, limit, sound);
    }
    fn music_delayed8(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game));
    }
    fn resume_movement_sound(&mut self) {
        #[cfg(sm_verify_dll)]
        samus_observer::event("command", 0x1c);
        self.foreign(|game| unsafe {
            crate::game::sm_90::CallSomeSamusCode(0x1c, game);
        });
    }
    fn upload(&mut self, request: Upload) {
        self.foreign(|game| animated_tiles::native::upload(request, game));
    }
    fn invalid_palette_instruction(&mut self) {
        self.foreign(|game| unsafe {
            crate::rtl::Unreachable(game);
        });
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.foreign(animated_tiles::native::unreachable)
    }
    fn color_written(&mut self, _index: u16, _value: u16) {}
}
impl visual_host::Composition for Runtime<'_> {
    fn spawn_palette(&mut self, definition: u16) {
        palette_effects::execution::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        use crate::game::constants::bank_86::{
            addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul,
        };
        let definition = match kind {
            StatueProjectile::EyeGlow => addr_kEproj_TourianStatueEyeGlow,
            StatueProjectile::Soul => addr_kEproj_TourianStatueSoul,
        };
        enemy_projectiles::lifecycle::spawn_room(self, definition as u16, parameter);
    }
}
impl visual_host::Host for Runtime<'_> {}
#[path = "../verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[path = "../verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
#[path = "../verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;
