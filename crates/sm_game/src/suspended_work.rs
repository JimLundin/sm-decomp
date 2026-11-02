//! Shared suspended work. The timer remains Session.messages.delay.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SuspendedWork {
    pub decompression_source: u32,
    pub queue_cursor: u16,
    pub pause: [u8; 3],
    pub unpause: [u8; 3],
}
#[derive(Clone, Copy)]
pub enum Hook {
    Pause,
    Unpause,
}
impl SuspendedWork {
    pub fn hook(&self, hook: Hook) -> &[u8; 3] {
        match hook {
            Hook::Pause => &self.pause,
            Hook::Unpause => &self.unpause,
        }
    }
    pub fn hook_mut(&mut self, hook: Hook) -> &mut [u8; 3] {
        match hook {
            Hook::Pause => &mut self.pause,
            Hook::Unpause => &mut self.unpause,
        }
    }
    /// Original Load24 consumes four native bytes before the mask. The caller
    /// acquires the unpause following byte from its earthquake-sound owner.
    pub fn native_hook(&self, hook: Hook, following: u8) -> u32 {
        let [a, b, c] = *self.hook(hook);
        u32::from_ne_bytes([a, b, c, following]) & 0xffffff
    }
    pub fn store_hook_byte(&mut self, hook: Hook, lane: usize, value: u8) {
        self.hook_mut(hook)[lane] = value;
    }
    pub fn store_hook_word(
        &mut self,
        hook: Hook,
        lane: usize,
        value: u16,
        order: crate::scene::state::WordOrder,
    ) {
        let [first, second] = order.encode(value);
        self.store_hook_byte(hook, lane, first);
        self.store_hook_byte(hook, lane + 1, second);
    }
    pub fn store_hook_long(&mut self, hook: Hook, value: u32) {
        let [low, high, bank, _] = value.to_le_bytes();
        self.store_hook_byte(hook, 0, low);
        self.store_hook_byte(hook, 1, high);
        self.store_hook_byte(hook, 2, bank);
    }
    pub(crate) fn projected_byte(&self, at: usize) -> Option<u8> {
        match at {
            0x601..=0x603 => Some(self.pause[at - 0x601]),
            0x604..=0x606 => Some(self.unpause[at - 0x604]),
            0x780..=0x781 => Some(self.queue_cursor.to_le_bytes()[at - 0x780]),
            _ => None,
        }
    }
    pub(crate) fn write_projected_byte(&mut self, at: usize, value: u8) -> bool {
        match at {
            0x601..=0x603 => self.pause[at - 0x601] = value,
            0x604..=0x606 => self.unpause[at - 0x604] = value,
            0x780..=0x781 => {
                let mut bytes = self.queue_cursor.to_le_bytes();
                bytes[at - 0x780] = value;
                self.queue_cursor = u16::from_le_bytes(bytes);
            }
            _ => return false,
        }
        true
    }
}
pub fn wait_for_nmi(session: &mut crate::session::Session) -> u8 {
    session
        .presentation
        .owner_mut()
        .expect("suspended display owner")
        .state
        .nmi
        .wait(true)
}
