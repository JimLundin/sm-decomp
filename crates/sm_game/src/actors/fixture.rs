//! Shared synthetic authored fixture; no gameplay or compatibility algorithm.
use super::{content::*, media::*};
use crate::{room_entry::world::*, session::Point};
use std::{collections::BTreeMap, sync::Arc};

pub fn binding(
    swap_frames: bool,
    default_appearance: AppearanceId,
    reuse_row: bool,
) -> Arc<WorldBinding> {
    let world = WorldDefinition {
        rooms: BTreeMap::new(),
        states: BTreeMap::new(),
        exits: BTreeMap::new(),
        starts: BTreeMap::new(),
        door_lists: BTreeMap::new(),
        tilesets: BTreeMap::new(),
        resources: BTreeMap::from([
            (ResourceId(1), ResourceKind::ActorSprites),
            (ResourceId(2), ResourceKind::ActorSprites),
            (ResourceId(3), ResourceKind::Palette),
            (ResourceId(4), ResourceKind::Palette),
        ]),
        closing_doors: BTreeMap::new(),
    };
    let keys = CompatibilityKeys {
        starts: BTreeMap::new(),
        rooms: BTreeMap::new(),
        states: BTreeMap::new(),
        exits: BTreeMap::new(),
        door_lists: BTreeMap::new(),
        resources: BTreeMap::from([
            (
                ResourceId(1),
                ResourceKey::Long(LegacyAddress24([0, 0x80, 0xa0])),
            ),
            (
                ResourceId(2),
                ResourceKey::Long(LegacyAddress24([0, 0x90, 0xa0])),
            ),
            (
                ResourceId(3),
                ResourceKey::Long(LegacyAddress24([0, 0x80, 0xb4])),
            ),
            (
                ResourceId(4),
                ResourceKey::Long(LegacyAddress24([0, 0x90, 0xb4])),
            ),
        ]),
    };
    let actors = ActorDefinitions {
        species: BTreeMap::from([(
            SpeciesId(7),
            SpeciesDefinition {
                initial_health: 99,
                contact_damage: 3,
                radius: Point { x: 8, y: 12 },
                hurt_time: 4,
                layer: 2,
                behavior: BehaviorId(8),
                default_appearance,
                multipart_count: 1,
            },
        )]),
        programs: BTreeMap::new(),
        populations: BTreeMap::new(),
        templates: BTreeMap::new(),
        poses: BTreeMap::from([(
            PoseId(9),
            PoseDefinition {
                collision: CollisionLayoutId(10),
                parts: vec![PosePart {
                    role: PartRoleId(0),
                    offset: Point { x: 0, y: 0 },
                }],
            },
        )]),
        collisions: BTreeMap::from([(
            CollisionLayoutId(10),
            CollisionLayout::Parts(vec![CollisionPart {
                offset: Point { x: 0, y: 0 },
                hitboxes: vec![CollisionBox {
                    left: 0xfff8,
                    top: 0xfff4,
                    right: 8,
                    bottom: 12,
                    touch: ActorCallbackId(60),
                    shot: ActorCallbackId(61),
                }],
            }]),
        )]),
    };
    let mut actor_keys = ActorCompatibilityKeys::default();
    actor_keys.species.insert(SpeciesId(7), 0xd000);
    actor_keys.behaviors.insert(BehaviorId(8), 0xd000);
    actor_keys.poses.insert(PoseId(9), (0xa0, 0xc000));
    actor_keys.callbacks.insert(ActorCallbackId(60), 0xa0804c);
    actor_keys.callbacks.insert(ActorCallbackId(61), 0xa0804c);
    let frame = |resource, material| {
        FrameDefinition::Sprites(vec![VisualPieceDefinition {
            resource: VisualResourceId(resource),
            source_origin: Point { x: 0, y: 0 },
            source_extent: Point { x: 0, y: 0 },
            offset: Point { x: 0, y: 0 },
            flip_x: false,
            flip_y: false,
            material: MaterialId(material),
        }])
    };
    let media = ActorMediaDefinitions {
        appearances: BTreeMap::from([
            (
                AppearanceId(20),
                AppearanceDefinition {
                    frames: BTreeMap::from([(
                        (PoseId(9), PartRoleId(0)),
                        FrameId(if swap_frames { 31 } else { 30 }),
                    )]),
                },
            ),
            (
                AppearanceId(21),
                AppearanceDefinition {
                    frames: BTreeMap::from([(
                        (PoseId(9), PartRoleId(0)),
                        FrameId(if swap_frames { 30 } else { 31 }),
                    )]),
                },
            ),
            (
                AppearanceId(22),
                AppearanceDefinition {
                    frames: BTreeMap::from([((PoseId(9), PartRoleId(0)), FrameId(32))]),
                },
            ),
        ]),
        frames: BTreeMap::from([
            (FrameId(30), frame(40, 50)),
            (FrameId(31), frame(41, 51)),
            (
                FrameId(32),
                FrameDefinition::Sprites(vec![
                    match frame(40, 50) {
                        FrameDefinition::Sprites(mut pieces) => pieces.remove(0),
                        _ => unreachable!(),
                    },
                    match frame(41, 51) {
                        FrameDefinition::Sprites(mut pieces) => pieces.remove(0),
                        _ => unreachable!(),
                    },
                ]),
            ),
        ]),
        materials: BTreeMap::from([
            (
                MaterialId(50),
                ActorMaterialDefinition {
                    palette: VisualResourceId(42),
                    cgram_row: 9,
                },
            ),
            (
                MaterialId(51),
                ActorMaterialDefinition {
                    palette: VisualResourceId(43),
                    cgram_row: if reuse_row { 9 } else { 10 },
                },
            ),
            (
                MaterialId(52),
                ActorMaterialDefinition {
                    palette: VisualResourceId(43),
                    cgram_row: 0,
                },
            ),
            (
                MaterialId(53),
                ActorMaterialDefinition {
                    palette: VisualResourceId(43),
                    cgram_row: 1,
                },
            ),
            (
                MaterialId(54),
                ActorMaterialDefinition {
                    palette: VisualResourceId(43),
                    cgram_row: 2,
                },
            ),
        ]),
        resources: BTreeMap::from([
            (
                VisualResourceId(40),
                ActorVisualResource::CompatibleSprites {
                    resource: ResourceId(1),
                },
            ),
            (
                VisualResourceId(41),
                ActorVisualResource::CompatibleSprites {
                    resource: ResourceId(2),
                },
            ),
            (
                VisualResourceId(42),
                ActorVisualResource::CompatiblePalette {
                    resource: ResourceId(3),
                },
            ),
            (
                VisualResourceId(43),
                ActorVisualResource::CompatiblePalette {
                    resource: ResourceId(4),
                },
            ),
        ]),
    };
    WorldBinding::prepared_with_actors(world, keys, actors, actor_keys, media).unwrap()
}
