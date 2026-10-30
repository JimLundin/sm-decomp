//! Population creation, shared slot initialization and semantic birth boundaries.
#![forbid(unsafe_code)]
use crate::enemy_population::{Creation, EnemyShape, PopulationEntry};
use {
    crate::actors::identity::{InstanceHandle, PendingActorBirth},
    crate::actors::{
        access::{AccessMode::Native, ActorKey},
        content::{
            ActorDefinitionCapture, ActorDefinitionKey, ActorPopulationCapture, ActorRoutine,
            DefinitionByte as DB, DefinitionRoutine, DefinitionWord as DW, PopulationWord as PW,
        },
        effects::ActorHost,
    },
};

fn read_entry<H: ActorHost>(h: &mut H, entry: &ActorPopulationCapture) -> PopulationEntry {
    PopulationEntry {
        definition: h.population_definition(entry),
        x: h.population_word(entry, PW::X),
        y: h.population_word(entry, PW::Y),
        instruction: h.population_word(entry, PW::Program),
        properties: h.population_word(entry, PW::Properties),
        extra_properties: h.population_word(entry, PW::ExtraProperties),
        parameters: [
            h.population_word(entry, PW::Parameter1),
            h.population_word(entry, PW::Parameter2),
        ],
    }
}
fn read_shape<H: ActorHost>(h: &mut H, definition: &ActorDefinitionCapture) -> EnemyShape {
    EnemyShape {
        width: h.definition_word(definition, DW::Width),
        height: h.definition_word(definition, DW::Height),
        health: h.definition_word(definition, DW::Health),
        layer: h.definition_byte(definition, DB::Layer),
        bank: h.definition_byte(definition, DB::Bank),
        hurt_time: h.definition_byte(definition, DB::HurtTime),
    }
}
fn configure<H: ActorHost>(h: &mut H, key: ActorKey, shape: EnemyShape) {
    h.edit_actor(key, Native, |a| a.radius.x = shape.width);
    h.edit_actor(key, Native, |a| a.radius.y = shape.height);
    h.edit_actor(key, Native, |a| a.health = shape.health);
    h.edit_actor(key, Native, |a| a.layer = u16::from(shape.layer));
    h.edit_actor(key, Native, |a| {
        a.bank = shape.bank;
        a.hurt_time = shape.hurt_time;
    });
}
/// Metadata is checked but unpublished before the first definition store.
fn place_entry<H: ActorHost>(
    h: &mut H,
    key: ActorKey,
    entry: PopulationEntry,
    birth: Option<PendingActorBirth>,
) -> Option<InstanceHandle> {
    h.edit_actor(key, Native, |a| a.definition = entry.definition);
    let handle = birth.map(|birth| h.session_mut().actor_identity.commit_birth(birth));
    h.edit_actor(key, Native, |a| a.x.whole = entry.x);
    h.edit_actor(key, Native, |a| a.y.whole = entry.y);
    h.edit_actor(key, Native, |a| a.instruction = entry.instruction);
    h.edit_actor(key, Native, |a| a.properties = entry.properties);
    h.edit_actor(key, Native, |a| a.extra_properties = entry.extra_properties);
    h.edit_actor(key, Native, |a| a.parameters[0] = entry.parameters[0]);
    h.edit_actor(key, Native, |a| a.parameters[1] = entry.parameters[1]);
    handle
}
fn place<H: ActorHost>(
    h: &mut H,
    key: ActorKey,
    entry: PopulationEntry,
    creation: Creation,
    birth: Option<PendingActorBirth>,
) -> Option<InstanceHandle> {
    let handle = place_entry(h, key, entry, birth);
    h.edit_actor(key, Native, |a| a.frame = 0);
    h.edit_actor(key, Native, |a| a.timer = 0);
    if creation.clear_script_variables() {
        h.edit_actor(key, Native, |a| {
            a.variables.fill(0);
            a.pre_instruction = 0;
        });
    }
    h.edit_actor(key, Native, |a| a.instruction_delay = 1);
    h.edit_actor(key, Native, |a| a.frame = 0);
    handle
}
fn finish_initialization<H: ActorHost>(
    h: &mut H,
    key: ActorKey,
    definition: &ActorDefinitionCapture,
    creation: Creation,
) {
    record_spawn(h, key);
    h.session_mut().actors.controls.selected = key.byte_index();
    let function = h.definition_routine_offset(definition, DefinitionRoutine::Initialize);
    if creation.run_initializer(function) {
        let bank = h.definition_byte(definition, DB::Bank);
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(6, key.byte_index());
        h.actor_ai(ActorRoutine::from_bank_offset(bank, function));
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(7, key.byte_index());
    }
    let properties = h.resolve_actor(key, Native).properties;
    let extra = h.resolve_actor(key, Native).extra_properties;
    use crate::constants::{bank_a0, bank_a4};
    let empty = match creation {
        Creation::Room => bank_a4::ADDR_K_SPRITEMAP_NOTHING_A4,
        Creation::Dynamic => bank_a0::ADDR_K_SPRITEMAP_NOTHING_A0,
    } as u16;
    if let Some(map) = creation.initial_spritemap(
        properties,
        extra,
        empty,
        bank_a4::ADDR_K_EXTENDED_SPRITEMAP_NOTHING_A4 as u16,
    ) {
        h.edit_actor(key, Native, |a| a.spritemap = map);
    }
}

