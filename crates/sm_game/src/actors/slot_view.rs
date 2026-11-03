//! An actor family's slot state, named by field, on the lease's Session.
//!
//! # Design
//!
//! A slot's state is the Session's [`Actor`] record and its 96 extended words
//! (`var_00`..`var_5f`, `actors.extended`). A family declares its view with
//! [`slot_view!`], naming each word it uses after the original's slot word
//! (`x_velocity = var_a`, `link = var_00`); the view reads those words as
//! typed fields, and its edit hands out `&mut u16`s to them. The common record
//! fields are the [`Actor`]'s own (`health`, `instruction`, `x.whole`, ...).
//!
//! The view reaches the storage the retired `session_slot!` resolved to: the actor
//! record is the codec's projection of $7E:0F78 + slot (no other owner
//! projects those bytes), and the extended words are the words of each block
//! the slot index selects. Words are little-endian in the Session, as the
//! projected bytes were.
//!
//! Stores to the record keep the one side effect the lease's projected store
//! had: the host refreshes its external observations (enemy 0's health, and
//! the rest of the inputs) after them, through [`SlotState::record_stored`].
//! The edit runs it once when it ends; the projected store ran it after each
//! byte, but nothing can read the observations while the edit holds the host,
//! so the result is the same. Extended words, as before, have none.
//!
//! Slot 0's extended words $7E:801E, $8020, $803C, $8818 and $883E are
//! Botwoon's death protocol's (`Session::botwoon_death`), which the projection
//! serves there for any family; the view reads and stores them there too, and
//! stores to them refresh the observations as record stores do.
use crate::actors::botwoon_death::DeathProtocol;
use crate::actors::state::{Actor, ExtendedRam};
use crate::session_access::SessionAccess;

/// A host whose actor slots are the Session's typed storage.
pub trait SlotState: SessionAccess {
    /// Refresh the host's external observations after stores to an actor's
    /// record.
    fn record_stored(&mut self);
}

/// The Session slot of the actor at byte index `k` ($7E:0F78 + `k`).
pub fn slot_index(k: u16) -> usize {
    debug_assert_eq!(k % 64, 0, "actor index {k:#x} is not a slot");
    usize::from(k) / 64
}

/// The block and word of extended word `var` of `slot`.
fn extended_at(slot: usize, var: usize) -> (usize, usize) {
    (var / 32, slot * 32 + var % 32)
}

/// Slot 0's extended word `var` when Botwoon's death protocol holds it.
macro_rules! death_word {
    ($death:expr, $var:expr, $($borrow:tt)*) => {
        match $var {
            0x2f => Some($($borrow)* $death.pending_hit),
            0x30 => Some($($borrow)* $death.active),
            0x3e => Some($($borrow)* $death.tail_ready),
            0x4c => Some($($borrow)* $death.previous_health),
            0x5f => Some($($borrow)* $death.body_landed),
            _ => None,
        }
    };
}

/// The extended word number of the layout word at byte `offset` of
/// `EnemyLayout104` (whose `var_00` lies at $7E:7800 for slot 0).
pub const fn extended_var(offset: usize) -> usize {
    const VAR_00: usize = 0x7800 - 0x0f78;
    let at = offset - VAR_00;
    assert!(at.is_multiple_of(2) && at / 0x800 < 3 && at % 0x800 < 64);
    (at / 0x800) * 32 + (at % 0x800) / 2
}

/// The record words a view may name, at the offsets the codec projects them.
const _: () = {
    use crate::actors::layouts::EnemyLayout104 as Layout;
    use core::mem::offset_of;
    assert!(offset_of!(Layout, var_a) == 0x30 && offset_of!(Layout, var_b) == 0x32);
    assert!(offset_of!(Layout, var_c) == 0x34 && offset_of!(Layout, var_d) == 0x36);
    assert!(offset_of!(Layout, var_e) == 0x38 && offset_of!(Layout, var_f) == 0x3a);
    assert!(offset_of!(Layout, parameter_1) == 0x3c && offset_of!(Layout, parameter_2) == 0x3e);
};

/// The record field the codec projects at a layout record word.
#[macro_export]
macro_rules! record_word {
    ($actor:expr, var_a) => {
        $actor.variables[0]
    };
    ($actor:expr, var_b) => {
        $actor.variables[1]
    };
    ($actor:expr, var_c) => {
        $actor.variables[2]
    };
    ($actor:expr, var_d) => {
        $actor.variables[3]
    };
    ($actor:expr, var_e) => {
        $actor.variables[4]
    };
    ($actor:expr, var_f) => {
        $actor.pre_instruction
    };
    ($actor:expr, parameter_1) => {
        $actor.parameters[0]
    };
    ($actor:expr, parameter_2) => {
        $actor.parameters[1]
    };
}

