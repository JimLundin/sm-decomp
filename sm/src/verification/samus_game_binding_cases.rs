//! Actual Game binding witnesses; these fixtures do not supply a prepared start.
#![cfg(sm_verify_dll)]
#![forbid(unsafe_code)]

use crate::game::scene::{
    records as sc,
    samus::SamusCatalog,
    samus_content::{
        ContentDescription,
        Registration::{Loaded, Native},
        Role::{BodyTop, PositionFallback},
    },
    state::DisplayError,
};
use crate::game_state::{
    Game,
    SamusAuthority::{Available, Revoked},
    SamusBindingDescription,
    SamusExecution::{self, Prepared, RetainedNative},
};
use crate::{rom::RomImage, samus_content_binding_cases as fixture};
use std::sync::Arc;

fn image() -> RomImage {
    let image = RomImage::zeroed(8);
    for (at, byte) in [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x7a, 0x8b]
        .into_iter()
        .enumerate()
    {
        image.write(at, byte);
    }
    image
}

fn description(
    game: &Game,
    roles: ContentDescription,
    execution: SamusExecution,
) -> SamusBindingDescription {
    let revision = sc::Revision([0x41; 32]);
    let reference = |id| sc::AssetRef {
        id: sc::AssetId(id),
        revision,
    };
    let request_media = Arc::new(
        sc::SceneMedia::new(
            revision,
            vec![
                sc::SceneAsset::Atlas(sc::TileAtlas {
                    reference: reference(201),
                    encoding: sc::TileEncoding::Affine8,
                    tile_count: 512,
                    data: vec![0x21; 32768].into(),
                }),
                sc::SceneAsset::Atlas(sc::TileAtlas {
                    reference: reference(202),
                    encoding: sc::TileEncoding::Planar4,
                    tile_count: 1,
                    data: vec![0x23; 16].into(),
                }),
                sc::SceneAsset::Room(reference(203)),
                sc::SceneAsset::Room(reference(204)),
            ],
            vec![],
            vec![],
        )
        .unwrap(),
    );
    SamusBindingDescription {
        world: game.world().clone(),
        roles,
        spritemaps: Arc::new(SamusCatalog {
            revision,
            pointers: vec![0].into(),
            maps: Default::default(),
        }),
        request_media,
        execution,
    }
}

fn installed(execution: SamusExecution) -> (Box<Game>, RomImage, RomImage) {
    let mut game = Game::new();
    let loaded = image();
    let native = image();
    game.register_rom(loaded.clone());
    game.register_native_rom(native.clone());
    let expected = description(&game, fixture::expected(), execution);
    let prepared = game
        .prepare_samus_binding(
            description(&game, fixture::supplied(), execution),
            &expected,
        )
        .unwrap();
    game.install_samus_binding(prepared).unwrap();
    (game, loaded, native)
}

// Movies cannot replace independently expected role metadata during unpublished preparation.
pub fn samus_game_rejected_expectation_preserves_binding() {
    let (game, _, _) = installed(Prepared);
    let owner = game.samus_spritemaps().unwrap().unwrap();
    let media = game.samus_request_media().unwrap().unwrap();
    let mut expected = description(&game, fixture::expected(), Prepared);
    expected
        .roles
        .roles
        .get_mut(&(BodyTop, Loaded))
        .unwrap()
        .mapped_base = 1;
    assert_eq!(
        game.prepare_samus_binding(description(&game, fixture::supplied(), Prepared), &expected)
            .err(),
        Some(DisplayError::Incomplete)
    );
    assert_eq!(game.samus_authority(), Available);
    assert_eq!(game.samus_execution(), Ok(Some(Prepared)));
    assert!(Arc::ptr_eq(
        &owner,
        &game.samus_spritemaps().unwrap().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &media,
        &game.samus_request_media().unwrap().unwrap()
    ));
    assert_eq!(
        game.prepared_samus_role(BodyTop, Loaded, 0)
            .unwrap()
            .unwrap()
            .native_word(0),
        Ok(0x2211)
    );
}

