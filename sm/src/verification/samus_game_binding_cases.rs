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
