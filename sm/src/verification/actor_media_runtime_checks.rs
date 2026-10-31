//! Actual room-palette decode and actual prepared SpriteMap emission on Runtime.
#![forbid(unsafe_code)]
use crate::{
    game::{
        room_entry::{
            content::{AssetRef, AssetSource},
            effects::{EntryEffects, InstallBuffer},
            world::ResourceId,
        },
        session::Point,
        session_runtime::Runtime,
    },
    game_state::Game,
    rom::{RomImage, lorom_offset},
};
use {
    crate::game::actors::identity::*,
    sm_game::actors::{access::ActorCollections, content::*, media::*, scene::*},
};

fn write_word(image: &mut [u8], at: usize, value: u16) {
    image[at..at + 2].copy_from_slice(&value.to_ne_bytes());
}
fn seed_actor(runtime: &mut Runtime<'_>) -> InstanceHandle {
    let pending = runtime
        .session()
        .actor_identity
        .prepare_birth(
            0,
            ActorProvenance {
                authored_species: Some(SpeciesId(7)),
                origin: ActorBirthOrigin::Dynamic {
                    template: ActorTemplateId(11),
                    part: 0,
                },
            },
            Some(AppearanceId(20)),
        )
        .unwrap();
    runtime.session_mut().actors.slots[0].definition = 0xd000;
    runtime.session_mut().actor_identity.commit_birth(pending)
}
fn stage_receipt(runtime: &mut Runtime<'_>, image: &RomImage, resource: ResourceId, address: u32) {
    let at = lorom_offset(address) as usize;
    let captured = ActorLiveRecord {
        image: image.clone(),
        base: at,
    };
    let mapped =
        sm_game::actors::media::palette_resource_for_capture(&captured, &runtime.session().world);
    assert_eq!(
        mapped,
        Some(resource),
        "actor.palette.fixture_source_maps_to_declared_resource"
    );
    let colors = core::array::from_fn(|lane| image.read(at + lane));
    runtime
        .session_mut()
        .actor_identity
        .record_palette_residency(9, None, image.clone());
    runtime.copy_palette_row(1, colors); // Actual OBJ staging copy into target row9.
    runtime
        .session_mut()
        .actor_identity
        .record_palette_residency(9, mapped, image.clone());
}
pub(crate) fn run_checked() {
    prepared_sprite_palette_modes_use_actual_oam_emission();
    actual_room_palette_decode_clears_obj_receipts_before_dynamic_reselection();
    full_room_catalog_preserves_actor_media_and_accepts_prepared_bg_receipt();
}

fn prepared_sprite_palette_modes_use_actual_oam_emission() {
    let world = sm_game::actors::fixture::binding(false, AppearanceId(20), false);
    let mut bytes = vec![0; 0x200000];
    let map = lorom_offset(0xa08000) as usize;
    write_word(&mut bytes, map, 1);
    write_word(&mut bytes, map + 2, 3);
    bytes[map + 4] = 4;
    write_word(&mut bytes, map + 5, 0x6a25); // Embedded palette5, plus non-palette flag bits.
    let image = RomImage::from(bytes);
    for (mode, palette, expected) in [
        (ActorPaletteMode::Normal, 0, 0x6238),
        (ActorPaletteMode::Flash, 0, 0x6a38),
        (ActorPaletteMode::Frozen, 0x0c00, 0x6e38),
    ] {
        let mut game = Game::with_world(world.clone());
        game.register_rom(image.clone());
        let mut runtime = Runtime::new(&mut game);
        stage_receipt(&mut runtime, &image, ResourceId(3), 0xb48000);
        // Fade-style writes to the actual owned palette buffers do not stage a new source.
        runtime.session_mut().presentation.visual_mut().colors[9 * 16] = 0x1234;
        runtime.session_mut().presentation.visual_mut().targets[9 * 16] = 0x5678;
        assert_eq!(
            world.actor_media().validate_material_residency(
                MaterialId(50),
                runtime.session().actor_identity.palette_residency(),
                &image
            ),
            Ok(()),
            "actor.media.palette_edits_preserve_residency"
        );
        let pose = &world.actors().definitions().unwrap().poses[&PoseId(9)];
        let visual = world
            .actor_media()
            .resolve(AppearanceId(20), PoseId(9), pose)
            .unwrap();
        crate::game::actors::media_runtime::draw(
            &mut runtime,
            ActorDrawRequest {
                visual: ActorVisualSelection::Prepared(visual),
                position: Point { x: 40, y: 50 },
                palette,
                tile: 0x13,
                palette_mode: mode,
                mode: ActorDrawMode::Base,
            },
        )
        .unwrap();
        assert_eq!([runtime.byte(0x370), runtime.byte(0x371)], [43, 54]);
        let tile = u16::from_ne_bytes([runtime.byte(0x372), runtime.byte(0x373)]);
        assert_eq!(
            tile, expected,
            "actor.media.actual_prepared_palette_mode mode={mode:?}"
        );
        assert_eq!(
            u16::from_ne_bytes([runtime.byte(0x590), runtime.byte(0x591)]),
            4,
            "actor.media.actual_sprite_emitted_once"
        );
    }
}

