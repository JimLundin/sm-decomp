#![forbid(unsafe_code)]
#[cfg(sm_verify_dll)]
use crate::{
    game_state::WorkRam,
    rom::RomImage,
    room_background_support::ram_word,
    room_reference_cases::{replacement, snapshot_parts},
};
use std::cell::RefCell;
struct Capture {
    id: u32,
    count: u32,
    bytes: Vec<u8>,
}
thread_local! {static ACTIVE:RefCell<Option<Capture>>=const{RefCell::new(None)};}
pub(crate) fn begin(id: u32) {
    ACTIVE.with(|a| {
        assert!(a.borrow().is_none());
        *a.borrow_mut() = Some(Capture {
            id,
            count: 0,
            bytes: Vec::new(),
        });
    });
}
pub(crate) fn finish() -> Vec<u8> {
    ACTIVE.with(|a| {
        let c = a.borrow_mut().take().unwrap();
        assert_eq!(c.count, if c.id == 32 || c.id == 33 { 1 } else { 0 });
        c.bytes
    })
}
#[cfg(sm_verify_dll)]
pub(crate) fn diagnostic_parts(
    address: u32,
    ram: &WorkRam,
    loaded_rom: &mut Option<RomImage>,
    fail: &bool,
) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        assert!(c.id == 32 || c.id == 33);
        assert_eq!(c.count, 0);
        assert_eq!(address, 0x8f0010);
        c.count += 1;
        c.bytes.extend_from_slice(&address.to_le_bytes());
        snapshot_parts(&mut c.bytes, ram, fail);
        *loaded_rom = Some(replacement(c.id));
        ram_word(
            ram,
            if c.id == 32 { 0x7bb } else { 0x79b },
            if c.id == 32 { 0xb000 } else { 0xa000 },
        );
        snapshot_parts(&mut c.bytes, ram, fail);
    });
}