// Movies cannot substitute a selector while retaining a captured allocation-bound role lease.
pub fn samus_game_native_capture_retains_allocation() {
    let (mut game, _, native) = installed(RetainedNative);
    let selected = game.capture_native_rom();
    assert!(selected.same_allocation(&native));
    let lease = game
        .capture_samus_role(PositionFallback, Native, 0, &selected)
        .unwrap()
        .unwrap();
    assert_eq!(lease.native_word(0), Ok(0x4433));
    selected.write(2, 0xa7);
    assert_eq!(lease.native_word(0), Ok(0x44a7));
    let replacement = image();
    game.register_native_rom(replacement.clone());
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.samus_execution(), Ok(Some(RetainedNative)));
    native.write(3, 0xbc);
    assert_eq!(lease.native_word(0), Ok(0xbca7));
    assert_eq!(replacement.read(2), 0x33);
    assert_eq!(
        game.capture_samus_role(PositionFallback, Native, 0, &selected)
            .err(),
        Some(DisplayError::Identity)
    );
    assert_eq!(
        game.capture_samus_role(PositionFallback, Native, 0, &replacement)
            .err(),
        Some(DisplayError::Identity)
    );
}

// Movies cannot bypass the registration setter with an equal-length, equal-byte new allocation.
pub fn samus_game_prepared_replacement_is_rejected() {
    let (mut game, _, _) = installed(Prepared);
    game.loaded_rom = Some(image());
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.samus_execution(), Err(DisplayError::Identity));
    assert_eq!(
        game.prepared_samus_role(BodyTop, Loaded, 0).err(),
        Some(DisplayError::Identity)
    );

    let (mut game, _, _) = installed(Prepared);
    game.register_rom(image());
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.samus_execution(), Err(DisplayError::Poisoned));
    assert_eq!(
        game.prepared_samus_role(BodyTop, Loaded, 0).err(),
        Some(DisplayError::Poisoned)
    );
}

// Movies cannot mutate a registered source between preparation and the final install check.
pub fn samus_game_failed_install_preserves_binding() {
    let (mut game, _, native) = installed(Prepared);
    let owner = game.samus_spritemaps().unwrap().unwrap();
    let media = game.samus_request_media().unwrap().unwrap();
    let expected = description(&game, fixture::expected(), Prepared);
    let candidate = game
        .prepare_samus_binding(description(&game, fixture::supplied(), Prepared), &expected)
        .unwrap();
    native.write(2, 0xa7);
    assert_eq!(
        game.install_samus_binding(candidate),
        Err(DisplayError::Source)
    );
    assert_eq!(game.samus_authority(), Available);
    assert_eq!(game.samus_execution(), Ok(Some(Prepared)));
    assert!(Arc::ptr_eq(
        &owner,
        &game.samus_spritemaps().unwrap().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &media,
        &game.samus_request_media().unwrap().unwrap()
    ));
    assert_eq!(native.read(2), 0xa7);
    assert_eq!(
        game.prepared_samus_role(PositionFallback, Native, 0)
            .unwrap()
            .unwrap()
            .native_word(0),
        Ok(0x4433)
    );
}

// Movies cannot nest a normal Runtime callback inside an imported reference allocation.
pub fn samus_game_reference_import_keeps_revoked_scope() {
    let (mut game, _, native) = installed(RetainedNative);
    let returned = game.with_reference_import(|game| {
        assert_eq!(game.samus_authority(), Revoked);
        let mut runtime = crate::game::session_runtime::Runtime::new(game);
        let result = runtime.foreign(|game| {
            assert_eq!(game.samus_authority(), Revoked);
            native.write(2, 0xa7);
            assert_eq!(game.capture_native_rom().read(2), 0xa7);
            0x6au16
        });
        assert!(runtime.has_session_for_verification());
        assert_eq!(runtime.samus_game().samus_authority(), Revoked);
        drop(runtime);
        assert_eq!(game.samus_authority(), Revoked);
        result
    });
    assert_eq!(returned, 0x6a);
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.preflight_samus_runtime(), Err(DisplayError::Poisoned));
    assert!(game.capture_native_rom().same_allocation(&native));
    assert_eq!(native.read(2), 0xa7);
}

