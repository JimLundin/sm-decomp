//! Population graphics staging and ordered transfer transitions.
#![forbid(unsafe_code)]
use super::effects::ActorHost;
use crate::game::enemy_projectiles::host::Host;

pub(in crate::game) fn queue_actor_tiles<H: ActorHost + Host>(h: &mut H) {
    if h.session().actors.drawing.tilemap_pending != 0
        && h.session().actors.diagnostics.freeze | h.inputs().frozen == 0
    {
        let cursor = h.vram_write_queue_tail();
        let size = h.session().actors.drawing.tilemap_size;
        h.set_vram_queue_size(cursor, size);
        h.set_vram_queue_source(cursor, 0x2000);
        h.write_collection_byte(
            super::access::CollectionWindow::VramQueue,
            usize::from(cursor) + 4,
            126,
        );
        h.set_vram_queue_destination(cursor, 0x4800);
        let tail = h.vram_write_queue_tail().wrapping_add(7);
        h.set_vram_write_queue_tail(tail);
    }
    h.session_mut().actors.drawing.tilemap_pending = 0;
}

pub(in crate::game) fn transfer_actor_tiles_and_initialize<H: ActorHost + Host>(h: &mut H) {
    let mut source = h.session().actors.staging.transfer_source;
    if h.session().actors.staging.transfer_source == 0 {
        source = 0x7000;
        h.session_mut().actors.staging.transfer_source = 0x7000;
        h.session_mut().actors.staging.transfer_destination = 0x6c00;
    }
    match source {
        0xffff => {}
        0xfffe => {
            super::population::install_population(h);
            h.session_mut().actors.staging.transfer_source = 0xffff;
        }
        0x9800 => h.session_mut().actors.staging.transfer_source = 0xfffe,
        _ => {
            let tail = h.vram_write_queue_tail();
            let cursor = h.vram_write_queue_tail();
            h.set_vram_queue_size(cursor, 2048);
            let source = h.session().actors.staging.transfer_source;
            let address = h.session().actors.staging.transfer_source;
            h.set_vram_queue_source(cursor, address);
            h.session_mut().actors.staging.transfer_source = source.wrapping_add(2048);
            h.set_vram_queue_bank_word(cursor, 126);
            let destination = h.session().actors.staging.transfer_destination;
            let address = h.session().actors.staging.transfer_destination;
            h.set_vram_queue_destination(cursor, address);
            h.session_mut().actors.staging.transfer_destination = destination.wrapping_add(1024);
            h.set_vram_write_queue_tail(tail.wrapping_add(7));
        }
    }
}

pub(super) fn load_graphics<H: ActorHost + Host>(
    h: &mut H,
    population: &super::content::PopulationCursor,
    key: super::access::ActorKey,
) {
    use super::access::AccessMode::Native;
    #[cfg(sm_verify_dll)]
    h.actor_reference_stage(1, key.byte_index());
    let mut address = h.session().room.entry.enemy_tilesets_key;
    let mut tiles = 0u16;
    loop {
        let tileset = h.capture_graphics_tileset(address);
        let entry = h.capture_actor_population(population);
        let found = h.population_definition(&entry) == h.tileset_definition(&tileset, 0);
        if found || h.tileset_definition(&tileset, 0) == 0xffff {
            let (palette, tiles) = if found {
                ((h.tileset_destination_word(&tileset, 0) & 15) << 9, tiles)
            } else {
                (2560, 0)
            };
            if found {
                h.edit_spawn(key, Native, |s| s.palette = palette);
                let value = h.resolve_spawn(key, Native).palette;
                h.edit_actor(key, Native, |a| a.palette = value);
                h.edit_spawn(key, Native, |s| s.tile = tiles);
                let value = h.resolve_spawn(key, Native).tile;
                h.edit_actor(key, Native, |a| a.tile = value);
            } else {
                h.edit_spawn(key, Native, |s| s.tile = tiles);
                let value = h.resolve_spawn(key, Native).tile;
                h.edit_actor(key, Native, |a| a.tile = value);
                h.edit_spawn(key, Native, |s| s.palette = palette);
                let value = h.resolve_spawn(key, Native).palette;
                h.edit_actor(key, Native, |a| a.palette = value);
            }
            #[cfg(sm_verify_dll)]
            h.actor_reference_stage(2, key.byte_index());
            return;
        }
        let raw = h.tileset_definition(&tileset, 0);
        let definition =
            h.capture_actor_definition(super::content::ActorDefinitionKey::from_legacy(raw));
        tiles = tiles.wrapping_add(h.definition_tile_size(&definition) >> 5);
        address = address.wrapping_add(4);
    }
}

