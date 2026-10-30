//! Finite checked-content and dispatch contracts through actual owned operations.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::{
    game::player_movement::{self, content::SpeedRecord, speed},
    game_state::Game,
    rom::RomImage,
};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn seed(g: &mut Game) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    g.loaded_rom = None;
    g.register_native_rom(RomImage::zeroed(0));
}
fn content(g: &mut Game, words: &[u16]) {
    let base = crate::rom::lorom_offset(0x90e000) as usize;
    assert_eq!(base, 548864);
    let image = RomImage::zeroed(base + words.len() * 2);
    for (i, value) in words.iter().enumerate() {
        for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
            image.write(base + i * 2 + lane, byte);
        }
    }
    g.register_native_rom(image);
}
fn caught(message: &'static str, action: impl FnOnce()) -> Result<(), Box<dyn Any + Send>> {
    let previous = Arc::new(std::panic::take_hook());
    let forward = previous.clone();
    std::panic::set_hook(Box::new(move |info| {
        let actual = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str));
        if actual != Some(message) {
            forward(info);
        }
    }));
    let result = catch_unwind(AssertUnwindSafe(action));
    drop(std::panic::take_hook());
    std::panic::set_hook(
        Arc::try_unwrap(previous)
            .ok()
            .expect("released edge panic hook"),
    );
    result
}
fn require_panic(result: Result<(), Box<dyn Any + Send>>, message: &str) {
    let payload = result.expect_err("actual movement edge must reach its selected failure");
    let actual = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    assert_eq!(actual, Some(message), "exact movement edge failure");
}
fn lazy_speed(g: &mut Game) {
    seed(g);
    // Exactly accel/fraction/max/fraction. Both deceleration words are absent.
    content(g, &[0, 1, 8, 0]);
    put(g, 0xb46, 1);
    put(g, 0xb48, 0xffff);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    expected[0xb46..0xb4a].copy_from_slice(&[2, 0, 0, 0]);
    {
        let mut runtime = Runtime::new(g);
        assert_eq!(
            speed::base_speed(&mut runtime, SpeedRecord::Selected(0xe000)),
            0x20000
        );
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "unused deceleration words are not read"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
fn partial_speed(g: &mut Game) {
    seed(g);
    // Acceleration words exist; the first maximum byte is exactly past the end.
    // The real algorithm publishes the base-speed addition before this read.
    content(g, &[0, 1]);
    put(g, 0xb46, 1);
    put(g, 0xb48, 0xffff);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    expected[0xb46..0xb4a].copy_from_slice(&[2, 0, 0, 0]);
    expected[0xaf6..0xaf8].copy_from_slice(&0x3456u16.to_le_bytes());
    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
    const MESSAGE: &str = "index out of bounds: the len is 548868 but the index is 548868";
    let result = caught(MESSAGE, || {
        // Match the real bank caller's temporary Runtime: Drop occurs during
        // unwinding, before the external caller catches the failure.
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        let _ = speed::base_speed(&mut runtime, SpeedRecord::Selected(0xe000));
    });
    require_panic(result, MESSAGE);
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "failed maximum read preserves accelerated prefix"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
fn invalid_normal(g: &mut Game) {
    seed(g);
    put(g, 0xa1e, 0x1c08);
    put(g, 0xdc6, 0x4567);
    put(g, 0xaa4, 0x5678);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    expected[0xaf6..0xaf8].copy_from_slice(&0x3456u16.to_le_bytes());
    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
    const MESSAGE: &str = "index out of bounds: the len is 28 but the index is 28";
    let result = caught(MESSAGE, || {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        player_movement::advance_normal(&mut runtime);
    });
    require_panic(result, MESSAGE);
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "invalid normal selector fails before movement writes"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
fn frozen_invalid_normal(g: &mut Game) {
    seed(g);
    put(g, 0xa1e, 0x1c08);
    put(g, 0xa78, 1);
    put(g, 0xb3e, 0x400);
    put(g, 0xaae, 74);
    let expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    {
        let mut runtime = Runtime::new(g);
        player_movement::advance_normal(&mut runtime);
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "frozen bypasses invalid dispatch and echo publication"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
fn invalid_crouch(g: &mut Game) {
    seed(g);
    put(g, 0xa1e, 0x0f08);
    put(g, 0xa1c, 0x41);
    put(g, 0xdc6, 0x4567);
    put(g, 0xaa4, 0x5678);
    let expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
    const MESSAGE: &str = "index out of bounds: the len is 12 but the index is 12";
    let result = caught(MESSAGE, || {
        let mut runtime = Runtime::new(g);
        player_movement::advance_normal(&mut runtime);
    });
    require_panic(result, MESSAGE);
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "invalid crouch selector fails before slope/AA4/movement writes"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
}
// Paste into the candidate-only player_movement_edges child module.
// Uses that module's seed, put, word, Runtime and player_movement imports.
fn echo_alias(g: &mut Game) {
    seed(g);
    use crate::ram_map::words as w;
    for (at, value) in [
        (0xa1e, 0x0908),
        (w::samus_x_pos, 0x1234),
        (w::samus_y_pos, 0x5678),
        (w::input_to_pose_calc, 0x7777),
        (w::speed_boost_counter, 0x0400),
        (w::game_time_frames, 4),
        (w::speed_echoes_index, 74),
        (w::samus_collision_direction, 0x9abc),
    ] {
        put(g, at, value);
    }
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<u8>>();
    let mut expected = before.clone();
    // Echo word index37: X destination AB0+74=AFA aliases owned player Y.
    // The subsequent Y destination AB8+74=B02 must read that newly written Y.
    for (at, value) in [
        (0xafa, 0x1234u16),
        (0xb02, 0x1234),
        (w::speed_echoes_index, 0),
    ] {
        expected[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    {
        let mut runtime = Runtime::new(g);
        player_movement::advance_normal(&mut runtime);
        assert_eq!(
            runtime.session().player.y,
            0x1234,
            "exceptional echo X updates owned player Y"
        );
        let mut projected = before.clone();
        crate::game::session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(
            projected, expected,
            "exceptional echo uses live Y after aliased X publication"
        );
    }
    assert_eq!(
        word(g, 0xb02),
        0x1234,
        "exceptional echo writes freshly reread Y"
    );
    assert_eq!(
        word(g, w::speed_echoes_index),
        0,
        "exceptional echo index wraps after publication"
    );
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<u8>>()[..],
        expected.as_slice(),
        "exceptional echo full publication"
    );
    assert!(!g.g_fail && !g.actor_identity_is_with_session());
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
        lazy_speed(g);
        partial_speed(g);
        invalid_normal(g);
        frozen_invalid_normal(g);
        invalid_crouch(g);
        echo_alias(g);
        6
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