// Movies cannot inject a panic after this callback's RAM and same-allocation ROM prefix.
pub fn samus_game_panicking_callback_keeps_prefix() {
    let (mut game, _, native) = installed(RetainedNative);
    let mut runtime = crate::game::session_runtime::Runtime::new(&mut game);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.foreign(|game| -> () {
            game.ram().write_byte(0xaf6, 0xd1);
            game.ram().write_byte(0xaf7, 0xe2);
            native.write(2, 0xa7);
            panic!("Samus callback prefix");
        });
    }));
    assert!(result.is_err());
    assert!(!runtime.has_session_for_verification());
    assert!(!runtime.samus_game().actor_identity_is_with_session());
    assert_eq!(runtime.samus_game().samus_authority(), Revoked);
    assert_eq!(runtime.samus_game().ram().read_byte(0xaf6), 0xd1);
    assert_eq!(runtime.samus_game().ram().read_byte(0xaf7), 0xe2);
    drop(runtime);
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.preflight_samus_runtime(), Err(DisplayError::Poisoned));
    assert_eq!(game.ram().read_byte(0xaf6), 0xd1);
    assert_eq!(game.ram().read_byte(0xaf7), 0xe2);
    assert_eq!(native.read(2), 0xa7);
}

// Movies cannot supply distinct before/selected registrations while a competing image is live.
pub fn samus_game_adoption_uses_captured_target() {
    let (mut game, before_loaded, before_native) = installed(Prepared);
    let before_owner = game.samus_spritemaps().unwrap().unwrap();
    let before = game.capture_samus_snapshot();
    let selected_loaded = image();
    let selected_native = image();
    let selected = game.with_reference_import(|game| {
        game.register_rom(selected_loaded.clone());
        game.register_native_rom(selected_native.clone());
        let expected = description(game, fixture::expected(), Prepared);
        let candidate = game
            .prepare_samus_binding(description(game, fixture::supplied(), Prepared), &expected)
            .unwrap();
        game.install_samus_binding(candidate).unwrap();
        assert_eq!(game.samus_authority(), Revoked);
        game.capture_samus_snapshot()
    });
    let competing_loaded = image();
    let competing_native = image();
    competing_loaded.write(0, 0xe2);
    competing_native.write(2, 0xf4);
    game.register_rom(competing_loaded.clone());
    game.register_native_rom(competing_native.clone());
    let before_ready = game.prepare_samus_adoption(&before).unwrap();
    let selected_ready = game.prepare_samus_adoption(&selected).unwrap();
    assert!(
        game.loaded_rom
            .as_ref()
            .unwrap()
            .same_allocation(&competing_loaded)
    );
    assert!(game.capture_native_rom().same_allocation(&competing_native));
    assert_eq!(game.samus_authority(), Revoked);

    selected_native.write(2, 0xa7);
    assert_eq!(
        game.commit_samus_adoption(selected_ready),
        Err(DisplayError::Source)
    );
    assert!(
        game.loaded_rom
            .as_ref()
            .unwrap()
            .same_allocation(&competing_loaded)
    );
    assert!(game.capture_native_rom().same_allocation(&competing_native));
    assert_eq!(game.samus_execution(), Err(DisplayError::Poisoned));
    assert_eq!(competing_loaded.read(0), 0xe2);
    assert_eq!(competing_native.read(2), 0xf4);

    game.commit_samus_adoption(before_ready).unwrap();
    assert!(
        game.loaded_rom
            .as_ref()
            .unwrap()
            .same_allocation(&before_loaded)
    );
    assert!(game.capture_native_rom().same_allocation(&before_native));
    assert!(Arc::ptr_eq(
        &before_owner,
        &game.samus_spritemaps().unwrap().unwrap()
    ));
    assert_eq!(game.samus_authority(), Available);

    selected_native.write(2, 0x33);
    let selected_ready = game.prepare_samus_adoption(&selected).unwrap();
    game.commit_samus_adoption(selected_ready).unwrap();
    assert!(
        game.loaded_rom
            .as_ref()
            .unwrap()
            .same_allocation(&selected_loaded)
    );
    assert!(game.capture_native_rom().same_allocation(&selected_native));
    assert!(!Arc::ptr_eq(
        &before_owner,
        &game.samus_spritemaps().unwrap().unwrap()
    ));
    assert_eq!(game.samus_authority(), Available);
    assert_eq!(game.samus_execution(), Ok(Some(Prepared)));
    assert_eq!(
        game.prepared_samus_role(BodyTop, Loaded, 0)
            .unwrap()
            .unwrap()
            .native_word(0),
        Ok(0x2211)
    );
}
// Movies cannot independently supply every body-coordinate lane or an overlapping odd word.
pub fn samus_driver_body_projection_preserves_odd_lanes() {
    use crate::game::{player_projectiles::access::Store, session_runtime::Runtime};
    let (mut game, _, _) = installed(Prepared);
    game.ram().copy_in(0xb04, &[0x7d, 0xf1, 0x68, 0xe2]);
    game.ram().copy_in(0xaf6, &0xc123u16.to_le_bytes());
    game.ram().copy_in(0xafa, &0xd456u16.to_le_bytes());
    game.ram().copy_in(0x911, &0x9110u16.to_le_bytes());
    game.ram().copy_in(0x915, &0x9220u16.to_le_bytes());
    let mut runtime = Runtime::new(&mut game);
    assert_eq!(
        (
            runtime.session().player.body_position.x,
            runtime.session().player.body_position.y
        ),
        (0xf17d, 0xe268)
    );
    assert_eq!(
        [
            runtime.byte(0xb04),
            runtime.byte(0xb05),
            runtime.byte(0xb06),
            runtime.byte(0xb07)
        ],
        [0x7d, 0xf1, 0x68, 0xe2]
    );
    assert_eq!(runtime.word(0xb05), 0x68f1);
    runtime.store(0xb05, 0xb4c3, Store::PrefixLe);
    assert_eq!((runtime.word(0xb04), runtime.word(0xb06)), (0xc37d, 0xe2b4));
    runtime.set_byte(0xb04, 0x81);
    runtime.set_byte(0xb07, 0xf2);
    assert_eq!((runtime.word(0xb04), runtime.word(0xb06)), (0xc381, 0xf2b4));
    runtime.store(0xb05, 0xa695, Store::SliceNative);
    assert_eq!(
        (
            runtime.session().player.body_position.x,
            runtime.session().player.body_position.y
        ),
        (0x9581, 0xf2a6)
    );
    assert_eq!(runtime.word(0xb05), 0xa695);
    assert_eq!(
        [
            runtime.samus_game().ram().read_byte(0xb04),
            runtime.samus_game().ram().read_byte(0xb05),
            runtime.samus_game().ram().read_byte(0xb06),
            runtime.samus_game().ram().read_byte(0xb07)
        ],
        [0x7d, 0xf1, 0x68, 0xe2]
    );
    runtime.session_mut().player.body_position.x = 0xe709;
    runtime.session_mut().player.body_position.y = 0xd823;
    assert_eq!(runtime.word(0xb05), 0x23e7);
    assert_eq!(
        (runtime.session().player.x, runtime.session().player.y),
        (0xc123, 0xd456)
    );
    assert_eq!(
        (runtime.inputs().camera_x, runtime.inputs().camera_y),
        (0x9110, 0x9220)
    );
    drop(runtime);
    assert_eq!(
        [
            game.ram().read_byte(0xb04),
            game.ram().read_byte(0xb05),
            game.ram().read_byte(0xb06),
            game.ram().read_byte(0xb07)
        ],
        [0x09, 0xe7, 0x23, 0xd8]
    );
    assert_eq!(game.samus_authority(), Available);
    assert!(!game.actor_identity_is_with_session());
    let runtime = Runtime::new(&mut game);
    assert_eq!(
        (
            runtime.session().player.body_position.x,
            runtime.session().player.body_position.y
        ),
        (0xe709, 0xd823)
    );
    assert_eq!(
        (
            runtime.word(0xaf6),
            runtime.word(0xafa),
            runtime.word(0x911),
            runtime.word(0x915)
        ),
        (0xc123, 0xd456, 0x9110, 0x9220)
    );
}

