//! Retain the doorway selected by ordered movement touches until the next dispatch.

use data::assets::world::Door;
use data::id::DoorwayId;
use data::level::BlockKind;

use crate::pose::Content;
use crate::state::{GameMode, State};

/// A checked passage edge that the host cannot continue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassageFault {
    /// A touched cell or room-door-list entry is absent.
    DoorContent,
    /// A selected destination or its source room is absent from the checked world.
    DestinationContent,
    /// Passage execution and its room, timing and sound consumers are not wired.
    MissingRuntime,
}

/// Check the retained destination against the same imported world and source door list.
/// # Errors
/// Missing registered source/destination content.
pub fn validate(state: &State, content: &Content) -> Result<(), PassageFault> {
    match state.mode {
        GameMode::Gameplay => Ok(()),
        GameMode::PassageEntry { doorway } => destination(state.room.room, doorway, content),
    }
}

fn destination(
    source: data::id::RoomId,
    doorway: DoorwayId,
    content: &Content,
) -> Result<(), PassageFault> {
    let world = content
        .world
        .as_ref()
        .ok_or(PassageFault::DestinationContent)?;
    let room = world.room(source).ok_or(PassageFault::DestinationContent)?;
    if !room.doors.contains(&Door::To(doorway)) {
        return Err(PassageFault::DestinationContent);
    }
    let arrival = world
        .doorway(doorway)
        .ok_or(PassageFault::DestinationContent)?;
    world
        .room(arrival.room)
        .ok_or(PassageFault::DestinationContent)?;
    Ok(())
}

fn last_destination(doors: impl IntoIterator<Item = Door>) -> Option<DoorwayId> {
    doors.into_iter().fold(None, |selected, door| match door {
        Door::To(doorway) => Some(doorway),
        Door::Nowhere => selected,
    })
}

