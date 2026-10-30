//! Two-room authored actor fixture. Synthetic records only; no exported ROM bytes.
#![forbid(unsafe_code)]
use crate::{
    game::{room_entry::world::*, session::Point},
    game_state::Game,
};
use sm_game::actors::{content::*, media::*};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) const SPECIES: SpeciesId = SpeciesId(0x17000007);
pub(crate) const APPEARANCES: [AppearanceId; 3] = [
    AppearanceId(0x21000021),
    AppearanceId(0x11000092),
    AppearanceId(0x31000043),
];
pub(crate) const POSE: PoseId = PoseId(0x37000039);
pub(crate) const TEMPLATE: ActorTemplateId = ActorTemplateId(0x29000019);
const PROGRAM: ActorProgramId = ActorProgramId(0x42000017);
const POP_A: PopulationId = PopulationId(0x13000053);
const POP_B: PopulationId = PopulationId(0x03000083);
const DEST_POP: ResourceId = ResourceId(0x72000017);
const DEST_TILES: ResourceId = ResourceId(0x72000018);
pub(crate) const PALETTES: [ResourceId; 3] = [
    ResourceId(0x71000031),
    ResourceId(0x71000032),
    ResourceId(0x71000033),
];
const SPRITES: [ResourceId; 3] = [
    ResourceId(0x71000041),
    ResourceId(0x71000042),
    ResourceId(0x71000043),
];
pub(crate) const MAP: u16 = 0xc800;

