//! Three bounded checks embedded in existing collision locals20/21 (globals36/37).
//! Child of session_runtime for observation of authority while Runtime remains live.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::room_background_support::{ram_bytes, ram_word};
use crate::{
    game::{
        player_collision::effects::{Effects, MechanismHeader},
        session_encoding,
    },
    game_state::Game,
    rom::RomImage,
};
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_word(image: &RomImage, address: u32, value: u16) {
    let at = crate::rom::lorom_offset(address) as usize;
    for (i, b) in value.to_le_bytes().into_iter().enumerate() {
        image.write(at + i, b);
    }
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
            .expect("released scoped panic hook"),
    );
    result
}
fn require_panic(result: Result<(), Box<dyn Any + Send>>, message: &str) {
    let payload = result.expect_err("selected actual call must reach its defined failure");
    let actual = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    assert_eq!(actual, Some(message), "exact failure, not arbitrary panic");
}
fn seed(g: &mut Game) {
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    ram_word(g.ram(), crate::ram_map::words::ROOM_WIDTH_IN_BLOCKS, 1);
    ram_word(g.ram(), crate::ram_map::words::ROOM_HEIGHT_IN_BLOCKS, 1);
    ram_word(g.ram(), crate::ram_map::words::SAMUS_X_POS, 0x120);
    g.ram().write_byte(0x16403, 0x7b);
    let loaded = RomImage::zeroed(0x200000);
    rom_word(&loaded, 0x84f100, 0xc6dc);
    rom_word(&loaded, 0x84f102, 0x9000);
    g.register_rom(loaded);
    g.register_native_rom(RomImage::zeroed(0x200000));
}
fn full(g: &mut Game) {
    seed(g);
    g.loaded_rom = None; // A full pool must not read either definition word.
    for i in 0..40 {
        ram_word(g.ram(), 0x1c37 + 2 * i, 0xf100);
    }
    let mut expected = ram_bytes(&g);
    put(&mut expected, 0xaf6, 0x3456);
    for (at, value) in [(0xe54, 0x0040), (0xe44, 0x2345), (0x185e, 0x3456)] {
        put(&mut expected, at, value);
    }
    expected[0xf8c..0xf8e].copy_from_slice(&0x1234u16.to_le_bytes());
    let mut delta = 0x12345678;
    {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        // Edit the owned projection without refreshing the external observations.
        for (at, value) in [
            (0xf8c, 0x1234u16),
            (0xe54, 0x0040),
            (0xe44, 0x2345),
            (0x185e, 0x3456),
        ] {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                assert!(session_encoding::write_projected_byte(
                    runtime.session_mut(),
                    at + lane,
                    byte
                ));
            }
        }
        assert_eq!(
            runtime.apply_mechanism(MechanismHeader(0xf100), &mut delta),
            0
        );
        assert_eq!(delta, 0x12345678, "full pool preserves caller displacement");
        assert_eq!(
            runtime.inputs.enemy_zero_health,
            u16::from_ne_bytes(0x1234u16.to_le_bytes())
        );
        assert_eq!(runtime.inputs.current_enemy, 0x0040);
        assert_eq!(runtime.inputs.random_enemy_counter, 0x2345);
        assert_eq!(runtime.inputs.debug_enemy_freeze, 0x3456);
        assert!(
            runtime.has_session_for_verification(),
            "normal return retains authority"
        );
        let mut projected = ram_bytes(&runtime.game);
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(projected, expected, "full pool performs no creation writes");
    }
    assert_eq!(ram_bytes(&g).as_slice(), expected.as_slice());
    assert!(!g.actor_identity_is_with_session());
}
fn stale(g: &mut Game) {
    seed(g);
    for i in 0..40 {
        ram_word(g.ram(), 0x1c37 + 2 * i, 0xf100);
    }
    let before = ram_bytes(&g);
    let mut expected = before.clone();
    put(&mut expected, 0xaf6, 0x3456);
    let world = g.world().clone();
    let mut delta = 0x12345678;
    let (result, held, unpublished) = {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        runtime.session_mut().world = Arc::new((*world).clone());
        let result = caught("Session belongs to a different world binding", || {
            runtime.apply_mechanism(MechanismHeader(0xf100), &mut delta);
        });
        let held = runtime.has_session_for_verification();
        let unpublished = ram_bytes(&runtime.game).as_slice() == before.as_slice();
        // Always restore the real binding before any assertion can unwind Drop.
        if held {
            runtime.session_mut().world = world;
        }
        (result, held, unpublished)
    };
    require_panic(result, "Session belongs to a different world binding");
    assert!(
        held && unpublished,
        "preflight fails before relinquishment or creation prefix"
    );
    assert_eq!(delta, 0x12345678);
    assert_eq!(ram_bytes(&g).as_slice(), expected.as_slice());
    assert!(!g.actor_identity_is_with_session());
}
fn failed_setup(g: &mut Game) {
    seed(g);
    let mut expected = ram_bytes(&g);
    put(&mut expected, 0xaf6, 0x3456);
    for (at, value) in [
        (0x1c37, 0xf100),
        (0x1c87, 0),
        (0x1cd7, 0x853d),
        (0x1d27, 0x9000),
        (0x1d77, 0),
        (0x1dc7, 0),
        (0xdf0c, 0),
        (0xde1c, 1),
        (0xde6c, 0x8da0),
    ] {
        put(&mut expected, at + 78, value);
    }
    put(&mut expected, 0x1c27, 78);
    expected[0x16402] = 0x10;
    let mut delta = 0x12345678;
    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
    const MESSAGE: &str = "index out of bounds: the len is 131072 but the index is 156674";
    let (result, held, prefix, identity_returned) = {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        let result = caught(MESSAGE, || {
            runtime.apply_mechanism(MechanismHeader(0xf100), &mut delta);
        });
        // Observe before Runtime drops: a later Drop must not hide retained authority.
        let held = runtime.has_session_for_verification();
        let prefix = ram_bytes(&runtime.game);
        let identity_returned = !runtime.game.actor_identity_is_with_session();
        (result, held, prefix, identity_returned)
    };
    require_panic(result, MESSAGE);
    assert!(
        !held && identity_returned,
        "failed setup returns authority even with outer Runtime alive"
    );
    assert_eq!(
        prefix, expected,
        "pure setup partial write published before caught caller resumes"
    );
    assert_eq!(
        ram_bytes(&g).as_slice(),
        expected.as_slice(),
        "subsequent empty Drop preserves prefix"
    );
    assert_eq!(
        delta, 0x12345678,
        "gate leaves caller displacement unchanged"
    );
}
/// Preserve the original selected movement capture and its exact output bytes.
pub(crate) fn run(g: &mut Game, local: u32) {
    assert!(matches!(local, 20 | 21));
    assert!(!g.actor_identity_is_with_session());
    let saved = ram_bytes(&g);
    let fail = g.g_fail;
    let loaded = g.loaded_rom.clone();
    let native = g.capture_native_rom_binding();
    let result = catch_unwind(AssertUnwindSafe(|| {
        if local == 20 {
            full(g);
        } else {
            stale(g);
            failed_setup(g);
            acceleration_stale(g);
            acceleration_loaded_failure(g);
            acceleration_success(g);
            assert_eq!(
                super::player_update_aliases::run(g),
                super::player_update_aliases::COUNT
            );
            assert_eq!(
                super::player_minimap_boss::run(g),
                super::player_minimap_boss::COUNT
            );
            assert_eq!(
                super::player_minimap_edges::run(g),
                super::player_minimap_edges::COUNT
            );
            assert_eq!(
                super::player_legacy_edges::run(g),
                super::player_legacy_edges::COUNT
            );
            assert_eq!(
                super::player_minimap_bytes::run(g),
                super::player_minimap_bytes::COUNT
            );
            assert_eq!(super::player_movement_edges::run(g), 6);
            assert_eq!(
                super::player_spin_bounds::run(g),
                super::player_spin_bounds::COUNT
            );
        }
        assert!(!g.g_fail);
    }));
    g.ram().copy_in(0, &saved);
    g.g_fail = fail;
    g.loaded_rom = loaded;
    g.restore_native_rom_binding(native);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

fn acceleration_step(runtime: &mut Runtime<'_>) {
    sm_game::player_projectiles::host::Host::player_step(
        runtime,
        sm_game::player_projectiles::host::PlayerStep::Acceleration,
    );
}
fn acceleration_stale(g: &mut Game) {
    seed(g);
    ram_word(g.ram(), crate::ram_map::words::EQUIPPED_ITEMS, 0x20);
    g.loaded_rom = None; // Gravity must need no pose content, but still preflight.
    let before = ram_bytes(&g);
    let mut expected = before.clone();
    put(&mut expected, 0xaf6, 0x3456);
    let world = g.world().clone();
    let (result, held, unpublished) = {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        runtime.session_mut().world = Arc::new((*world).clone());
        let result = caught("Session belongs to a different world binding", || {
            acceleration_step(&mut runtime);
        });
        let held = runtime.has_session_for_verification();
        let unpublished = ram_bytes(&runtime.game).as_slice() == before.as_slice();
        if held {
            runtime.session_mut().world = world;
        }
        (result, held, unpublished)
    };
    require_panic(result, "Session belongs to a different world binding");
    assert!(
        held && unpublished,
        "acceleration preflight precedes Gravity no-read path"
    );
    assert_eq!(ram_bytes(&g).as_slice(), expected.as_slice());
    assert!(!g.actor_identity_is_with_session());
}
fn acceleration_loaded_failure(g: &mut Game) {
    seed(g);
    g.loaded_rom = None;
    ram_word(g.ram(), crate::ram_map::words::SAMUS_Y_ACCEL, 0x5678);
    ram_word(g.ram(), crate::ram_map::words::SAMUS_Y_SUBACCEL, 0x9abc);
    let mut expected = ram_bytes(&g);
    put(&mut expected, 0xaf6, 0x3456);
    let (result, held, prefix, identity_returned) = {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        let result = caught("ROM must be loaded", || acceleration_step(&mut runtime));
        let held = runtime.has_session_for_verification();
        let prefix = ram_bytes(&runtime.game);
        let identity_returned = !runtime.game.actor_identity_is_with_session();
        (result, held, prefix, identity_returned)
    };
    require_panic(result, "ROM must be loaded");
    assert!(
        !held && identity_returned,
        "acceleration Loaded failure returns authority before outer Drop"
    );
    assert_eq!(
        prefix, expected,
        "acceleration Loaded failure publishes prior owner without acceleration writes"
    );
    assert_eq!(
        ram_bytes(&g).as_slice(),
        expected.as_slice(),
        "empty outer Drop preserves failed acceleration prefix"
    );
}
fn acceleration_success(g: &mut Game) {
    seed(g);
    ram_word(g.ram(), crate::ram_map::words::EQUIPPED_ITEMS, 0x20);
    ram_word(g.ram(), crate::ram_map::words::SAMUS_Y_ACCEL, 0x5678);
    ram_word(g.ram(), crate::ram_map::words::SAMUS_Y_SUBACCEL, 0x9abc);
    g.loaded_rom = None;
    let mut expected = ram_bytes(&g);
    put(&mut expected, 0xaf6, 0x3456);
    put(&mut expected, crate::ram_map::words::SAMUS_Y_ACCEL, 0);
    put(
        &mut expected,
        crate::ram_map::words::SAMUS_Y_SUBACCEL,
        0x1c00,
    );
    for (at, value) in [
        (0xf8c, 0x1234),
        (0xe54, 0x0040),
        (0xe44, 0x2345),
        (0x185e, 0x3456),
    ] {
        put(&mut expected, at, value);
    }
    {
        let mut runtime = Runtime::new(g);
        runtime.session_mut().player.x = 0x3456;
        for (at, value) in [
            (0xf8c, 0x1234u16),
            (0xe54, 0x0040),
            (0xe44, 0x2345),
            (0x185e, 0x3456),
        ] {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                assert!(session_encoding::write_projected_byte(
                    runtime.session_mut(),
                    at + lane,
                    byte
                ));
            }
        }
        acceleration_step(&mut runtime);
        assert_eq!(
            runtime.inputs.enemy_zero_health,
            u16::from_ne_bytes(0x1234u16.to_le_bytes())
        );
        assert_eq!(runtime.inputs.current_enemy, 0x0040);
        assert_eq!(runtime.inputs.random_enemy_counter, 0x2345);
        assert_eq!(runtime.inputs.debug_enemy_freeze, 0x3456);
        let mut projected = ram_bytes(&runtime.game);
        session_encoding::encode(runtime.session(), |at, span| {
            projected[at..at + span.len()].copy_from_slice(span)
        });
        assert_eq!(
            projected, expected,
            "actual Gravity acceleration and refreshed aliases"
        );
    }
    assert_eq!(ram_bytes(&g).as_slice(), expected.as_slice());
    assert!(!g.actor_identity_is_with_session());
}