fn actual_room_palette_decode_clears_obj_receipts_before_dynamic_reselection() {
    let world = sm_game::actors::fixture::binding(false, AppearanceId(20), true);
    let mut bytes = vec![0; 0x200000];
    let stream = lorom_offset(0xb49000) as usize;
    bytes[stream] = 31; // Two actual literal decode commands,32 bytes each.
    for (i, byte) in bytes[stream + 1..stream + 33].iter_mut().enumerate() {
        *byte = 0x20 + i as u8;
    }
    bytes[stream + 33] = 31;
    for (i, byte) in bytes[stream + 34..stream + 66].iter_mut().enumerate() {
        *byte = 0x40 + i as u8;
    }
    bytes[stream + 66] = 0xff;
    let image = RomImage::from(bytes);
    let mut game = Game::with_world(world.clone());
    game.register_rom(image.clone());
    let mut runtime = Runtime::new(&mut game);
    let handle = seed_actor(&mut runtime);
    stage_receipt(&mut runtime, &image, ResourceId(3), 0xb48000);
    runtime
        .session_mut()
        .actor_identity
        .record_palette_residency(2, Some(ResourceId(4)), image.clone());
    assert_eq!(
        world.actor_media().validate_appearance_residency(
            AppearanceId(20),
            runtime.session().actor_identity.palette_residency(),
            &image
        ),
        Ok(())
    );
    <Runtime<'_> as EntryEffects>::decompress(
        &mut runtime,
        AssetSource::Bound(AssetRef::Prepared(ResourceId(4))),
        InstallBuffer::Palette,
    );
    for i in 0..64 {
        assert_eq!(
            runtime.byte(0xc200 + i),
            0x20 + i as u8,
            "actor.palette.actual_room_decode_bytes"
        );
    }
    let identity = &runtime.session().actor_identity;
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(50),
            identity.palette_residency(),
            &image
        ),
        Err(ActorContentError::PaletteNotResident(MaterialId(50))),
        "actor.palette.next_room_revokes_previous_obj"
    );
    for material in [MaterialId(52), MaterialId(53)] {
        assert_eq!(
            world.actor_media().validate_material_residency(
                material,
                identity.palette_residency(),
                &image
            ),
            Ok(()),
            "actor.palette.actual_decode_records_complete_bg_rows"
        );
    }
    assert_eq!(
        world.actor_media().validate_material_residency(
            MaterialId(54),
            identity.palette_residency(),
            &image
        ),
        Err(ActorContentError::PaletteNotResident(MaterialId(54))),
        "actor.palette.actual_decode_drops_unproduced_bg_row"
    );
    let before = identity.clone();
    let result = {
        let state = runtime.session_mut();
        state
            .actor_identity
            .set_appearance(handle, AppearanceId(21), &state.actors, &world, &image)
    };
    assert_eq!(
        result,
        Err(ActorIdentityError::Content(
            ActorContentError::PaletteNotResident(MaterialId(51))
        )),
        "actor.palette.new_room_selection_requires_new_obj_staging"
    );
    assert_eq!(
        runtime.session().actor_identity,
        before,
        "actor.palette.actual_transition_rejection_before_mutation"
    );
    stage_receipt(&mut runtime, &image, ResourceId(4), 0xb49000);
    let state = runtime.session_mut();
    state
        .actor_identity
        .set_appearance(handle, AppearanceId(21), &state.actors, &world, &image)
        .unwrap();
    assert_eq!(
        state
            .actor_identity
            .observe(0, true, Some(SpeciesId(7)))
            .appearance,
        Some(AppearanceId(21)),
        "actor.palette.same_row_reused_for_next_room_material"
    );
}