/// Select from resolved terminal touches; inside cells are not an input to this edge.
/// # Errors
/// Out-of-range resolved cells or missing checked door/destination content.
pub fn select(
    room: &crate::state::room::RoomScoped,
    touches: &[types::Touch],
    content: &Content,
) -> Result<Option<DoorwayId>, PassageFault> {
    let mut doors = Vec::new();
    for touch in touches {
        if touch.cell.x >= room.width.0 || touch.cell.y >= room.height.0 {
            return Err(PassageFault::DoorContent);
        }
        let position = usize::from(touch.cell.y)
            .checked_mul(usize::from(room.width.0))
            .and_then(|row| row.checked_add(usize::from(touch.cell.x)))
            .ok_or(PassageFault::DoorContent)?;
        let block = room.blocks.get(position).ok_or(PassageFault::DoorContent)?;
        if let BlockKind::Door(index) = block.kind {
            let door = content
                .room_doors
                .of(room.room)
                .get(index.place())
                .copied()
                .ok_or(PassageFault::DoorContent)?;
            if let Door::To(doorway) = door {
                destination(room.room, doorway, content)?;
            }
            doors.push(door);
        }
    }
    Ok(last_destination(doors))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::last_destination;
    use data::assets::world::Door;
    use data::id::DoorwayId;

    #[test]
    fn last_real_destination_wins_and_nowhere_preserves_it() {
        let first = DoorwayId(1);
        let last = DoorwayId(2);
        assert_eq!(last_destination([]), None);
        assert_eq!(last_destination([Door::Nowhere]), None);
        assert_eq!(
            last_destination([Door::To(first), Door::Nowhere]),
            Some(first)
        );
        assert_eq!(
            last_destination([Door::To(first), Door::To(last), Door::Nowhere]),
            Some(last)
        );
    }
    #[test]
    fn missing_arrival_content_rejects_a_restored_pending_event() {
        let state = crate::state::State {
            mode: crate::state::GameMode::PassageEntry {
                doorway: DoorwayId(1),
            },
            ..crate::state::State::default()
        };
        assert_eq!(
            super::validate(&state, &crate::pose::Content::default()),
            Err(super::PassageFault::DestinationContent)
        );
        assert_eq!(
            super::validate(
                &crate::state::State::default(),
                &crate::pose::Content::default()
            ),
            Ok(())
        );
    }

    #[test]
    fn a_nowhere_high_bit_door_is_not_a_passage_and_missing_list_is_rejected() {
        let room = crate::state::room::RoomScoped {
            width: crate::state::room::BlockColumns(1),
            height: crate::state::room::BlockRows(1),
            blocks: vec![data::level::Block {
                kind: data::level::BlockKind::Door(data::level::DoorIndex(0x80)),
                ..data::level::Block::default()
            }],
            ..crate::state::room::RoomScoped::default()
        };
        let touch = types::Touch {
            cell: types::Pair { x: 0, y: 0 },
            way: types::Way::Look,
        };
        let mut content = crate::pose::Content::default();
        assert_eq!(
            super::select(&room, &[touch], &content),
            Err(super::PassageFault::DoorContent)
        );
        content.room_doors.0.push((room.room, vec![Door::Nowhere]));
        assert_eq!(super::select(&room, &[touch], &content), Ok(None));
        assert_eq!(super::select(&room, &[], &content), Ok(None));
        let moved = types::Touch {
            way: types::Way::Move(types::Heading::Right),
            ..touch
        };
        assert_eq!(super::select(&room, &[moved], &content), Ok(None));
        let outside = types::Touch {
            cell: types::Pair { x: 1, y: 0 },
            ..touch
        };
        assert_eq!(
            super::select(&room, &[outside], &content),
            Err(super::PassageFault::DoorContent)
        );
    }

    pub fn registered_content() -> crate::pose::Content {
        use data::assets::world::{Arrival, Back, Doorway, MapPlace, Room, Version, World};
        use data::id::{RoomId, TilesetId};
        let source = Room {
            id: RoomId(1),
            place: MapPlace {
                area: data::room::Area::Ceres,
                x: 0,
                y: 0,
            },
            size: data::room::Screens {
                width: 1,
                height: 1,
            },
            scrollers: data::room::Scrollers { up: 0, down: 0 },
            doors: vec![
                Door::To(DoorwayId(1)),
                Door::To(DoorwayId(2)),
                Door::Nowhere,
            ],
            conditional: Vec::new(),
            default: Version {
                tileset: TilesetId(0),
                back: Back::Level,
                plms: data::plm::PlmPopulation {
                    entries: Vec::new(),
                    scroll_changes: Vec::new(),
                },
            },
        };
        let target = Room {
            id: RoomId(2),
            doors: Vec::new(),
            ..source.clone()
        };
        let arrival = Arrival {
            way: types::Heading::Right,
            view: types::Pair {
                x: data::assets::world::ScreenCoordinate(0),
                y: data::assets::world::ScreenCoordinate(0),
            },
            speed: None,
        };
        let world = World::new(
            vec![source.clone(), target],
            vec![
                Doorway {
                    id: DoorwayId(1),
                    room: RoomId(2),
                    arrival,
                },
                Doorway {
                    id: DoorwayId(2),
                    room: RoomId(2),
                    arrival,
                },
            ],
        )
        .unwrap();
        crate::pose::Content {
            room_doors: crate::collision::RoomDoors(vec![(source.id, source.doors)]),
            world: Some(world),
            ..crate::pose::Content::default()
        }
    }

    #[test]
    fn ordered_resolved_moves_and_looks_select_the_last_registered_destination() {
        let room = crate::state::room::RoomScoped {
            room: data::id::RoomId(1),
            width: crate::state::room::BlockColumns(3),
            height: crate::state::room::BlockRows(1),
            blocks: [0, 1, 2]
                .into_iter()
                .map(|index| data::level::Block {
                    kind: data::level::BlockKind::Door(data::level::DoorIndex(index)),
                    ..data::level::Block::default()
                })
                .collect(),
            ..crate::state::room::RoomScoped::default()
        };
        let touches = [
            types::Touch {
                cell: types::Pair { x: 0, y: 0 },
                way: types::Way::Move(types::Heading::Right),
            },
            types::Touch {
                cell: types::Pair { x: 1, y: 0 },
                way: types::Way::Look,
            },
            types::Touch {
                cell: types::Pair { x: 2, y: 0 },
                way: types::Way::Look,
            },
        ];
        assert_eq!(
            super::select(&room, &touches, &registered_content()),
            Ok(Some(DoorwayId(2)))
        );
    }

    #[test]
    fn restored_pending_destination_refuses_next_dispatch_without_publishing_any_work() {
        use crate::state::strand::{Frame, MainLoopAt, MainStrand, Wait};
        let state = crate::state::State {
            mode: crate::state::GameMode::PassageEntry {
                doorway: DoorwayId(1),
            },
            room: crate::state::room::RoomScoped {
                room: data::id::RoomId(1),
                ..crate::state::room::RoomScoped::default()
            },
            main: MainStrand {
                frames: vec![Frame::MainLoop(MainLoopAt::Start)],
                waiting: Some(Wait::Nmi),
            },
            ..crate::state::State::default()
        };
        let restored =
            canonical::from_bytes::<crate::state::State>(&canonical::to_bytes(&state)).unwrap();
        let mut session = crate::Session::open(
            restored,
            crate::Rules {
                auto_repeat: crate::AutoRepeat {
                    initial: crate::state::slots::Frames(0),
                    subsequent: crate::state::slots::Frames(0),
                },
                switches: crate::Switches::REFERENCE,
            },
            std::sync::Arc::new(registered_content()),
        )
        .unwrap();
        let result = session.step(crate::TickInput {
            pad: crate::state::Buttons::default(),
            timing: crate::state::NmiTiming::OnTime,
        });
        assert_eq!(
            result.err(),
            Some(crate::SessionFault::Passage(
                super::PassageFault::MissingRuntime
            ))
        );
        assert_eq!(session.state(), &state);
    }
}
