use super::*;
use crate::state::slots::Frames;
use crate::state::video::{OamEntry, SamusTileUploads, SpriteCount};
use crate::state::{FrameCount, State};
use data::pose::Poses;
use data::pose::spritemap::{SamusSpritemaps, SpritemapIndex};
use data::pose::tiles::{FrameTiles, SamusTiles, TileDefinition, TileDefinitionId};
use data::spritemap::Spritemap;

/// A pose definition: facing, movement type, Y offset.
fn definition(facing: u8, movement: u8, y_offset: u8) -> [u8; 8] {
    [facing, movement, 0xFF, 0xFF, y_offset, 0, 0x15, 0]
}

/// Pose 0 faces forward; pose 1 stands facing right, its graphics 8 pixels up; pose 2 is
/// the morph ball. Each pose's top half is spritemap 1 plus the frame, its bottom half
/// spritemap 3; spritemap 1 is one sprite at (-4, -8), 2 one at (0, 0), 3 two sprites.
fn content() -> Content {
    let poses = [
        definition(0, 0, 0),
        definition(8, 0, 8),
        definition(8, 4, 0),
    ]
    .concat();
    let sprite = |x: u8, y: u8, tile: u8| [x, 0x00, y, tile, 0x28];
    let map = |entries: &[[u8; 5]]| Some(Spritemap::decode(&entries.concat()));
    Content {
        poses: Poses::decode(&poses).unwrap(),
        samus_spritemaps: SamusSpritemaps::new(
            vec![SpritemapIndex(1); 3],
            vec![SpritemapIndex(3); 3],
            vec![
                None,
                map(&[sprite(0xFC, 0xF8, 1)]),
                map(&[sprite(0, 0, 2)]),
                map(&[sprite(0, 0, 3), sprite(8, 0, 4)]),
            ],
            PositionOffsets::default(),
        ),
        samus_tiles: SamusTiles::new(
            vec![
                Vec::new(),
                vec![FrameTiles {
                    top: TileDefinitionId(1),
                    bottom: Some(TileDefinitionId(2)),
                }],
                vec![
                    FrameTiles::default(),
                    FrameTiles {
                        top: TileDefinitionId(0),
                        bottom: None,
                    },
                ],
            ],
            vec![TileDefinition::default(); 3],
        ),
        ..Content::default()
    }
}

/// Samus at (0x150, 0x90) in pose `pose`, the camera at (0x100, 0x20).
fn state(pose: u16) -> State {
    let mut state = State::default();
    state.samus.pose = PoseId(pose);
    state.samus.motion.position.x = crate::model::num::Fixed::new(0x150, 0);
    state.samus.motion.position.y = crate::model::num::Fixed::new(0x90, 0);
    state.room.camera.x.pixels = 0x100;
    state.room.camera.y.pixels = 0x20;
    state
}

fn tiles(state: &State) -> Vec<u8> {
    let drawn = usize::from(state.video.batch.sprites.0);
    state.video.batch.oam.low[..drawn]
        .iter()
        .map(|entry| entry.tile)
        .collect()
}

#[test]
fn the_top_half_then_the_bottom_half_draw_from_her_screen_position() {
    let mut drawn = state(1);
    draw(&mut drawn, &content());
    assert_eq!(tiles(&drawn), [1, 3, 4]);
    // X: $150 - $100 - 4; Y: $90 - 8 - $20 - 8.
    assert_eq!(
        drawn.video.batch.oam.low[0],
        OamEntry {
            x: 0x4C,
            y: 0x60,
            tile: 1,
            attributes: 0x28
        }
    );
    assert_eq!(drawn.video.batch.oam.low[2].x, 0x58);
    assert_eq!(drawn.video.batch.oam.low[2].y, 0x68);
}

#[test]
fn the_atmospheric_graphics_draw_before_her_body() {
    use crate::state::atmospheric::{
        AtmosphericFrame, AtmosphericGraphic, AtmosphericKind, AtmosphericTimer, LANDING_SLOT,
    };
    let mut drawn = state(1);
    drawn.atmospherics.slots[LANDING_SLOT] = Some(AtmosphericGraphic {
        kind: AtmosphericKind::LandingDust,
        frame: AtmosphericFrame(0),
        timer: AtmosphericTimer(3),
        x: 0x150,
        y: 0x90,
    });
    draw(&mut drawn, &content());
    // `$90:EB7C`: the dust, then her top and bottom halves.
    assert_eq!(tiles(&drawn), [0x48, 1, 3, 4]);
}

