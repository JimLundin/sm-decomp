//! Records real completed byte writes; mutations occur only at source effects.
use crate::{game_state::Game, rom::RomImage, room_background_support::*};
use std::cell::RefCell;
struct Capture {
    id: u32,
    writes: Vec<(u16, u8)>,
    at: usize,
    diagnostics: Vec<u32>,
    diag_at: usize,
    a: RomImage,
    b: RomImage,
    mutations: Vec<usize>,
    bytes: Vec<u8>,
}
thread_local! {static ACTIVE:RefCell<Option<Capture>>=const{RefCell::new(None)};}
pub(crate) fn begin(
    id: u32,
    writes: Vec<(u16, u8)>,
    diagnostics: Vec<u32>,
    a: RomImage,
    b: RomImage,
    mutations: Vec<usize>,
) {
    ACTIVE.with(|s| {
        assert!(s.borrow().is_none());
        *s.borrow_mut() = Some(Capture {
            id,
            writes,
            at: 0,
            diagnostics,
            diag_at: 0,
            a,
            b,
            mutations,
            bytes: Vec::new(),
        });
    });
}
pub(crate) fn finish() -> Vec<u8> {
    ACTIVE.with(|s| {
        let c = s.borrow_mut().take().unwrap();
        assert_eq!(c.at, c.writes.len(), "ordered write count");
        assert_eq!(c.diag_at, c.diagnostics.len(), "diagnostic count");
        c.bytes
    })
}
pub(crate) unsafe fn written(address: u16, value: u8, g: &mut Game) {
    unsafe {
        ACTIVE.with(|s| {
            let mut active = s.borrow_mut();
            let Some(c) = active.as_mut() else {
                return;
            };
            assert_eq!(
                c.writes.get(c.at),
                Some(&(address, value)),
                "background {} write {}",
                c.id,
                c.at
            );
            let index = c.at;
            c.at += 1;
            let mut payload = address.to_le_bytes().to_vec();
            payload.push(value);
            record(&mut c.bytes, 1, &payload);
            if address == 0x420b {
                record(&mut c.bytes, 2, &snapshot(g));
            }
            if c.mutations.contains(&index) {
                record(&mut c.bytes, 3, &snapshot(g));
                g.loaded_rom = Some(c.b.clone());
                let source = if c.id == 4 && address == 0x4311 {
                    0x6200
                } else {
                    0x6100
                };
                rw(&c.a, 0x8fa002, source);
                bytes(&c.a, 0x8fa004, &[0x7e]);
                rw(&c.a, 0x8fa005, 0x7777);
                rw(&c.a, 0x8fa007, if source == 0x6200 { 8 } else { 6 });
                record(&mut c.bytes, 4, &snapshot(g));
                record(
                    &mut c.bytes,
                    5,
                    &(0..9)
                        .map(|i| c.a.read(offset(0x8fa002) + i))
                        .collect::<Vec<_>>(),
                );
            }
        });
    }
}
pub(crate) unsafe fn diagnostic(address: u32, g: &mut Game) {
    unsafe {
        ACTIVE.with(|s| {
            let mut active = s.borrow_mut();
            let Some(c) = active.as_mut() else {
                return;
            };
            assert_eq!(
                c.diagnostics.get(c.diag_at),
                Some(&address),
                "diagnostic order"
            );
            c.diag_at += 1;
            record(&mut c.bytes, 6, &address.to_le_bytes());
            record(&mut c.bytes, 3, &snapshot(g));
            if c.id == 29 && c.diag_at == 1 {
                g.loaded_rom = Some(c.b.clone());
                put(g, 0x7bb, 0x80);
            }
            record(&mut c.bytes, 4, &snapshot(g));
        });
    }
}
