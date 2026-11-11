use std::marker::PhantomData;

use data::enemy::program::{EnemyPrograms, Spritemap, SpritemapId};
use data::rom::{Lorom, RomAddress};
use data::spritemap::SpriteAttributes;

use super::{SpriteBase, draw, palette, queue};
use crate::pose::Content;
use crate::state::State;
use crate::state::room::{
    EnemyBody, EnemyCommon, EnemyDrawing, EnemyExtraProperty, EnemyProperty, FaceBlock,
    PaletteIndex, TilesIndex,
};
use crate::state::slots::{Frames, Occupancy, SlotIndex};
use crate::state::video::{OamEntry, SpriteCount};

/// A file offset in a `LoROM` image.
fn file(bank: u8, offset: u16) -> usize {
    (usize::from(bank & 0x7f) << 15) | usize::from(offset & 0x7fff)
}

/// Programs showing two spritemaps in bank $A8: $9100, one sprite of tile $10 at (-4, -8),
/// and $9110, two sprites of tile $20 at (0, 0) and (8, 0).
fn programs() -> EnemyPrograms {
    let mut image = vec![0; 0x15_0000];
    let mut put = |offset: u16, bytes: &[u8]| {
        image[file(0xa8, offset)..][..bytes.len()].copy_from_slice(bytes);
    };
    // Show $9100 for 1, show $9110 for 1, sleep (`$A0:812F`).
    put(0x9000, &[1, 0, 0x00, 0x91, 1, 0, 0x10, 0x91, 0x2f, 0x81]);
    put(0x9100, &[1, 0, 0xFC, 0x01, 0xF8, 0x10, 0x20]);
    put(0x9110, &[2, 0, 0, 0, 0, 0x20, 0x20, 8, 0, 0, 0x20, 0x20]);
    EnemyPrograms::load(Lorom::new(&image), &[RomAddress::new(0xa8, 0x9000)], &[]).unwrap()
}

fn id(programs: &EnemyPrograms, offset: u16) -> SpritemapId {
    programs
        .spritemap_id(Spritemap(RomAddress::new(0xa8, offset)))
        .unwrap()
}

/// An enemy at (`x`, `y`) in the room on `layer`, visible.
fn enemy(x: u16, y: u16, layer: u16) -> EnemyCommon {
    let mut common = EnemyCommon {
        layer,
        radius_x: 8,
        radius_y: 8,
        ..EnemyCommon::default()
    };
    common.position.x.pixels = x;
    common.position.y.pixels = y;
    common
}

fn slots(queue: &[SlotIndex<crate::state::room::Enemy>]) -> Vec<u8> {
    queue.iter().map(|slot| slot.0).collect()
}

#[test]
fn only_a_visible_enemy_in_the_window_is_queued_on_its_layer() {
    let mut drawing = EnemyDrawing::default();
    queue(&mut drawing, 4, &enemy(0x80, 0x80, 2), (0, 0));
    // Far right of the camera's window.
    queue(&mut drawing, 5, &enemy(0x300, 0x80, 2), (0, 0));
    let mut invisible = enemy(0x80, 0x80, 2);
    invisible.properties.insert(EnemyProperty::Invisible);
    queue(&mut drawing, 6, &invisible, (0, 0));
    let mut deleted = enemy(0x80, 0x80, 2);
    deleted.properties.insert(EnemyProperty::Deleted);
    queue(&mut drawing, 7, &deleted, (0, 0));
    // Extended spritemaps skip the window test.
    let mut extended = enemy(0x300, 0x80, 5);
    extended
        .extra_properties
        .insert(EnemyExtraProperty::ExtendedSpritemap);
    queue(&mut drawing, 8, &extended, (0, 0));
    // No ninth layer.
    queue(&mut drawing, 9, &enemy(0x80, 0x80, 8), (0, 0));
    let queued: Vec<Vec<u8>> = drawing.queues.iter().map(|queue| slots(queue)).collect();
    assert_eq!(
        queued,
        [
            vec![],
            vec![],
            vec![4],
            vec![],
            vec![],
            vec![8],
            vec![],
            vec![]
        ]
    );
}