#[test]
fn a_hidden_samus_leaves_the_atmospheric_graphics_alone() {
    use crate::state::atmospheric::{
        AtmosphericFrame, AtmosphericGraphic, AtmosphericKind, AtmosphericTimer,
    };
    let mut hidden = state(1);
    hidden.samus.drawing = DrawingHandler::Hidden;
    let dust = Some(AtmosphericGraphic {
        kind: AtmosphericKind::LandingDust,
        frame: AtmosphericFrame(0),
        timer: AtmosphericTimer(3),
        x: 0x150,
        y: 0x90,
    });
    hidden.atmospherics.slots[0] = dust;
    draw(&mut hidden, &content());
    assert_eq!(hidden.atmospherics.slots[0], dust);
    assert_eq!(tiles(&hidden), [0_u8; 0]);
}

#[test]
fn the_frame_picks_the_spritemap_and_the_morph_ball_has_no_bottom_half() {
    let mut rolling = state(2);
    rolling.samus.animation.frame = AnimationFrame(1);
    draw(&mut rolling, &content());
    assert_eq!(tiles(&rolling), [2]);
}

#[test]
fn facing_forward_adds_the_chest_sprite_between_the_halves() {
    let mut forward = state(0);
    draw(&mut forward, &content());
    assert_eq!(tiles(&forward), [1, 0x21, 3, 4]);
    // $150 - 7 - $100 and $90 - 17 - $20, palette 4 and priority 3.
    assert_eq!(
        forward.video.batch.oam.low[1],
        OamEntry {
            x: 0x49,
            y: 0x5F,
            tile: 0x21,
            attributes: 0x38
        }
    );
    // From the third frame the forward pose's origin is a pixel up, not its graphics offset;
    // frame 2 draws spritemap 3, whose first sprite is at (0, 0).
    let mut later = state(0);
    later.samus.animation.frame = AnimationFrame(2);
    draw(&mut later, &content());
    assert_eq!(later.video.batch.oam.low[0].y, 0x90 - 1 - 0x20);
}

#[test]
fn an_invincible_samus_flickers_on_odd_frames() {
    let mut odd = state(1);
    odd.samus.invincibility = Frames(30);
    odd.clock.counters.completed = FrameCount(7);
    draw(&mut odd, &content());
    assert_eq!(odd.video.batch.sprites, SpriteCount(0));
    odd.clock.counters.completed = FrameCount(8);
    draw(&mut odd, &content());
    assert_eq!(tiles(&odd), [1, 3, 4]);
}

#[test]
fn drawing_starts_from_the_sprites_already_drawn() {
    let mut after = state(1);
    after.video.batch.sprites = SpriteCount(5);
    draw(&mut after, &content());
    assert_eq!(after.video.batch.sprites, SpriteCount(8));
    assert_eq!(after.video.batch.oam.low[5].tile, 1);
}

#[test]
fn the_bottom_half_handlers_follow_the_movement_type() {
    let frame = AnimationFrame;
    assert_eq!(
        bottom_half(MovementType::SpinJumping, PoseId(0x19), frame(1)),
        BottomHalf::Hidden
    );
    assert_eq!(
        bottom_half(MovementType::SpinJumping, PoseId(0x19), frame(0)),
        BottomHalf::Drawn
    );
    assert_eq!(
        bottom_half(MovementType::CrouchingTransition, PoseId(0x37), frame(0)),
        BottomHalf::Hidden
    );
    assert_eq!(
        bottom_half(MovementType::CrouchingTransition, PoseId(0x35), frame(0)),
        BottomHalf::Drawn
    );
    assert_eq!(
        bottom_half(MovementType::Standing, PoseId(0x9B), frame(0)),
        BottomHalf::Drawn
    );
}

#[test]
fn the_elevator_handler_draws_her_on_even_frames_without_the_flicker() {
    let mut riding = state(1);
    riding.samus.drawing = DrawingHandler::UsingElevator;
    riding.samus.invincibility = Frames(30);
    riding.clock.counters.completed = FrameCount(7);
    draw(&mut riding, &content());
    assert_eq!(riding.video.batch.sprites, SpriteCount(0));
    // An even frame draws her though she is invincible: `$90:8A00` has no flicker.
    riding.clock.counters.completed = FrameCount(8);
    draw(&mut riding, &content());
    assert_eq!(tiles(&riding), [1, 3, 4]);
}

