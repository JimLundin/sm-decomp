use super::*;
use data::assets::samus::{SamusAssets, SpriteSheet};
use std::collections::BTreeMap;

fn fixture() -> (ImportedAssets, SamusTiles, Graphics, Clips) {
    let first = TileDefinition {
        source: GraphicsRef {
            bank: 0x9b,
            offset: 0x8000,
        },
        first: 2,
        second: 2,
    };
    let second = TileDefinition {
        source: GraphicsRef {
            offset: 0x8004,
            ..first.source
        },
        ..first
    };
    let tiles = SamusTiles::new(Vec::new(), vec![first, second]);
    let upload = TileUpload {
        first: vec![1, 2],
        second: vec![3, 4],
    };
    let imported = ImportedAssets {
        assets: SamusAssets {
            sheet: SpriteSheet {
                sprites: Vec::new(),
                uploads: vec![upload.clone(), upload],
            },
            ..SamusAssets::default()
        },
        upload_bindings: BTreeMap::from([
            (TileDefinitionId(0), TilesId(1)),
            (TileDefinitionId(1), TilesId(0)),
        ]),
    };
    (
        imported,
        tiles,
        Graphics::new(vec![(0x9b, vec![1, 2, 3, 4, 1, 2, 3, 4])]),
        Clips::default(),
    )
}

#[test]
fn equal_upload_bytes_do_not_replace_actual_definition_identity() {
    let (imported, tiles, graphics, clips) = fixture();
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    let upload = imported.assets.sheet.upload(TilesId(0)).unwrap();
    let bound = content
        .upload(TilesId(0), TileDefinitionId(1), upload)
        .unwrap();
    assert_eq!(
        bound.descriptor,
        tiles.pointer(TileDefinitionId(1)).unwrap()
    );
    assert_eq!(bound.first.start.offset, 0x8004);
    assert_eq!(bound.second.unwrap().start.offset, 0x8006);
    assert_eq!(
        content.upload(TilesId(0), TileDefinitionId(0), upload),
        Err(Fault::Binding)
    );
    assert_eq!(
        TileDefinition::decode(header(*tiles.definition(TileDefinitionId(1)).unwrap())),
        *tiles.definition(TileDefinitionId(1)).unwrap()
    );
}

#[test]
fn ambiguous_assignments_wrong_current_bytes_and_unchecked_zero_first_reject() {
    let (mut imported, tiles, graphics, clips) = fixture();
    imported
        .upload_bindings
        .insert(TileDefinitionId(0), TilesId(0));
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    assert_eq!(
        content.upload(
            TilesId(0),
            TileDefinitionId(1),
            imported.assets.sheet.upload(TilesId(0)).unwrap()
        ),
        Err(Fault::Binding)
    );
    let (imported, _, graphics, clips) = fixture();
    let definition = TileDefinition {
        source: GraphicsRef {
            bank: 0x9b,
            offset: 0x8004,
        },
        first: 0,
        second: 0,
    };
    let tiles = SamusTiles::new(Vec::new(), vec![definition, definition]);
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    assert_eq!(first_length(0), 65536);
    assert_eq!(
        content.upload(
            TilesId(0),
            TileDefinitionId(1),
            imported.assets.sheet.upload(TilesId(0)).unwrap()
        ),
        Err(Fault::Bytes)
    );
    let (imported, tiles, graphics, clips) = fixture();
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    let wrong = TileUpload {
        first: vec![8, 9],
        second: vec![3, 4],
    };
    assert_eq!(
        content.upload(TilesId(0), TileDefinitionId(1), &wrong),
        Err(Fault::Bytes)
    );
}

#[test]
fn encoded_zero_second_has_no_region_and_preserves_first_source() {
    let (mut imported, _, graphics, clips) = fixture();
    imported.assets.sheet.uploads[0].second.clear();
    let tiles = SamusTiles::new(
        Vec::new(),
        vec![
            TileDefinition::default(),
            TileDefinition {
                source: GraphicsRef {
                    bank: 0x9b,
                    offset: 0x8004,
                },
                first: 2,
                second: 0,
            },
        ],
    );
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    let bound = content
        .upload(
            TilesId(0),
            TileDefinitionId(1),
            imported.assets.sheet.upload(TilesId(0)).unwrap(),
        )
        .unwrap();
    assert_eq!(bound.first.bytes, [1, 2]);
    assert_eq!(bound.second, None);
    assert_eq!(
        TileDefinition {
            source: GraphicsRef {
                bank: 0x9b,
                offset: 0xfffe
            },
            first: 4,
            second: 2
        }
        .second_source(),
        GraphicsRef {
            bank: 0x9b,
            offset: 2
        }
    );
}

fn checked_frames(imported: &ImportedAssets, tiles: &SamusTiles) -> u64 {
    use data::assets::samus::map;
    let mut frames = 0_u64;
    for (clip, entry) in imported.assets.clips.iter().zip(map::MAP) {
        for (index, frame) in clip.frames.iter().enumerate() {
            let Some(actual) = frame.tiles else {
                continue;
            };
            let native = tiles
                .frame(
                    entry.id,
                    data::pose::animation::AnimationFrame(u16::try_from(index).unwrap()),
                )
                .expect("actual native frame exists");
            assert!(
                imported.upload_bindings.get(&native.top) == Some(&actual.top),
                "frame top identity differs"
            );
            let bottom = native.bottom.map(|definition| {
                *imported
                    .upload_bindings
                    .get(&definition)
                    .expect("actual bottom assignment exists")
            });
            assert!(bottom == actual.bottom, "frame bottom identity differs");
            frames = frames.checked_add(1).unwrap();
        }
    }
    frames
}