pub fn delete_connected<H: ActorHost>(h: &mut H) {
    let key = ActorKey::from_legacy(h.session().actors.controls.selected);
    let raw = h.resolve_actor(key, Native).definition;
    let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
    let parts = h.definition_word(&definition, DW::MultipartCount).max(1);
    for _ in 0..parts {
        let key = ActorKey::from_legacy(h.session().actors.controls.selected);
        h.edit_actor(key, Native, |a| a.definition = 0);
        if key.byte_index() < 2048 && key.byte_index() & 63 == 0 {
            h.session_mut().actor_identity.retire(key.byte_index() >> 6);
        }
        let selected = h.session().actors.controls.selected.wrapping_add(64);
        h.session_mut().actors.controls.selected = selected;
    }
}

/// Each save is a separate source observation and publication, including odd legacy windows.
pub fn record_spawn<H: ActorHost>(h: &mut H, key: ActorKey) {
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(8, key.byte_index());
    let value = h.resolve_actor(key, Native).definition;
    h.edit_spawn(key, Native, |s| s.definition = value);
    let value = h.resolve_actor(key, Native).x.whole;
    h.edit_spawn(key, Native, |s| s.position.x = value);
    let value = h.resolve_actor(key, Native).y.whole;
    h.edit_spawn(key, Native, |s| s.position.y = value);
    let value = h.resolve_actor(key, Native).instruction;
    h.edit_spawn(key, Native, |s| s.instruction = value);
    let value = h.resolve_actor(key, Native).properties;
    h.edit_spawn(key, Native, |s| s.properties = value);
    let value = h.resolve_actor(key, Native).extra_properties;
    h.edit_spawn(key, Native, |s| s.extra_properties = value);
    let value = h.resolve_actor(key, Native).parameters[0];
    h.edit_spawn(key, Native, |s| s.parameters[0] = value);
    let value = h.resolve_actor(key, Native).parameters[1];
    h.edit_spawn(key, Native, |s| s.parameters[1] = value);
    let raw = h.resolve_actor(key, Native).definition;
    let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
    let mut words = [0; 6];
    if let Some(text) = h.definition_name(&definition) {
        use crate::actors::content::ActorNameWord::*;
        for (word, field) in words.iter_mut().zip([At0, At2, At4, At6, At8, At12]) {
            *word = h.name_word(&text, field);
        }
    }
    for (i, value) in words.into_iter().enumerate() {
        h.edit_spawn(key, Native, |s| {
            s.name[2 * i..2 * i + 2].copy_from_slice(&value.to_ne_bytes())
        });
    }
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(9, key.byte_index());
}