#[test]
fn the_inanimate_handler_draws_every_frame_and_the_hidden_one_never() {
    for completed in [7, 8] {
        let mut inanimate = state(1);
        inanimate.samus.drawing = DrawingHandler::Inanimate;
        inanimate.samus.invincibility = Frames(30);
        inanimate.clock.counters.completed = FrameCount(completed);
        draw(&mut inanimate, &content());
        assert_eq!(tiles(&inanimate), [1, 3, 4]);

        let mut hidden = state(1);
        hidden.samus.drawing = DrawingHandler::Hidden;
        hidden.clock.counters.completed = FrameCount(completed);
        draw(&mut hidden, &content());
        assert_eq!(hidden.video.batch.sprites, SpriteCount(0));
    }
}

#[test]
fn drawing_queues_the_frames_tile_uploads_flickering_or_not() {
    let queued = |top, bottom: Option<u16>| SamusTileUploads {
        top: Some(TileDefinitionId(top)),
        bottom: bottom.map(TileDefinitionId),
    };
    let mut drawn = state(1);
    drawn.samus.invincibility = Frames(30);
    drawn.clock.counters.completed = FrameCount(1);
    draw(&mut drawn, &content());
    assert_eq!(tiles(&drawn), []);
    assert_eq!(drawn.video.batch.samus_tiles, queued(1, Some(2)));
    // A frame without bottom tiles leaves the bottom half's upload as it was.
    let mut rolling = state(2);
    rolling.samus.animation.frame = AnimationFrame(1);
    rolling.video.batch.samus_tiles.bottom = Some(TileDefinitionId(2));
    draw(&mut rolling, &content());
    assert_eq!(rolling.video.batch.samus_tiles, queued(0, Some(2)));
}

#[test]
fn the_elevator_queues_no_tiles_on_odd_frames_and_hidden_samus_none() {
    let mut riding = state(1);
    riding.samus.drawing = DrawingHandler::UsingElevator;
    riding.clock.counters.completed = FrameCount(1);
    draw(&mut riding, &content());
    assert_eq!(riding.video.batch.samus_tiles, SamusTileUploads::default());
    riding.clock.counters.completed = FrameCount(2);
    draw(&mut riding, &content());
    assert_eq!(
        riding.video.batch.samus_tiles.top,
        Some(TileDefinitionId(1))
    );
    let mut hidden = state(1);
    hidden.samus.drawing = DrawingHandler::Hidden;
    draw(&mut hidden, &content());
    assert_eq!(hidden.video.batch.samus_tiles, SamusTileUploads::default());
}

/// The ROM's content for drawing her, from `SM_ROM` or the checkout's `rom/sm.smc`; `None`
/// where it isn't present.
fn rom_content() -> Option<Content> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let rom = data::rom::Rom::new(std::fs::read(path).ok()?).unwrap();
    let lorom = rom.lorom();
    let poses = Poses::load(lorom).unwrap();
    let delays = data::pose::animation::AnimationDelays::load(lorom, &poses).unwrap();
    Some(Content {
        samus_tiles: SamusTiles::load(lorom, &delays, &poses).unwrap(),
        samus_spritemaps: SamusSpritemaps::load(lorom).unwrap(),
        graphics: data::pose::tiles::Graphics::load(lorom, data::pose::tiles::GRAPHICS_BANKS)
            .unwrap(),
        poses,
        delays,
        ..Content::default()
    })
}

/// The bytes a tiles definition uploads, as the commit reads them.
fn uploaded(content: &Content, id: TileDefinitionId) -> data::assets::samus::TileUpload {
    let definition = content.samus_tiles.definition(id).unwrap();
    let first = match definition.first {
        0 => 0x1_0000,
        size => usize::from(size),
    };
    let second = usize::from(definition.second);
    let bytes = |from, count| content.graphics.bytes(from, count).unwrap().to_vec();
    data::assets::samus::TileUpload {
        first: bytes(definition.source, first),
        second: if second == 0 {
            Vec::new()
        } else {
            bytes(definition.second_source(), second)
        },
    }
}