#[test]
fn the_layers_draw_from_zero_up_and_their_queues_empty() {
    let programs = programs();
    let mut state = State::default();
    state.room.camera.x.pixels = 0x100;
    state.room.camera.y.pixels = 0x20;
    let maps = [(0x9110, 5), (0x9100, 1)];
    for (slot, (offset, layer)) in state.room.enemies.slots.iter_mut().zip(maps) {
        slot.common = EnemyCommon {
            spritemap: id(&programs, offset),
            ..enemy(0x180, 0x90, layer)
        };
        slot.occupancy = Occupancy::Live(EnemyBody::FaceBlock(FaceBlock::default()));
    }
    // Slot 0 on layer 5 queued, then slot 1 on layer 1.
    state.room.drawing.queues[5].push(SlotIndex(0, PhantomData));
    state.room.drawing.queues[1].push(SlotIndex(1, PhantomData));
    let content = Content {
        enemy_programs: programs,
        ..Content::default()
    };
    draw(&mut state, &content);
    // Layer 1's sprite first: ($80 - 4, $70 - 8); then layer 5's two at ($80, $70), ($88, $70).
    let written: Vec<OamEntry> = state.video.batch.oam.low.iter().take(3).copied().collect();
    let sprite = |x, y, tile| OamEntry {
        x,
        y,
        tile,
        attributes: 0x20,
    };
    assert_eq!(
        written,
        [
            sprite(0x7C, 0x68, 0x10),
            sprite(0x80, 0x70, 0x20),
            sprite(0x88, 0x70, 0x20)
        ]
    );
    assert_eq!(state.video.batch.sprites, SpriteCount(3));
    assert!(state.room.drawing.queues.iter().all(Vec::is_empty));
}

#[test]
fn a_shaking_enemy_moves_a_pixel_and_counts_down_with_its_tiles_and_palette() {
    let programs = programs();
    let mut state = State::default();
    let common = EnemyCommon {
        spritemap: id(&programs, 0x9100),
        shake: Frames(3),
        frame_counter: 2,
        tiles: TilesIndex(0x0100),
        palette: PaletteIndex(0x0200),
        ..enemy(0x40, 0x40, 0)
    };
    if let Some(slot) = state.room.enemies.slots.first_mut() {
        slot.common = common;
    }
    state.room.drawing.queues[0].push(SlotIndex(0, PhantomData));
    let content = Content {
        enemy_programs: programs,
        ..Content::default()
    };
    draw(&mut state, &content);
    // Bit 1 of its frame counter set: a pixel left. Tile $10 + $100, palette 1.
    assert_eq!(
        state.video.batch.oam.low[0],
        OamEntry {
            x: 0x3B,
            y: 0x38,
            tile: 0x10,
            attributes: 0x23,
        }
    );
    assert_eq!(state.room.enemies.slots[0].common.shake, Frames(2));
}

#[test]
fn flashing_and_frozen_enemies_change_palette() {
    let normal = EnemyCommon {
        palette: PaletteIndex(0x0400),
        ..EnemyCommon::default()
    };
    assert_eq!(palette(&normal, 2), PaletteIndex(0x0400));
    let flashing = EnemyCommon {
        flash: Frames(5),
        ..normal
    };
    // The flash shows on the routine count's bit 1.
    assert_eq!(palette(&flashing, 2), PaletteIndex(0));
    assert_eq!(palette(&flashing, 1), PaletteIndex(0x0400));
    let frozen = |frames| EnemyCommon {
        frozen: Frames(frames),
        ..normal
    };
    assert_eq!(palette(&frozen(0x5A), 0), PaletteIndex(0x0C00));
    // Under $5A frames, only on the timer's bit 1.
    assert_eq!(palette(&frozen(0x59), 0), PaletteIndex(0x0400));
    assert_eq!(palette(&frozen(0x52), 0), PaletteIndex(0x0C00));
}

#[test]
fn an_enemy_base_adds_its_tiles_and_ors_its_palette() {
    let base = SpriteBase {
        tiles: TilesIndex(0x0100),
        palette: PaletteIndex(0x0A00),
    };
    let look = |tile, palette| SpriteAttributes {
        tile,
        palette,
        priority: 2,
        flip_x: false,
        flip_y: false,
    };
    // Tile $110 + $100 is $210: the carry raises the palette from 0 to 1, and 1 | 5 is 5.
    assert_eq!(base.apply(look(0x110, 0)), look(0x010, 5));
    // Tile $1FF + $100 is $2FF: the carry raises the palette from 2 to 3, and 3 | 5 is 7.
    assert_eq!(base.apply(look(0x1FF, 2)), look(0x0FF, 7));
}