// Movies cannot compose Prepared Actor media with a full catalog before owner construction.
fn full_room_catalog_preserves_actor_media_and_accepts_prepared_bg_receipt() {
    use sm_game::room_entry::world::*;
    let original = sm_game::actors::fixture::binding(false, AppearanceId(20), true);
    let mut world = original.definition().unwrap().clone();
    let mut keys = original.compatibility_keys().clone();
    world
        .resources
        .insert(ResourceId(5), ResourceKind::TileTable);
    world.resources.insert(ResourceId(6), ResourceKind::Tiles);
    keys.resources.insert(
        ResourceId(5),
        ResourceKey::Long(LegacyAddress24([0, 0x90, 0xc0])),
    );
    keys.resources.insert(
        ResourceId(6),
        ResourceKey::Long(LegacyAddress24([0, 0x91, 0xc0])),
    );
    world.tilesets.insert(
        TilesetId(9),
        TilesetDefinition {
            tile_table: ResourceId(5),
            tiles: ResourceId(6),
            palette: ResourceId(4),
        },
    );
    let binding = WorldBinding::prepared_with_actors(
        world.clone(),
        keys.clone(),
        original.actors().definitions().unwrap().clone(),
        original.actors().keys().unwrap().clone(),
        original.actor_media().definitions().unwrap().clone(),
    )
    .unwrap();
    let revisions = (
        binding.actor_gameplay_revision(),
        binding.actor_media_revision(),
    );
    let mut source = vec![0xff; 0x400000];
    let at = lorom_offset(0xb49000) as usize;
    source[at..at + 4].copy_from_slice(&[0xe4, 0x1f, 0x66, 0xff]);
    let assets = PreparedRoomAssets::import(&source, &world, &keys).unwrap();
    let revision = assets.fingerprint();
    let binding = WorldBinding::with_room_assets(binding, assets).unwrap();
    assert_eq!(
        (
            binding.actor_gameplay_revision(),
            binding.actor_media_revision()
        ),
        revisions
    );
    let owner = crate::game_state::GameOwner::with_world(binding.clone());
    owner.with(|game| {
        let image = RomImage::zeroed(1);
        let mut h = Runtime::new(game);
        let check = |h: &Runtime<'_>, image: &RomImage| {
            binding.actor_media().validate_material_residency(
                MaterialId(52),
                h.session().actor_identity.palette_residency(),
                image,
            )
        };
        let missing = Err(ActorContentError::PaletteNotResident(MaterialId(52)));
        assert_eq!(check(&h, &image), missing);
        EntryEffects::decompress(
            &mut h,
            AssetSource::Bound(AssetRef::Prepared(ResourceId(4))),
            InstallBuffer::Palette,
        );
        assert_eq!(check(&h, &image), Ok(()));
        h.session_mut()
            .actor_identity
            .record_prepared_palette_residency(0, ResourceId(4), [0; 32]);
        assert_eq!(check(&h, &image), missing);
        h.session_mut()
            .actor_identity
            .record_prepared_palette_residency(0, ResourceId(4), revision);
        assert_eq!(check(&h, &RomImage::zeroed(1)), Ok(()));
    });
}