// Movies cannot inject this short loaded-ROM cannon failure before Disabled, Empty or Unknown.
pub fn samus_driver_cannon_failure_preserves_prefix() {
    use crate::game::{player_transitions::state::DrawMode, session_runtime::Runtime};
    for mode in [
        DrawMode::Disabled,
        DrawMode::Empty,
        DrawMode::Unknown(0xbeef),
    ] {
        let (mut game, loaded, _) = installed(RetainedNative);
        assert_eq!(loaded.len(), 8);
        game.ram().copy_in(0xdfe, &0xc35eu16.to_le_bytes());
        game.ram().copy_in(0xe00, &0xd46fu16.to_le_bytes());
        let mut runtime = Runtime::new(&mut game);
        runtime.session_mut().player.transitions.draw_handler = mode;
        runtime.session_mut().player.contact.auto_jump = 0x6b5a;
        let shots = &mut runtime.session_mut().projectiles;
        shots.controls.cannon_flags = 0xab01;
        shots.controls.cannon_frame = 2;
        shots.controls.cannon_drawing_mode = 0xe1d2;
        shots.controls.selected_weapon = 0xffff;
        shots.selected = 0x6a5b;
        shots.trails[0].left.remaining = 7;
        shots.trails[0].right.remaining = 9;
        assert!(runtime.has_session_for_verification());
        assert!(runtime.samus_game().actor_identity_is_with_session());
        assert_eq!(runtime.samus_game().samus_authority(), Available);
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.produce_player_scene();
        }))
        .expect_err("the cannon's loaded table must be out of bounds");
        let message = failure
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| failure.downcast_ref::<&str>().copied())
            .unwrap();
        assert!(message.contains("index out of bounds: the len is 8 but the index is 542687"));
        assert!(!runtime.has_session_for_verification());
        assert!(!runtime.samus_game().actor_identity_is_with_session());
        assert_eq!(runtime.samus_game().samus_authority(), Revoked);
        let ram = runtime.samus_game().ram();
        let word = |at| u16::from_le_bytes([ram.read_byte(at), ram.read_byte(at + 1)]);
        assert_eq!((word(0xaa6), word(0xaa8), word(0xaac)), (1, 3, 0xe1d2));
        assert_eq!(word(crate::ram_map::words::projectile_index), 0x6a5b);
        assert_eq!((word(0xd658), word(0xd67c)), (7, 9));
        assert_eq!(
            (word(0xaf4), word(0xdfe), word(0xe00)),
            (0x6b5a, 0xc35e, 0xd46f)
        );
        drop(runtime);
        assert_eq!(game.samus_authority(), Revoked);
        assert_eq!(game.preflight_samus_runtime(), Err(DisplayError::Poisoned));
        assert!(!game.actor_identity_is_with_session());
        assert_eq!(
            [
                game.ram().read_byte(0xaa6),
                game.ram().read_byte(0xaa7),
                game.ram().read_byte(0xaa8),
                game.ram().read_byte(0xaa9)
            ],
            [1, 0, 3, 0]
        );
    }
}