fn entry(id: u32, appearance: AppearanceId, x: u16, y: u16) -> PopulationEntry {
    PopulationEntry {
        id: PopulationEntryId(id),
        species: SPECIES,
        appearance: Some(appearance),
        position: Point { x, y },
        initial_program: PROGRAM,
        properties: 0x2800,
        extra_properties: 0,
        parameters: [id as u16, 0],
    }
}
pub(crate) fn binding(reversed: bool) -> Arc<WorldBinding> {
    bound(reversed, None)
}
pub(crate) fn binding_with_tables(reversed: bool, table: u8) -> Arc<WorldBinding> {
    bound(reversed, Some(table))
}
fn bound(reversed: bool, table: Option<u8>) -> Arc<WorldBinding> {
    let (mut world, mut keys) = crate::room_world_fixture::definition(false);
    world
        .resources
        .insert(DEST_POP, ResourceKind::EnemyPopulation);
    world
        .resources
        .insert(DEST_TILES, ResourceKind::EnemyTilesets);
    keys.resources.insert(
        DEST_POP,
        ResourceKey::Word {
            bank: 0xa1,
            offset: 0x9200,
        },
    );
    keys.resources.insert(
        DEST_TILES,
        ResourceKey::Word {
            bank: 0xb4,
            offset: 0x9200,
        },
    );
    let destination = world
        .states
        .get_mut(&crate::room_world_fixture::STATE_B)
        .unwrap();
    destination.enemy_population = DEST_POP;
    destination.enemy_tilesets = DEST_TILES;
    for i in 0..3 {
        world.resources.insert(PALETTES[i], ResourceKind::Palette);
        world
            .resources
            .insert(SPRITES[i], ResourceKind::ActorSprites);
        keys.resources.insert(
            PALETTES[i],
            ResourceKey::Long(LegacyAddress24([0, 0xc0 + i as u8, 0xa2])),
        );
        keys.resources.insert(
            SPRITES[i],
            ResourceKey::Long(LegacyAddress24([0, 0xc4 + i as u8, 0xa2])),
        );
    }
    let mut entries = vec![
        entry(0x190000e1, APPEARANCES[0], 0x140, 0x240),
        entry(0x09000017, APPEARANCES[1], 0x180, 0x240),
    ];
    if reversed {
        entries.reverse();
    }
    let definitions = ActorDefinitions {
        species: BTreeMap::from([(
            SPECIES,
            SpeciesDefinition {
                initial_health: 99,
                contact_damage: 3,
                radius: Point { x: 8, y: 12 },
                hurt_time: 4,
                layer: 2,
                behavior: BehaviorId(8),
                default_appearance: APPEARANCES[0],
                multipart_count: 1,
            },
        )]),
        programs: BTreeMap::from([(PROGRAM, ActorProgramDefinition::CompatibleEnemyScript)]),
        populations: BTreeMap::from([
            (
                POP_A,
                PopulationDefinition {
                    entries,
                    death_quota: 0,
                },
            ),
            (
                POP_B,
                PopulationDefinition {
                    entries: vec![entry(0x490000b3, APPEARANCES[2], 0x240, 0x340)],
                    death_quota: 0,
                },
            ),
        ]),
        templates: BTreeMap::from([(TEMPLATE, entry(0x590000d5, APPEARANCES[1], 0x1c0, 0x240))]),
        poses: BTreeMap::from([(
            POSE,
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
                // This ordinary-body fixture never uses extended hitboxes.
                hitboxes: Vec::new(),
            }]),
        )]),
    };
    let mut actor_keys = ActorCompatibilityKeys::default();
    actor_keys.species.insert(SPECIES, 0xd000);
    actor_keys.behaviors.insert(BehaviorId(8), 0xd000);
    actor_keys.programs.insert(PROGRAM, (0xa2, 0xe000));
    actor_keys
        .populations
        .extend([(POP_A, 0x9000), (POP_B, 0x9200)]);
    actor_keys.room_populations.extend([
        (crate::room_world_fixture::ENEMIES, POP_A),
        (DEST_POP, POP_B),
    ]);
    actor_keys.templates.insert(TEMPLATE, (0xa1, 0x9400));
    actor_keys.poses.insert(POSE, (0xa2, MAP));
    let media = ActorMediaDefinitions {
        appearances: (0..3)
            .map(|i| {
                (
                    APPEARANCES[i],
                    AppearanceDefinition {
                        frames: BTreeMap::from([((POSE, PartRoleId(0)), FrameId(30 + i as u32))]),
                    },
                )
            })
            .collect(),
        frames: (0..3)
            .map(|i| {
                (
                    FrameId(30 + i as u32),
                    FrameDefinition::Sprites(vec![VisualPieceDefinition {
                        resource: VisualResourceId(40 + i as u32),
                        source_origin: Point { x: 0, y: 0 },
                        source_extent: Point { x: 0, y: 0 },
                        offset: Point { x: 0, y: 0 },
                        flip_x: false,
                        flip_y: false,
                        material: MaterialId(50 + i as u32),
                    }]),
                )
            })
            .collect(),
        materials: (0..3)
            .map(|i| {
                (
                    MaterialId(50 + i as u32),
                    ActorMaterialDefinition {
                        palette: VisualResourceId(60 + i as u32),
                        cgram_row: if i == 1 { 10 } else { 9 },
                    },
                )
            })
            .collect(),
        resources: (0..3)
            .flat_map(|i| {
                [
                    (
                        VisualResourceId(40 + i as u32),
                        ActorVisualResource::CompatibleSprites {
                            resource: SPRITES[i],
                        },
                    ),
                    (
                        VisualResourceId(60 + i as u32),
                        ActorVisualResource::CompatiblePalette {
                            resource: PALETTES[i],
                        },
                    ),
                ]
            })
            .collect(),
    };
    let tables = table.map(|value| {
        for room in world.rooms.values_mut() {
            room.cre = 2;
        }
        crate::room_world_resources::tables(&world, &keys, value)
    });
    let binding = WorldBinding::prepared_with_actors(world, keys, definitions, actor_keys, media)
        .expect("composed authored fixture");
    match tables {
        Some(tables) => WorldBinding::with_tables(binding, tables).unwrap(),
        None => binding,
    }
}
fn write(image: &crate::rom::RomImage, address: u32, bytes: &[u8]) {
    let base = crate::rom::lorom_offset(address) as usize;
    for (i, &byte) in bytes.iter().enumerate() {
        image.write(base + i, byte);
    }
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn definition(palette: u16) -> Vec<u8> {
    use crate::types::EnemyDef;
    use std::mem::{offset_of, size_of};
    let mut bytes = vec![0; size_of::<EnemyDef>()];
    for (at, value) in [
        (offset_of!(EnemyDef, tile_data_size), 32),
        (offset_of!(EnemyDef, palette_ptr), palette),
        (offset_of!(EnemyDef, health), 99),
        (offset_of!(EnemyDef, damage), 3),
        (offset_of!(EnemyDef, x_radius), 8),
        (offset_of!(EnemyDef, y_radius), 12),
        (offset_of!(EnemyDef, num_parts), 1),
        (offset_of!(EnemyDef, ai_init), 0x804c),
        (offset_of!(EnemyDef, main_ai), 0x804c),
        (offset_of!(EnemyDef, grapple_ai), 0x804c),
        (offset_of!(EnemyDef, hurt_ai), 0x804c),
        (offset_of!(EnemyDef, frozen_ai), 0x804c),
        (offset_of!(EnemyDef, time_is_frozen_ai), 0x804c),
        (offset_of!(EnemyDef, touch_ai), 0x804c),
        (offset_of!(EnemyDef, shot_ai), 0x804c),
    ] {
        put(&mut bytes, at, value);
    }
    bytes[offset_of!(EnemyDef, bank)] = 0xa2;
    bytes[offset_of!(EnemyDef, hurt_ai_time)] = 4;
    bytes[offset_of!(EnemyDef, layer)] = 2;
    let at = offset_of!(EnemyDef, tile_data);
    bytes[at..at + 3].copy_from_slice(&[0, 0xd0, 0xb4]);
    bytes
}
pub(crate) fn install(g: &mut Game) {
    crate::room_world_resources::install(g);
    let loaded = g.loaded_rom.clone().expect("initialized loaded image");
    let native = g.capture_native_rom();
    // Both actual consumer allocations are initialized; do not register only loaded ROM.
    for image in [&loaded, &native] {
        for i in 0..3 {
            write(
                image,
                0xa0d000 + i * 0x100,
                &definition(0xc000 + i as u16 * 0x100),
            );
            write(image, 0xa2c000 + i * 0x100, &[0x11 + i as u8; 32]);
            // One small OAM entry: count1, x0, y0, tile1/2/3. Palette bits initially clear.
            write(
                image,
                0xa2c400 + i * 0x100,
                &[1, 0, 0, 0, 0, 1 + i as u8, 0],
            );
        }
        write(image, 0xa2e000, &[0xff, 0x7f, 0x00, 0xc8]);
        write(image, 0xa2c800, &[1, 0, 0, 0, 0, 1, 0]);
        write(image, 0xb4d000, &[0x5a; 32]);
        write(image, 0xb49000, &[0, 0xd0, 1, 0, 0, 0xd1, 2, 0, 0xff, 0xff]);
        write(image, 0xb49200, &[0, 0xd2, 1, 0, 0xff, 0xff]);
        // Authored populations supply all scalar records; raw capture backing stays finite.
        write(image, 0xa19000, &[0; 36]);
        write(image, 0xa19200, &[0; 20]);
        write(image, 0xa19400, &[0; 16]);
    }
}
