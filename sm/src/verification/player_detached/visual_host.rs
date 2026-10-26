//! Shared visual host: actual core composition and explicitly prepared external edges.
#![allow(unused_variables)]
use super::game::{
    session_access::SessionAccess,
    visual_host::*,
    visual_state::{PaletteBuffer, Upload},
};
use super::{Composed, Event, RomRecord, animated, colors};
impl Record for RomRecord {
    fn byte(&self, offset: usize) -> u8 {
        RomRecord::byte(self, offset)
    }
    fn native_word(&self, offset: usize) -> u16 {
        RomRecord::word(self, offset)
    }
    fn bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        let start = self
            .base
            .checked_add(offset)
            .expect("prepared visual record overflow");
        let end = start
            .checked_add(N)
            .expect("prepared visual record extent overflow");
        assert!(end <= self.image.len(), "prepared visual record extent");
        core::array::from_fn(|i| self.image.read(start + i))
    }
}
impl Content for Composed {
    type Record = RomRecord;
    fn record(&mut self, source: Source, bank: u8, address: u16) -> RomRecord {
        if bank == 0xb4 && address & 0x8000 == 0 {
            if let Some(callback) = self.on_sprite_record {
                callback(self, address);
            } else {
                assert!(address & 0x8000 != 0, "unprepared sprite low-window record");
            }
        } else {
            assert!(address & 0x8000 != 0, "unprepared visual low-window record");
        }
        self.visual_records.push((source, bank, address));
        if let Some(callback) = self.on_visual_record {
            callback(self, source, bank, address);
        }
        RomRecord {
            image: self.transition_image(source),
            base: usize::from(bank & 0x7f) * 0x8000 + usize::from(address & 0x7fff),
        }
    }
    fn table(&self, source: Source, address: u32) -> RomRecord {
        RomRecord {
            image: self.transition_image(source),
            base: ((address as usize >> 16) & 0x7f) * 0x8000 + (address as usize & 0x7fff),
        }
    }
}
impl Aliases for Composed {
    fn palette_word(&self, _: PaletteMember, _: u16) -> u16 {
        panic!("unprepared palette field alias")
    }
    fn set_palette_word_slice(&mut self, _: PaletteMember, _: u16, _: u16) {
        panic!("unprepared palette field alias")
    }
    fn set_palette_counter_byte(&mut self, _: u16, _: u8) {
        panic!("unprepared palette counter alias")
    }
    fn palette_flag_native(&self, _: u16) -> u16 {
        panic!("unprepared palette flag alias")
    }
    fn animation_word(&self, _: AnimationMember, _: u16) -> u16 {
        panic!("unprepared animation field alias")
    }
    fn set_animation_word_prefix(&mut self, _: AnimationMember, _: u16, _: u16) {
        panic!("unprepared animation field alias")
    }
    fn set_animation_counter_byte(&mut self, _: u16, _: u8) {
        panic!("unprepared animation counter alias")
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        let start = 0xc000 + usize::from(displacement & !1);
        // Actual aggregate projection, with whole-word preflight before slice write.
        assert!(super::game::session_encoding::projected_byte(&self.world.player, start).is_some());
        assert!(
            super::game::session_encoding::projected_byte(&self.world.player, start + 1).is_some()
        );
        for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
            assert!(super::game::session_encoding::write_projected_byte(
                &mut self.world.player,
                start + offset,
                byte
            ));
        }
    }
    fn set_color_prefix(&mut self, _: PaletteBuffer, _: usize, _: u16) {
        panic!("unprepared animation color alias")
    }
}
impl Effects for Composed {
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        self.events.push(Event::SamusSprite(index, x, y));
        if let Some(callback) = self.on_sprite_draw {
            callback(self);
        }
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        let s = &self.world.player.atmospheric.slots[slot];
        self.events.push(Event::AtmosphericTile(
            slot,
            kind,
            s.frame_and_kind,
            s.x,
            s.y,
        ));
        if let Some(callback) = self.on_sprite_draw {
            callback(self);
        }
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        self.events.push(Event::Sound(channel, sound));
        if let Some(callback) = self.on_visual_sound {
            callback(self);
        }
    }
    fn music_delayed8(&mut self, track: u16) {
        self.events.push(Event::Music(track));
        if let Some(callback) = self.on_visual_music {
            callback(self);
        }
    }
    fn resume_movement_sound(&mut self) {
        panic!("unprepared visual movement sound")
    }
    fn upload(&mut self, request: Upload) {
        self.events.push(Event::Upload(request));
        if let Some(update) = self.on_upload.take() {
            update(&mut self.world);
        }
        assert!(!self.fail_upload, "intentional upload failure");
    }
    fn invalid_palette_instruction(&mut self) {
        panic!("invalid palette command")
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.animation_invalid_result
            .unwrap_or_else(|| panic!("invalid animation instruction"))
    }
    fn color_written(&mut self, index: u16, value: u16) {
        self.events.push(Event::Color(usize::from(index), value));
    }
}
impl Composition for Composed {
    fn spawn_palette(&mut self, definition: u16) {
        self.events.push(Event::PaletteSpawn(definition));
        colors::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        self.events.push(Event::Projectile(kind, parameter));
        let definition = match kind {
            StatueProjectile::EyeGlow => 0xba6a,
            StatueProjectile::Soul => 0xba94,
        };
        super::game::enemy_projectiles::spawn_room(self, definition, parameter);
    }
}
impl Host for Composed {}