// Literal body workspace and player coordinates are supplied independently.
fn supply_player_body(game: &mut Game) {
    game.ram()
        .copy_in(0xb03, &[0x9c, 0x11, 0x22, 0x33, 0x44, 0x8d]);
    game.ram()
        .copy_in(0xaf6, &[0x5a, 0x6b, 0x7c, 0x8d, 0x9e, 0xaf, 0xb0, 0xc1]);
}
fn player_body_bytes(game: &Game) -> [u8; 6] {
    std::array::from_fn(|lane| game.ram().read_byte(0xb03 + lane))
}
fn assert_player_body_neighbors(game: &Game) {
    assert_eq!(game.ram().read_byte(0xb03), 0x9c);
    assert_eq!(game.ram().read_byte(0xb08), 0x8d);
    let player: [u8; 8] = std::array::from_fn(|lane| game.ram().read_byte(0xaf6 + lane));
    assert_eq!(player, [0x5a, 0x6b, 0x7c, 0x8d, 0x9e, 0xaf, 0xb0, 0xc1]);
}

// Legacy Game construction supplies fixture RAM, not an authored prepared start.
pub fn samus_game_body_projection_roundtrip() {
    let mut game = Game::new();
    supply_player_body(&mut game);
    {
        let runtime = crate::game::session_runtime::Runtime::new(&mut game);
        assert_eq!(runtime.session().player.body_position.x, 0x2211);
        assert_eq!(runtime.session().player.body_position.y, 0x4433);
        assert_eq!(runtime.session().player.x, 0x6b5a);
        assert_eq!(runtime.session().player.y, 0xaf9e);
        assert_eq!(runtime.word(0xb04), 0x2211);
        assert_eq!(runtime.word(0xb06), 0x4433);
    }
    assert_eq!(
        player_body_bytes(&game),
        [0x9c, 0x11, 0x22, 0x33, 0x44, 0x8d]
    );
    assert_player_body_neighbors(&game);
    let runtime = crate::game::session_runtime::Runtime::new(&mut game);
    assert_eq!(runtime.session().player.body_position.x, 0x2211);
    assert_eq!(runtime.session().player.body_position.y, 0x4433);
    assert_eq!(runtime.word(0xb04), 0x2211);
    assert_eq!(runtime.word(0xb06), 0x4433);
}

