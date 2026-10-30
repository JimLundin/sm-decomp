//! Prepared sprite edges on the same Composed owner, with actual aggregate projection.
use super::game::{session_encoding, sprite_effect_host::*, sprite_effect_state::*};
use super::{Composed, Event};
impl Composed {
    fn sprite_bytes(&self, address: usize) -> [u8; 2] {
        core::array::from_fn(|i| {
            session_encoding::projected_byte(&self.world.player, address + i)
                .expect("unprepared sprite alias outside shared Session")
        })
    }
    pub(super) fn write_sprite_bytes(&mut self, address: usize, bytes: [u8; 2]) {
        // Existing slice semantics preflight the complete destination before mutation.
        self.sprite_bytes(address);
        for (offset, value) in bytes.into_iter().enumerate() {
            assert!(session_encoding::write_projected_byte(
                &mut self.world.player,
                address + offset,
                value
            ));
        }
    }
}
impl Aliases for Composed {
    fn sprite_word(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        u16::from_le_bytes(self.sprite_bytes(member.address(index)))
    }
    fn set_sprite_word_le_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_sprite_bytes(member.address(index), value.to_le_bytes());
    }
    fn sprite_word_native(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        u16::from_ne_bytes(self.sprite_bytes(member.address(index)))
    }
    fn set_sprite_word_native_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_sprite_bytes(member.address(index), value.to_ne_bytes());
    }
}
impl Effects for Composed {
    fn submit_sprite(&mut self, request: SpriteDraw) {
        self.events.push(Event::Sprite(
            request.map.0,
            request.position.x,
            request.position.y,
            request.palette,
            request.base_tile,
        ));
        if let Some(callback) = self.on_sprite_draw {
            callback(self);
        }
    }
    fn invalid_sprite_instruction(&mut self) {
        panic!("invalid actual sprite instruction");
    }
}
impl Host for Composed {}

impl ControllerInputs for Composed {
    fn attachment_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes(self.sprite_bytes(at))
    }
}
