//! Finite live actor content acquisition at the retained backend boundary.
#![forbid(unsafe_code)]
use crate::{
    rom::{RomImage, RomRecord, lorom_offset},
    types::{EnemyDef, ExtendedSpriteMap, Hitbox},
};
use core::mem::{offset_of, size_of};
use sm_game::actors::{
    access::{AccessMode, ActorAccess, ActorKey},
    content::*,
};
use std::sync::Arc;
use {crate::game::session_runtime::Runtime, sm_game::room_entry::world::WorldBinding};

fn live(record: RomRecord) -> ActorLiveRecord {
    ActorLiveRecord {
        image: record.image,
        base: record.base,
    }
}
fn byte(record: &ActorLiveRecord, offset: usize) -> u8 {
    record.image.read(
        record
            .base
            .checked_add(offset)
            .expect("ROM record offset overflow"),
    )
}
fn word(record: &ActorLiveRecord, offset: usize) -> u16 {
    u16::from_ne_bytes([byte(record, offset), byte(record, offset + 1)])
}
fn source<P>(
    world: Arc<WorldBinding>,
    position: Option<P>,
    record: ActorLiveRecord,
) -> ActorRecordCapture<P> {
    ActorRecordCapture {
        world,
        position,
        record,
    }
}
fn population_position(
    world: &WorldBinding,
    bank: u8,
    address: u16,
    room_key: u16,
) -> Option<PopulationPosition> {
    if bank != 0xa1 {
        return None;
    }
    let catalog = world.actors();
    let keys = catalog.keys()?;
    let definitions = catalog.definitions()?;
    // Prefer the actual room population when compatible streams overlap.
    let candidates = catalog
        .population_for_key(room_key)
        .into_iter()
        .chain(keys.populations.keys().copied());
    for population in candidates {
        let base = keys.populations[&population];
        let delta = address.wrapping_sub(base);
        if delta % 16 == 0 {
            let entry = usize::from(delta / 16);
            if entry <= definitions.populations[&population].entries.len() {
                return Some(PopulationPosition::Room { population, entry });
            }
        }
    }
    None
}
fn prepared_entry(source: &ActorPopulationCapture) -> Option<&PopulationEntry> {
    source
        .position
        .and_then(|position| source.world.actors().entry(position))
}
fn prepared_species(source: &ActorDefinitionCapture) -> Option<&SpeciesDefinition> {
    source
        .position
        .and_then(|id| source.world.actors().definitions()?.species.get(&id))
}
fn prepared_pose(source: &ActorPoseCapture) -> Option<&PoseDefinition> {
    source
        .position
        .and_then(|id| source.world.actors().definitions()?.poses.get(&id))
}
fn prepared_collision(source: &ActorCollisionCapture) -> Option<&CollisionPart> {
    let position = source.position?;
    match source
        .world
        .actors()
        .definitions()?
        .collisions
        .get(&position.layout)?
    {
        CollisionLayout::Parts(parts) => parts.get(position.part),
        CollisionLayout::Body { .. } => None,
    }
}

