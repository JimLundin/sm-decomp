use super::*;
use crate::state::slots::Frames;
use crate::state::video::{OamEntry, SpriteCount};
use crate::state::{FrameCount, State};
use data::pose::Poses;
use data::pose::spritemap::{SamusSpritemaps, SpritemapIndex};
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
