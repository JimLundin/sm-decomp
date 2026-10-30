//! Layer-ordered actor scene contribution; presentation changes do not own collision.
#![forbid(unsafe_code)]
use super::{
    access::{AccessMode, ActorAccess, ActorCollections, ActorKey, CollectionWindow},
    combat::current,
    content::{ActorCountWidth, ActorPartTraversal, ActorPartWord},
    effects::{ActorAssertion, ActorHost},
    media::AppearanceId,
    scene::{self, ActorDrawMode, ActorDrawRequest, ActorPaletteMode},
};
use sm_game::geometry::Point;
use sm_game::player_projectiles::access::Store;
use sm_game::{
    enemy_projectiles::{host::Host as EprojHost, presentation as eproj_presentation},
    player_projectiles::{host::Host as ShotHost, presentation as shot_presentation},
    sprite_objects,
};

fn appearance(h: &impl ActorAccess, key: ActorKey) -> Option<AppearanceId> {
    let index = key.byte_index();
    if index < 2048 && index & 63 == 0 {
        let live = h.resolve_actor(key, AccessMode::Native).definition != 0;
        h.session()
            .actor_identity
            .observe(index / 64, live, None)
            .appearance
    } else {
        None
    }
}
fn queue_address(h: &mut impl ActorHost, address: u16) -> u16 {
    if address >= 0x2000 {
        h.actor_assertion(ActorAssertion::DrawingQueueAddress);
    }
    address
}
pub(in crate::game) fn queue_current_actor<H: ActorHost>(h: &mut H) {
    let key = current(h);
    let layer = h
        .resolve_actor(key, AccessMode::Native)
        .layer
        .wrapping_mul(2)
        >> 1;
    let index = h.session().actors.controls.selected;
    let destination =
        queue_size(h, usize::from(layer)).wrapping_add(h.actor_draw_queue_base(layer));
    let address = queue_address(h, destination);
    write_queue_entry(h, address, index);
    let size = queue_size(h, usize::from(layer)).wrapping_add(2);
    set_queue_size(h, usize::from(layer), size);
}
pub(in crate::game) fn draw_scene<H: ActorHost + EprojHost + ShotHost>(h: &mut H) {
    sprite_objects::draw(h);
    shot_presentation::draw_explosions(h);
    eproj_presentation::draw(eproj_presentation::Priority::Low, h);
    for layer in 0..8usize {
        match layer {
            3 => h.draw_player_scene(),
            6 => eproj_presentation::draw(eproj_presentation::Priority::High, h),
            _ => {}
        }
        let size = queue_size(h, layer);
        if size == 0 {
            continue;
        }
        let base = h.actor_draw_queue_base(layer as u16);
        let mut cursor = 0u16;
        set_queue_size(h, layer, 0);
        loop {
            let address = queue_address(h, base.wrapping_add(cursor));
            let index = queue_entry(h, address);
            write_queue_entry(h, address, 0);
            h.session_mut().actors.controls.selected = index;
            draw_actor(h);
            if u32::from(cursor) + 2 == u32::from(size) {
                break;
            }
            cursor = cursor.wrapping_add(2);
        }
    }
    let hook = u32::from_ne_bytes(h.session().actors.drawing.hook_address) & 0xffffff;
    h.actor_graphics_hook(super::content::ActorRoutine(hook));
}
fn draw_actor<H: ActorHost + EprojHost>(h: &mut H) {
    let key = current(h);
    let spawn_x = h.resolve_spawn(key, AccessMode::Native).draw_offset.x;
    let mut x = spawn_x
        .wrapping_add(h.resolve_actor(key, AccessMode::Native).x.whole)
        .wrapping_sub(h.inputs().camera_x);
    let spawn_y = h.resolve_spawn(key, AccessMode::Native).draw_offset.y;
    let y = spawn_y
        .wrapping_add(h.resolve_actor(key, AccessMode::Native).y.whole)
        .wrapping_sub(h.inputs().camera_y);
    h.session_mut().compatibility_enemy_scratch = y;
    let shake = h.resolve_actor(key, AccessMode::Native).shake;
    if shake != 0 {
        x = x.wrapping_add(if h.resolve_actor(key, AccessMode::Native).frame & 2 == 0 {
            1
        } else {
            0xffff
        });
        h.edit_actor(key, AccessMode::Native, |actor| {
            actor.shake = shake.wrapping_sub(1)
        });
    }
    let frozen = h.resolve_actor(key, AccessMode::Native).frozen;
    let (palette, palette_mode) = if h.resolve_actor(key, AccessMode::Native).flash != 0
        && h.session().actors.controls.phase_counter & 2 != 0
    {
        (0, ActorPaletteMode::Flash)
    } else if frozen != 0 && (frozen >= 0x5a || frozen & 2 != 0) {
        (0xc00, ActorPaletteMode::Frozen)
    } else {
        (
            h.resolve_actor(key, AccessMode::Native).palette,
            ActorPaletteMode::Normal,
        )
    };
    let tile = h.resolve_actor(key, AccessMode::Native).tile;
    if h.resolve_actor(key, AccessMode::Native).extra_properties & 4 == 0 {
        h.session_mut().room.enemy_processing_stage = 1;
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let map = h.resolve_actor(key, AccessMode::Native).spritemap;
        let visual = scene::select_visual(
            &h.session().world,
            appearance(h, key),
            bank,
            map,
            ActorDrawMode::Base,
        )
        .expect("validated actor appearance");
        h.draw_actor_visual(ActorDrawRequest {
            visual,
            position: Point { x, y },
            palette,
            tile,
            palette_mode,
            mode: ActorDrawMode::Base,
        });
        h.probe("actors.scene.base.emitted");
        #[cfg(sm_verify_dll)]
        if h.session().world.room_key(&h.session().room.entry.room) == 0x91f8 {
            h.probe("actors.scene.landing_site.base.emitted");
        }
        return;
    }
    if h.resolve_actor(key, AccessMode::Native).spritemap & 0x8000 == 0 {
        h.unreachable();
    }
    let pose = h.capture_actor_pose(key);
    let pose_id = scene::pose_identity(&pose);
    let mut remaining = i32::from(h.actor_pose_count(&pose, ActorCountWidth::Byte));
    let mut cursor = h
        .resolve_actor(key, AccessMode::Native)
        .spritemap
        .wrapping_add(2);
    let mut ordinal = 0usize;
    loop {
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let part = h.capture_actor_part_at(
            &pose,
            ordinal,
            ActorPartTraversal::WrappingReacquire {
                bank,
                address: cursor,
            },
        );
        let bank = h.resolve_actor(key, AccessMode::Native).bank;
        let map = h.actor_part_word(&part, ActorPartWord::Map);
        let tilemap = h.actor_visual_header(bank, map) == 0xfffe;
        let part_x = x.wrapping_add(h.actor_part_word(&part, ActorPartWord::X));
        let part_y = y.wrapping_add(h.actor_part_word(&part, ActorPartWord::Y));
        if tilemap {
            if h.resolve_actor(key, AccessMode::Native).extra_properties & 0x8000 != 0 {
                let bank = h.resolve_actor(key, AccessMode::Native).bank;
                let address = h.actor_part_word(&part, ActorPartWord::Map);
                let source = scene::select_tilemap_source(
                    &h.session().world,
                    appearance(h, key),
                    pose_id,
                    ordinal,
                    bank,
                    address,
                );
                h.copy_actor_tilemap(source);
                h.probe("actors.scene.tilemap.copied");
            }
        } else if (u32::from(part_x) + 128) & 0xfe00 == 0 && (u32::from(part_y) + 128) & 0xfe00 == 0
        {
            let mode = if part_y.to_ne_bytes()[1] != 0 {
                ActorDrawMode::ExtendedOffscreen
            } else {
                ActorDrawMode::Extended
            };
            let bank = h.resolve_actor(key, AccessMode::Native).bank;
            let map = h.actor_part_word(&part, ActorPartWord::Map);
            let visual = scene::select_part_visual(
                &h.session().world,
                appearance(h, key),
                pose_id,
                ordinal,
                bank,
                map,
                mode,
            )
            .expect("validated actor part appearance");
            h.draw_actor_visual(ActorDrawRequest {
                visual,
                position: Point {
                    x: part_x,
                    y: part_y,
                },
                palette,
                tile,
                palette_mode,
                mode,
            });
            h.probe("actors.scene.extended.emitted");
        }
        cursor = cursor.wrapping_add(8);
        ordinal = ordinal.wrapping_add(1);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
}

fn queue_size(h: &impl ActorCollections, index: usize) -> u16 {
    h.collection_word(
        CollectionWindow::DrawSizes,
        index.checked_mul(2).unwrap(),
        AccessMode::LittleEndian,
    )
}
fn set_queue_size(h: &mut impl ActorCollections, index: usize, value: u16) {
    h.write_collection_word(
        CollectionWindow::DrawSizes,
        index.checked_mul(2).unwrap(),
        value,
        Store::PrefixLe,
    );
}
fn queue_entry(h: &impl ActorCollections, address: u16) -> u16 {
    h.collection_word(
        CollectionWindow::AbsoluteDrawAddress(address),
        0,
        AccessMode::Native,
    )
}
fn write_queue_entry(h: &mut impl ActorCollections, address: u16, value: u16) {
    h.write_collection_word(
        CollectionWindow::AbsoluteDrawAddress(address),
        0,
        value,
        Store::SliceNative,
    );
}