fn captured_origin(
    cursor: &crate::actors::content::PopulationCursor,
) -> Option<crate::actors::content::PopulationPosition> {
    use crate::actors::content::{PopulationCursor as C, PopulationPosition as P};
    match *cursor {
        C::Prepared { population, entry } => Some(P::Room { population, entry }),
        C::Template { template, part } => Some(P::Template { template, part }),
        C::Legacy { .. } => None,
    }
}
fn prepare_birth<H: ActorHost>(
    h: &H,
    key: ActorKey,
    capture: &ActorPopulationCapture,
    cursor: &crate::actors::content::PopulationCursor,
    definition: u16,
    appearance: Option<crate::actors::media::AppearanceId>,
    respawn: bool,
) -> Option<PendingActorBirth> {
    let (provenance, appearance) = crate::actors::content::birth_metadata_from_capture(
        capture,
        captured_origin(cursor),
        definition,
        appearance,
        respawn,
    )
    .expect("actor population metadata must be valid");
    validate_selected_appearance(h, capture, appearance)
        .expect("actor appearance must be resident");
    h.session()
        .actor_identity
        .prepare_legacy_birth(key, provenance, appearance)
        .expect("actor identity counter overflow")
}

pub fn install_population<H: ActorHost + crate::enemy_projectiles::host::Host>(h: &mut H) {
    h.reset_room_enemy_workspace();
    h.session_mut().actors.controls.population_count = 0;
    h.session_mut().room.enemy_deaths = 0;
    h.session_mut().actors.controls.process_all = 0;
    crate::enemy_projectiles::drops::reset_room_metadata(h);
    let mut cursor = crate::actors::content::PopulationCursor::Legacy {
        bank: 0xa1,
        address: h.session().room.entry.enemy_population_key,
    };
    let check_cursor = crate::actors::content::PopulationCursor::Legacy {
        bank: 0xa1,
        address: h.session().room.entry.enemy_population_key,
    };
    let mut check = h.capture_actor_population(&check_cursor);
    let mut checked_definition = h.population_definition(&check);
    if checked_definition == 0xffff {
        return;
    }
    h.session_mut().actors.controls.population_scratch = 0;
    let mut index = 0u16;
    loop {
        let key = ActorKey::from_legacy(index);
        validate_birth_appearance(h, &check, &cursor, checked_definition, None, false)
            .expect("initial actor appearance must be resident");
        crate::actors::staging::load_graphics(h, &cursor, key);
        let capture = h.capture_actor_population(&cursor);
        let raw = h.population_definition(&capture);
        let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(3, key.byte_index());
        let shape = read_shape(h, &definition);
        configure(h, key, shape);
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(4, key.byte_index());
        let boss = h.definition_word(&definition, DW::BossFight);
        if boss != 0 {
            h.session_mut().actors.schedule.boss_id = boss;
        }
        let entry = read_entry(h, &capture);
        let birth = prepare_birth(h, key, &capture, &cursor, entry.definition, None, false);
        place(h, key, entry, Creation::Room, birth);
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(5, key.byte_index());
        finish_initialization(h, key, &definition, Creation::Room);
        index = index.wrapping_add(64);
        cursor = next_cursor(cursor);
        check = h.capture_actor_population(&cursor);
        checked_definition = h.population_definition(&check);
        if checked_definition == 0xffff {
            break;
        }
    }
    h.session_mut().actors.controls.first_free = index;
    h.session_mut().actors.controls.population_count = index >> 6;
    let end = h.capture_actor_population(&cursor);
    let deaths = h.population_death_quota(&end);
    h.session_mut().room.enemy_quota = deaths;
    h.probe("actors.population.installed.nonempty");
}

