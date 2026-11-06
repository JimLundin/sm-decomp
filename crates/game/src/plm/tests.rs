use data::level::ScrollArea;
use data::plm::{ItemPlm, ScrollChange, ScrollChangesId};
use data::rom::RomAddress;

use super::*;
use crate::state::CollectedItems;
use crate::state::room::{BlockColumns, Crumble, ItemStep, PlmChoice, Tile, TileNumber};

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

/// A scroll PLM at (3, 2) turning screen 1 green, the upwards extension above it, a
/// rightwards one beside it, and the Morph Ball at (20, 5).
fn population() -> PlmPopulation {
    let changes = ScrollChangesId(0);
    PlmPopulation {
        entries: vec![
            entry(PlmKind::Scroll(changes), 3, 2),
            entry(PlmKind::Extension(Extension::Up), 3, 1),
            entry(PlmKind::Extension(Extension::Right), 4, 2),
            entry(PlmKind::Item(ItemPlm::MorphBall, 0x1a), 20, 5),
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

fn loaded() -> RoomScoped {
    let mut room = room();
    spawn_population(&mut room, &population());
    room
}

fn scroll(room: &RoomScoped, slot: usize) -> Option<ScrollPlm> {
    if let Occupancy::Live(PlmBody::Scroll(scroll)) = room.plms.slots[slot].occupancy {
        Some(scroll)
    } else {
        None
    }
}

#[test]
fn setup_writes_each_plms_block_keeping_its_tile() {
    let room = loaded();
    let kind = |index: u16| room.blocks[usize::from(index)].kind;
    assert_eq!(kind(TRIGGER.0), BlockKind::SpecialAir(Bts(0x46)));
    assert_eq!(kind(32 + 3), BlockKind::VerticalExtension(Offset(1)));
    assert_eq!(kind(2 * 32 + 4), BlockKind::HorizontalExtension(Offset(-1)));
    assert_eq!(kind(5 * 32 + 20), BlockKind::Air(Bts(0x45)));
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
    assert_eq!(written, 4);
}

#[test]
fn spawns_take_the_highest_free_slot_and_extensions_free_theirs() {
    let room = loaded();
    let slots = &room.plms.slots;
    // The scroll PLM takes 39; each extension takes 38 and frees it; the item takes 38.
    assert_eq!(
        scroll(&room, 39),
        Some(ScrollPlm {
            changes: ScrollChangesId(0),
            triggered: false,
            step: ScrollStep::Sleep,
        })
    );
    assert_eq!(
        slots[38].occupancy,
        Occupancy::Live(PlmBody::Item {
            item: ItemPlm::MorphBall,
            collected_bit: 0x1a,
            step: ItemStep::Shown {
                next: ItemFrame::First,
            },
        })
    );
    assert_eq!(slots[38].generation.0, 3);
    assert_eq!(slots[39].common.block, TRIGGER);
    assert_eq!(slots[39].common.cursor.delay, Frames(1));
    assert!(
        slots[..38]
            .iter()
            .all(|slot| slot.occupancy == Occupancy::Free)
    );
}

#[test]
fn a_sleeping_scroll_plm_changes_nothing() {
    let mut room = loaded();
    let before = room.clone();
    for _ in 0..3 {
        process(&mut room, &mut CollectedItems::default(), &population());
    }
    assert_eq!(room.scroll_areas, before.scroll_areas);
    // Only the item's draw writes a block.
    let written: Vec<_> = (0_u16..)
        .zip(room.blocks.iter().zip(&before.blocks))
        .filter_map(|(index, (after, before))| (after != before).then_some(index))
        .collect();
    assert_eq!(written, [5 * 32 + 20]);
    // The sleep leaves the timer at 0, and it wraps.
    assert_eq!(room.plms.slots[39].common.cursor.delay, Frames(0xfffe));
}

#[test]
fn touching_the_trigger_changes_the_scroll_areas_on_the_next_pass() {
    let mut room = loaded();
    process(&mut room, &mut CollectedItems::default(), &population());
    touch_scroll_block(&mut room, TRIGGER).unwrap();
    let touch = &room.plms.slots[37];
    assert_eq!(touch.occupancy, Occupancy::Live(PlmBody::ScrollTouch));
    assert_eq!(touch.common.block, BlockIndex(0));
    let triggered = scroll(&room, 39).unwrap();
    assert!(triggered.triggered);
    assert_eq!(triggered.step, ScrollStep::Update);

    // A second touch before the update spawns another touch PLM but changes nothing.
    touch_scroll_block(&mut room, TRIGGER).unwrap();
    assert_eq!(scroll(&room, 39), Some(triggered));

    process(&mut room, &mut CollectedItems::default(), &population());
    assert_eq!(room.scroll_areas, [ScrollArea::Red, ScrollArea::Green]);
    assert_eq!(
        scroll(&room, 39),
        Some(ScrollPlm {
            triggered: false,
            step: ScrollStep::Sleep,
            ..triggered
        })
    );
    assert_eq!(
        room.blocks[usize::from(TRIGGER.0)].kind,
        BlockKind::SpecialAir(Bts(0x46))
    );
    // The touch PLMs deleted themselves.
    assert_eq!(room.plms.slots[37].occupancy, Occupancy::Free);
    assert_eq!(room.plms.slots[36].occupancy, Occupancy::Free);
}

#[test]
fn a_touch_with_no_plm_on_its_block_is_the_originals_hang() {
    let mut room = loaded();
    assert_eq!(
        touch_scroll_block(&mut room, BlockIndex(7)),
        Err(TouchFault::NoPlmOnBlock(BlockIndex(7)))
    );
}

#[test]
fn a_touch_with_no_free_slot_spawns_nothing_and_triggers_nothing() {
    let mut room = loaded();
    for slot in &mut room.plms.slots {
        if slot.occupancy == Occupancy::Free {
            slot.occupancy = Occupancy::Live(PlmBody::ScrollTouch);
        }
    }
    let before = room.clone();
    touch_scroll_block(&mut room, TRIGGER).unwrap();
    assert_eq!(room, before);
}

#[test]
fn the_pass_deletes_an_item_collision_plm_and_holds_a_shot_block() {
    let mut room = loaded();
    let crumbling = PlmBody::ShotBlock {
        crumble: Crumble::First,
    };
    for (index, body) in [(30, PlmBody::ItemCollision), (31, crumbling)] {
        let slot = &mut room.plms.slots[index];
        slot.occupancy = Occupancy::Live(body);
        slot.common.cursor.delay = Frames(1);
    }
    process(&mut room, &mut CollectedItems::default(), &population());
    assert_eq!(room.plms.slots[30].occupancy, Occupancy::Free);
    assert_eq!(room.plms.slots[31].occupancy, Occupancy::Live(crumbling));
    assert_eq!(room.plms.slots[31].common.cursor.delay, Frames(1));
}

#[test]
fn an_items_timer_shows_each_frame_for_4_passes() {
    let mut room = loaded();
    assert_eq!(room.plms.slots[38].common.cursor.delay, Frames(1));
    let mut delays = Vec::new();
    for _ in 0..6 {
        process(&mut room, &mut CollectedItems::default(), &population());
        delays.push(room.plms.slots[38].common.cursor.delay.0);
    }
    assert_eq!(delays, [4, 3, 2, 1, 4, 3]);
}

/// The Morph Ball's block and slot in [`loaded`].
const ITEM: BlockIndex = BlockIndex(5 * 32 + 20);
const ITEM_SLOT: usize = 38;

fn item_step(room: &RoomScoped) -> Option<ItemStep> {
    if let Occupancy::Live(PlmBody::Item { step, .. }) = room.plms.slots[ITEM_SLOT].occupancy {
        Some(step)
    } else {
        None
    }
}

#[test]
fn an_items_draw_makes_its_block_the_special_block_samus_touches() {
    let mut room = loaded();
    process(&mut room, &mut CollectedItems::default(), &population());
    assert_eq!(
        room.blocks[usize::from(ITEM.0)].kind,
        BlockKind::Special(PlmChoice::Common(0x45))
    );
}

/// The item block's tile number after each of 10 passes from the spawn: frame 0's draw on the
/// first pass, then each frame for 4.
#[test]
fn an_items_draws_turn_its_block_between_its_two_frames_tiles() {
    let mut room = loaded();
    let mut tiles = Vec::new();
    for _ in 0..10 {
        process(&mut room, &mut CollectedItems::default(), &population());
        tiles.push(room.blocks[usize::from(ITEM.0)].tile.number.0);
    }
    assert_eq!(
        tiles,
        [0x8e, 0x8e, 0x8e, 0x8e, 0x8f, 0x8f, 0x8f, 0x8f, 0x8e, 0x8e]
    );
    assert_eq!(
        item_step(&room),
        Some(ItemStep::Shown {
            next: ItemFrame::Second
        })
    );
}

#[test]
fn touching_an_item_spawns_the_collision_plm_and_triggers_the_item() {
    let mut room = loaded();
    touch_item_block(&mut room, ITEM).unwrap();
    // The collision PLM takes the highest free slot, its block moved to 0, its timer 1.
    let collision = &room.plms.slots[37];
    assert_eq!(collision.occupancy, Occupancy::Live(PlmBody::ItemCollision));
    assert_eq!(collision.common.block, BlockIndex(0));
    assert_eq!(collision.common.cursor.delay, Frames(1));
    assert_eq!(item_step(&room), Some(ItemStep::Triggered));
}

#[test]
fn an_item_touch_with_no_plm_on_its_block_is_the_originals_crash() {
    let mut room = loaded();
    assert_eq!(
        touch_item_block(&mut room, BlockIndex(7)),
        Err(TouchFault::NoPlmOnBlock(BlockIndex(7)))
    );
}

/// A touch, then the pass: the item's pre-instruction goes to its link, its list sets the
/// collected bit and the pickup calls out of the pass before the lower slots run.
#[test]
fn a_triggered_item_collects_its_bit_and_calls_out_of_the_pass() {
    let mut room = loaded();
    // The item's timer mid-wait: the pre-instruction resets it to 1 all the same.
    room.plms.slots[ITEM_SLOT].common.cursor.delay = Frames(3);
    touch_item_block(&mut room, ITEM).unwrap();
    let mut collected = CollectedItems::default();
    let callout = process(&mut room, &mut collected, &population());
    assert_eq!(callout, Some(SlotIndex(38, PhantomData)));
    assert!(collected.contains(0x1a));
    assert_eq!(collected.0[1], 1 << 10);
    assert_eq!(item_step(&room), Some(ItemStep::PickingUp));
    // The collision PLM below waits for the rest of the pass.
    assert_eq!(
        room.plms.slots[37].occupancy,
        Occupancy::Live(PlmBody::ItemCollision)
    );
    assert_eq!(room.plms.slots[37].common.cursor.delay, Frames(1));
}