/// Store `value` to byte `lane` (0 the low, 1 the high) of `word`, as a byte
/// store to the word's RAM does.
pub fn store_byte(word: &mut u16, lane: usize, value: u8) {
    let mut bytes = word.to_le_bytes();
    bytes[lane] = value;
    *word = u16::from_le_bytes(bytes);
}

/// One slot's state, read.
#[derive(Clone, Copy)]
pub struct SlotRef<'a> {
    pub actor: &'a Actor,
    extended: &'a ExtendedRam,
    death: &'a DeathProtocol,
    slot: usize,
}
impl<'a> SlotRef<'a> {
    pub fn of<H: SessionAccess + ?Sized>(host: &'a H, k: u16) -> Self {
        let session = host.session();
        let slot = slot_index(k);
        Self {
            actor: &session.actors.slots[slot],
            extended: &session.actors.extended,
            death: &session.botwoon_death,
            slot,
        }
    }
    /// Extended word `var`.
    pub fn extended(&self, var: usize) -> u16 {
        if self.slot == 0
            && let Some(word) = death_word!(self.death, var, &)
        {
            return *word;
        }
        let (block, word) = extended_at(self.slot, var);
        self.extended.blocks[block][word]
    }
}

/// One slot's state, edited; the host's observations refresh when the edit
/// ends if the record was borrowed mutably.
pub struct SlotMut<'a, H: SlotState + ?Sized> {
    host: &'a mut H,
    slot: usize,
    record_stored: bool,
}
impl<'a, H: SlotState + ?Sized> SlotMut<'a, H> {
    pub fn of(host: &'a mut H, k: u16) -> Self {
        Self {
            host,
            slot: slot_index(k),
            record_stored: false,
        }
    }
    /// The record, to store to.
    pub fn actor(&mut self) -> &mut Actor {
        self.record_stored = true;
        &mut self.host.session_mut().actors.slots[self.slot]
    }
    /// Extended word `var`, to store to.
    pub fn extended(&mut self, var: usize) -> &mut u16 {
        let session = self.host.session_mut();
        if self.slot == 0 && death_word!(session.botwoon_death, var, &).is_some() {
            self.record_stored = true;
            return death_word!(session.botwoon_death, var, &mut).expect("a Botwoon death word");
        }
        let (block, word) = extended_at(self.slot, var);
        &mut session.actors.extended.blocks[block][word]
    }
}
impl<H: SlotState + ?Sized> Drop for SlotMut<'_, H> {
    fn drop(&mut self) {
        if self.record_stored {
            self.host.record_stored();
        }
    }
}

/// A family's slot view `$name` and edit `$edit`: each record field names the
/// layout record word (`var_a`..`var_f`, `parameter_1`, `parameter_2`) and
/// each extended field the layout word (`var_00`..`var_5f`) it is.
#[macro_export]
macro_rules! slot_view {
    ($(#[$doc:meta])* $name:ident, $edit:ident {
        record: [$($field:ident = $word:ident),* $(,)?],
        extended: [$($xfield:ident = $xword:ident),* $(,)?] $(,)?
    }) => {
        $(#[$doc])*
        #[derive(Clone, Copy)]
        pub struct $name<'a>($crate::actors::slot_view::SlotRef<'a>);
        impl<'a> $name<'a> {
            pub fn of<H: $crate::session_access::SessionAccess + ?Sized>(host: &'a H, k: u16) -> Self {
                Self($crate::actors::slot_view::SlotRef::of(host, k))
            }
            /// The common record.
            pub fn actor(&self) -> &'a $crate::actors::state::Actor {
                self.0.actor
            }
            $(pub fn $field(&self) -> u16 {
                $crate::record_word!(self.0.actor, $word)
            })*
            $(pub fn $xfield(&self) -> u16 {
                self.0.extended(const {
                    $crate::actors::slot_view::extended_var(::core::mem::offset_of!(
                        $crate::actors::layouts::EnemyLayout104, $xword))
                })
            })*
        }
        #[doc = concat!("[`", stringify!($name), "`], edited.")]
        pub struct $edit<'a, H: $crate::actors::slot_view::SlotState + ?Sized>(
            $crate::actors::slot_view::SlotMut<'a, H>,
        );
        impl<'a, H: $crate::actors::slot_view::SlotState + ?Sized> $edit<'a, H> {
            pub fn of(host: &'a mut H, k: u16) -> Self {
                Self($crate::actors::slot_view::SlotMut::of(host, k))
            }
            /// The common record, to store to.
            pub fn actor(&mut self) -> &mut $crate::actors::state::Actor {
                self.0.actor()
            }
            $(pub fn $field(&mut self) -> &mut u16 {
                &mut $crate::record_word!(self.0.actor(), $word)
            })*
            $(pub fn $xfield(&mut self) -> &mut u16 {
                self.0.extended(const {
                    $crate::actors::slot_view::extended_var(::core::mem::offset_of!(
                        $crate::actors::layouts::EnemyLayout104, $xword))
                })
            })*
        }
    };
}
pub use crate::slot_view;
