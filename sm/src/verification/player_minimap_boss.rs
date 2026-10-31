//! Authored physical-byte contracts through actual owned boss-map suppression.
//! Candidate only: no unaligned predecessor Rust loads are executed as an oracle.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{
    game::{player_update, session_encoding},
    game_state::Game,
    ram_map::words as w,
    rom::RomImage,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) const COUNT: usize = 6;
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_word(image: &mut [u8], offset: usize, value: u16) {
    image[offset..offset + 2].copy_from_slice(&value.to_ne_bytes());
}
fn payload_text(payload: &(dyn std::any::Any + Send)) -> Option<&str> {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
}
/// Suppress only the exact expected bounded-ROM failure. Other panics retain
/// their normal diagnostic and cannot satisfy this contract.
fn bounded_rom_failure(length: usize, action: impl FnOnce()) {
    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
    let message = format!("index out of bounds: the len is {length} but the index is {length}");
    let wanted = message.clone();
    let matched_source = Arc::new(AtomicBool::new(false));
    let observed = matched_source.clone();
    let previous = Arc::new(std::panic::take_hook());
    let forward = previous.clone();
    std::panic::set_hook(Box::new(move |info| {
        if payload_text(info.payload()) == Some(wanted.as_str())
            && info
                .location()
                .is_some_and(|at| at.file().ends_with("rom.rs"))
        {
            observed.store(true, Ordering::Relaxed);
        } else {
            forward(info);
        }
    }));
    let result = catch_unwind(AssertUnwindSafe(action));
    drop(std::panic::take_hook());
    std::panic::set_hook(
        Arc::try_unwrap(previous)
            .ok()
            .expect("released scoped panic hook"),
    );
    let payload = result.expect_err("boss map must reach selected truncated Native read");
    assert_eq!(
        payload_text(payload.as_ref()),
        Some(message.as_str()),
        "exact bounded ROM failure"
    );
    assert!(
        matched_source.load(Ordering::Relaxed),
        "failure must originate in rom.rs"
    );
}
fn one(g: &mut Game, case: usize) {
    assert!(!g.actor_identity_is_with_session());
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    for index in 0..96 {
        g.ram()
            .copy_in(0xc608 + 2 * index, &(0x5000 + index as u16).to_le_bytes());
    }
    g.ram().fill(0x7f7..0x8f7, 0x20);
    g.ram()
        .copy_in(w::DEBUG_DISABLE_MINIMAP, &0x8000u16.to_le_bytes());
    g.ram()
        .copy_in(w::ROOM_X_COORDINATE_ON_MAP, &7u16.to_le_bytes());
    g.ram()
        .copy_in(w::ROOM_Y_COORDINATE_ON_MAP, &2u16.to_le_bytes());
    g.ram().copy_in(w::BOSS_ID, &7u16.to_le_bytes());
    let descriptors = crate::rom::lorom_offset(0x90a83a) as usize;
    // Deliberately odd coordinate address: only checked Native word reads run.
    let coordinates = crate::rom::lorom_offset(0x90a901) as usize;
    let length = match case {
        0 => descriptors + 24,
        1 => coordinates + 14,
        2 => coordinates + 2,
        3 => descriptors + 21, // Descriptor 5 ID has only its first byte.
        4 => descriptors + 23, // Matched descriptor 5 pointer lacks high byte.
        5 => coordinates + 7,  // Second coordinate lacks its final Y byte.
        _ => unreachable!(),
    };
    let mut native = vec![0; length];
    // Nonmatching pointer values remain zero (invalid low-window addresses).
    // No-match and reverse-scan cases must not dereference those pointers.
    for index in 0..6 {
        let at = descriptors + 4 * index;
        if at + 1 < length {
            rom_word(&mut native, at, 0x100 + index as u16);
        }
    }
    if case == 3 {
        native[descriptors + 20] = 7;
    } else if case != 0 {
        let selected = if case == 2 { 0 } else { 5 };
        rom_word(&mut native, descriptors + 4 * selected, 7);
        if case == 4 {
            native[descriptors + 22] = 1;
        } else {
            rom_word(&mut native, descriptors + 4 * selected + 2, 0xa901);
        }
        if case == 1 {
            // Both endpoints match. Reverse order must choose descriptor 5;
            // descriptor 0's poisoned pointer must remain unused.
            rom_word(&mut native, descriptors, 7);
            for (index, value) in [0x0055, 0x00aa, 0x0155, 0x01aa, 0x2055, 0x02aa, 0x8000]
                .into_iter()
                .enumerate()
            {
                rom_word(&mut native, coordinates + 2 * index, value);
            }
        } else if case == 2 {
            // Sentinel is the last complete word: its Y must never be read.
            rom_word(&mut native, coordinates, 0x8000);
        } else if case == 5 {
            for (index, value) in [0x0055, 0x00aa, 0x0155].into_iter().enumerate() {
                rom_word(&mut native, coordinates + 2 * index, value);
            }
            native[coordinates + 6] = 0xaa;
        }
    }
    let native = RomImage::from(native);
    g.loaded_rom = None;
    g.register_native_rom(native.clone());
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let mut expected = before.clone();
    put(&mut expected, w::DEBUG_DISABLE_MINIMAP, 1);
    // Independent literal HUD offsets, including all three rows and neighbors.
    for at in [
        0xc63c, 0xc63e, 0xc640, 0xc642, 0xc644, 0xc67c, 0xc67e, 0xc680, 0xc682, 0xc684, 0xc6bc,
        0xc6be, 0xc6c0, 0xc6c2, 0xc6c4,
    ] {
        put(&mut expected, at, 0x2c1f);
    }
    if case == 1 || case == 5 {
        // Map (7,2), coordinate screen (0,0): row 3, column 7.
        expected[0x803] = 0x21;
    }
    if case == 1 {
        // Screen (1,1): row 4, column 8. Screen (32,2): right half, row 5, column 7.
        expected[0x808] = 0xa0;
        expected[0x88b] = 0x21;
    }
    if case < 3 {
        let mut runtime = Runtime::new(g);
        assert!(runtime.game.actor_identity_is_with_session());
        player_update::minimap::disable_for_boss(&mut runtime);
        assert!(runtime.game.actor_identity_is_with_session());
        let mut projected = before.clone();
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(
            projected, expected,
            "boss map live physical contract case {case}"
        );
    } else {
        bounded_rom_failure(length, || {
            let mut runtime = Runtime::new(g);
            assert!(runtime.game.actor_identity_is_with_session());
            player_update::minimap::disable_for_boss(&mut runtime);
        });
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "boss map full RAM publication case {case}"
    );
    assert!(
        !g.actor_identity_is_with_session(),
        "boss map must return authority case {case}"
    );
    assert!(
        !g.g_fail,
        "valid native window must not set failure flag case {case}"
    );
    assert!(g.loaded_rom.is_none());
    assert!(g.capture_native_rom().same_allocation(&native));
}
pub(crate) fn run(g: &mut Game) -> usize {
    assert!(!g.actor_identity_is_with_session());
    let ram = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = catch_unwind(AssertUnwindSafe(|| {
        for case in 0..COUNT {
            one(g, case);
        }
        COUNT
    }));
    g.ram().copy_in(0, &ram);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    match result {
        Ok(count) => count,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