// An odd native store changes X's high byte and Y's low byte in one live projection.
pub fn samus_game_body_projection_crossing_writes() {
    let mut game = Game::new();
    supply_player_body(&mut game);
    {
        let mut runtime = crate::game::session_runtime::Runtime::new(&mut game);
        runtime.write_word_le(0x6655, 0xb04);
        assert_eq!(runtime.session().player.body_position.x, 0x6655);
        assert_eq!(runtime.session().player.body_position.y, 0x4433);
        runtime.write_slice_native(0xb05, u16::from_ne_bytes([0xa7, 0xb8]));
        assert_eq!(runtime.session().player.body_position.x, 0xa755);
        assert_eq!(runtime.word(0xb04), 0xa755);
        assert_eq!(runtime.session().player.body_position.y, 0x44b8);
        assert_eq!(runtime.word(0xb06), 0x44b8);
        assert_eq!(runtime.word(0xb05), 0xb8a7);
        runtime.write_slice_le(0xb06, 0xddee);
        assert_eq!(runtime.session().player.body_position.x, 0xa755);
        assert_eq!(runtime.session().player.body_position.y, 0xddee);
        assert_eq!(runtime.word(0xb04), 0xa755);
        assert_eq!(runtime.word(0xb06), 0xddee);
        assert_eq!(runtime.byte(0xb03), 0x9c);
        assert_eq!(runtime.byte(0xb08), 0x8d);
        assert_eq!(runtime.session().player.x, 0x6b5a);
        assert_eq!(runtime.session().player.y, 0xaf9e);
    }
    assert_eq!(
        player_body_bytes(&game),
        [0x9c, 0x55, 0xa7, 0xee, 0xdd, 0x8d]
    );
    assert_player_body_neighbors(&game);
}

