//! Actual aggregate Runtime native-spawn authority checks; no transport model.
//! New composed-owner invariants, not old/new predecessor observations.
#![forbid(unsafe_code)]
use crate::game::{session::Session, session_runtime::Runtime};
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) const CASES: u32 = 3;
// Independent physical map: no use of the production codec's visitor or helpers.
const WORDS: [usize; 12] = [
    0x1993, 0x1995, 0x19b9, 0x1a6d, 0x1a49, 0x9c6, 0x9c8, 0x9d6, 0x9d8, 0x1c27, 0xc04, 0x10002,
];
#[derive(Clone, Copy)]
struct Values {
    native: [u16; 8],
    words: [u16; 12],
    angle: u8,
}
const INITIAL: Values = Values {
    native: [
        0xa102, 0xb203, 0xc304, 0xd405, 0xe506, 0xf607, 0x1808, 0x2909,
    ],
    words: [
        0x1112, 0x2324, 0x3536, 0x4748, 0x595a, 0x6b6c, 0x7d7e, 0x8f90, 0xa1a2, 0x12, 0xb3b4,
        0xc5c6,
    ],
    angle: 0xd7,
};
const OWNED: Values = Values {
    native: [
        0x3111, 0x4222, 0x5333, 0x6444, 0x7555, 0x8666, 0x9777, 0xa888,
    ],
    words: [
        0x1314, 0x2526, 0x3738, 0x494a, 0x5b5c, 0x6d6e, 0x7f80, 0x9192, 0xa3a4, 0x22, 0xb5b6,
        0xc7c8,
    ],
    angle: 0xd9,
};
const CALLBACK: Values = Values {
    native: [
        0xb999, 0xcaaa, 0xdbbb, 0xeccc, 0xfddd, 0x1eee, 0x2fff, 0x3001,
    ],
    words: [
        0x1516, 0x2728, 0x393a, 0x4b4c, 0x5d5e, 0x6f70, 0x8182, 0x9394, 0xa5a6, 0x32, 0xb7b8,
        0xc9ca,
    ],
    angle: 0xdb,
};
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn expected_ram(ram: &[u8], values: Values) -> Vec<u8> {
    let mut expected = ram.to_vec();
    for (at, value) in WORDS.into_iter().zip(values.words) {
        put(&mut expected, at, value);
    }
    expected[0x60b] = values.angle;
    expected
}
fn native_game(game: &Game) -> [u16; 8] {
    [
        game.eproj_spawn_rect.x,
        game.eproj_spawn_rect.y,
        game.eproj_spawn_rect.w,
        game.eproj_spawn_rect.h,
        game.eproj_spawn_r22,
        game.eproj_spawn_pt.x,
        game.eproj_spawn_pt.y,
        game.eproj_spawn_varE24,
    ]
}
fn set_native_game(game: &mut Game, values: [u16; 8]) {
    game.eproj_spawn_rect.x = values[0];
    game.eproj_spawn_rect.y = values[1];
    game.eproj_spawn_rect.w = values[2];
    game.eproj_spawn_rect.h = values[3];
    game.eproj_spawn_r22 = values[4];
    game.eproj_spawn_pt.x = values[5];
    game.eproj_spawn_pt.y = values[6];
    game.eproj_spawn_varE24 = values[7];
}
fn native_session(state: &Session) -> [u16; 8] {
    let spawn = &state.enemy_projectiles.spawn;
    [
        spawn.rectangle.origin.x,
        spawn.rectangle.origin.y,
        spawn.rectangle.extent.x,
        spawn.rectangle.extent.y,
        spawn.variant,
        spawn.point.x,
        spawn.point.y,
        spawn.drop_definition,
    ]
}
fn session_words(state: &Session) -> [u16; 12] {
    let enemy = &state.enemy_projectiles;
    [
        enemy.spawn.primary,
        enemy.spawn.secondary,
        enemy.slots[17].definition,
        enemy.slots[17].x.whole,
        enemy.slots[17].x.fraction,
        state.inventory.ammunition[0].current,
        state.inventory.ammunition[0].maximum,
        state.inventory.reserve_health,
        state.inventory.reserve_missiles,
        state.mechanisms.selected,
        state.projectiles.shots[0].direction,
        state.room.terrain.cells()[0].level,
    ]
}
fn set_session(state: &mut Session, values: Values) {
    let enemy = &mut state.enemy_projectiles;
    enemy.spawn.rectangle.origin.x = values.native[0];
    enemy.spawn.rectangle.origin.y = values.native[1];
    enemy.spawn.rectangle.extent.x = values.native[2];
    enemy.spawn.rectangle.extent.y = values.native[3];
    enemy.spawn.variant = values.native[4];
    enemy.spawn.point.x = values.native[5];
    enemy.spawn.point.y = values.native[6];
    enemy.spawn.drop_definition = values.native[7];
    enemy.spawn.primary = values.words[0];
    enemy.spawn.secondary = values.words[1];
    enemy.spawn.angle = values.angle;
    enemy.slots[17].definition = values.words[2];
    enemy.slots[17].x.whole = values.words[3];
    enemy.slots[17].x.fraction = values.words[4];
    state.inventory.ammunition[0].current = values.words[5];
    state.inventory.ammunition[0].maximum = values.words[6];
    state.inventory.reserve_health = values.words[7];
    state.inventory.reserve_missiles = values.words[8];
    state.mechanisms.selected = values.words[9];
    state.projectiles.shots[0].direction = values.words[10];
    state.room.terrain.cells_mut()[0].level = values.words[11];
}
fn assert_native(actual: [u16; 8], expected: [u16; 8], stage: &str) {
    for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
        assert_eq!(actual, expected, "{stage}: native word {index}");
    }
}
fn assert_session(state: &Session, values: Values, stage: &str) {
    assert_native(native_session(state), values.native, stage);
    assert_eq!(
        session_words(state),
        values.words,
        "{stage}: shared WRAM fields"
    );
    assert_eq!(
        state.enemy_projectiles.spawn.angle, values.angle,
        "{stage}: angle"
    );
}
fn record(stage: u8, values: Values, trace: &mut Vec<u8>) {
    trace.push(stage);
    for value in values.native.into_iter().chain(values.words) {
        trace.extend_from_slice(&value.to_le_bytes());
    }
    trace.push(values.angle);
}
fn callback(game: &mut Game, published: &[u8], callback_ram: &[u8], trace: &mut Vec<u8>) {
    assert_native(
        native_game(game),
        OWNED.native,
        "native callback publication",
    );
    assert_eq!(
        &game.g_ram[..],
        published,
        "native callback whole RAM publication"
    );
    // Capture only after observing the actual published state against literals.
    record(1, OWNED, trace);
    set_native_game(game, CALLBACK.native);
    game.g_ram.copy_from_slice(callback_ram);
    let image = RomImage::zeroed(3);
    image.write(0, 0x92);
    game.loaded_rom = Some(image);
    record(2, CALLBACK, trace);
}
fn intentional_unwind(result: Result<(), Box<dyn std::any::Any + Send>>, message: &str) {
    let error = result.expect_err("native scope must unwind");
    assert_eq!(
        error.downcast_ref::<&str>().copied(),
        Some(message),
        "native scope must reach the designated intentional panic"
    );
}

