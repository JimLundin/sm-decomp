use data::level::ScrollArea;
use data::plm::{Extension, PlmEntry, PlmKind, ScrollChange};
use data::rom::RomAddress;
use types::{Heading, Point, Way};

use super::fixture::{CRUMBLE, SHOT, content};
use super::*;
use crate::plm::{process_unseen, spawn_population};
use crate::state::CollectedItems;
use crate::state::room::{BlockColumns, Tile};
use crate::state::slots::Frames as Timer;

/// A two-screen-wide, one-screen-high room of solid blocks, its screens red.
fn room() -> RoomScoped {
    let solid = Block {
        kind: BlockKind::Solid(Bts(0)),
        tile: Tile {
            number: TileNumber(0x55),
            x_flip: false,
            y_flip: false,
        },
    };
    RoomScoped {
        blocks: vec![solid; 512],
        width: BlockColumns(32),
        scroll_areas: vec![ScrollArea::Red; 2],
        ..RoomScoped::default()
    }
}

const fn entry(kind: PlmKind, x: u8, y: u8) -> PlmEntry {
    PlmEntry { kind, x, y }
}

/// A scroll cell at (3, 2) turning screen 1 green, the upwards stand-in above it and a
/// rightwards one beside it.
fn population() -> PlmPopulation {
    PlmPopulation {
        entries: vec![
            entry(PlmKind::Scroll(ScrollChangesId(0)), 3, 2),
            entry(PlmKind::Extension(Extension::Up), 3, 1),
            entry(PlmKind::Extension(Extension::Right), 4, 2),
        ],
        scroll_changes: vec![(
            RomAddress::new(0x8f, 0x9f0b),
            vec![ScrollChange {
                screen: 1,
                area: ScrollArea::Green,
            }],
        )],
    }
}

const TRIGGER: BlockIndex = BlockIndex(2 * 32 + 3);
const TRIGGER_CELL: Cell = Cell { x: 3, y: 2 };

fn loaded(content: &BlockContent) -> RoomScoped {
    let mut room = room();
    spawn_population(&mut room, content, &population()).unwrap();
    room
}

fn reads<'a>(content: &'a BlockContent, population: &'a PlmPopulation) -> Reads<'a> {
    Reads {
        content,
        population,
    }
}

fn scroll(room: &RoomScoped, slot: usize) -> Option<ScrollPlm> {
    if let Occupancy::Live(PlmBody::Scroll(scroll)) = room.plms.slots[slot].occupancy {
        Some(scroll)
    } else {
        None
    }
}

fn pass(room: &mut RoomScoped, content: &BlockContent) {
    process_unseen(room, &CollectedItems::default(), content, &population()).unwrap();
}

#[test]
fn the_rooms_start_writes_each_placements_cell_keeping_its_look() {
    let content = content();
    let room = loaded(&content);
    let kind = |index: u16| room.blocks[usize::from(index)].kind;
    assert_eq!(kind(TRIGGER.0), SCROLL_CELL);
    assert_eq!(kind(32 + 3), BlockKind::VerticalExtension(Offset(1)));
    assert_eq!(kind(2 * 32 + 4), BlockKind::HorizontalExtension(Offset(-1)));
    assert!(
        room.blocks
            .iter()
            .all(|block| block.tile.number == TileNumber(0x55))
    );
    let written = room
        .blocks
        .iter()
        .filter(|block| block.kind != BlockKind::Solid(Bts(0)))
        .count();
    assert_eq!(written, 3);
}

/// The scroll cell takes 39; each stand-in passes through 38, leaving its cell and timer there.
#[test]
fn the_scroll_cell_keeps_its_place_and_the_stand_ins_leave_theirs() {
    let content = content();
    let room = loaded(&content);
    let slots = &room.plms.slots;
    assert_eq!(
        scroll(&room, 39),
        Some(ScrollPlm {
            changes: ScrollChangesId(0),
            triggered: false,
            step: ScrollStep::Sleep,
        })
    );
    assert_eq!(slots[39].common.block, TRIGGER);
    assert_eq!(slots[39].common.cursor.delay, Timer(1));
    assert_eq!(slots[38].occupancy, Occupancy::Free);
    assert_eq!(slots[38].common.block, BlockIndex(2 * 32 + 4));
    assert_eq!(slots[38].generation.0, 2);
    assert!(
        slots[..39]
            .iter()
            .all(|slot| slot.occupancy == Occupancy::Free)
    );
}