impl ActorContent for Runtime<'_> {
    fn actor_script_image(&self) -> RomImage {
        self.loaded_image().expect("ROM must be loaded")
    }
    fn capture_actor_population(&mut self, cursor: &PopulationCursor) -> ActorPopulationCapture {
        let world = self.session().world.clone();
        let (bank, address, position) = match *cursor {
            PopulationCursor::Legacy { bank, address } => (
                bank,
                address,
                population_position(
                    &world,
                    bank,
                    address,
                    self.session().room.entry.enemy_population_key,
                ),
            ),
            PopulationCursor::Prepared { population, entry } => {
                let base = world
                    .actors()
                    .keys()
                    .expect("prepared actor keys")
                    .populations[&population];
                (
                    0xa1,
                    base.wrapping_add((entry as u16).wrapping_mul(16)),
                    Some(PopulationPosition::Room { population, entry }),
                )
            }
            PopulationCursor::Template { template, part } => {
                let (bank, base) = world
                    .actors()
                    .keys()
                    .expect("prepared actor keys")
                    .templates[&template];
                (
                    bank,
                    base.wrapping_add((part as u16).wrapping_mul(16)),
                    Some(PopulationPosition::Template { template, part }),
                )
            }
        };
        let record = live(self.actor_legacy_record(bank, address));
        ActorPopulationCapture {
            world,
            position,
            record,
            cursor: cursor.clone(),
        }
    }
    fn capture_actor_definition(&mut self, key: ActorDefinitionKey) -> ActorDefinitionCapture {
        let world = self.session().world.clone();
        let position = world.actors().species_for_key(key.0);
        let record = live(self.actor_legacy_record(0xa0, key.0));
        source(world, position, record)
    }
    fn capture_actor_pose(&mut self, actor: ActorKey) -> ActorPoseCapture {
        let bank = self.resolve_actor(actor, AccessMode::Native).bank;
        let address = self.resolve_actor(actor, AccessMode::Native).spritemap;
        let world = self.session().world.clone();
        let position = world.actors().pose_for_key(bank, address);
        let record = live(self.actor_legacy_record(bank, address));
        source(world, position, record)
    }
    fn actor_pose_count(&mut self, captured: &ActorPoseCapture, width: ActorCountWidth) -> u16 {
        if let Some(pose) = prepared_pose(captured) {
            let count =
                u16::try_from(pose.parts.len()).expect("actor pose count fits compatible word");
            return match width {
                ActorCountWidth::Byte => u16::from(count as u8),
                ActorCountWidth::NativeWord => count,
            };
        }
        match width {
            ActorCountWidth::Byte => u16::from(byte(&captured.record, 0)),
            ActorCountWidth::NativeWord => word(&captured.record, 0),
        }
    }
    fn capture_actor_part_at(
        &mut self,
        captured: &ActorPoseCapture,
        part: usize,
        traversal: ActorPartTraversal,
    ) -> ActorPartCapture {
        let record = match traversal {
            ActorPartTraversal::CapturedLinear => ActorLiveRecord {
                image: captured.record.image.clone(),
                base: captured
                    .record
                    .base
                    .checked_add(
                        2 + part
                            .checked_mul(size_of::<ExtendedSpriteMap>())
                            .expect("actor part extent"),
                    )
                    .expect("actor part extent"),
            },
            ActorPartTraversal::WrappingReacquire { bank, address } => {
                live(self.actor_legacy_record(bank, address))
            }
        };
        source(
            captured.world.clone(),
            captured
                .position
                .map(|pose| PosePartPosition { pose, part }),
            record,
        )
    }
    fn actor_part_word(&mut self, captured: &ActorPartCapture, field: ActorPartWord) -> u16 {
        if let Some(position) = captured.position {
            let pose = &captured
                .world
                .actors()
                .definitions()
                .expect("prepared actors")
                .poses[&position.pose];
            if let Some(part) = pose.parts.get(position.part) {
                match field {
                    ActorPartWord::X => return part.offset.x,
                    ActorPartWord::Y => return part.offset.y,
                    _ => {}
                }
            }
        }
        word(
            &captured.record,
            match field {
                ActorPartWord::X => offset_of!(ExtendedSpriteMap, xpos),
                ActorPartWord::Y => offset_of!(ExtendedSpriteMap, ypos),
                ActorPartWord::Map => offset_of!(ExtendedSpriteMap, spritemap),
                ActorPartWord::Hitboxes => offset_of!(ExtendedSpriteMap, hitbox_ptr_),
            },
        )
    }
    fn capture_actor_collision_at(
        &mut self,
        captured: &ActorPartCapture,
        bank: u8,
    ) -> ActorCollisionCapture {
        let address = self.actor_part_word(captured, ActorPartWord::Hitboxes);
        let record = live(self.actor_legacy_record(bank, address));
        let position = captured.position.map(|position| CollisionPosition {
            layout: captured
                .world
                .actors()
                .definitions()
                .expect("prepared actors")
                .poses[&position.pose]
                .collision,
            part: position.part,
            hitbox: 0,
        });
        source(captured.world.clone(), position, record)
    }
    fn actor_collision_count(&mut self, captured: &ActorCollisionCapture) -> u16 {
        if let Some(part) = prepared_collision(captured) {
            return u16::try_from(part.hitboxes.len())
                .expect("actor hitbox count fits compatible word");
        }
        word(&captured.record, 0)
    }
    fn actor_collision_word(
        &mut self,
        captured: &ActorCollisionCapture,
        index: usize,
        field: ActorCollisionWord,
    ) -> u16 {
        if let Some(part) = prepared_collision(captured) {
            let hit = &part.hitboxes[index];
            return match field {
                ActorCollisionWord::Left => hit.left,
                ActorCollisionWord::Right => hit.right,
                ActorCollisionWord::Top => hit.top,
                ActorCollisionWord::Bottom => hit.bottom,
                ActorCollisionWord::Touch | ActorCollisionWord::Shot => {
                    panic!("use full actor callback target")
                }
            };
        }
        let offset = match field {
            ActorCollisionWord::Left => offset_of!(Hitbox, left),
            ActorCollisionWord::Right => offset_of!(Hitbox, right),
            ActorCollisionWord::Top => offset_of!(Hitbox, top),
            ActorCollisionWord::Bottom => offset_of!(Hitbox, bottom),
            ActorCollisionWord::Touch => offset_of!(Hitbox, func_ptr),
            ActorCollisionWord::Shot => offset_of!(Hitbox, func_ptrA),
        };
        word(
            &captured.record,
            2 + index
                .checked_mul(size_of::<Hitbox>())
                .expect("actor hitbox extent")
                + offset,
        )
    }
    fn actor_collision_target(
        &mut self,
        captured: &ActorCollisionCapture,
        index: usize,
        field: ActorCollisionWord,
    ) -> ActorCallbackTarget {
        if let Some(part) = prepared_collision(captured) {
            let hit = &part.hitboxes[index];
            let id = match field {
                ActorCollisionWord::Touch => hit.touch,
                ActorCollisionWord::Shot => hit.shot,
                _ => panic!("actor callback field required"),
            };
            let address = captured
                .world
                .actors()
                .keys()
                .expect("prepared actor keys")
                .callbacks[&id];
            ActorCallbackTarget::Prepared(ActorRoutine::from_legacy(address))
        } else {
            ActorCallbackTarget::CompatibleOffset(self.actor_collision_word(captured, index, field))
        }
    }
    fn actor_visual_header(&mut self, bank: u8, address: u16) -> u16 {
        self.actor_legacy_record(bank, address).word(0)
    }
    fn actor_draw_queue_base(&self, layer: u16) -> u16 {
        self.actor_script_image().table_word(
            lorom_offset(0xa0b133) as usize,
            isize::try_from(layer).unwrap(),
        )
    }
    fn actor_spike_mechanism(&mut self, variant: u16) -> u16 {
        let image = self.actor_script_image();
        let at = loaded_table_offset(&image, 0xa0c2da, usize::from(variant) * 2, 2);
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
    fn actor_slope_multiplier(&mut self, index: u16) -> u16 {
        let image = self.actor_script_image();
        let at = loaded_table_offset(&image, 0xa0c49f, usize::from(index) * 2, 2);
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
    fn actor_slope_surface(&mut self, index: u16) -> u16 {
        let image = self.actor_script_image();
        u16::from(image.read(loaded_table_offset(&image, 0x948b2b, usize::from(index), 1)))
    }
    fn actor_slope_surface_native(&mut self, index: u16) -> u8 {
        self.native_image()
            .read(lorom_offset(0x948b2b) as usize + usize::from(index))
    }
    fn actor_sine_sample(&mut self, phase: u8) -> u8 {
        let image = self.actor_script_image();
        image.read(loaded_table_offset(
            &image,
            0xa0b143,
            usize::from(phase & 0x7f),
            1,
        ))
    }
}

impl ActorCaptureReads for Runtime<'_> {
    fn population_definition(&mut self, captured: &ActorPopulationCapture) -> u16 {
        if let Some(entry) = prepared_entry(captured) {
            return captured
                .world
                .actors()
                .keys()
                .expect("prepared actor keys")
                .species[&entry.species];
        }
        if matches!(captured.position, Some(PopulationPosition::Room { .. })) {
            return 0xffff;
        }
        word(&captured.record, 0)
    }
    fn population_word(&mut self, captured: &ActorPopulationCapture, field: PopulationWord) -> u16 {
        if let Some(entry) = prepared_entry(captured) {
            return match field {
                PopulationWord::X => entry.position.x,
                PopulationWord::Y => entry.position.y,
                PopulationWord::Program => {
                    captured
                        .world
                        .actors()
                        .keys()
                        .expect("prepared actor keys")
                        .programs[&entry.initial_program]
                        .1
                }
                PopulationWord::Properties => entry.properties,
                PopulationWord::ExtraProperties => entry.extra_properties,
                PopulationWord::Parameter1 => entry.parameters[0],
                PopulationWord::Parameter2 => entry.parameters[1],
            };
        }
        word(
            &captured.record,
            match field {
                PopulationWord::X => 2,
                PopulationWord::Y => 4,
                PopulationWord::Program => 6,
                PopulationWord::Properties => 8,
                PopulationWord::ExtraProperties => 10,
                PopulationWord::Parameter1 => 12,
                PopulationWord::Parameter2 => 14,
            },
        )
    }
    fn population_terminated(&mut self, captured: &ActorPopulationCapture) -> bool {
        self.population_definition(captured) == 0xffff
    }
    fn population_death_quota(&mut self, captured: &ActorPopulationCapture) -> u16 {
        if let Some(PopulationPosition::Room { population, .. }) = captured.position {
            return captured
                .world
                .actors()
                .definitions()
                .expect("prepared actors")
                .populations[&population]
                .death_quota;
        }
        u16::from(byte(&captured.record, 2))
    }
    fn definition_word(&mut self, captured: &ActorDefinitionCapture, field: DefinitionWord) -> u16 {
        if let Some(species) = prepared_species(captured) {
            match field {
                DefinitionWord::Width => return species.radius.x,
                DefinitionWord::Height => return species.radius.y,
                DefinitionWord::Health => return species.initial_health,
                DefinitionWord::Damage => return species.contact_damage,
                DefinitionWord::MultipartCount => return species.multipart_count,
                DefinitionWord::NativeBank => {
                    return u16::from_ne_bytes([
                        byte(&captured.record, offset_of!(EnemyDef, bank)),
                        species.hurt_time,
                    ]);
                }
                _ => {}
            }
        }
        word(
            &captured.record,
            match field {
                DefinitionWord::Width => offset_of!(EnemyDef, x_radius),
                DefinitionWord::Height => offset_of!(EnemyDef, y_radius),
                DefinitionWord::Health => offset_of!(EnemyDef, health),
                DefinitionWord::NativeBank => offset_of!(EnemyDef, bank),
                DefinitionWord::Damage => offset_of!(EnemyDef, damage),
                DefinitionWord::HurtSound => offset_of!(EnemyDef, hurt_sfx),
                DefinitionWord::DeathAnimation => offset_of!(EnemyDef, death_anim),
                DefinitionWord::MultipartCount => offset_of!(EnemyDef, num_parts),
                DefinitionWord::BossFight => offset_of!(EnemyDef, boss_fight_value),
            },
        )
    }
    fn definition_byte(&mut self, captured: &ActorDefinitionCapture, field: DefinitionByte) -> u8 {
        if let Some(species) = prepared_species(captured) {
            match field {
                DefinitionByte::HurtTime => return species.hurt_time,
                DefinitionByte::Layer => return species.layer,
                DefinitionByte::Bank => {}
            }
        }
        byte(
            &captured.record,
            match field {
                DefinitionByte::Bank => offset_of!(EnemyDef, bank),
                DefinitionByte::HurtTime => offset_of!(EnemyDef, hurt_ai_time),
                DefinitionByte::Layer => offset_of!(EnemyDef, layer),
            },
        )
    }
    fn definition_routine_offset(
        &mut self,
        captured: &ActorDefinitionCapture,
        routine: DefinitionRoutine,
    ) -> u16 {
        word(
            &captured.record,
            match routine {
                DefinitionRoutine::Initialize => offset_of!(EnemyDef, ai_init),
                DefinitionRoutine::Main => offset_of!(EnemyDef, main_ai),
                DefinitionRoutine::Grapple => offset_of!(EnemyDef, grapple_ai),
                DefinitionRoutine::Hurt => offset_of!(EnemyDef, hurt_ai),
                DefinitionRoutine::Frozen => offset_of!(EnemyDef, frozen_ai),
                DefinitionRoutine::TimeFrozen => offset_of!(EnemyDef, time_is_frozen_ai),
                DefinitionRoutine::Touch => offset_of!(EnemyDef, touch_ai),
                DefinitionRoutine::Shot => offset_of!(EnemyDef, shot_ai),
                DefinitionRoutine::PowerBomb => offset_of!(EnemyDef, powerbomb_reaction),
            },
        )
    }
    fn definition_name(&mut self, captured: &ActorDefinitionCapture) -> Option<ActorLiveRecord> {
        let address = word(&captured.record, offset_of!(EnemyDef, name_ptr));
        (address != 0).then(|| live(self.actor_legacy_record(0xb4, address)))
    }
    fn name_word(&mut self, captured: &ActorLiveRecord, offset: ActorNameWord) -> u16 {
        word(
            captured,
            match offset {
                ActorNameWord::At0 => 0,
                ActorNameWord::At2 => 2,
                ActorNameWord::At4 => 4,
                ActorNameWord::At6 => 6,
                ActorNameWord::At8 => 8,
                ActorNameWord::At12 => 12,
            },
        )
    }
    fn definition_palette(&mut self, captured: &ActorDefinitionCapture) -> ActorLiveRecord {
        let bank = byte(&captured.record, offset_of!(EnemyDef, bank));
        let address = word(&captured.record, offset_of!(EnemyDef, palette_ptr));
        live(self.actor_legacy_record(bank, address))
    }
    fn palette_bytes(&mut self, captured: &ActorLiveRecord) -> [u8; 32] {
        core::array::from_fn(|offset| byte(captured, offset))
    }
    fn definition_tile_size(&mut self, captured: &ActorDefinitionCapture) -> u16 {
        word(&captured.record, offset_of!(EnemyDef, tile_data_size))
    }
    fn definition_tile_source(&mut self, captured: &ActorDefinitionCapture) -> [u8; 3] {
        core::array::from_fn(|offset| {
            byte(&captured.record, offset_of!(EnemyDef, tile_data) + offset)
        })
    }
    fn capture_tile_chunk(&mut self, source: [u8; 3], offset: usize) -> ActorLiveRecord {
        let address = u16::from_le_bytes([source[0], source[1]]).wrapping_add(offset as u16);
        live(self.actor_legacy_record(source[2], address))
    }
    fn capture_tilesets(&mut self, key: u16) -> ActorLiveRecord {
        live(self.actor_legacy_record(0xb4, key))
    }
    fn tileset_definition(&mut self, captured: &ActorLiveRecord, index: usize) -> u16 {
        word(captured, index.checked_mul(4).expect("tileset extent"))
    }
    fn tileset_destination_word(&mut self, captured: &ActorLiveRecord, index: usize) -> u16 {
        word(captured, index.checked_mul(4).expect("tileset extent") + 2)
    }
    fn tileset_destination_byte(&mut self, captured: &ActorLiveRecord, index: usize) -> u8 {
        byte(captured, index.checked_mul(4).expect("tileset extent") + 2)
    }
    fn capture_graphics_tileset(&mut self, address: u16) -> ActorLiveRecord {
        live(self.actor_legacy_record(0xb4, address))
    }
    fn definition_vulnerability_address(&mut self, captured: &ActorDefinitionCapture) -> u16 {
        let address = word(&captured.record, offset_of!(EnemyDef, vulnerability_ptr));
        if address == 0 { 0xec1c } else { address }
    }
    fn capture_vulnerability_at(&mut self, address: u16) -> ActorLiveRecord {
        live(self.actor_legacy_record(0xb4, address))
    }
    fn contact_vulnerability(&mut self, captured: &ActorLiveRecord, wrapping_offset: u16) -> u16 {
        let relocated = vulnerability_offset(captured, wrapping_offset);
        word(&relocated, 0)
    }
    fn projectile_vulnerability(
        &mut self,
        captured: &ActorLiveRecord,
        wrapping_offset: u16,
        linear_field: usize,
    ) -> u8 {
        let relocated = vulnerability_offset(captured, wrapping_offset);
        byte(&relocated, linear_field)
    }
}
fn vulnerability_offset(captured: &ActorLiveRecord, offset: u16) -> ActorLiveRecord {
    // B4 is fixed for this source. Adding before mapping is distinct from the
    // projectile field addition, which is linear after this captured address.
    let address = ((captured.base & 0x7fff) as u16).wrapping_add(offset);
    ActorLiveRecord {
        image: captured.image.clone(),
        base: lorom_offset(0xb40000 | u32::from(address)) as usize,
    }
}

fn loaded_table_offset(image: &RomImage, address: u32, offset: usize, width: usize) -> usize {
    assert!(
        address & 0x8000 != 0,
        "ROM address must select a LoROM window"
    );
    let start = (lorom_offset(address) as usize)
        .checked_add(offset)
        .expect("ROM offset overflow");
    let end = start.checked_add(width).expect("ROM offset overflow");
    assert!(end <= image.len(), "ROM read outside loaded allocation");
    start
}
