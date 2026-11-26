//! Camera exposures of the room's front layer.

use super::passage::Room;
use crate::exact::tilemap::{commit_exposures, expose};
use crate::intent::{BlockLine, BlockLook, LayerExposure, LevelLayer};
use crate::state::render::BlockPoint;
use crate::state::video::{LevelUploads, TilemapEntry, VramWord};
use data::assets::tilesets::Tilesets;
use data::level::Tile;
use types::{Pair, Parts, Position, Px};

const BLOCK_PIXELS: u16 = 16;

impl super::State {
    pub(super) fn apply_passage_view(
        &mut self,
        content: &super::Content,
        view: Position,
        seen: Pair<Px>,
        step: Option<Position>,
    ) {
        let before = self.camera.0.position();
        // The slide's raster interrupts request the upload independently of the main loop.
        if let Some(step) = step
            && view.x == before.x + step.x
            && view.y == before.y + step.y
        {
            self.prepare_bg1(content, view);
            commit_exposures(&mut self.room_video.video);
        }
        self.camera.0 = super::camera::passed(self.camera.0, view, seen);
    }

    pub(super) fn prepare_bg1(&mut self, content: &super::Content, view: Position) {
        prepare(
            &mut self.room_video.video.batch.level,
            &self.passage.room,
            &content.rooms.tilesets,
            self.camera.0.position(),
            view,
        );
    }
}

/// Prepare the columns and rows that enter the tiled view, where BG1 follows layer one.
fn prepare(
    uploads: &mut LevelUploads,
    room: &Room,
    tilesets: &Tilesets,
    previous: Position,
    current: Position,
) {
    if room.entry.view.affine {
        return;
    }
    let Some(tileset) = room.tileset else {
        return;
    };
    let at = |view: Position| BlockPoint {
        x: view.x.pixel().0 / BLOCK_PIXELS,
        y: view.y.pixel().0 / BLOCK_PIXELS,
    };
    let before = at(previous);
    let now = at(current);
    let reads = [
        (
            before.x != now.x,
            BlockLine::Column,
            BlockPoint {
                x: if before.x < now.x {
                    now.x.wrapping_add(16)
                } else {
                    now.x
                },
                y: now.y,
            },
        ),
        (
            before.y != now.y,
            BlockLine::Row,
            BlockPoint {
                x: now.x,
                y: now.y.wrapping_add(if before.y < now.y { 15 } else { 1 }),
            },
        ),
    ];
    for (crossed, line, start) in reads {
        if !crossed {
            continue;
        }
        let width = usize::from(room.entry.width);
        let first = usize::from(start.y)
            .wrapping_mul(width)
            .wrapping_add(usize::from(start.x));
        let stride = match line {
            BlockLine::Column => width,
            BlockLine::Row => 1,
        };
        let looks = (0..usize::from(line.blocks()))
            .map(|offset| {
                let index = first.wrapping_add(offset.wrapping_mul(stride));
                let tile = room.entry.cells.get(index)?.tile;
                let look = tilesets.block(tileset, tile.number.0)?;
                Some(oriented(tile, look))
            })
            .collect::<Option<Vec<_>>>();
        if let Some(looks) = looks {
            expose(
                uploads,
                &LayerExposure {
                    layer: LevelLayer::One,
                    line,
                    level: start,
                    background: start,
                },
                VramWord(0),
                looks,
            );
        }
    }
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