pub(crate) fn prepare_room_assets<H: ActorHost + Host>(h: &mut H) {
    h.session_mut()
        .actor_identity
        .clear_actor_palette_residency();
    h.session_mut().actors.diagnostics.freeze = 0;
    h.session_mut().actors.drawing.hook_address[2] = 0xa0;
    h.session_mut().actors.drawing.hook_address[..2].copy_from_slice(&0x804cu16.to_ne_bytes());
    h.session_mut().actors.drawing.tilemap_size = 2048;
    h.session_mut().actors.schedule.compatibility_179e[0] = 0;
    h.session_mut().actors.schedule.compatibility_179e[1] = 0;
    h.session_mut().actors.schedule.boss_id = 0;
    for i in 0..32 {
        h.session_mut().actors.slots[i] = Default::default();
        h.session_mut().actor_identity.retire(i as u16);
    }
    let cursor = super::content::PopulationCursor::Legacy {
        bank: 0xa1,
        address: h.session().room.entry.enemy_population_key,
    };
    let entry = h.capture_actor_population(&cursor);
    if !h.population_terminated(&entry) {
        stage_tilesets(h);
    }
    load_tile_data(h);
    h.session_mut().actors.staging.transfer_source = 0;
    h.session_mut().actors.controls.projectile_contacts_disabled = 0;
    crate::game::sprite_objects::clear_for_population_load(h.session_mut());
}

fn load_tile_data<H: ActorHost + Host>(h: &mut H) {
    use super::access::{AccessMode::Native, ActorKey};
    use crate::game::visual_host::{Content, Record, Source};
    // Capture once, then read/write each native word in descending source order.
    let defaults = Content::table(h, Source::Loaded, 0x9ad200);
    for i in 0..256 {
        let value = defaults.native_word(2 * (3327 - i));
        let key = ActorKey::from_legacy((0x1fe - 2 * i) as u16);
        h.edit_spawn(key, Native, |s| s.contact_mode = value);
    }
    if h.session().actors.staging.descriptor_write_position == 0 {
        return;
    }
    let (mut load, mut consumed) = (0usize, 0u16);
    loop {
        let mut source = load_word(h, load, 2);
        let end = load_word(h, load, 0).wrapping_add(source);
        let mut destination = load_word(h, load, 5);
        let bank = load_byte(h, load, 4);
        loop {
            let bytes = source.to_le_bytes();
            let chunk = h.capture_tile_chunk([bytes[0], bytes[1], bank], 0);
            h.copy_actor_tile_chunk(
                super::scene::ActorTileChunkSource::from_record(chunk),
                super::scene::ActorTileDestination::from_legacy(destination),
            );
            destination = destination.wrapping_add(8);
            source = source.wrapping_add(8);
            if source == end {
                break;
            }
        }
        load += 7;
        consumed = consumed.wrapping_add(7);
        if consumed == h.session().actors.staging.descriptor_write_position {
            break;
        }
    }
    h.session_mut().actors.staging.descriptor_write_position = 0;
}