#[test]
fn a_sleeping_scroll_cells_timer_wraps_and_nothing_changes() {
    let content = content();
    let mut room = loaded(&content);
    let before = room.clone();
    for _ in 0..3 {
        pass(&mut room, &content);
    }
    assert_eq!(room.scroll_areas, before.scroll_areas);
    assert_eq!(room.blocks, before.blocks);
    assert_eq!(room.plms.slots[39].common.cursor.delay, Timer(0xfffe));
}

/// `ScrollCells`: a touch takes the highest free place until the run and touches the scroll
/// reaction once; the run sets its screen and frees the holds.
#[test]
fn touching_the_scroll_cell_changes_its_screen_in_the_next_run() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    pass(&mut room, &content);
    let down = Touch {
        cell: TRIGGER_CELL,
        way: Way::Move(Heading::Down),
    };
    touches(&mut room, reads(&content, &population), &[down]).unwrap();
    let hold = &room.plms.slots[38];
    assert_eq!(hold.occupancy, Occupancy::Live(PlmBody::ScrollTouch));
    assert_eq!(hold.common.block, BlockIndex(0));
    let touched = scroll(&room, 39).unwrap();
    assert!(touched.triggered);
    assert_eq!(touched.step, ScrollStep::Update);
    assert_eq!(room.plms.slots[39].common.cursor.delay, Timer(1));
    // At her centre it is touched again: another hold, the reaction as it was.
    inside(
        &mut room,
        reads(&content, &population),
        &[(TRIGGER, Point::Centre)],
    )
    .unwrap();
    assert_eq!(
        room.plms.slots[37].occupancy,
        Occupancy::Live(PlmBody::ScrollTouch)
    );
    assert_eq!(scroll(&room, 39), Some(touched));
    pass(&mut room, &content);
    assert_eq!(room.scroll_areas, [ScrollArea::Red, ScrollArea::Green]);
    assert_eq!(
        scroll(&room, 39),
        Some(ScrollPlm {
            triggered: false,
            step: ScrollStep::Sleep,
            ..touched
        })
    );
    assert_eq!(room.plms.slots[39].common.cursor.delay, Timer(0));
    assert_eq!(room.plms.slots[38].occupancy, Occupancy::Free);
    assert_eq!(room.plms.slots[37].occupancy, Occupancy::Free);
}

#[test]
fn her_bottom_in_the_scroll_cell_touches_nothing() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    let before = room.clone();
    inside(
        &mut room,
        reads(&content, &population),
        &[(TRIGGER, Point::Bottom), (TRIGGER, Point::Top)],
    )
    .unwrap();
    assert_eq!(room, before);
}

/// A scroll cell no scroll reaction holds a place for is the original's hang, which the
/// room's loading refuses.
#[test]
fn a_scroll_cell_with_no_scroll_reaction_is_refused() {
    let content = content();
    let population = population();
    let mut room = room();
    room.blocks[7].kind = SCROLL_CELL;
    let down = Touch {
        cell: Cell { x: 7, y: 0 },
        way: Way::Move(Heading::Down),
    };
    assert_eq!(
        touches(&mut room, reads(&content, &population), &[down]),
        Err(BlocksFault::NoScrollReaction(BlockIndex(7)))
    );
}

#[test]
fn with_every_place_taken_a_touch_takes_none() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    for slot in &mut room.plms.slots {
        if slot.occupancy == Occupancy::Free {
            slot.occupancy = Occupancy::Live(PlmBody::ScrollTouch);
        }
    }
    let before = room.clone();
    let down = Touch {
        cell: TRIGGER_CELL,
        way: Way::Move(Heading::Down),
    };
    touches(&mut room, reads(&content, &population), &[down]).unwrap();
    assert_eq!(room, before);
}

/// A shot block at (10, 4).
const TARGET: BlockIndex = BlockIndex(4 * 32 + 10);

fn with_shot_block(content: &BlockContent) -> RoomScoped {
    let mut room = loaded(content);
    room.blocks[usize::from(TARGET.0)].kind = SHOT;
    room
}

