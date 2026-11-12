use data::level::ScrollArea;
use data::plm::{Extension, PlmEntry, PlmKind, PlmPopulation, ScrollChange, ScrollChangesId};

use super::*;
use crate::plm::spawn_population;
use crate::state::CollectedItems;
use crate::state::room::{
    Block, BlockColumns, BlockRows, PlmBody, ScrollPlm, ScrollStep, Tile, TileNumber,
};
use crate::state::room::{Bts, Offset};
use crate::state::slots::Occupancy;
use crate::world_blocks_bridge::fixture::content;

/// The scroll PLM's slot: the first spawn takes the highest.
const SCROLL_SLOT: usize = 39;

/// A scroll PLM at (3, 2) turning screen 1 green, with a rightwards extension at (4, 2).
fn population() -> PlmPopulation {
    PlmPopulation {
        entries: vec![
            PlmEntry {
                kind: PlmKind::Scroll(ScrollChangesId(0)),
                x: 3,
                y: 2,
            },
            PlmEntry {
                kind: PlmKind::Extension(Extension::Right),
                x: 4,
                y: 2,
            },
        ],
        scroll_changes: vec![vec![ScrollChange {
            screen: 1,
            area: ScrollArea::Green,
        }]],
    }
}

/// A two-screen-wide room of air, its screens red, with the population spawned.
fn room() -> RoomScoped {
    let air = Block {
        kind: BlockKind::Air(Bts(0)),
        tile: Tile {
            number: TileNumber(0),
            x_flip: false,
            y_flip: false,
        },
    };
    let mut room = RoomScoped {
        blocks: vec![air; 512],
        width: BlockColumns(32),
        height: BlockRows(16),
        scroll_areas: vec![ScrollArea::Red; 2],
        ..RoomScoped::default()
    };
    spawn_population(&mut room, &content(), &population()).unwrap();
    room
}

/// Her inside handling, with the invented blocks' content.
fn handle(room: &mut RoomScoped, samus: &Samus) -> Result<(), BlocksFault> {
    let population = population();
    let content = content();
    super::handle(
        room,
        samus,
        Reads {
            content: &content,
            population: &population,
        },
    )
}

/// The PLM pass, with the invented blocks' content.
fn process(room: &mut RoomScoped, collected: &CollectedItems, population: &PlmPopulation) {
    crate::plm::process_unseen(room, collected, &content(), population).unwrap();
}

/// Samus standing at `x`, `y` with a Y radius of 15h.
fn samus(x: u16, y: u16) -> Samus {
    let mut samus = Samus::default();
    samus.motion.position.x = crate::model::num::Fixed::new(x, 0);
    samus.motion.position.y = crate::model::num::Fixed::new(y, 0);
    samus.y_radius = 0x15;
    samus
}

fn triggered(room: &RoomScoped) -> bool {
    matches!(
        room.plms.slots[SCROLL_SLOT].occupancy,
        Occupancy::Live(PlmBody::Scroll(ScrollPlm {
            triggered: true,
            step: ScrollStep::Update,
            ..
        }))
    )
}

/// Walking right along row 2, her centre's block reaches the trigger at column 3: the
/// inside handling triggers the scroll PLM, and that frame's PLM pass changes screen 1.
#[test]
fn walking_into_the_trigger_changes_the_scroll_area_on_the_plm_pass() {
    let mut room = room();
    // Her centre in row 2, her bottom in row 3.
    let y = 2 * 16 + 8;
    for x in (0x10..0x30).step_by(4) {
        handle(&mut room, &samus(x, y)).unwrap();
        process(&mut room, &CollectedItems::default(), &population());
        assert!(!triggered(&room), "x {x:#x}");
        assert_eq!(room.scroll_areas, [ScrollArea::Red; 2], "x {x:#x}");
    }
    handle(&mut room, &samus(0x30, y)).unwrap();
    assert!(triggered(&room));
    process(&mut room, &CollectedItems::default(), &population());
    assert_eq!(room.scroll_areas, [ScrollArea::Red, ScrollArea::Green]);
    assert!(!triggered(&room));
}

/// The extension beside the trigger stands for it.
#[test]
fn her_centre_in_the_extension_triggers_the_scroll_plm() {
    let mut room = room();
    handle(&mut room, &samus(4 * 16 + 8, 2 * 16 + 8)).unwrap();
    assert!(triggered(&room));
}

/// Only her centre's reaction spawns the touch PLM (`$94:9956`): her bottom or top in the
/// trigger does nothing.
#[test]
fn her_bottom_or_top_in_the_trigger_does_nothing() {
    let x = 3 * 16 + 8;
    let mut bottom = room();
    // Centre in row 1, bottom (y + 14h) in row 2.
    handle(&mut bottom, &samus(x, 16 + 4)).unwrap();
    assert!(!triggered(&bottom));
    let mut top = room();
    // Centre in row 3, top (y - 15h) in row 2, bottom in row 4.
    handle(&mut top, &samus(x, 3 * 16 + 8)).unwrap();
    assert!(!triggered(&top));
}

/// Her centre in the same row as her bottom skips the centre's reaction (`$94:9BB0`).
#[test]
fn her_centre_in_her_bottoms_row_is_not_looked_up() {
    let mut room = room();
    let mut standing = samus(3 * 16 + 8, 2 * 16 + 2);
    standing.y_radius = 4;
    handle(&mut room, &standing).unwrap();
    assert!(!triggered(&room));
}

/// Through a Session: a Tick's gameplay pass runs the inside handling in Samus's current
/// state and the PLM pass after her new state, so the frame she stands in the trigger
/// changes the scroll area.
#[test]
fn a_gameplay_pass_inside_the_trigger_changes_the_scroll_area() {
    use crate::session::{AutoRepeat, Content, Rules, Session, Switches, TickInput};
    use crate::state::State;
    use crate::state::slots::Frames;
    use data::pose::{PoseId, Poses};

    // Pose 0 stands facing right with a Y radius of 15h.
    let content = Content {
        poses: Poses::decode(&[8, 0, 0xff, 2, 0, 0, 0x15, 0]).unwrap(),
        room_plms: crate::plm::RoomPlms(vec![(data::id::RoomId(0), population())]),
        blocks: content(),
        ..Content::default()
    };
    let state = State {
        room: room(),
        samus: Samus {
            pose: PoseId(0),
            ..samus(3 * 16 + 8, 2 * 16 + 8)
        },
        ..State::default()
    };
    let rules = Rules {
        switches: Switches::REFERENCE,
        auto_repeat: AutoRepeat {
            initial: Frames(3),
            subsequent: Frames(2),
        },
    };
    let mut session = Session::open(state, rules, std::sync::Arc::new(content)).unwrap();
    session.step(TickInput::default()).unwrap();
    let room = &session.state().room;
    assert_eq!(room.scroll_areas, [ScrollArea::Red, ScrollArea::Green]);
    assert!(!triggered(room));
}

/// A chain of stand-ins a reaction's show closed into a loop as the room ran, which the original
/// follows without end, is a fault at the cell her point is in, not a hang.
#[test]
fn a_stand_in_chain_that_loops_is_a_fault() {
    let mut room = room();
    let at = |x: usize| 2 * 32 + x;
    room.blocks[at(5)].kind = BlockKind::HorizontalExtension(Offset(1));
    room.blocks[at(6)].kind = BlockKind::HorizontalExtension(Offset(-1));
    assert_eq!(
        handle(&mut room, &samus(5 * 16 + 8, 2 * 16 + 8)),
        Err(BlocksFault::StandInLoop(BlockIndex(2 * 32 + 5)))
    );
}