// The retained-native fixture exports, adopts normal return, and poisons an unwind.
pub fn samus_game_body_foreign_adoption_and_prefix() {
    let (mut game, _, _) = installed(RetainedNative);
    supply_player_body(&mut game);
    let mut runtime = crate::game::session_runtime::Runtime::new(&mut game);
    runtime.write_word_le(0x6655, 0xb04);
    let returned = runtime.foreign(|game| {
        assert_eq!(game.samus_authority(), Revoked);
        assert_eq!(
            player_body_bytes(game),
            [0x9c, 0x55, 0x66, 0x33, 0x44, 0x8d]
        );
        game.ram().copy_in(0xb04, &[0x77, 0x88, 0x99, 0xaa]);
        0x6au16
    });
    assert_eq!(returned, 0x6a);
    assert!(runtime.has_session_for_verification());
    assert_eq!(runtime.samus_game().samus_authority(), Available);
    assert_eq!(runtime.session().player.body_position.x, 0x8877);
    assert_eq!(runtime.session().player.body_position.y, 0xaa99);
    assert_eq!(runtime.word(0xb04), 0x8877);
    assert_eq!(runtime.word(0xb06), 0xaa99);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.foreign(|game| -> () {
            assert_eq!(
                player_body_bytes(game),
                [0x9c, 0x77, 0x88, 0x99, 0xaa, 0x8d]
            );
            game.ram().write_byte(0xb05, 0xd1);
            game.ram().write_byte(0xb06, 0xe2);
            panic!("body foreign prefix");
        });
    }))
    .expect_err("body callback must panic");
    assert_eq!(failure.downcast_ref::<&str>(), Some(&"body foreign prefix"));
    assert!(!runtime.has_session_for_verification());
    assert!(!runtime.samus_game().actor_identity_is_with_session());
    assert_eq!(runtime.samus_game().samus_authority(), Revoked);
    assert_eq!(
        player_body_bytes(runtime.samus_game()),
        [0x9c, 0x77, 0xd1, 0xe2, 0xaa, 0x8d]
    );
    assert_player_body_neighbors(runtime.samus_game());
    drop(runtime);
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.preflight_samus_runtime(), Err(DisplayError::Poisoned));
    assert_eq!(
        player_body_bytes(&game),
        [0x9c, 0x77, 0xd1, 0xe2, 0xaa, 0x8d]
    );
    assert_player_body_neighbors(&game);
}

// The initialized loader fixture keeps real ROM allocations and authored role metadata.
fn returning_die_load_roles(
    mut roles: ContentDescription,
    loaded: &Arc<[u8]>,
    native: &Arc<[u8]>,
) -> ContentDescription {
    use crate::game::scene::samus_content::AllocationDeclaration;
    for (&(_, registration), role) in &roles.roles {
        let bytes = match registration {
            Loaded => loaded,
            Native => native,
        };
        let old = &roles.allocations[&role.allocation];
        let replacement = AllocationDeclaration {
            identity: old.identity,
            revision: old.revision,
            extent: bytes.len(),
            bytes: bytes.clone(),
        };
        roles
            .allocations
            .insert(role.allocation, Arc::new(replacement));
    }
    roles
}

