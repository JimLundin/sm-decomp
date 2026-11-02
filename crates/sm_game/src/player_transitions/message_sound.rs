//! Message completion sounds on the current player and inventory owner.
#![forbid(unsafe_code)]
use crate::session_access::SessionAccess;

pub trait MessageState {
    fn message_state(&self) -> &crate::session::Session;
}
impl<T: SessionAccess> MessageState for T {
    fn message_state(&self) -> &crate::session::Session {
        self.session()
    }
}
pub fn resume_movement<H: MessageState>(h: &mut H, mut sound: impl FnMut(&mut H, u8, u8, u16)) {
    if h.message_state().player.contact.speed_boost & 0xff00 == 0x400 {
        sound(h, 3, 6, 0x2b);
    }
    if h.message_state()
        .projectiles
        .controls
        .charge
        .wrapping_sub(16)
        & 0x8000
        == 0
    {
        sound(h, 1, 6, 0x41);
    }
}
pub fn restore_control<H: MessageState>(h: &mut H, mut sound: impl FnMut(&mut H, u8, u8, u16)) {
    if h.message_state().inventory.health.current.wrapping_sub(31) & 0x8000 != 0 {
        sound(h, 3, 6, 2);
    }
    if h.message_state().player.grapple != 0xc4f0 {
        sound(h, 1, 6, 6);
    } else if h.message_state().player.movement_and_facing.to_le_bytes()[0] == 3 {
        let cue = match h.message_state().player.pose {
            0x81 | 0x82 => 0x33,
            0x1b | 0x1c => 0x3e,
            _ => 0x31,
        };
        sound(h, 1, 6, cue);
    }
}
