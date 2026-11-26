//! Camera exposures of the room's front layer.

use super::passage::Room;
use crate::exact::tilemap::{commit_exposures, expose};
use crate::intent::{BlockLine, BlockLook, LayerExposure, LevelLayer};
use crate::state::render::BlockPoint;
use crate::state::video::{Scroll, TilemapEntry, VramWord};
use data::assets::tilesets::Tilesets;
use data::assets::world::DoorRule;
use data::level::Tile;
use types::{Pair, Parts, Position, Px};
use world_doors::{Doors, Passage, Phase};

const BLOCK_PIXEL_SHIFT: u32 = 4;

impl super::State {
    pub(super) fn apply_passage_view(
        &mut self,
        content: &super::Content,
        view: Position,
        seen: Pair<Px>,
        step: Option<Position>,
    ) {
        let before = self.camera.0.position();
        if let Some(step) = step {
            let setting_off = matches!(
                self.passage.doors,
                Doors::Passing(Passage {
                    phase: Phase::BeforeTheSlide(_),
                    ..
                })
            );
            if setting_off && step.x.0 != 0 {
                self.start_horizontal_bg1(content, before, view, step);
            } else if view.x == before.x + step.x && view.y == before.y + step.y {
                self.prepare_bg1(content, view);
            }
            // The slide's raster interrupts request the upload independently of the main loop.
            commit_exposures(&mut self.room_video.video);
        }
        self.camera.0 = super::camera::passed(self.camera.0, view, seen);
    }

    fn start_horizontal_bg1(
        &mut self,
        content: &super::Content,
        before: Position,
        view: Position,
        step: Position,
    ) {
        // The incoming room starts a screen away. Keep the outgoing background's place on
        // the ring while the camera is re-anchored to the incoming room.
        let starting = Pair {
            x: view.x - step.x,
            y: view.y - step.y,
        };
        if step.x.negative()
            && self.station.stage == boss_ceres_ridley::Stage::Escape
            && self.passage.doorway.is_some_and(|doorway| {
                doorway.through == data::assets::world::doorways::FALLING_TILE_TO_ELEVATOR
            })
        {
            self.room_video.start_shaft_bg2_scroll(starting);
        }
        let offset = &mut self.room_video.bg1_offset;
        offset.x = offset
            .x
            .wrapping_add(before.x.pixel().0.wrapping_sub(starting.x.pixel().0));
        offset.y = offset
            .y
            .wrapping_add(before.y.pixel().0.wrapping_sub(starting.y.pixel().0));
        let ahead = if step.x.negative() { 0 } else { 16 };
        let layer = block_at(view, Scroll::default());
        let background = block_at(view, *offset);
        self.expose_bg1(
            content,
            BlockLine::Column,
            across(layer, ahead),
            across(background, ahead),
        );
    }

    pub(super) fn prepare_bg1(&mut self, content: &super::Content, view: Position) {
        let before = block_at(self.camera.0.position(), Scroll::default());
        let layer = block_at(view, Scroll::default());
        let background = block_at(view, self.room_video.bg1_offset);
        if before.x != layer.x {
            let ahead = if layer.x.wrapping_sub(before.x).cast_signed() > 0 {
                16
            } else {
                0
            };
            self.expose_bg1(
                content,
                BlockLine::Column,
                across(layer, ahead),
                across(background, ahead),
            );
        }
        if before.y != layer.y {
            let ahead = if layer.y.wrapping_sub(before.y).cast_signed() > 0 {
                15
            } else {
                1
            };
            self.expose_bg1(
                content,
                BlockLine::Row,
                below(layer, ahead),
                below(background, ahead),
            );
        }
    }

    fn expose_bg1(
        &mut self,
        content: &super::Content,
        line: BlockLine,
        level: BlockPoint,
        background: BlockPoint,
    ) {
        let room = &self.passage.room;
        if room.entry.view.affine && !self.tiled_shaft_exit(content) {
            return;
        }
        let Some(looks) = line_looks(room, &content.rooms.tilesets, line, level) else {
            return;
        };
        expose(
            &mut self.room_video.video.batch.level,
            &LayerExposure {
                layer: LevelLayer::One,
                line,
                level,
                background,
            },
            VramWord(0),
            looks,
        );
    }

    fn tiled_shaft_exit(&self, content: &super::Content) -> bool {
        // The destination's tiled scroll starts before the room view's fetch completes.
        matches!(
            self.passage.doors,
            Doors::Passing(Passage {
                phase: Phase::BeforeTheSlide(_) | Phase::Arriving { .. },
                ..
            })
        ) && self
            .passage
            .doorway
            .and_then(|doorway| content.rooms.world.doorway(doorway.through))
            .is_some_and(|doorway| doorway.rule == DoorRule::FromCeresElevatorShaft)
    }
}

fn block_at(view: Position, offset: Scroll) -> BlockPoint {
    let block = |pixel: u16, offset: u16| {
        (pixel.wrapping_add(offset).cast_signed() >> BLOCK_PIXEL_SHIFT).cast_unsigned()
    };
    BlockPoint {
        x: block(view.x.pixel().0, offset.x),
        y: block(view.y.pixel().0, offset.y),
    }
}

const fn across(at: BlockPoint, columns: u16) -> BlockPoint {
    BlockPoint {
        x: at.x.wrapping_add(columns),
        y: at.y,
    }
}

const fn below(at: BlockPoint, rows: u16) -> BlockPoint {
    BlockPoint {
        x: at.x,
        y: at.y.wrapping_add(rows),
    }
}

fn line_looks(
    room: &Room,
    tilesets: &Tilesets,
    line: BlockLine,
    start: BlockPoint,
) -> Option<Vec<BlockLook>> {
    let tileset = room.tileset?;
    let width = usize::from(room.entry.width);
    let first = usize::from(start.y)
        .wrapping_mul(width)
        .wrapping_add(usize::from(start.x));
    let stride = match line {
        BlockLine::Column => width,
        BlockLine::Row => 1,
    };
    (0..usize::from(line.blocks()))
        .map(|offset| {
            let index = first.wrapping_add(offset.wrapping_mul(stride));
            let tile = room.entry.cells.get(index)?.tile;
            let look = tilesets.block(tileset, tile.number.0)?;
            Some(oriented(tile, look))
        })
        .collect()
}

fn oriented(tile: Tile, look: data::assets::art::BlockLook) -> BlockLook {
    let [mut tl, mut tr, mut bl, mut br] = look.cells().map(|mut cell| {
        cell.flip_x ^= tile.x_flip;
        cell.flip_y ^= tile.y_flip;
        TilemapEntry::from(cell)
    });
    if tile.x_flip {
        core::mem::swap(&mut tl, &mut tr);
        core::mem::swap(&mut bl, &mut br);
    }
    if tile.y_flip {
        core::mem::swap(&mut tl, &mut bl);
        core::mem::swap(&mut tr, &mut br);
    }
    BlockLook {
        top_left: tl,
        top_right: tr,
        bottom_left: bl,
        bottom_right: br,
    }
}
