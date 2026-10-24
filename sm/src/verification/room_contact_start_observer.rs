//! Existing low-window RomPtr effect-point observations only.
#![forbid(unsafe_code)]
use crate::{
    game_state::Game,
    rom::RomImage,
    room_contact_start_cases::{offset, snapshot},
};
use std::cell::RefCell;
struct Capture {
    id: u32,
    count: u32,
    expected: Option<Vec<u8>>,
    bytes: Vec<u8>,
}
thread_local! {static ACTIVE:RefCell<Option<Capture>>=const{RefCell::new(None)};}
pub(crate) fn begin(id: u32, expected: Option<Vec<u8>>) {
    ACTIVE.with(|a| {
        assert!(a.borrow().is_none());
        *a.borrow_mut() = Some(Capture {
            id,
            count: 0,
            expected,
            bytes: Vec::new(),
        });
    });
}
pub(crate) fn finish() -> Vec<u8> {
    ACTIVE.with(|a| {
        let c = a.borrow_mut().take().unwrap();
        assert_eq!(
            c.count,
            u32::from(c.expected.is_some()),
            "exact low-window diagnostic count"
        );
        c.bytes
    })
}
pub(crate) fn diagnostic(address: u32, g: &mut Game) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        assert_eq!(c.count, 0, "unexpected extra diagnostic");
        let expected_address = match c.id {
            6 => 0x8f0000,
            7 | 10 => 0x830010,
            15 => 0x80001a,
            19 => 0x8f0010,
            _ => panic!("unexpected low-window diagnostic"),
        };
        assert_eq!(address, expected_address);
        let mut before = Vec::new();
        snapshot(&mut before, g);
        assert_eq!(
            Some(&before),
            c.expected.as_ref(),
            "exact source prefix at diagnostic"
        );
        c.count += 1;
        c.bytes.extend_from_slice(&address.to_le_bytes());
        c.bytes.extend(before);
        if c.id == 10 {
            g.loaded_rom = Some(RomImage::zeroed(offset(0x838010) + 1));
        }
        let mut after = Vec::new();
        snapshot(&mut after, g);
        assert_eq!(
            Some(&after),
            c.expected.as_ref(),
            "injection preserves RAM/native ownership"
        );
        c.bytes.extend(after);
    });
}