/// For every pose id of hers, in every dress that shows it, every frame of the imported
/// asset draws, holds and uploads what the engine does for that id and frame today.
#[test]
fn the_imported_assets_draw_what_the_engine_draws() {
    use data::assets::samus::import::{self, map};
    use data::assets::samus::key::{Dress, SpinSprite, Suit};
    use data::pose::animation::{AnimationFrame, DelayItem};

    let Some(content) = rom_content() else {
        return;
    };
    let assets = import::from_tables(import::Tables {
        poses: &content.poses,
        delays: &content.delays,
        spritemaps: &content.samus_spritemaps,
        tiles: &content.samus_tiles,
        graphics: &content.graphics,
    })
    .unwrap();
    assert_eq!(assets.chest.attributes, CHEST);
    assert_eq!((assets.chest.left, assets.chest.up), CHEST_OFFSET);
    let running = content.delays.running();
    assert_eq!(assets.running.len(), running.len());
    for (at, hold) in assets.running.iter().enumerate() {
        let frame = AnimationFrame(u16::try_from(at).unwrap());
        assert_eq!(Some(hold.timer_byte()), running.byte(frame));
    }
    let dresses = [
        SpinSprite::Plain,
        SpinSprite::SpaceJump,
        SpinSprite::ScrewAttack,
    ]
    .into_iter()
    .flat_map(|spin| {
        [Suit::First, Suit::VariaOrGravity]
            .into_iter()
            .map(move |suit| Dress { spin, suit })
    });
    let mut compared = 0_usize;
    for dress in dresses {
        for entry in map::MAP {
            if map::id_of(entry.key, dress) != Some(entry.id) {
                continue;
            }
            let asset = assets.clip(entry.key, dress).unwrap();
            let list = content.delays.of(entry.id).unwrap();
            assert_eq!(asset.frames.len(), list.len(), "{:?}", entry.id);
            for (at, frame) in asset.frames.iter().enumerate() {
                let number = AnimationFrame(u16::try_from(at).unwrap());
                let id = entry.id;
                // How long it holds.
                assert_eq!(Some(frame.hold.timer_byte()), list.byte(number));
                assert_eq!(
                    matches!(frame.hold, data::assets::samus::Hold::Shows(_)),
                    matches!(list.at(number), Some(DelayItem::Delay(_))),
                );
                // What it draws.
                let mut engine = state(id.0);
                engine.samus.animation.frame = number;
                draw_body(&mut engine, &content);
                let mut ours = State::default();
                let (x, y) = (0x150_u16 - 0x100, 0x90_u16 - 0x20);
                let origin = (x, y.wrapping_add_signed(frame.origin_y));
                let batch = &mut ours.video.batch;
                if let Some(top) = frame.top {
                    batch.add_spritemap(assets.sheet.sprite(top).unwrap(), origin);
                }
                if frame.chest {
                    let chest = assets.chest;
                    let [y_low, _] = y.wrapping_sub(chest.up).to_le_bytes();
                    batch.add_low_sprite(x.wrapping_sub(chest.left), y_low, chest.attributes);
                }
                if let Some(bottom) = frame.bottom {
                    batch.add_spritemap(assets.sheet.sprite(bottom).unwrap(), origin);
                }
                assert_eq!(ours.video.batch.oam, engine.video.batch.oam, "{id:?} {at}");
                assert_eq!(
                    ours.video.batch.sprites, engine.video.batch.sprites,
                    "{id:?} {at}"
                );
                // What it uploads.
                let engine_tiles = content.samus_tiles.frame(id, number);
                assert_eq!(frame.tiles.is_some(), engine_tiles.is_some(), "{id:?} {at}");
                if let (Some(tiles), Some(engine_tiles)) = (frame.tiles, engine_tiles) {
                    assert_eq!(
                        assets.sheet.upload(tiles.top),
                        Some(&uploaded(&content, engine_tiles.top))
                    );
                    assert_eq!(
                        tiles
                            .bottom
                            .and_then(|bottom| assets.sheet.upload(bottom))
                            .cloned(),
                        engine_tiles.bottom.map(|bottom| uploaded(&content, bottom))
                    );
                }
                compared += 1;
            }
        }
    }
    assert!(compared > 164, "{compared}");
}