pub fn respawn<H: ActorHost + crate::enemy_projectiles::host::Host>(h: &mut H, key: ActorKey) {
    h.session_mut().actors.controls.selected = key.byte_index();
    let cursor = crate::actors::content::PopulationCursor::Legacy {
        bank: 0xa1,
        address: h
            .session()
            .room
            .entry
            .enemy_population_key
            .wrapping_add(key.byte_index() >> 2),
    };
    let population = h.capture_actor_population(&cursor);
    let entry = read_entry(h, &population);
    let birth = prepare_birth(h, key, &population, &cursor, entry.definition, None, true);
    place_entry(h, key, entry, birth);
    let palette = h.resolve_spawn(key, Native).palette;
    h.edit_actor(key, Native, |a| a.palette = palette);
    let tile = h.resolve_spawn(key, Native).tile;
    h.edit_actor(key, Native, |a| a.tile = tile);
    h.edit_actor(key, Native, |a| a.frozen = 0);
    h.edit_actor(key, Native, |a| a.flash = 0);
    h.edit_actor(key, Native, |a| a.invincibility = 0);
    h.edit_actor(key, Native, |a| a.timer = 0);
    h.edit_actor(key, Native, |a| a.frame = 0);
    h.edit_actor(key, Native, |a| {
        a.variables.fill(0);
        a.pre_instruction = 0;
    });
    h.edit_actor(key, Native, |a| a.instruction_delay = 1);
    let raw = h.resolve_actor(key, Native).definition;
    let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(10, key.byte_index());
    let width = h.definition_word(&definition, DW::Width);
    h.edit_actor(key, Native, |a| a.radius.x = width);
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(11, key.byte_index());
    let height = h.definition_word(&definition, DW::Height);
    h.edit_actor(key, Native, |a| a.radius.y = height);
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(12, key.byte_index());
    let health = h.definition_word(&definition, DW::Health);
    h.edit_actor(key, Native, |a| a.health = health);
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(13, key.byte_index());
    let layer = h.definition_byte(&definition, DB::Layer);
    h.edit_actor(key, Native, |a| a.layer = u16::from(layer));
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(14, key.byte_index());
    let bank = h.definition_word(&definition, DW::NativeBank).to_ne_bytes();
    h.edit_actor(key, Native, |a| {
        a.bank = bank[0];
        a.hurt_time = bank[1];
    });
    let bank = h.definition_byte(&definition, DB::Bank);
    let offset = h.definition_routine_offset(&definition, DefinitionRoutine::Initialize);
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(6, key.byte_index());
    h.actor_ai(ActorRoutine::from_bank_offset(bank, offset));
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(7, key.byte_index());
}

fn spawn_from_cursor<H: ActorHost + crate::enemy_projectiles::host::Host>(
    h: &mut H,
    mut cursor: crate::actors::content::PopulationCursor,
    appearance: Option<crate::actors::media::AppearanceId>,
) -> Result<Option<(u16, Option<InstanceHandle>)>, crate::actors::identity::ActorIdentityError> {
    use crate::enemy_population::{SearchResult, SpawnSearch, SpawnSequence};
    let saved = h.session().actors.controls.selected;
    let entry = h.capture_actor_population(&cursor);
    let raw = h.population_definition(&entry);
    let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
    let parts = h.definition_word(&definition, DW::MultipartCount);
    // Validate selected content before any slot or presentation store.
    validate_birth_appearance(h, &entry, &cursor, raw, appearance, false)?;
    let mut search = SpawnSearch::new(parts);
    let mut index = loop {
        let occupied = h
            .resolve_actor(ActorKey::from_legacy(search.probe), Native)
            .definition
            != 0;
        match search.observe(occupied) {
            SearchResult::Continue => {}
            SearchResult::Found(index) => break index,
            SearchResult::Full => return Ok(None),
        }
    };
    let mut sequence = SpawnSequence::new(index, 0, parts);
    // Each part assigns its handle; the loop only exits after one.
    let mut last_handle;
    loop {
        let key = ActorKey::from_legacy(index);
        let capture = h.capture_actor_population(&cursor);
        let mut graphics = None;
        let mut observed_definition = 0;
        for i in 0..4 {
            let definition = h.population_definition(&capture);
            observed_definition = definition;
            if definition == h.session().actors.graphics.definitions[i] {
                graphics = Some(i);
                break;
            }
        }
        validate_birth_appearance(h, &capture, &cursor, observed_definition, appearance, false)?;
        let (tiles, palette) = graphics
            .map(|i| {
                (
                    h.session().actors.graphics.tile_offsets[i],
                    h.session().actors.graphics.palette_routes[i]
                        .swap_bytes()
                        .wrapping_mul(2),
                )
            })
            .unwrap_or((0, 0));
        h.edit_actor(key, Native, |a| a.tile = tiles);
        h.edit_actor(key, Native, |a| a.palette = palette);
        let raw = h.population_definition(&capture);
        let definition = h.capture_actor_definition(ActorDefinitionKey::from_legacy(raw));
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(3, key.byte_index());
        let shape = read_shape(h, &definition);
        configure(h, key, shape);
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(4, key.byte_index());
        let entry = read_entry(h, &capture);
        let (provenance, selected) = crate::actors::content::birth_metadata_from_capture(
            &capture,
            captured_origin(&cursor),
            entry.definition,
            appearance,
            false,
        )
        .map_err(crate::actors::identity::ActorIdentityError::Content)?;
        validate_selected_appearance(h, &capture, selected)?;
        let birth = h
            .session()
            .actor_identity
            .prepare_legacy_birth(key, provenance, selected)?;
        let handle = place(h, key, entry, Creation::Dynamic, birth);
        #[cfg(sm_verify_dll)]
        h.actor_reference_stage(5, key.byte_index());
        last_handle = handle;
        finish_initialization(h, key, &definition, Creation::Dynamic);
        // Reuse the original one-fewer-part stop rule; the typed cursor advances separately.
        if !sequence.advance() {
            break;
        }
        index = sequence.slot;
        cursor = match cursor {
            crate::actors::content::PopulationCursor::Legacy { bank, .. } => {
                let address = h.session().compatibility_enemy_scratch.wrapping_add(16);
                h.session_mut().compatibility_enemy_scratch = address;
                crate::actors::content::PopulationCursor::Legacy { bank, address }
            }
            other => next_cursor(other),
        };
    }
    h.session_mut().actors.controls.selected = saved;
    Ok(Some((index, last_handle)))
}

