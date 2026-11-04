//! Explicit injected diagnostic/draw transitions for actual predecessor execution.
#![forbid(unsafe_code)]
use super::sprite_effect_cases::{Operation, put, word};
use crate::game_state::Game;
use std::cell::RefCell;
#[cfg(sm_verify_dll)]
use {super::sprite_effect_cases::put_parts, crate::game_state::WorkRam, crate::rom::RomImage};
struct Capture {
    id: u32,
    diagnostics: usize,
    draws: usize,
    bytes: Vec<u8>,
}
thread_local! {static ACTIVE:RefCell<Option<Capture>>=const{RefCell::new(None)};}
pub(crate) fn begin(id: u32) {
    ACTIVE.with(|a| {
        assert!(a.borrow().is_none());
        *a.borrow_mut() = Some(Capture {
            id,
            diagnostics: 0,
            draws: 0,
            bytes: if id >= 111 {
                b"SPSE1".to_vec()
            } else {
                Vec::new()
            },
        });
    });
}
/// Whether a sprite-effect case is capturing; its draws run on the lease.
#[cfg(sm_verify_dll)]
pub(crate) fn observing() -> bool {
    ACTIVE.with(|a| a.borrow().is_some())
}
pub(crate) fn finish() -> Vec<u8> {
    let captured = ACTIVE.with(|a| a.borrow_mut().take().unwrap());
    if captured.id >= 111 {
        let expected =
            [(1, 0), (2, 0), (1, 0), (0, 3), (1, 1), (0, 2)][(captured.id - 111) as usize];
        assert_eq!(
            (captured.diagnostics, captured.draws),
            expected,
            "sprite.observer.{}.hook-counts",
            captured.id
        );
    }
    captured.bytes
}
#[cfg(sm_verify_dll)]
fn replacement(words: &[(u32, u16)]) -> RomImage {
    let mut image = vec![0; 0x200000];
    for &(address, value) in words {
        word(&mut image, address, value);
    }
    RomImage::from(image)
}
#[cfg(sm_verify_dll)]
pub(crate) fn diagnostic_parts(address: u32, ram: &WorkRam, loaded_rom: &mut Option<RomImage>) {
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        if c.id < 111 {
            return;
        }
        c.bytes.push(1);
        c.bytes.extend_from_slice(&address.to_le_bytes());
        let step = c.diagnostics;
        c.diagnostics += 1;
        match (c.id, step) {
            (111, 0) => {
                assert_eq!(address, 0xb40010);
                put_parts(ram, 0x1844, 58);
                *loaded_rom = Some(replacement(&[(0xb48010, 7)]));
            }
            (112, 0) => {
                assert_eq!(address, 0xb41000);
                put_parts(ram, 0x1844, 58);
                *loaded_rom = Some(replacement(&[(0xb49002, 0x1200)]));
            }
            (112, 1) => {
                assert_eq!(address, 0xb41200);
                put_parts(ram, 0x1844, 54);
                *loaded_rom = Some(replacement(&[(0xb49200, 9)]));
            }
            (113, 0) => {
                assert_eq!(address, 0xb41000);
                put_parts(ram, 0x1844, 58);
                *loaded_rom = Some(replacement(&[(0xb49000, 11)]));
            }
            (115, 0) => {
                assert_eq!(address, 0xb41000);
                *loaded_rom = Some(replacement(&[
                    (0xb49002, 0xa100),
                    (0xb4a100, 1),
                    (0xb4a102, 6),
                    (0xb4a104, 2),
                    (0xb4a105, 0x15),
                ]));
            }
            _ => panic!("unexpected sprite diagnostic transition"),
        }
    });
}
/// Record one draw and, on a case's first draw, store its slot changes
/// through the lease's byte path, as the effect table's next reads see them.
#[cfg(sm_verify_dll)]
fn draw(
    map: u16,
    x: u16,
    y: u16,
    palette: u16,
    tile: u16,
    lease: &mut crate::game::session_runtime::Runtime<'_>,
) {
    let mut put = |at: usize, value: u16| lease.write_slice_le(at, value);
    ACTIVE.with(|a| {
        let mut active = a.borrow_mut();
        let Some(c) = active.as_mut() else {
            return;
        };
        if c.id < 111 {
            return;
        }
        c.bytes.push(2);
        for value in [map, x, y, palette, tile] {
            c.bytes.extend_from_slice(&value.to_le_bytes());
        }
        if c.draws == 0 {
            match c.id {
                114 => {
                    put(0xef78 + 10, 0x9100);
                    put(0xf0f8 + 10, 99);
                    put(0xf1f8 + 10, 77);
                    put(0xf078 + 10, 0xffff);
                }
                116 => put(0xef78 + 10, 0),
                _ => (),
            }
        }
        c.draws += 1;
    });
}
pub(crate) fn seed(id: u32, g: &mut Game, image: &mut [u8]) -> Operation {
    match id {
        111 => {
            put(g, 0xef78 + 62, 0xc);
            put(g, 0xeff8 + 62, 1);
            put(g, 0xf2f8 + 62, 0);
            word(image, 0xb48010, 3);
            for slot in [30, 28] {
                put(g, 0xef78 + slot * 2, 0x9000);
                put(g, 0xeff8 + slot * 2, 2);
                put(g, 0xf2f8 + slot * 2, 0);
            }
            Operation::Update
        }
        112 => {
            put(g, 0xef78 + 62, 0x1000);
            put(g, 0xeff8 + 62, 0xbd12);
            put(g, 0xf2f8 + 62, 0);
            word(image, 0xb49002, 0x9200);
            put(g, 0xef78 + 52, 0x9000);
            put(g, 0xeff8 + 52, 2);
            put(g, 0xf2f8 + 52, 0);
            Operation::Update
        }
        113 => {
            word(image, 0xb4bda8, 0x1000);
            Operation::Spawn
        }
        114 | 116 => {
            for (slot, x) in [(31, 63), (5, 47), (0, 31)] {
                put(g, 0xef78 + slot * 2, 0x9000);
                put(g, 0xf0f8 + slot * 2, x);
                put(g, 0xf1f8 + slot * 2, 30);
                put(g, 0xf078 + slot * 2, 0xe00 + slot as u16);
            }
            Operation::Draw
        }
        115 => {
            put(g, 0xef78 + 62, 0x1000);
            put(g, 0xf0f8 + 62, 30);
            put(g, 0xf1f8 + 62, 40);
            word(image, 0xb4a100, 1);
            word(image, 0xb4a102, 4);
            word(image, 0xb4a104, 2);
            word(image, 0xb4a105, 9);
            Operation::Draw
        }
        _ => unreachable!(),
    }
}
/// A sprite-effect case's draw on the scheduler's lease: while a case is
/// capturing, its observer stores through the lease and the sprite draws on
/// the lease's objects, as an uncaptured draw does. False when no case is
/// capturing.
#[cfg(sm_verify_dll)]
pub(crate) fn draw_on_lease(
    runtime: &mut crate::game::session_runtime::Runtime<'_>,
    map: u16,
    position: (u16, u16),
    palette: u16,
    tile: u16,
) -> bool {
    use sm_game::{sprite_drawing::Clip, sprite_output};
    if !observing() {
        return false;
    }
    let (x, y) = position;
    draw(map, x, y, palette, tile, runtime);
    runtime.draw_objects(|drawing| {
        sprite_output::with_base_tile(0xb4, map, position, palette, tile, Clip::None, drawing);
    });
    true
}
