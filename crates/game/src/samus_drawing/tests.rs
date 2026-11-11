use super::*;
use crate::state::samus::ClipStep;
use crate::state::slots::Frames;
use crate::state::video::{OamEntry, SamusTileUploads, SpriteCount};
use crate::state::{FrameCount, State};
use data::assets::samus::import::map::key_of;
use data::assets::samus::key::Worn;
use data::assets::samus::{Chest, Clip, ClipEnd, Frame, Hold, SamusAssets, SpriteId, SpriteSheet};
use data::pose::PoseId;
use data::pose::animation::{AnimationDelays, DelayList, RunPaces};
use data::pose::tiles::{FrameTiles, SamusTiles, TileDefinition, TileDefinitionId};
use data::spritemap::Spritemap;

/// A frame drawing sprite lists `top` and `bottom` with the chest or not, its origin `origin_y`
/// pixels down from her position.
const fn frame(top: u16, bottom: Option<u16>, chest: bool, origin_y: i16) -> Frame {
    Frame {
        hold: Hold::Shows(1),
        top: Some(SpriteId(top)),
        bottom: match bottom {
            Some(bottom) => Some(SpriteId(bottom)),
            None => None,
        },
        chest,
        origin_y,
        tiles: None,
    }
}

/// The clip of the original's pose `id`.
fn clip(id: u16, frames: Vec<Frame>) -> Clip {
    Clip {
        key: key_of(PoseId(id)).unwrap(),
        worn: Worn::Always,
        frames,
        end: ClipEnd::Loop {
            from: data::assets::samus::FrameNumber(0),
        },
    }
}

/// Sprite list 0 is one sprite at (-4, -8), 1 one at (0, 0), 2 two sprites. Pose 0 faces the
/// screen: the chest between its halves, and from its third frame its origin a pixel up. Pose
/// 1 stands facing right, its origin 8 pixels up. Pose 2's second frame has no bottom half.
/// The chest is tile $21, palette 4, priority 3, 7 pixels left of her and 17 up.
fn content() -> Content {
    let sprite = |x: u8, y: u8, tile: u8| [x, 0x00, y, tile, 0x28];
    let map = |entries: &[[u8; 5]]| Spritemap::decode(&entries.concat());
    let assets = SamusAssets {
        sheet: SpriteSheet {
            sprites: vec![
                map(&[sprite(0xFC, 0xF8, 1)]),
                map(&[sprite(0, 0, 2)]),
                map(&[sprite(0, 0, 3), sprite(8, 0, 4)]),
            ],
            uploads: Vec::new(),
        },
        clips: vec![
            Clip {
                worn: Worn::Suit(data::assets::samus::key::Suit::First),
                ..clip(
                    0,
                    vec![
                        frame(0, Some(2), true, 0),
                        frame(1, Some(2), true, 0),
                        frame(2, Some(2), true, -1),
                    ],
                )
            },
            clip(1, vec![frame(0, Some(2), false, -8)]),
            clip(
                2,
                vec![frame(0, Some(2), false, 0), frame(1, None, false, 0)],
            ),
        ],
        chest: Chest {
            left: 7,
            up: 17,
            attributes: data::spritemap::SpriteAttributes::from_word(0x3821),
        },
        ..SamusAssets::default()
    };
    Content {
        samus_assets: assets,
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
        samus_clips: clips(),
        ..Content::default()
    }
}

/// Every pose of hers with a clip of 28 shown steps that starts over, so each step is the
/// original's frame of the same number.
fn clips() -> crate::samus_looks_world::Clips {
    let bytes: Vec<u8> = [[1_u8; 0x1C].as_slice(), &[0xFF]].concat();
    let list = DelayList::decode(&bytes).unwrap();
    let delays = AnimationDelays::new(vec![list; data::pose::POSE_COUNT], DelayList::default());
    crate::samus_looks_world::Clips::import(&delays, &RunPaces::default()).unwrap()
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
fn the_frame_picks_the_sprites_and_a_frame_may_have_no_bottom_half() {
    let mut rolling = state(2);
    rolling.samus.animation.step = ClipStep(1);
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
    later.samus.animation.step = ClipStep(2);
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
    rolling.samus.animation.step = ClipStep(1);
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