pub fn spawn<H: ActorHost + crate::enemy_projectiles::host::Host>(
    h: &mut H,
    request: crate::actors::api::ActorSpawnRequest,
) -> Result<Option<InstanceHandle>, crate::actors::identity::ActorIdentityError> {
    let definitions = h
        .session()
        .world
        .actors()
        .definitions()
        .ok_or(crate::actors::content::ActorContentError::UnsupportedForm)?;
    if !definitions.templates.contains_key(&request.template) {
        return Err(
            crate::actors::content::ActorContentError::MissingTemplate(request.template).into(),
        );
    }
    let cursor = crate::actors::content::PopulationCursor::Template {
        template: request.template,
        part: 0,
    };
    Ok(spawn_from_cursor(h, cursor, request.appearance)?.and_then(|(_, handle)| handle))
}

pub fn spawn_legacy<H: ActorHost + crate::enemy_projectiles::host::Host>(
    h: &mut H,
    bank: u8,
    address: u16,
) -> u16 {
    h.session_mut().compatibility_enemy_scratch = address;
    spawn_from_cursor(
        h,
        crate::actors::content::PopulationCursor::Legacy { bank, address },
        None,
    )
    .expect("legacy actor spawn identity overflow")
    .map(|(last, _)| last)
    .unwrap_or(0xffff)
}

fn next_cursor(
    cursor: crate::actors::content::PopulationCursor,
) -> crate::actors::content::PopulationCursor {
    use crate::actors::content::PopulationCursor as C;
    match cursor {
        C::Legacy { bank, address } => C::Legacy {
            bank,
            address: address.wrapping_add(16),
        },
        C::Prepared { population, entry } => C::Prepared {
            population,
            entry: entry + 1,
        },
        C::Template { template, part } => C::Template {
            template,
            part: part + 1,
        },
    }
}

fn validate_selected_appearance<H: ActorHost>(
    h: &H,
    capture: &ActorPopulationCapture,
    appearance: Option<crate::actors::media::AppearanceId>,
) -> Result<(), crate::actors::content::ActorContentError> {
    if let Some(appearance) = appearance {
        capture.world.actor_media().validate_appearance_residency(
            appearance,
            h.session().actor_identity.palette_residency(),
            &capture.record.image,
        )?;
    }
    Ok(())
}

fn validate_birth_appearance<H: ActorHost>(
    h: &H,
    capture: &ActorPopulationCapture,
    cursor: &crate::actors::content::PopulationCursor,
    definition: u16,
    requested: Option<crate::actors::media::AppearanceId>,
    respawn: bool,
) -> Result<(), crate::actors::content::ActorContentError> {
    let (_, appearance) = crate::actors::content::birth_metadata_from_capture(
        capture,
        captured_origin(cursor),
        definition,
        requested,
        respawn,
    )?;
    validate_selected_appearance(h, capture, appearance)
}