fn stage_tilesets<H: ActorHost + Host>(h: &mut H) {
    h.session_mut().actors.staging.descriptor_write_position = 0;
    h.session_mut().actors.graphics.definitions.fill(0);
    h.session_mut().actors.graphics.tile_offsets.fill(0);
    h.session_mut().actors.graphics.palette_routes.fill(0);
    let key = h.session().room.entry.enemy_tilesets_key;
    let tileset = h.capture_tilesets(key);
    let (mut entry, mut load, mut graphics, mut tiles, mut next_ram) =
        (0usize, 0usize, 0u16, 0u16, 2048u16);
    while h.tileset_definition(&tileset, entry) != 0xffff {
        let raw = h.tileset_definition(&tileset, entry);
        let definition =
            h.capture_actor_definition(super::content::ActorDefinitionKey::from_legacy(raw));
        h.session_mut().compatibility_enemy_scratch =
            h.definition_word(&definition, super::content::DefinitionWord::NativeBank);
        // Native destination byte is observed before palette acquisition/diagnostic.
        let palette_row = h.tileset_destination_byte(&tileset, entry);
        let palette = h.definition_palette(&definition);
        let colors = h.palette_bytes(&palette);
        let row = u16::from(palette_row) + 8;
        let resource = super::media::palette_resource_for_capture(&palette, &h.session().world);
        // Source capture is complete. Revoke before the write-capable copy; a
        // failed/unknown overwrite cannot leave a receipt for previous pixels.
        h.session_mut()
            .actor_identity
            .record_palette_residency(row, None, palette.image.clone());
        h.copy_palette_row(palette_row, colors);
        h.session_mut().actor_identity.record_palette_residency(
            row,
            resource,
            palette.image.clone(),
        );
        let bytes = h.definition_tile_size(&definition) & 0x7fff;
        set_load_word(h, load, 0, bytes);
        let source = h.definition_tile_source(&definition);
        set_load_source(h, load, source);
        let destination = if h.definition_tile_size(&definition) & 0x8000 != 0 {
            (h.tileset_destination_word(&tileset, entry) & 0x3000) >> 3
        } else {
            next_ram
        };
        set_load_word(h, load, 5, destination);
        let position = h
            .session()
            .actors
            .staging
            .descriptor_write_position
            .wrapping_add(7);
        h.session_mut().actors.staging.descriptor_write_position = position;
        load += 7;
        let slot = usize::from(graphics >> 1);
        set_graphics_tiles(h, slot, tiles);
        let raw = h.tileset_definition(&tileset, entry);
        set_graphics_definition(h, slot, raw);
        let route = h.tileset_destination_word(&tileset, entry);
        set_graphics_route(h, slot, route);
        graphics = graphics.wrapping_add(2);
        tiles = tiles.wrapping_add(h.definition_tile_size(&definition) >> 5);
        next_ram = next_ram.wrapping_add(h.definition_tile_size(&definition));
        entry += 1;
    }
}

// These are fixed collection projections, not addresses or independent RAM owners.
fn load_word<H: ActorHost>(h: &H, descriptor: usize, field: usize) -> u16 {
    h.collection_word(
        super::access::CollectionWindow::TileLoad,
        descriptor + field,
        super::access::AccessMode::Native,
    )
}
fn load_byte<H: ActorHost>(h: &H, descriptor: usize, field: usize) -> u8 {
    h.collection_byte(
        super::access::CollectionWindow::TileLoad,
        descriptor + field,
    )
}
fn set_load_word<H: ActorHost>(h: &mut H, descriptor: usize, field: usize, value: u16) {
    h.write_collection_word(
        super::access::CollectionWindow::TileLoad,
        descriptor + field,
        value,
        super::access::CollectionStore::SliceNative,
    );
}
fn set_load_source<H: ActorHost>(h: &mut H, descriptor: usize, source: [u8; 3]) {
    h.write_tile_source(descriptor, source);
}
fn set_graphics_tiles<H: ActorHost>(h: &mut H, index: usize, value: u16) {
    h.write_collection_word(
        super::access::CollectionWindow::GraphicsTiles,
        2 * index,
        value,
        super::access::CollectionStore::SliceNative,
    );
}
fn set_graphics_definition<H: ActorHost>(h: &mut H, index: usize, value: u16) {
    h.write_collection_word(
        super::access::CollectionWindow::GraphicsDefinitions,
        2 * index,
        value,
        super::access::CollectionStore::PrefixLe,
    );
}
fn set_graphics_route<H: ActorHost>(h: &mut H, index: usize, value: u16) {
    h.write_collection_word(
        super::access::CollectionWindow::GraphicsRoutes,
        2 * index,
        value,
        super::access::CollectionStore::PrefixLe,
    );
}
