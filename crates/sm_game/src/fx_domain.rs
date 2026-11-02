//! Exclusive FX source authority outside a Session interval.
#![forbid(unsafe_code)]
use crate::fx_state::FxState;
use crate::scene::hdma_sources::{HdmaLiveDescription, LiveOwner};
use crate::scene::state::WordOrder;
pub const fn native_order() -> WordOrder {
    if cfg!(target_endian = "little") {
        WordOrder::Little
    } else {
        WordOrder::Big
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub numeric: FxState,
    pub lighting: crate::visual_state::FirefleaLighting,
    pub sources: HdmaLiveDescription,
}
pub enum Domain {
    Unowned,
    Owned { state: Box<State>, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomBlending {
    pub primary: u16,
    pub liquid: u16,
}
pub trait Controls {
    fn room_blending(&self) -> RoomBlending;
    fn publish_room_blending(&mut self, value: RoomBlending, primary: bool);
    fn fx_controls(&self) -> FxState;
    fn publish_fx_controls(&mut self, value: FxState, fields: &[crate::fx_state::Field]);
}

/// Source roles and byte offsets select current domain buffers. Physical bank
/// mapping and out-of-owner native aliases are the compatibility host's concern.
pub trait Sources {
    /// Signed source offsets preserve native neighboring aliases on legacy hosts.
    /// Prepared hosts retain the declared allocation's bounds failure.
    fn store_source_signed_byte(&mut self, owner: LiveOwner, offset: isize, byte: u8);
    fn store_source_signed_word(
        &mut self,
        owner: LiveOwner,
        offset: isize,
        order: WordOrder,
        value: u16,
    ) {
        let bytes = order.encode(value);
        self.store_source_signed_byte(owner, offset, bytes[0]);
        self.store_source_signed_byte(
            owner,
            offset.checked_add(1).expect("source offset overflow"),
            bytes[1],
        );
    }
    /// Captured content identity. The host resolves native allocation provenance
    /// or a prepared source key; the core never interprets it as a RAM address.
    fn add_parallax_source(&mut self, selector: u16, delta: u32);
    fn source_byte(&self, owner: LiveOwner, offset: usize) -> u8;
    fn store_source_byte(&mut self, owner: LiveOwner, offset: usize, byte: u8);
    fn store_source_word(&mut self, owner: LiveOwner, offset: usize, order: WordOrder, value: u16) {
        let bytes = order.encode(value);
        self.store_source_byte(owner, offset, bytes[0]);
        self.store_source_byte(
            owner,
            offset.checked_add(1).expect("source offset overflow"),
            bytes[1],
        );
    }
    fn source_word(&self, owner: LiveOwner, offset: usize, order: WordOrder) -> u16 {
        let first = self.source_byte(owner, offset);
        let next = offset.checked_add(1).expect("source offset overflow");
        order.decode([first, self.source_byte(owner, next)])
    }
}

/// Provenance of the selected physical byte, not a byte mirror or a replay log.
/// Normalization selects an explicit snapshot range without a gameplay stamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceOrigin {
    pub address: usize,
    pub normalized: bool,
}
