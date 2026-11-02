//! Finite services for the remaining ordinary state machines.
#![forbid(unsafe_code)]
use crate::actors::content_records::ContentSource;
use crate::actors::owned_common_lifecycle::BossState;
use crate::actors::owned_execution::PreparedActorState;

pub trait Core:
    BossState
    + ContentSource
    + crate::actors::session_slots::SessionSlots
    + crate::actors::movement::MovementHost
    + crate::room_mechanisms::host::Host
    + crate::player_projectiles::host::Host
    + crate::sprite_effect_host::Host
    + crate::player_position::PlayerPosition
    + crate::effects::SoundEffects
    + crate::records::Records
    + crate::progress::FlagHost
{
    fn remaining_unknown(&mut self) -> !;
    fn remaining_player_command(&mut self, command: u16) -> u16;
    /// Only odd/out-of-palette identities reach these explicit color aliases.
    fn remaining_color_alias_byte(&self, target: bool, byte_offset: usize) -> u8;
    fn set_remaining_color_alias_byte(&mut self, target: bool, byte_offset: usize, value: u8);
    fn remaining_color(&self, target: bool, byte_offset: usize, native: bool) -> u16 {
        let bytes = if byte_offset.is_multiple_of(2) && byte_offset < 512 {
            let visual = self.session().presentation.visual();
            let color = if target {
                visual.targets[byte_offset / 2]
            } else {
                visual.colors[byte_offset / 2]
            };
            color.to_le_bytes()
        } else {
            [
                self.remaining_color_alias_byte(target, byte_offset),
                self.remaining_color_alias_byte(target, byte_offset + 1),
            ]
        };
        if native {
            u16::from_ne_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        }
    }
    fn set_remaining_color(&mut self, target: bool, byte_offset: usize, value: u16, native: bool) {
        let bytes = if native {
            value.to_ne_bytes()
        } else {
            value.to_le_bytes()
        };
        if byte_offset.is_multiple_of(2) && byte_offset < 512 {
            let visual = self.session_mut().presentation.visual_mut();
            let color = u16::from_le_bytes(bytes);
            if target {
                visual.targets[byte_offset / 2] = color;
            } else {
                visual.colors[byte_offset / 2] = color;
            }
        } else {
            for (lane, byte) in bytes.into_iter().enumerate() {
                self.set_remaining_color_alias_byte(target, byte_offset + lane, byte);
            }
        }
    }
    fn remaining_spawn_plm(&mut self, x: u8, y: u8, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded { x, y, header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn remaining_create_sprite(&mut self, x: u16, y: u16, definition: u16, appearance: u16) -> u16 {
        let result = crate::sprite_objects::spawn(
            self,
            crate::sprite_effect_state::SpriteSpawn {
                position: crate::geometry::Point { x, y },
                definition: crate::sprite_effect_state::LegacySpriteDefinition(definition),
                appearance,
            },
        );
        crate::sprite_objects::legacy_result(result).0
    }
    /// The native allocation is captured at the original speed observation.
    /// Read all four bytes lazily in native order, including off-table offsets.
    fn remaining_speed32(&self, offset: usize) -> u32 {
        let image = self.native_content_image();
        let start = usize::try_from(sm_data::cartridge::lorom_offset(
            sm_data::enemy_speeds::ADDRESS32,
        ))
        .expect("speed table offset")
            + offset;
        u32::from_ne_bytes(core::array::from_fn(|i| image.read(start + i)))
    }
    fn remaining_cosine_full(&mut self, phase: u16, amplitude: u16) -> u32 {
        let phase = u16::from(phase.wrapping_add(64).to_le_bytes()[0]);
        crate::actors::queries::sine_product(self, amplitude, phase)
    }
    fn remaining_sine_full(&mut self, phase: u16, amplitude: u16) -> u32 {
        let phase = u16::from(phase.wrapping_add(128).to_le_bytes()[0]);
        crate::actors::queries::sine_product(self, amplitude, phase)
    }
}
impl Core for PreparedActorState<'_> {
    fn remaining_unknown(&mut self) -> ! {
        self.unknown_function()
    }
    fn remaining_player_command(&mut self, command: u16) -> u16 {
        self.projectile_player_command(command)
    }
    fn remaining_color_alias_byte(&self, target: bool, byte_offset: usize) -> u8 {
        self.current_byte((if target { 0xc200 } else { 0xc000 }) + byte_offset)
    }
    fn set_remaining_color_alias_byte(&mut self, target: bool, byte_offset: usize, value: u8) {
        self.set_current_byte((if target { 0xc200 } else { 0xc000 }) + byte_offset, value);
    }
}

/// Only owners with the original exhaustive species table provide this capability.
pub trait PreInstruction {
    fn remaining_run_pre_instruction(&mut self, address: u32);
}
impl PreInstruction for PreparedActorState<'_> {
    fn remaining_run_pre_instruction(&mut self, address: u32) {
        crate::actors::remaining_owned::bindings::pre_instruction(address, self);
    }
}