fn selected_model_receipt(
    rendering: &game::production::frame::rendering_stage::Content,
    clips: &Clips,
) -> NormalDrawingReceipt {
    use core::num::Wrapping;
    use game::production::{
        appearance::Work,
        frame::{
            appearance_stage, borrowed::SpriteBuilderOrigin, movement::Movement, rendering_stage,
        },
    };
    use samus_appearance::{AppearanceRead, Frames, LapsLeft, Playhead, SuspendedAppearance};
    let figure = clips
        .poses()
        .iter()
        .find(|clip| {
            clip.figure.pose.action == types::Action::Running
                && clip.figure.pose.facing == types::Facing::Right
                && clip.figure.pose.gun == types::Gun::Stowed
                && clip.figure.pose.legs == types::Legs::Together
        })
        .expect("actual running content exists")
        .figure;
    let movement = Movement::new(samus_movement::Samus {
        pose: figure.pose,
        position: types::Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        },
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: types::Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: Wrapping(0),
            phase: samus_movement::Phase::Ground,
            bounces: samus_movement::Bounces::None,
        },
        spin: samus_movement::Spin::Off,
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
    });
    let mut appearance = Work::new(SuspendedAppearance::new(
        samus_appearance::State {
            playhead: Playhead {
                step: samus_appearance::Step::FIRST,
                time_left: Frames::LAST,
            },
            stage: types::BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        None,
        Vec::new(),
    ));
    appearance_stage::choose_read(
        &mut appearance,
        AppearanceRead::Draw {
            previous: figure.pose.action,
            current: figure.pose.action,
        },
    )
    .unwrap();
    appearance_stage::complete(&mut appearance).unwrap();
    let contacts = rendering_stage::Contacts::default();
    let mut builder = SpriteBuilderOrigin {
        backing: [0; 544],
        cursor: 0,
        prepared_flags: 0,
        prepared_top: 0,
        prepared_bottom: 0,
    };
    let mut stage = rendering_stage::Stage::default();
    let receipt = stage
        .choose(
            &mut builder,
            rendering,
            &rendering_stage::Inputs {
                movement: &movement,
                appearance: &appearance,
                contacts: &contacts,
                figure,
                premises: rendering_stage::Premises {
                    camera: [0, 0],
                    shake: [0, 0],
                    normal_visible: true,
                    no_additional_selected_pieces: true,
                    unselected_draws_empty: true,
                },
            },
        )
        .expect("constructed selected model draw completes");
    receipt.clone()
}

#[test]
#[ignore = "requires explicit pinned SM_ROM; exercised by the focused importer binding proof"]
fn actual_rom_import_bindings_headers_and_selected_model_uploads_are_exact() {
    use crate::lock::Sha256;
    use data::{
        pose::{
            Poses,
            animation::{AnimationDelays, RunPaces},
        },
        rom::Rom,
    };
    let path = std::env::var_os("SM_ROM").expect("explicit SM_ROM is required for this proof");
    let image = std::fs::read(path).expect("ROM source is readable");
    let pin = Sha256::try_from("12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72")
        .unwrap();
    assert!(Sha256::of(&image) == pin, "ROM source pin differs");
    let cartridge = Rom::new(image).expect("ROM source shape is valid");
    let rom = cartridge.lorom();
    let poses = Poses::load(rom).expect("pose content decodes");
    let delays = AnimationDelays::load(rom, &poses).expect("delay content decodes");
    let tiles = SamusTiles::load(rom, &delays, &poses).expect("tile content decodes");
    let graphics =
        Graphics::load(rom, data::pose::tiles::GRAPHICS_BANKS).expect("graphics content decodes");
    let paces = RunPaces::load(rom).expect("pace content decodes");
    let clips = Clips::import(&delays, &paces).expect("clean clip content imports");
    let imported = import::samus::import_with_bindings(rom).expect("actual assets import");
    let content = Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips: &clips,
    };
    let mut checked = 0_u64;
    for (definition, id) in &imported.upload_bindings {
        let actual = imported
            .assets
            .sheet
            .upload(*id)
            .expect("actual pool ID resolves");
        let bound = content
            .upload(*id, *definition, actual)
            .expect("actual preserved binding validates");
        assert!(
            content.source(&bound, rom).is_ok(),
            "bound source header differs"
        );
        checked = checked.checked_add(1).unwrap();
    }
    assert!(checked > 0);
    let frames = checked_frames(&imported, &tiles);
    let rendering =
        game::production::frame::rendering_stage::Content::import(&imported.assets, &clips)
            .expect("actual rendering bindings import");
    let receipt = selected_model_receipt(&rendering, &clips);
    let selected = rendering
        .uploads(&receipt)
        .expect("actual selected upload outputs resolve");
    let bound = content
        .bind(&receipt, &selected)
        .expect("actual selected model uploads bind");
    assert!(
        content.source(&bound.top, rom).is_ok(),
        "selected top source header differs"
    );
    if let Some(bottom) = &bound.bottom {
        assert!(
            content.source(bottom, rom).is_ok(),
            "selected bottom source header differs"
        );
    }
    if let Some(path) = std::env::var_os("SM_TILES_BINDING_REPORT") {
        std::fs::write(path, format!("format = \"actual-tiles-importer-binding-proof-v1\"\nrom_sha256 = \"{pin}\"\nchecked_bindings = {checked}\nchecked_frames = {frames}\nselected_uploads = {}\n", selected.len())).expect("safe binding receipt writes");
    }
}
