use data::level::ScrollArea;
use data::plm::{ItemPlm, ScrollChange, ScrollChangesId};
use data::rom::RomAddress;

use data::plm::{Extension, PlmEntry};

use super::*;
use crate::state::room::{
    BlockColumns, ItemGfx, ItemGfxSlot, ItemStep, PlmChoice, ScrollPlm, ScrollStep, Tile,
    TileNumber,
};
use crate::state::video::Video;
use crate::state::{COLLECTED_ITEM_WORDS, CollectedItems};
use crate::world_blocks_bridge::fixture::content;

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
    spawn_population(&mut room, &content(), &population()).unwrap();
    room
}

/// The pass with the invented blocks' content.
fn process_unseen(
    room: &mut RoomScoped,
    collected: &CollectedItems,
    population: &PlmPopulation,
) -> Option<SlotIndex<Plm>> {
    super::process_unseen(room, collected, &content(), population).unwrap()
}

#[test]
fn spawns_take_the_highest_free_slot_and_extensions_free_theirs() {
    let room = loaded();
    let slots = &room.plms.slots;
    // The scroll PLM takes 39; each extension takes 38 and frees it; the item takes 38.
    assert_eq!(
        slots[39].occupancy,
        Occupancy::Live(PlmBody::Scroll(ScrollPlm {
            changes: ScrollChangesId(0),
            triggered: false,
            step: ScrollStep::Sleep,
        }))
    );
    assert_eq!(
        slots[38].occupancy,
        Occupancy::Live(PlmBody::Item {
            item: ItemPlm::MorphBall,
            collected_bit: 0x1a,
            step: ItemStep::Spawned,
            slot: ItemGfxSlot(0),
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
fn the_pass_deletes_an_item_collision_plm() {
    let mut room = loaded();
    let slot = &mut room.plms.slots[30];
    slot.occupancy = Occupancy::Live(PlmBody::ItemCollision);
    slot.common.cursor.delay = Frames(1);
    process_unseen(&mut room, &CollectedItems::default(), &population());
    assert_eq!(room.plms.slots[30].occupancy, Occupancy::Free);
}

#[test]
fn an_items_timer_shows_each_frame_for_4_passes() {
    let mut room = loaded();
    assert_eq!(room.plms.slots[38].common.cursor.delay, Frames(1));
    let mut delays = Vec::new();
    for _ in 0..6 {
        process_unseen(&mut room, &CollectedItems::default(), &population());
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

/// The room past the PLM pass after its load: the item's first run done.
fn shown() -> RoomScoped {
    let mut room = loaded();
    process_unseen(&mut room, &CollectedItems::default(), &population());
    room
}

/// The item's first run loads its GFX into the first slot, its blocks' entries now the
/// Morph Ball's tiles, and the next load would take the second.
#[test]
fn an_items_first_run_loads_its_gfx_into_the_next_slot() {
    let room = loaded();
    assert_eq!(room.item_gfx, ItemGfx::default());
    let room = shown();
    assert_eq!(
        room.item_gfx,
        ItemGfx {
            next: ItemGfxSlot(1),
            loaded: [Some(ItemPlm::MorphBall), None, None, None],
        }
    );
    assert_eq!(
        item_step(&room),
        Some(ItemStep::Shown {
            next: ItemFrame::Second
        })
    );
    assert_eq!(room.plms.slots[ITEM_SLOT].common.cursor.delay, Frames(4));
}

/// A second item in the room takes the second slot (`$1C2D` steps by 2).
#[test]
fn a_rooms_second_item_loads_into_the_second_slot() {
    let mut population = population();
    population
        .entries
        .push(entry(PlmKind::Item(ItemPlm::MorphBall, 0x1b), 22, 5));
    let mut room = room();
    spawn_population(&mut room, &content(), &population).unwrap();
    process_unseen(&mut room, &CollectedItems::default(), &population);
    assert_eq!(room.item_gfx.next, ItemGfxSlot(2));
    assert_eq!(
        room.item_gfx.loaded,
        [
            Some(ItemPlm::MorphBall),
            Some(ItemPlm::MorphBall),
            None,
            None
        ]
    );
}

/// A collected item's first run still loads its GFX, then goes to the empty item: air with
/// tile `FFh` for a frame, then the delete (`$84:887C`, `$84:DFA9`).
#[test]
fn a_collected_items_first_run_draws_the_empty_item_and_deletes_it() {
    let mut room = loaded();
    // Place 1Ah: word 1's bit Ah.
    let mut words = [0; COLLECTED_ITEM_WORDS];
    words[1] = 1 << 0xa;
    let collected = CollectedItems(words);
    process_unseen(&mut room, &collected, &population());
    assert_eq!(room.item_gfx.next, ItemGfxSlot(1));
    assert_eq!(item_step(&room), Some(ItemStep::Emptied));
    let block = room.blocks[usize::from(ITEM.0)];
    assert_eq!(block.kind, BlockKind::Air(Bts(0x45)));
    assert_eq!(block.tile.number, TileNumber(0xff));
    process_unseen(&mut room, &collected, &population());
    assert_eq!(room.plms.slots[ITEM_SLOT].occupancy, Occupancy::Free);
}

#[test]
fn an_items_draw_makes_its_block_the_special_block_samus_touches() {
    let mut room = loaded();
    process_unseen(&mut room, &CollectedItems::default(), &population());
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
        process_unseen(&mut room, &CollectedItems::default(), &population());
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
    let mut room = shown();
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

/// A touch, then the pass: the item's pre-instruction goes to its link and the pickup calls out
/// of the pass before the lower slots run; its place is emptied with the pickup, on its return.
#[test]
fn a_triggered_item_calls_out_of_the_pass() {
    let mut room = shown();
    // The item's timer mid-wait: the pre-instruction resets it to 1 all the same.
    room.plms.slots[ITEM_SLOT].common.cursor.delay = Frames(3);
    touch_item_block(&mut room, ITEM).unwrap();
    let callout = process_unseen(&mut room, &CollectedItems::default(), &population());
    assert_eq!(callout, Some(SlotIndex(38, PhantomData)));
    assert_eq!(item_step(&room), Some(ItemStep::PickingUp));
    // The collision PLM below waits for the rest of the pass.
    assert_eq!(
        room.plms.slots[37].occupancy,
        Occupancy::Live(PlmBody::ItemCollision)
    );
    assert_eq!(room.plms.slots[37].common.cursor.delay, Frames(1));
}

/// A second item draws its slot's blocks, `90h` and `91h` (`$84:A31B`, `$84:A321`), and keeps
/// the slot its load took.
#[test]
fn a_rooms_second_item_draws_the_blocks_of_its_slot() {
    let mut population = population();
    population
        .entries
        .push(entry(PlmKind::Item(ItemPlm::MorphBall, 0x1b), 22, 5));
    let mut room = room();
    spawn_population(&mut room, &content(), &population).unwrap();
    let second = BlockIndex(5 * 32 + 22);
    let tile = |room: &RoomScoped| room.blocks[usize::from(second.0)].tile.number.0;
    let mut tiles = Vec::new();
    for _ in 0..5 {
        process_unseen(&mut room, &CollectedItems::default(), &population);
        tiles.push(tile(&room));
    }
    assert_eq!(tiles, [0x90, 0x90, 0x90, 0x90, 0x91]);
    let slots: Vec<_> = room
        .plms
        .slots
        .iter()
        .filter_map(|slot| {
            if let Occupancy::Live(PlmBody::Item { slot, .. }) = slot.occupancy {
                Some(slot)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(slots, [ItemGfxSlot(1), ItemGfxSlot(0)]);
}

/// The draws of the pass queue the item's block into the tilemap while it is on screen: the
/// first frame at the load, the second when its timer runs out.
#[test]
fn an_on_screen_items_draws_queue_its_blocks_tiles() {
    let mut room = loaded();
    // The item's column, 14h, is on screen once the camera is a screen across.
    room.camera.x.pixels = 0x100;
    let mut video = Video::default();
    let tables = TileTables::default();
    let pass = |room: &mut RoomScoped, video: &mut Video| {
        process(
            &mut Drawing::new(video, &tables),
            room,
            &CollectedItems::default(),
            &content(),
            &population(),
        )
        .unwrap();
    };
    pass(&mut room, &mut video);
    let words = |video: &Video| -> Vec<Vec<u16>> {
        video
            .batch
            .vram
            .iter()
            .filter_map(|transfer| match &transfer.source {
                crate::state::video::VramSource::Words { words, .. } => Some(words.clone()),
                crate::state::video::VramSource::Content(_) => None,
            })
            .collect()
    };
    assert_eq!(words(&video), [[0x3e0, 0x3e1], [0x3e2, 0x3e3]]);
    for _ in 0..3 {
        pass(&mut room, &mut video);
    }
    assert_eq!(video.batch.vram.len(), 2);
    pass(&mut room, &mut video);
    assert_eq!(words(&video)[2..], [[0x3e4, 0x3e5], [0x3e6, 0x3e7]]);
}