fn assert_returning_die_load_prefix(game: &Game) {
    let recorder = &game.state_recorder;
    assert_eq!(recorder.total_frames, 0x1020_3040);
    assert_eq!(recorder.log.size, 3);
    assert_eq!(recorder.log.as_slice(), &[0x31, 0x42, 0xf3]);
    assert_eq!([recorder.log.byte(3), recorder.log.byte(4)], [0xa4, 0xb5]);
    assert_eq!(recorder.base_snapshot.size, 0);
    assert_eq!(
        [
            recorder.base_snapshot.byte(0),
            recorder.base_snapshot.byte(3)
        ],
        [0x91, 0xc4]
    );
    assert_eq!(recorder.snapshot_flags, 0x6677_8899);
    assert_eq!(
        (
            recorder.last_inputs,
            recorder.frames_since_last,
            recorder.replay_pos,
            recorder.replay_pos_last_complete,
            recorder.replay_frame_counter,
            recorder.replay_next_cmd_at,
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert!(recorder.replay_mode);
    assert_eq!(recorder.replay_cmd, 0xad);
    assert!(!recorder.has_base_audio_queue);
    assert_eq!(recorder.base_audio_queue, [0x5c; 71]);
    assert_eq!(game.snes_frame_counter, 0);
    assert_eq!(game.ram().read_byte(0x611), 1);
    assert!((0..0x2000).all(|at| game.g_sram.read(at) == 0));
}

// The host callback invokes the real unsafe StateRecorder_Load, with one returning Die.
// Movies cannot establish this deliberately incomplete load or observe lost Runtime ownership.
pub fn samus_game_returning_die_load_witness(game: &mut Game, load: impl FnOnce(&mut Game)) {
    use crate::game::session_runtime::Runtime;
    assert!(
        !game.g_snes.is_null(),
        "the host must initialize the actual machine"
    );
    let loaded = game.loaded_rom.as_ref().expect("initialized loaded ROM");
    let native = game.capture_native_rom();
    assert!(loaded.len() > 8 && native.len() > 8);
    let loaded: Arc<[u8]> = (0..loaded.len()).map(|at| loaded.read(at)).collect();
    let native: Arc<[u8]> = (0..native.len()).map(|at| native.read(at)).collect();
    let supplied = description(
        game,
        returning_die_load_roles(fixture::supplied(), &loaded, &native),
        RetainedNative,
    );
    let expected = description(
        game,
        returning_die_load_roles(fixture::expected(), &loaded, &native),
        RetainedNative,
    );
    let binding = game.prepare_samus_binding(supplied, &expected).unwrap();
    game.install_samus_binding(binding).unwrap();
    assert_eq!(game.samus_authority(), Available);
    assert_eq!(game.samus_execution(), Ok(Some(RetainedNative)));
    assert_eq!(game.preflight_samus_runtime(), Ok(()));

    let recorder = &mut game.state_recorder;
    recorder.log.resize(5).unwrap();
    recorder
        .log
        .as_mut_slice()
        .copy_from_slice(&[0xd1, 0xe2, 0xf3, 0xa4, 0xb5]);
    recorder.base_snapshot.resize(4).unwrap();
    recorder
        .base_snapshot
        .as_mut_slice()
        .copy_from_slice(&[0x91, 0xa2, 0xb3, 0xc4]);
    recorder.total_frames = 0xdead_beef;
    recorder.last_inputs = 0xabcd;
    recorder.frames_since_last = 0x1357_2468;
    recorder.snapshot_flags = 0xface_cafe;
    recorder.replay_pos = 0x2132;
    recorder.replay_pos_last_complete = 0x3243;
    recorder.replay_frame_counter = 0x4354;
    recorder.replay_next_cmd_at = 0x5465;
    recorder.replay_cmd = 0xad;
    recorder.replay_mode = false;
    recorder.has_base_audio_queue = true;
    recorder.base_audio_queue = [0x5c; 71];
    game.snes_frame_counter = 0x6789;
    game.ram().write_byte(0x611, 0xa6);
    game.g_sram.fill(0..0x2000, 0x7b);

    let mut runtime = Runtime::new(game);
    assert!(runtime.has_session_for_verification());
    assert!(runtime.samus_game().actor_identity_is_with_session());
    assert_eq!(runtime.samus_game().samus_authority(), Available);
    let mut load_returned = false;
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.foreign(|game| {
            assert_eq!(game.samus_authority(), Revoked);
            load(game);
            load_returned = true;
            assert_returning_die_load_prefix(game);
            assert_eq!(game.samus_authority(), Revoked);
            assert!(!game.actor_identity_is_with_session());
        });
    }))
    .expect_err("an incomplete returning-Die load must reject Samus resumption");
    let message = failure
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| failure.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(message, "Samus resumption rejected: Poisoned");
    assert!(
        load_returned,
        "the loader must complete its original reset continuation"
    );
    assert!(!runtime.has_session_for_verification());
    assert!(!runtime.samus_game().actor_identity_is_with_session());
    assert_eq!(runtime.samus_game().samus_authority(), Revoked);
    assert_returning_die_load_prefix(runtime.samus_game());
    drop(runtime);
    assert_eq!(game.samus_authority(), Revoked);
    assert_eq!(game.preflight_samus_runtime(), Err(DisplayError::Poisoned));
    assert_eq!(game.samus_execution(), Ok(Some(RetainedNative)));
    assert!(!game.actor_identity_is_with_session());
    assert_returning_die_load_prefix(game);
}
