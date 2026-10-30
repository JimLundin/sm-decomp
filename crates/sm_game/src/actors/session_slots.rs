//! Enemy slot words read and written on the owned Session, for code running
//! inside a lease.
//!
//! # Design
//!
//! A species view (`part_view!` or a hand-written view checked by
//! `same_words!`) still names each word by its byte offset in the slot, as
//! `slot!` does over the RAM image. Inside a lease, [`session_slot!`] takes the
//! same paths, resolves the same address (and checks the field is a word), and
//! then reads and writes the Session's typed storage instead of the RAM image:
//! the extended RAM at $7E:7800..$8FFF is `Actors::extended`'s word for that
//! address, and every other slot word (the 64-byte record, Botwoon's death
//! words) goes through the host's Session projection, which already owns it. The bytes
//! a store leaves are the ones the RAM image would hold, so neighbour-slot and
//! off-table paths behave exactly as they do through `slot!`.
//!
//! Routines shared between a lease and a host that still holds the RAM image
//! are written against [`SlotAccess`], which both implement: the RAM image's
//! native-order bytes there, the Session here.
//!
//! Words are native order in the RAM image and little-endian in the Session;
//! the two agree on the little-endian hosts the actor codec borrows directly.
#![forbid(unsafe_code)]
use crate::session_access::SessionAccess;

/// The extended RAM's span, less the Botwoon death words its own cohort keeps.
fn extended_index(at: usize) -> Option<(usize, usize)> {
    match at {
        0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e => None,
        0x7800..0x9000 if at % 2 == 0 => Some(((at - 0x7800) / 0x800, (at - 0x7800) % 0x800 / 2)),
        _ => None,
    }
}

/// Slot words and bytes, by RAM address: the Session's inside a lease, the RAM
/// image's on a host that still holds it.
pub trait SlotAccess {
    fn slot_word(&self, at: usize) -> u16;
    fn set_slot_word(&mut self, at: usize, value: u16);
    fn slot_byte(&self, at: usize) -> u8;
    fn set_slot_byte(&mut self, at: usize, value: u8);
}

/// Slot words and bytes on the Session. The extended RAM is served here; the
/// host serves the other slot bytes through its projection, which also keeps
/// its observations (enemy 0's health) current.
pub trait SessionSlots: SessionAccess {
    /// The projected byte at `at`.
    fn projected_slot_byte(&self, at: usize) -> u8;
    /// Store the projected byte at `at`.
    fn store_projected_slot_byte(&mut self, at: usize, value: u8);
}

impl<T: SessionSlots + ?Sized> SlotAccess for T {
    fn slot_word(&self, at: usize) -> u16 {
        if let Some((block, index)) = extended_index(at) {
            return self.session().actors.extended.blocks[block][index];
        }
        u16::from_le_bytes([self.slot_byte(at), self.slot_byte(at + 1)])
    }
    fn set_slot_word(&mut self, at: usize, value: u16) {
        if let Some((block, index)) = extended_index(at) {
            self.session_mut().actors.extended.blocks[block][index] = value;
            return;
        }
        let [low, high] = value.to_le_bytes();
        self.set_slot_byte(at, low);
        self.set_slot_byte(at + 1, high);
    }
    fn slot_byte(&self, at: usize) -> u8 {
        if let Some((block, index)) = extended_index(at & !1) {
            return self.session().actors.extended.blocks[block][index].to_le_bytes()[at & 1];
        }
        self.projected_slot_byte(at)
    }
    fn set_slot_byte(&mut self, at: usize, value: u8) {
        if let Some((block, index)) = extended_index(at & !1) {
            let word = &mut self.session_mut().actors.extended.blocks[block][index];
            let mut bytes = word.to_le_bytes();
            bytes[at & 1] = value;
            *word = u16::from_le_bytes(bytes);
            return;
        }
        self.store_projected_slot_byte(at, value);
    }
}

/// `slot!` on the Session: the same view paths and forms (word read, store,
/// ordered stores, read-modify-write and byte lanes), served from the Session.
#[macro_export]
macro_rules! session_slot {
    (byte $h:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {
        match ($value, $crate::ram_slot!(@at $layout, $slot, $($field).+)) {
            (value, at) => {
                let value: ::core::primitive::u8 = value;
                $crate::actors::session_slots::SlotAccess::set_slot_byte(
                    $h, at + ($lane) as usize, value)
            }
        }
    };
    (byte $h:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::session_slots::SlotAccess::slot_byte(
            &*$h, $crate::ram_slot!(@at $layout, $slot, $($field).+) + ($lane) as usize)
    };
    // Split-word pairs, as `slot!`'s: the value first, then low is read before
    // high and written before high.
    ($h:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {
        match $value {
            value => {
                let value: ::core::primitive::u32 = value;
                let low = ::core::primitive::u32::from($crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$lo));
                let high = ::core::primitive::u32::from($crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$hi));
                let sum = (low | (high << 16)).wrapping_add(value);
                $crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$lo = sum as ::core::primitive::u16);
                $crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as ::core::primitive::u16);
            }
        }
    };
    ($h:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) = $value:expr) => {
        match $value {
            value => {
                let value: ::core::primitive::u32 = value;
                $crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$lo = value as ::core::primitive::u16);
                $crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$hi = (value >> 16) as ::core::primitive::u16);
            }
        }
    };
    // Ordered word stores to several fields; the slot is re-evaluated per store.
    ($h:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($stores:tt)+) => {
        $crate::session_slot!(@stores $h, $layout[$slot] ($($prefix)*); $($stores)+)
    };
    (@stores $h:ident, $layout:ident[$slot:expr] $prefix:tt; $($field:ident = $value:expr),+ $(,)?) => {
        $($crate::session_slot!(@store $h, $layout[$slot] $prefix $field = $value);)+
    };
    (@store $h:ident, $layout:ident[$slot:expr] ($($prefix:ident)*) $field:ident = $value:expr) => {
        $crate::session_slot!($h, $layout[$slot] $(.$prefix)* .$field = $value)
    };
    ($h:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {
        match $crate::ram_slot!(@at $layout, $slot, $($field).+) {
            at => {
                $crate::ram::word_field::<$layout>(|record| record $(.$field)+);
                let $old = $crate::actors::session_slots::SlotAccess::slot_word(&*$h, at);
                let value: ::core::primitive::u16 = $value;
                $crate::actors::session_slots::SlotAccess::set_slot_word($h, at, value);
            }
        }
    };
    ($h:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {
        match ($value, $crate::ram_slot!(@at $layout, $slot, $($field).+)) {
            (value, at) => {
                $crate::ram::word_field::<$layout>(|record| record $(.$field)+);
                $crate::actors::session_slots::SlotAccess::set_slot_word($h, at, value);
            }
        }
    };
    ($h:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        match $crate::ram_slot!(@at $layout, $slot, $($field).+) {
            at => {
                $crate::ram::word_field::<$layout>(|record| record $(.$field)+);
                $crate::actors::session_slots::SlotAccess::slot_word(&*$h, at)
            }
        }
    };
}
pub use crate::session_slot;
