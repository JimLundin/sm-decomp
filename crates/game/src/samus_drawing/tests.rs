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
    state.samus.position.x.pixels = 0x150;
    state.samus.position.y.pixels = 0x90;
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