fn capture_checked(id: u32) -> Vec<u8> {
    let mut game = Game::new();
    for (index, byte) in game.g_ram.iter_mut().enumerate() {
        *byte = (index.wrapping_mul(37) ^ (index >> 8)) as u8;
    }
    let initial_ram = expected_ram(&game.g_ram, INITIAL);
    game.g_ram.copy_from_slice(&initial_ram);
    set_native_game(&mut game, INITIAL.native);
    game.g_ram[0x613] = 0x77;
    let old_image = RomImage::zeroed(2);
    old_image.write(0, 0x31);
    game.loaded_rom = Some(old_image.clone());
    let published = expected_ram(&game.g_ram, OWNED);
    let mut callback_ram = expected_ram(&published, CALLBACK);
    callback_ram[0x20] = 0xaf;
    callback_ram[0x613] = 0x88;
    put(&mut callback_ram, 0x911, 0x6666);
    let mut trace = Vec::new();
    let returned = match id {
        0 => {
            let mut final_values = CALLBACK;
            final_values.native[5] = 0x4152;
            final_values.words[5] = 0x6172;
            let final_ram = expected_ram(&callback_ram, final_values);
            let returned;
            {
                let mut runtime = Runtime::new(&mut game);
                assert_session(runtime.session(), INITIAL, "native acquisition");
                set_session(runtime.session_mut(), OWNED);
                returned = runtime.foreign(|game| {
                    callback(game, &published, &callback_ram, &mut trace);
                    0x91u16
                });
                assert_eq!(returned, 0x91, "native callback return value");
                assert_session(runtime.session(), CALLBACK, "native return import");
                assert_eq!(
                    runtime.inputs().camera_x,
                    0x6666,
                    "native callback observation refresh"
                );
                let image = runtime.loaded_image().expect("callback ROM");
                assert_eq!((image.len(), image.read(0)), (3, 0x92));
                assert_eq!(old_image.read(0), 0x31, "captured old ROM remains live");
                runtime.session_mut().enemy_projectiles.spawn.point.x = 0x4152;
                runtime.session_mut().inventory.ammunition[0].current = 0x6172;
            }
            assert_native(
                native_game(&game),
                final_values.native,
                "native final Drop publication",
            );
            assert_eq!(
                &game.g_ram[..],
                &final_ram[..],
                "native final Drop whole RAM"
            );
            Some(usize::from(returned))
        }
        1 => {
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                let mut runtime = Runtime::new(&mut game);
                assert_session(runtime.session(), INITIAL, "native acquisition");
                set_session(runtime.session_mut(), OWNED);
                runtime.foreign(|game| {
                    callback(game, &published, &callback_ram, &mut trace);
                    panic!("intentional native spawn callback unwind");
                });
            }));
            intentional_unwind(unwind, "intentional native spawn callback unwind");
            assert_native(
                native_game(&game),
                CALLBACK.native,
                "native foreign panic retained callback",
            );
            assert_eq!(
                &game.g_ram[..],
                &callback_ram[..],
                "native foreign panic whole RAM"
            );
            None
        }
        _ => {
            // This explicitly tests ordinary-panic Drop transport. It does not
            // claim actual instruction failure/prefix coverage; those retain
            // their separate short-definition regression cases.
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                let mut runtime = Runtime::new(&mut game);
                assert_session(runtime.session(), INITIAL, "native acquisition");
                set_session(runtime.session_mut(), OWNED);
                assert_session(runtime.session(), OWNED, "native core before panic");
                panic!("intentional native spawn core unwind");
            }));
            intentional_unwind(unwind, "intentional native spawn core unwind");
            assert_native(
                native_game(&game),
                OWNED.native,
                "native core panic publication",
            );
            assert_eq!(
                &game.g_ram[..],
                &published[..],
                "native core panic whole RAM"
            );
            None
        }
    };
    assert!(!game.g_fail, "native Runtime diagnostic {id}");
    let mut output = game.g_ram.to_vec();
    for value in native_game(&game) {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output.push(u8::from(returned.is_some()));
    output.extend_from_slice(&(returned.unwrap_or(0) as u64).to_le_bytes());
    output.push(u8::from(id != 0));
    output.push(u8::from(game.g_fail));
    output.extend_from_slice(&(trace.len() as u32).to_le_bytes());
    output.extend_from_slice(&trace);
    output
}

pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| capture_checked(id)));
    assert_eq!(
        outcome.is_ok(),
        true,
        "native Runtime case {} failed",
        1440 + id
    );
    outcome.ok()
}
