//! Drawing the enemies: the main enemy routine queues each one it processed by its layer,
//! and the Draw phase draws the layers from 0 up, Samus with layer 3.
//!
//! Bank $A0 log (<https://patrickjohnston.org/bank/A0>): the main enemy routine (`$A0:8FD4`)
//! adds an enemy to its layer's drawing queue (`$A0:9423`) after processing it, if it is in
//! the processing window (`$A0:ADE7`) and neither invisible nor deleted. Draw Samus,
//! projectiles, enemies and enemy projectiles (`$A0:884D`) runs the layers 0 to 7: at layer 3
//! it draws Samus (`$90:EB35`) before the layer's queue, and each queue's enemies are
//! written to OAM in order (`$A0:944A`), then the queue is empty.

use std::marker::PhantomData;

use clean_types::Pair;
use data::enemy::program::EnemyPrograms;
use data::spritemap::SpriteAttributes;

use crate::intent::{SpriteDraw, on_screen};
use crate::pose::Content;
use crate::state::State;
use crate::state::room::{
    EnemyCommon, EnemyDrawing, EnemyExtraProperty, EnemyProperty, GraphicsDrawnHook, PaletteIndex,
    TilesIndex,
};
use crate::state::slots::{Frames, SlotIndex};

/// The layer Samus draws with (`$A0:8871`).
const SAMUS_LAYER: usize = 3;

/// The palette a flashing enemy shows on its bright frames: palette 0 (`$A0:94A4`).
const FLASH_PALETTE: PaletteIndex = PaletteIndex(0);
/// The palette a frozen enemy shows: palette 6 (`$A0:94B8`).
const FROZEN_PALETTE: PaletteIndex = PaletteIndex(0x0C00);
/// Frozen frames left from which a frozen enemy shows the frozen palette on every frame;
/// under it, only on frames with the timer's bit 1 set (`$A0:94AE`).
const FROZEN_STEADY: u16 = 0x5A;

/// Queue slot `index` to draw if the main enemy routine draws it (`$A0:90F0` to `$A0:9115`):
/// in the processing window unless its spritemaps are extended, and neither invisible nor
/// deleted.
// open: the enemy drawing disable flag (`$17A2` bit 0) joins with the species that set it;
// a layer past the eighth, whose queue address the original reads past its table, is
// dropped.
pub fn queue(drawing: &mut EnemyDrawing, index: usize, common: &EnemyCommon, camera: (u16, u16)) {
    let (camera_x, camera_y) = camera;
    let in_window = common
        .extra_properties
        .contains(EnemyExtraProperty::ExtendedSpritemap)
        || super::on_screen(common, camera_x, camera_y);
    let hidden = common.properties.contains(EnemyProperty::Invisible)
        || common.properties.contains(EnemyProperty::Deleted);
    if !in_window || hidden {
        return;
    }
    let (Ok(slot), Some(queue)) = (
        u8::try_from(index),
        drawing.queues.get_mut(usize::from(common.layer)),
    ) else {
        return;
    };
    queue.push(SlotIndex(slot, PhantomData));
}

/// The Draw phase (`$A0:884D`): each layer's queued enemies, Samus with layer 3, then the
/// enemy graphics drawn hook (`$A0:88BD`).
// open: the sprite objects (`$B4:BD32`), bombs and projectile explosions (`$93:834D`) and
// the high-priority enemy projectiles (`$86:8390`) before the layers, Samus's projectiles
// with her and the low-priority enemy projectiles at layer 6 (`$86:83B2`) join with their
// domains.
pub fn draw(state: &mut State, content: &Content) {
    let queues = std::mem::take(&mut state.room.drawing.queues);
    let camera = (state.room.camera.x.pixels, state.room.camera.y.pixels);
    let count = state.room.drawing.routine_count;
    for (layer, queue) in queues.iter().enumerate() {
        if layer == SAMUS_LAYER {
            crate::samus_drawing::draw(state, content);
        }
        for slot in queue {
            if let Some(enemy) = state.room.enemies.slots.get_mut(usize::from(slot.0)) {
                let drawn = draw_enemy(&mut enemy.common, camera, count, &content.enemy_programs);
                state.video.batch.draw_sprites(&drawn);
            }
        }
    }
    graphics_drawn(state, content);
}

/// The enemy graphics drawn hook (`$A0:88C4`).
fn graphics_drawn(state: &mut State, content: &Content) {
    match state.room.drawing.hook {
        GraphicsDrawnHook::None => {}
        GraphicsDrawnHook::FaceBlockGlow => super::face_block::glow(
            &mut state.enemy_palette_cycle,
            &mut state.video.batch.cgram,
            &content.face_block_glow,
        ),
    }
}

/// The palette it shows this frame (`$A0:9497` to `$A0:94BD`).
fn palette(common: &EnemyCommon, routine_count: u16) -> PaletteIndex {
    let frozen = common.frozen.0;
    if common.flash != Frames(0) && routine_count & 2 != 0 {
        FLASH_PALETTE
    } else if frozen != 0 && (frozen >= FROZEN_STEADY || frozen & 2 != 0) {
        FROZEN_PALETTE
    } else {
        common.palette
    }
}

/// What an enemy's spritemap adds to each sprite's tile and attributes word.
///
/// Bank `$81` log, add spritemap to OAM with base tile number `$81:8AB8`: the tiles index is
/// added to the word and the palette bits are combined into the sum by OR, so a carry out of
/// the tile number reaches the attributes, as in the original.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SpriteBase {
    /// Added to the word.
    tiles: TilesIndex,
    /// Combined into the sum by OR.
    palette: PaletteIndex,
}

impl SpriteBase {
    /// `attributes` with the base applied.
    fn apply(self, attributes: SpriteAttributes) -> SpriteAttributes {
        SpriteAttributes::from_word(attributes.word().wrapping_add(self.tiles.0) | self.palette.0)
    }
}

/// One enemy's spritemap (`$A0:944A`): from its position less the camera's, a pixel left or
/// right while it shakes, which counts its shake down, with its palette and tiles applied.
// open: the graphical offsets (`$7E:7010`, `$7E:7012`) and the extended spritemaps
// (`$A0:94F4`) join with the species that use them; until then the offsets are zero and an
// extended enemy draws nothing.
fn draw_enemy(
    common: &mut EnemyCommon,
    camera: (u16, u16),
    routine_count: u16,
    programs: &EnemyPrograms,
) -> SpriteDraw {
    let (camera_x, camera_y) = camera;
    let mut at = Pair {
        x: on_screen(common.position.x.pixels, camera_x),
        y: on_screen(common.position.y.pixels, camera_y),
    };
    if common.shake != Frames(0) {
        at.x = if common.frame_counter & 2 == 0 {
            at.x.wrapping_add(1)
        } else {
            at.x.wrapping_sub(1)
        };
        common.shake = Frames(common.shake.0.wrapping_sub(1));
    }
    let base = SpriteBase {
        tiles: common.tiles,
        palette: palette(common, routine_count),
    };
    let mut drawn = SpriteDraw::default();
    if common
        .extra_properties
        .contains(EnemyExtraProperty::ExtendedSpritemap)
    {
        return drawn;
    }
    if let Some(map) = programs.sprites(common.spritemap) {
        drawn.place(map, at);
        for piece in &mut drawn.pieces {
            piece.look = base.apply(piece.look);
        }
    }
    drawn
}

#[cfg(test)]
mod tests;