/// `ShotBlocks`: the beam starts it in the highest free place, solid with its own look; its
/// script's shows run from that frame's pass, and its end frees the place, the cell air with
/// its BTS kept.
#[test]
fn a_shot_block_breaks_through_its_script() {
    let content = content();
    let population = population();
    let mut room = with_shot_block(&content);
    hit(&mut room, reads(&content, &population), TARGET, HitBy::Beam).unwrap();
    let slot = &room.plms.slots[38];
    assert_eq!(
        slot.occupancy,
        Occupancy::Live(PlmBody::Reaction(ReactionPlace {
            reaction: ReactionId(0),
            script: ReactionScript::Broken,
            next: 0,
            grows_back_as: None,
        }))
    );
    assert_eq!(slot.common.block, TARGET);
    assert_eq!(slot.common.cursor.delay, Timer(1));
    let block = room.blocks[usize::from(TARGET.0)];
    assert_eq!(block.kind, BlockKind::Solid(Bts(4)));
    assert_eq!(block.tile.number, TileNumber(0x55));
    let mut passes = Vec::new();
    for _ in 0..4 {
        pass(&mut room, &content);
        let slot = &room.plms.slots[38];
        let next = match slot.occupancy {
            Occupancy::Live(PlmBody::Reaction(running)) => Some(running.next),
            Occupancy::Live(_) | Occupancy::Free | Occupancy::Deleting(_) => None,
        };
        let block = room.blocks[usize::from(TARGET.0)];
        passes.push((next, slot.common.cursor.delay.0, block.tile.number.0));
    }
    assert_eq!(
        passes,
        [
            (Some(2), 2, 0x12),
            (Some(2), 1, 0x12),
            (Some(3), 1, 0x13),
            (None, 0, 0x13)
        ]
    );
    assert_eq!(
        room.blocks[usize::from(TARGET.0)].kind,
        BlockKind::Air(Bts(4))
    );
}

/// `Regrowing`: a crumble block landed on turns solid with its whole look, remembers it was a
/// block, and grows back as one with the whole look.
#[test]
fn a_crumble_block_grows_back_as_the_block_it_was() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    room.blocks[usize::from(TARGET.0)].kind = CRUMBLE;
    let down = Touch {
        cell: Cell { x: 10, y: 4 },
        way: Way::Move(Heading::Down),
    };
    touches(&mut room, reads(&content, &population), &[down]).unwrap();
    let block = room.blocks[usize::from(TARGET.0)];
    assert_eq!(block.kind, BlockKind::Solid(Bts(1)));
    assert_eq!(block.tile.number, TileNumber(0x20));
    let grows = |room: &RoomScoped| match room.plms.slots[38].occupancy {
        Occupancy::Live(PlmBody::Reaction(running)) => running.grows_back_as,
        Occupancy::Live(_) | Occupancy::Free | Occupancy::Deleting(_) => None,
    };
    assert_eq!(grows(&room), Some(GrowsBackAs::Block));
    let mut kinds = Vec::new();
    for _ in 0..6 {
        pass(&mut room, &content);
        let block = room.blocks[usize::from(TARGET.0)];
        kinds.push((block.kind, block.tile.number.0));
    }
    let solid = (BlockKind::Solid(Bts(1)), 0x20);
    let air = (BlockKind::Air(Bts(1)), 0x21);
    assert_eq!(
        kinds,
        [solid, solid, air, air, (CRUMBLE, 0x20), (CRUMBLE, 0x20)]
    );
    assert_eq!(room.plms.slots[38].occupancy, Occupancy::Free);
}

/// Every request names a cell of the room: one past its last row stops the call, and the room
/// is left as it was.
#[test]
fn a_request_past_the_rooms_last_row_is_refused() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    let before = room.clone();
    let past = Cell { x: 3, y: 16 };
    let down = Touch {
        cell: past,
        way: Way::Move(Heading::Down),
    };
    assert_eq!(
        touches(&mut room, reads(&content, &population), &[down]),
        Err(BlocksFault::Outside(past))
    );
    assert_eq!(room, before);
}

/// The cells kept from one call to the next: a block another writer changes in between is
/// imported again, so the next call meets it as it is now.
#[test]
fn a_block_written_between_calls_is_met_as_it_is_now() {
    let content = content();
    let population = population();
    let mut room = loaded(&content);
    pass(&mut room, &content);
    room.blocks[usize::from(TARGET.0)].kind = SHOT;
    hit(&mut room, reads(&content, &population), TARGET, HitBy::Beam).unwrap();
    assert!(matches!(
        room.plms.slots[38].occupancy,
        Occupancy::Live(PlmBody::Reaction(_))
    ));
    room.blocks[usize::from(TARGET.0)].kind = BlockKind::Solid(Bts(0));
    let before = room.plms.slots[37].occupancy;
    hit(&mut room, reads(&content, &population), TARGET, HitBy::Beam).unwrap();
    assert_eq!(room.plms.slots[37].occupancy, before);
}
