//! Crocomire phases and attack scripts retain native actor words and captured script images.
//! Hardware effects remain in the visual owner; RAM remains authoritative.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::actors::host::{ActorLease, Death, Slopes};
use crate::actors::layouts::EnemyLayout72 as Crocomire;
use crate::constants::{
    bank_86::{
        addr_kEproj_CrocomireSpikeWallPieces, addr_kEproj_DustCloudExplosion, addr_stru_868F9D,
    },
    bank_a4::*,
    enemy_properties::kEnemyProps_Tangible,
};
use crate::enemy_script::EnemyScript;
use crate::menus::prelude::do_until;
use crate::ram::RawRam as _;
use crate::ram::{checked_word, slot, word, words as ram};
use sm_data::crocomire::{
    BRIDGE_DEBRIS_ADDRESS, FLESH_TILEMAP_ADDRESS, MELT_TILES, SCROLL_SPRITEMAP_ADDRESS,
    SKELETON_TILEMAP_ADDRESS, SPIKE_WALL_DESTINATION_ADDRESS, SPIKE_WALL_TILE_ADDRESS,
};

/// What Crocomire needs beyond the shared boss effects: its imported tables,
/// the BG2 alignment, the native boss word, projectile clearing and its drop.
pub trait Host: crate::actors::host::BossEffects + crate::actors::host::LeaseActors {
    /// The word at `index` of the Crocomire table at `address` (scroll
    /// spritemaps, spike wall tiles and destinations, bridge debris).
    fn crocomire_table_word(&self, address: u32, index: isize) -> u16;
    /// The 256 tiles of the melt tilemap at `table` while it is imported.
    fn crocomire_melt_tilemap(&self, table: u32) -> Option<[u16; MELT_TILES]>;
    fn align_crocomire_background(&mut self);
    /// The boss word for `area`, in native order.
    fn boss_word_native(&mut self, area: u16) -> u16;
    fn clear_enemy_projectiles(&mut self);
    /// Crocomire's death drop for `area`.
    fn scatter_crocomire_drop(&mut self, area: u16);
}

pub fn dispatch<H: Host + ?Sized>(k: u16, jp: EnemyScript, game: &mut H) -> EnemyScript {
    crate::actors::crocomire_attack::dispatch(k, jp, &mut game.lease())
}
pub fn choose_spit<H: Host + ?Sized>(k: u16, jp: EnemyScript, game: &mut H) -> EnemyScript {
    crate::actors::crocomire_attack::choose_spit(k, jp, &mut game.lease())
}

fn cap_byte(value: u8, limit: u8) -> u8 {
    if value.wrapping_sub(limit) as i8 >= 0 {
        limit
    } else {
        value
    }
}
pub fn fill_scroll<H: Host + ?Sized>(game: &mut H) {
    let scroll = word!(game.reg_BG2VOFS);
    crate::word_transfers!(game, descending, 256; le(crocomire_bg2_scroll_hdma_data, 255) <- value(scroll));
}
pub fn melt_down<H: Host + ?Sized>(game: &mut H) {
    fill_scroll(game);
    slot!(game, Crocomire[0].var_B, |old| old & !0x800);
    update_background(word!(game.cur_enemy_index), game);
    let key = word!(game.cur_enemy_index) as i32;
    if slot!(game, Crocomire[key].base.y_pos) as i32 - 280 & 0x8000 != 0 {
        let mut cursor = (8 * (word!(game.reg_BG2VOFS) as i32 + 288 & 0xfff8)) as u16;
        for _ in 0..32 {
            word!(game.tilemap_stuff[cursor >> 1] = 824);
            cursor = cursor.wrapping_add(2);
        }
        word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
        slot!(game, Crocomire[0].base.extra_properties, |old| old
            & !0x8000);
        let low = slot!(game, Crocomire[key].var_D) as u8;
        slot!(byte game, Crocomire[key].var_D[0] = low.wrapping_add(0x80));
        let next = (low.overflowing_add(0x80).1 as u8)
            .wrapping_add(slot!(byte game, Crocomire[key].var_D[1]))
            .wrapping_add(3);
        let high = cap_byte(next, 48);
        slot!(byte game, Crocomire[key].var_D[1] = high);
        let carry = slot!(byte game, Crocomire[key].var_E[0])
            .overflowing_add(high)
            .1;
        slot!(byte game, Crocomire[key].var_E[0] = match slot!(byte game, Crocomire[key].var_E[0]) { old => old.wrapping_add(high) });
        let high = cap_byte(
            (carry as u8).wrapping_add(slot!(byte game, Crocomire[key].var_E[1])),
            3,
        );
        slot!(byte game, Crocomire[key].var_E[1] = high);
        let velocity = slot!(game, Crocomire[key].var_E) as u8;
        let carry = slot!(byte game, Crocomire[key].var_F[1])
            .overflowing_add(velocity)
            .1;
        slot!(byte game, Crocomire[key].var_F[1] = match slot!(byte game, Crocomire[key].var_F[1]) { old => old.wrapping_add(velocity) });
        let (step, carry) =
            (carry as u8).overflowing_add(slot!(byte game, Crocomire[key].var_E[1]));
        let carry = carry
            | slot!(byte game, Crocomire[key].base.y_pos[0])
                .overflowing_add(step)
                .1;
        slot!(byte game, Crocomire[key].base.y_pos[0] = match slot!(byte game, Crocomire[key].base.y_pos[0]) { old => old.wrapping_add(step) });
        slot!(byte game, Crocomire[key].base.y_pos[1] = match slot!(byte game, Crocomire[key].base.y_pos[1]) { old => old.wrapping_add(carry as u8) });
    } else {
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[0].var_D = 48);
    }
}
pub fn melt_up<H: Host + ?Sized>(game: &mut H) {
    fill_scroll(game);
    let key = word!(game.cur_enemy_index) as i32;
    if slot!(game, Crocomire[key].base.y_pos) as i32 - 218 & 0x8000 != 0 {
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[key].var_A, |old| old.wrapping_add(1));
    } else {
        update_background(word!(game.cur_enemy_index), game);
        let mut acceleration = (slot!(game, Crocomire[0].var_D) as i16).wrapping_add(256);
        if slot!(game, Crocomire[0].var_D) as i32 - 7680 & 0x8000 == 0 {
            acceleration = 7936;
        }
        slot!(game, Crocomire[0].var_D = acceleration as u16);
        let low = slot!(game, Crocomire[0].var_E) as u8;
        let borrow = low < slot!(byte game, Crocomire[0].var_D[1]);
        slot!(byte game, Crocomire[0].var_E[0] = low.wrapping_sub(slot!(byte game, Crocomire[0].var_D[1])));
        let mut high = (slot!(byte game, Crocomire[0].var_E[1]) as i8).wrapping_sub(borrow as i8);
        if high < 0 {
            slot!(byte game, Crocomire[0].var_E[0] = 0xff);
            high = 0;
        }
        slot!(byte game, Crocomire[0].var_E[1] = high as u8);
        let fraction = slot!(byte game, Crocomire[0].var_F[1]);
        let borrow = fraction < slot!(byte game, Crocomire[0].var_E[0]);
        slot!(byte game, Crocomire[0].var_F[1] = fraction.wrapping_sub(slot!(byte game, Crocomire[0].var_E[0])));
        let y = slot!(game, Crocomire[0].base.y_pos) as u8;
        let step = (borrow as u8).wrapping_add(slot!(byte game, Crocomire[0].var_E[1]));
        slot!(byte game, Crocomire[0].base.y_pos[0] = y.wrapping_sub(step));
        slot!(byte game, Crocomire[0].base.y_pos[1] = match slot!(byte game, Crocomire[0].base.y_pos[1]) { old => old.wrapping_sub((y < step) as u8) });
    }
}
pub fn update_background<H: Host + ?Sized>(k: u16, game: &mut H) {
    word!(game.reg_BG2VOFS = 67u16.wrapping_sub(slot!(game, Crocomire[k as i32].base.y_pos)));
    let mut index = 32u16;
    let spritemap = loop {
        let candidate = slot!(game, Crocomire[0].base.spritemap_pointer);
        let spritemap = game.crocomire_table_word(SCROLL_SPRITEMAP_ADDRESS, (index >> 1) as isize);
        if candidate == spritemap {
            break candidate;
        }
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            game.align_crocomire_background();
            return;
        }
    };
    let offset = crate::cartridge::native_record(0xa4, spritemap, game).table_word(14);
    word!(game.reg_BG2VOFS += offset);
    game.align_crocomire_background();
}
pub fn update_palette<H: Host + ?Sized>(game: &mut H) {
    if word!(game.door_transition_flag_enemies) == 0 {
        if slot!(game, Crocomire[0].base.flash_timer) != 0
            && (word!(game.random_enemy_counter) & 2) != 0
        {
            crate::word_transfers!(game, descending, 8; native(0xc0ee) <- value(0x7fff));
        } else {
            crate::word_transfers!(game, descending, 8; native(0xc0ee) <- rom(0xa4b89d, 7));
        }
    }
}
fn wall_counter(runtime: &impl ActorAccess, key: u16) -> u16 {
    runtime
        .resolve_actor(Key::from_legacy(key), AccessMode::Native)
        .variables[3]
}
fn set_wall_counter(runtime: &mut impl ActorAccess, key: u16, value: u16) {
    runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
        enemy.variables[3] = value
    });
}
pub fn wall_step<H: Host + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let mut cursor = wall_counter(&runtime, 0);
    let image = runtime.leased_image().expect("ROM must be loaded");
    let target = image.table_word(
        sm_data::cartridge::lorom_offset(0xa498ca) as usize,
        (cursor >> 1) as isize,
    );
    if target == 0x8080 {
        set_wall_counter(&mut runtime, 0x40, 0x8080);
        set_wall_counter(&mut runtime, 0, 128);
        drop(runtime);
        crate::word_transfers!(game, descending, 16; le(palette_buffer, 191) <- rom(0xa4b91d, 15));
        game.lease()
            .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
                enemy.variables[0] = enemy.variables[0].wrapping_add(2);
            });
        return;
    }
    let current = wall_counter(&runtime, 0x40);
    if current != target {
        let step = wall_counter(&runtime, 0xc0);
        let next = if (current as i16).wrapping_sub(target as i16) >= 0 {
            current.wrapping_sub(step)
        } else {
            current.wrapping_add(step)
        };
        set_wall_counter(&mut runtime, 0x40, next);
        return;
    }
    if target & 0x8000 != 0 {
        let countdown = wall_counter(&runtime, 0x80);
        if countdown != 0 {
            set_wall_counter(&mut runtime, 0x80, countdown.wrapping_sub(1));
            set_wall_counter(&mut runtime, 0, cursor.wrapping_sub(2));
            drop(runtime);
            game.queue_sound(1, 0x2b, 6);
            return;
        }
        cursor = cursor.wrapping_add(2);
        let countdown = image.table_word(
            sm_data::cartridge::lorom_offset(0xa498ca) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0x80, countdown);
        cursor = cursor.wrapping_add(2);
        let step = image.table_word(
            sm_data::cartridge::lorom_offset(0xa498ca) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0xc0, step);
    }
    set_wall_counter(&mut runtime, 0, cursor.wrapping_add(2));
}
pub fn rumble<H: Host + ?Sized>(game: &mut H) {
    let remaining = slot!(game, Crocomire[0].var_20);
    if remaining != 0 {
        slot!(game, Crocomire[0].var_20 = remaining - 1);
        if remaining == 1 {
            slot!(game, Crocomire[0].var_20 = 32);
            game.queue_sound(2, 0x22, 6);
        }
    }
}
fn contact_edge<H: Host + ?Sized>(game: &mut H) -> i32 {
    slot!(game, Crocomire[0].base.x_pos) as i32
        - slot!(game, Crocomire[0].base.x_width) as i32
        - word!(game.samus_x_radius) as i32
}
fn contact<H: Host + ?Sized>(game: &mut H) {
    game.contact_reaction(Death::Finalize);
}
pub fn touch<H: Host + ?Sized>(game: &mut H) {
    contact(game);
    slot!(game, Crocomire[0].var_B, |flags| flags | 0x4000);
    word!(game.extra_samus_x_displacement = -4i32 as u16);
}
pub fn push_player<H: Host + ?Sized>(game: &mut H) {
    if slot!(game, Crocomire[0].var_A) == 0
        && ((contact_edge(game) - word!(game.samus_x_pos) as i32) as i16) < 0
    {
        contact(game);
        word!(game.samus_x_pos = contact_edge(game) as u16);
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        word!(game.extra_samus_x_displacement = -4i32 as u16);
        word!(game.extra_samus_y_displacement = -1i32 as u16);
    }
}
fn next_hit_count(flags: u16) -> u16 {
    ((flags & 0xf) + 1).min(15)
}
pub fn count_hit<H: Host + ?Sized>(game: &mut H) {
    let count = next_hit_count(slot!(game, Crocomire[0].var_B));
    slot!(game, Crocomire[0].var_B, |flags| flags | count);
}
pub fn impact<H: Host + ?Sized>(game: &mut H) {
    let index = word!(game.collision_detection_index) as i32;
    let x = word!(game.projectile_x_pos[index]);
    let y = word!(game.projectile_y_pos[index]);
    game.set_projectile_spawn_point(x, y);
    let appearance = if word!(game.projectile_type[index]) & 0x200 == 0 {
        6
    } else {
        29
    };
    game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, appearance);
}
pub fn hit<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Crocomire[0].base.invincibility_timer = 0);
    let mut advance = ((slot!(game, Crocomire[0].base.x_pos) as i32
        - slot!(game, Crocomire[0].base.x_width) as i32
        - 256
        - word!(game.layer1_x_pos) as i32) as i16)
        >= 0;
    if !advance {
        let projectile = word!(game.projectile_type[word!(game.collision_detection_index)]);
        if projectile & 0xf10 == 0 {
            slot!(game, Crocomire[0].base.instruction_timer = 8);
            impact(game);
            return;
        }
        let recoil = [2u16, 1, 3]
            .get(((projectile & 0xf00) >> 8) as usize)
            .copied()
            .unwrap_or(0);
        advance = recoil != 0;
        if advance {
            slot!(game, Crocomire[0].var_D, |old| old.wrapping_add(recoil));
        }
    }
    if advance {
        let count = next_hit_count(slot!(game, Crocomire[0].var_B));
        if slot!(game, Crocomire[0].var_B) & 0x800 == 0 {
            let mut delay = 8;
            if slot!(game, Crocomire[0].var_C) == 8 {
                delay = 8;
            }
            slot!(game, Crocomire[0].base.instruction_timer, |old| old
                .wrapping_add(delay));
        }
        slot!(
            game,
            Crocomire[0].var_B = count | slot!(game, Crocomire[0].var_B) & 0xb7f0 | 0x800
        );
        slot!(game, Crocomire[0].var_E = 10);
    }
    slot!(game, Crocomire[0].base.flash_timer, |old| old
        .wrapping_add(14));
    slot!(game, Crocomire[0].base.ai_handler_bits, |old| old | 2);
}

pub fn select_melt_animation<const SKELETON: bool, H: Host + ?Sized>(key: Key, game: &mut H) {
    game.lease().edit_actor(key, AccessMode::Native, |enemy| {
        let instruction = match enemy.y.whole as i16 {
            y if y.wrapping_sub(280) >= 0 => addr_kCrocomire_Ilist_BF64 as u16,
            y if y.wrapping_sub(264) >= 0 => addr_kCrocomire_Ilist_BF6C as u16,
            y if y.wrapping_sub(248) >= 0 => addr_kCrocomire_Ilist_BF72 as u16,
            _ => addr_kCrocomire_Ilist_BF78 as u16,
        };
        // Each skeleton list starts 0x1a bytes after its corresponding flesh list.
        enemy.instruction = instruction + if SKELETON { 0x1a } else { 0 };
        enemy.instruction_delay = 1;
    });
}

pub fn melt_skeleton_up<H: Host + ?Sized>(game: &mut H) {
    select_melt_animation::<true, _>(Key::from_legacy(word!(game.cur_enemy_index)), game);
    emit_lava_plume(game);
    melt_up(game);
}

pub fn melt_flesh_down<H: Host + ?Sized>(game: &mut H) {
    emit_lava_plume(game);
    select_melt_animation::<false, _>(Key::from_legacy(0), game);
    melt_down(game);
}

pub fn emit_lava_plume<H: Host + ?Sized>(game: &mut H) {
    if word!(game.g_word_7E9018 -= 1, reread) == 0 {
        word!(game.g_word_7E9018 = 6);
        let mut v0: u16 = (word!(game.random_number)) & (0x3fu16);
        if (word!(game.random_number) & 2) == 0 {
            v0 = !(v0 as i32) as u16;
        }
        let x: u16 = (slot!(game, Crocomire[0i32].base.x_pos)).wrapping_add(v0);
        let y: u16 = (word!(game.lava_acid_y_pos) as i32 + 16
            - (((word!(game.random_number)) & (0x1f00u16)) as i32 >> 8))
            as u16;
        game.create_sprite(x, y, 21, 0);
    }
}

pub fn melt_rumble_down<H: Host + ?Sized>(game: &mut H) {
    rumble(game);
    emit_lava_plume(game);
    melt_down(game);
}

pub fn melt_flesh_up<H: Host + ?Sized>(game: &mut H) {
    emit_lava_plume(game);
    select_melt_animation::<false, _>(Key::from_legacy(word!(game.cur_enemy_index)), game);
    melt_up(game);
}

pub fn melt_rumble_up<H: Host + ?Sized>(game: &mut H) {
    rumble(game);
    emit_lava_plume(game);
    melt_up(game);
}

pub fn queue_tilemap_upload<H: Host + ?Sized>(size: u16, bg_mask: u8, game: &mut H) {
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    // The native bank word overlaps the first destination byte, including on BE.
    crate::word_transfers!(game, ascending, 1;
        native(entry) <- value(size),
        native(entry + 2) <- value(0x2000),
        native(entry + 4) <- value(126),
    );
    crate::word_transfers!(game, ascending, 1;
        native(entry + 5) <- value(u16::from(game.ram().read_byte(0x59) & bg_mask) << 8),
    );
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

fn load_melt_tilemap<H: Host + ?Sized>(table: u32, game: &mut H) {
    // The tilemap serves its 256 tiles while the loaded image is the imported
    // allocation; otherwise the original loads scan for the $FFFF end.
    if let Some(tiles) = game.crocomire_melt_tilemap(table) {
        for (index, tile) in tiles.into_iter().enumerate() {
            word!(game.tilemap_stuff[(index + 32)] = tile);
        }
        queue_tilemap_upload((2 * MELT_TILES + 1024) as u16, 0xfc, game);
        return;
    }
    let mut i: i32 = 0;
    loop {
        let index: i32 = i >> 1;
        if crate::cartridge::rom_table_word!(game, table, index as isize) == 0xffff {
            break;
        }
        word!(
            game.tilemap_stuff[(index + 32)] =
                crate::cartridge::rom_table_word!(game, table, index as isize)
        );
        i += 2;
    }
    queue_tilemap_upload((i + 1024) as u16, 0xfc, game);
}

pub fn prepare_flesh_tilemap<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(0x68c, 48);
    word!(game.croco_target_0688 = 48);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_BF64 as u16, instruction_timer = 1);
    word!(game.nmi_flag_bg2_enemy_vram_transfer = 1);
    let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
    slot!(game, Crocomire[E1].base; current_instruction = addr_kCrocomire_Ilist_BF98 as u16, instruction_timer = 1);
    slot!(game, Crocomire[E1].base.properties, |old| old & 0xd2ff
        | 0x2c00);
    slot!(
        game,
        Crocomire[E1].base.x_pos = slot!(game, Crocomire[E].base.x_pos)
    );
    slot!(
        game,
        Crocomire[E1].base.y_pos = (slot!(game, Crocomire[E].base.y_pos)).wrapping_add(16)
    );
    crate::word_transfers!(game, (4), 256;
        le(tilemap_stuff, 0) <- value(824),
        le(tilemap_stuff, 1) <- value(824),
    );
    load_melt_tilemap(FLESH_TILEMAP_ADDRESS, game);
}

pub fn prepare_skeleton_tilemap<H: Host + ?Sized>(game: &mut H) {
    fill_scroll(game);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].base.instruction_timer = 1);
    game.ram().write_byte(0x68c, 48);
    word!(game.croco_target_0688 = 48);
    slot!(
        game,
        Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_BF7E as u16
    );
    crate::word_transfers!(game, ascending, 1024; le(tilemap_stuff, 0) <- value(824));
    load_melt_tilemap(SKELETON_TILEMAP_ADDRESS, game);
}

macro_rules! native_graphics_word {
    ($game:ident, $address:expr, $offset:expr) => {{
        let address = $address;
        let record = crate::cartridge::native_record((address >> 16) as u8, address as u16, $game);
        record.image.table_word(record.base + $offset as usize, 0)
    }};
}

pub fn queue_melt_graphics<H: Host + ?Sized>(offset: u16, live_size: bool, game: &mut H) {
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    for (source, destination) in [(0, 0), (6, 2), (4, 4), (2, 5)] {
        let value = native_graphics_word!(game, 0xa49bc5 + source, {
            if source == 0 && live_size {
                word!(game.g_word_7E068A)
            } else {
                offset
            }
        });
        crate::word_transfers!(game, ascending, 1; native(entry + destination) <- value(value));
    }
}

pub fn advance_melt_upload<H: Host + ?Sized>(game: &mut H) {
    let offset = word!(game.g_word_7E068A);
    let tail = word!(game.vram_write_queue_tail);
    if native_graphics_word!(game, 0xa49bc5, word!(game.g_word_7E068A)) == 0xffff {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        word!(game.g_word_7E068A = 0);
    } else {
        queue_melt_graphics(offset, true, game);
        word!(game.g_word_7E068A = offset.wrapping_add(8));
        word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    };
}

pub fn prepare_melt_graphics<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| (old).wrapping_add(2));
    word! { game;
        g_word_7E0692 = 256;
        croco_cur_vline_idx = 0;
        g_word_7E0698 = native_graphics_word!(game, 0xa49bc5, word!(game.croco_word_7E069A));
        g_word_7E0694 = word!(game.g_word_7E0698);
        g_word_7E0696 = native_graphics_word!(game, 0xa49bc7, word!(game.croco_word_7E069A));
        g_word_7E068E = native_graphics_word!(game, 0xa49bc9, word!(game.croco_word_7E069A));
    }
    let bank: u8 = native_graphics_word!(game, 0xa49bcb, word!(game.croco_word_7E069A)) as u8;
    let mut i: i32 = word!(game.croco_word_7E069A) as i32 + 8;
    loop {
        let mut v2: u16 = native_graphics_word!(game, 0xa49bc5, i);
        if v2 == 0xffff {
            break;
        }
        let mut v3: u16 = native_graphics_word!(game, 0xa49bc7, i);
        for _ in 0..=word!(game.g_word_7E068E) {
            crate::word_transfers!(game, ascending, 1; native(v3) <- value(native_graphics_word!(game, (u32::from(bank) << 16) | u32::from(v2), 0)));
            v3 = (v3).wrapping_add(2);
            v2 = (v2).wrapping_add(2);
        }
        i = i as u16 as i32 + 4;
    }
    word! {game; croco_word_7E069A = (i + 2) as u16; g_word_7E068A = (i + 2) as u16; }
    crate::word_transfers!(game, descending, 65; native(0x71c) <- value(0));
}

pub fn power_bomb<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    if slot!(game, Crocomire[E].var_A) == 0 {
        slot!(game, Crocomire[E].var_D = 3);
        if slot!(game, Crocomire[E].var_C) != 24 {
            slot!(game, Crocomire[E].var_B, |old| old & 0x3ff0 | 0x8000);
            slot!(game, Crocomire[E].var_E = 10);
            slot!(game, Crocomire[E].base.flash_timer, |old| old
                .wrapping_add(4));
            slot!(game, Crocomire[E].base.ai_handler_bits, |old| old | 2u16);
            slot!(game, Crocomire[E].var_C = 24);
            let mut spritemap_pointer: u16 = slot!(game, Crocomire[E].base.spritemap_pointer);
            let mut n: i32 =
                native_graphics_word!(game, 0xa40000 | u32::from(spritemap_pointer), 0) as i32;
            let instruction = loop {
                let tag = native_graphics_word!(game, 0xa40000 | u32::from(spritemap_pointer), 6);
                if u32::from(tag) == addr_kCrocomire_BigSprmap_D600 {
                    break addr_kCrocomire_Ilist_BDAE as u16;
                }
                if u32::from(tag) == addr_kCrocomire_BigSprmap_D51C {
                    break addr_kCrocomire_Ilist_BDB2 as u16;
                }
                spritemap_pointer = (spritemap_pointer).wrapping_add(8);
                n -= 1;
                if n == 0 {
                    break addr_kCrocomire_Ilist_BDB6 as u16;
                }
            };
            slot!(game, Crocomire[E].base; current_instruction = instruction, instruction_timer = 1);
        }
    }
}

pub fn approach_wall<H: Host + ?Sized>(game: &mut H) {
    update_wall(game);
    crate::word_transfers!(game, ascending, 1; native(0xcd24) <- value(257));
    if word!(game.samus_x_pos) as i32 - 1312i32 & 0x8000 == 0 {
        crate::word_transfers!(game, ascending, 1; native(0xcd24) <- value(256));
    }
    update_background(word!(game.cur_enemy_index), game);
}

fn update_wall<H: Host + ?Sized>(game: &mut H) {
    let E0: i32 = 0;
    if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1536i32 & 0x8000 != 0 {
        word! { game, 0; kraid_unk9000, g_word_7E9002, g_word_7E9006, g_word_7E900A }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1552i32 & 0x8000 != 0 {
        if word!(game.kraid_unk9000) == 0 {
            word!(game.kraid_unk9000 = 1);
            game.set_projectile_spawn_point(1536, 176);
            game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
        }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1552i32 & 0x8000 != 0 {
        word! { game, 0; g_word_7E9002, g_word_7E9006, g_word_7E900A }
    } else if slot!(game, Crocomire[E0].base.x_pos) as i32 - 1568i32 & 0x8000 != 0 {
        if word!(game.g_word_7E9002) == 0 {
            word!(game.g_word_7E9002 = 1);
            game.spawn_plm(0x61, 0xb, 0xb74b);
            game.set_projectile_spawn_point(1568, 176);
            game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
        }
    } else {
        let x_pos: u16 = slot!(game, Crocomire[E0].base.x_pos);
        if x_pos as i32 - 1568i32 & 0x8000 != 0 {
            word! { game, 0; g_word_7E9006, g_word_7E900A }
        } else if x_pos as i32 - 1584i32 & 0x8000 != 0 {
            if word!(game.g_word_7E9006) == 0 {
                word!(game.g_word_7E9006 = 1);
                game.spawn_plm(0x62, 0xb, 0xb74b);
                game.spawn_plm(0x63, 0xb, 0xb74b);
                game.set_projectile_spawn_point(1584, 176);
                game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
            }
        } else if ((x_pos) as i16).wrapping_sub((1600u16) as i16) >= 0 {
            demolish_wall(game);
            slot!(game, Crocomire[E0].var_20 = 1);
            word!(game.g_word_7E9018 = 1);
            let _v3: u16 = word!(game.cur_enemy_index);
            let EK: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, Crocomire[EK].var_A, |old| (old).wrapping_add(2));
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].var_00 = 2);
            }
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].var_01 = 0);
            }
            game.queue_sound(1, 0x3b, 6);
            crate::word_transfers!(game, ascending, 1; native(0x9016) <- value(0));
            game.spawn_plm(0x4e, 0x3, 0xb757);
            slot!(game, Crocomire[EK].base; current_instruction = addr_kCrocomire_Ilist_BFB0 as u16, instruction_timer = 1);
            slot!(game, Crocomire[EK].base.properties, |old| (old)
                | ((kEnemyProps_Tangible) as u16));
            let E1: i32 = word!(game.cur_enemy_index) as i32 + 64;
            slot!(game, Crocomire[E1].base; instruction_timer = 0x7fff, current_instruction = addr_kCrocomire_Ilist_BF62 as u16);
            slot!(
                game,
                Crocomire[0x40i32].base.properties =
                    slot!(game, Crocomire[0x40i32].base.properties) | (0x100u16)
            );

            slot!(game, Crocomire[EK]; var_E = 0, var_F = 0);
            slot!(game, Crocomire[EK].var_D = 2048);
            slot!(game, Crocomire[0i32].base.y_height = 16);
        }
    };
}

fn demolish_wall<H: Host + ?Sized>(game: &mut H) {
    for x in 0x61..=0x6a {
        game.spawn_plm(x, 0xb, 0xb74f);
    }
    for (offset, y) in [176, 192, 176, 192, 192, 192, 192].into_iter().enumerate() {
        game.set_projectile_spawn_point(1536 + 16 * offset as u16, y);
        game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
    }
}

pub fn advance_phase<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
}

pub fn wait_for_melt<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    let crocom_var_D: u16 = slot!(game, Crocomire[E].var_D);
    if crocom_var_D != 0 {
        slot!(game, Crocomire[E].var_D = (crocom_var_D).wrapping_sub(1));
    } else {
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
        slot!(game, Crocomire[E].var_E = 768);
    };
}

pub fn rumble_and_wait<H: Host + ?Sized>(game: &mut H) {
    rumble(game);
    game.align_crocomire_background();
    wait_for_melt(game);
}

pub fn prepare_spike_wall<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    let crocom_var_D: u16 = slot!(game, Crocomire[E].var_D);
    if crocom_var_D != 0 {
        slot!(game, Crocomire[E].var_D = (crocom_var_D).wrapping_sub(1));
        let v3: i16 = word!(game.croco_target_0688) as i16;
        let v4: i32 = word!(game.croco_target_0688) as i32 >> 1;
        let tile = game.crocomire_table_word(SPIKE_WALL_TILE_ADDRESS, v4 as isize);
        if tile != 0xffff {
            let v5: u16 = word!(game.vram_write_queue_tail);
            let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
            crate::word_transfers!(game, ascending, 1; native(entry) <- value(512));
            crate::word_transfers!(game, ascending, 1;
                native(entry + 2) <- value(game.crocomire_table_word(SPIKE_WALL_DESTINATION_ADDRESS, v4 as isize,
                )),
            );
            crate::word_transfers!(game, ascending, 1; native(entry + 4) <- value(173));
            crate::word_transfers!(game, ascending, 1;
                native(entry + 5) <- value((game.crocomire_table_word(SPIKE_WALL_TILE_ADDRESS, v4 as isize,
                ) as i32
                    + ((game.ram().read_byte(0x52) as i32 & 7) << 13)) as u16),
            );
            word!(game.vram_write_queue_tail = (v5).wrapping_add(7));
            word!(game.croco_target_0688 = ((v3) as u16).wrapping_add(2));
        }
    } else {
        slot!(game, Crocomire[E].base.x_pos = 480);
        slot!(game, Crocomire[E].base.y_pos = 54);
        slot!(game, Crocomire[0x80i32].var_D = 80);
        slot!(game, Crocomire[E]; var_E = 0, var_F = 0);
        game.spawn_plm(0x20, 0x3, 0xb753);
        game.spawn_plm(0x1e, 0x3, 0xb757);
        game.spawn_plm(0x70, 0xb, 0xb747);
        game.queue_sound(1, 0x29, 6);
        slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E158 as u16, instruction_timer = 1);
        slot!(game, Crocomire[E].base.palette_index = 0);
        crate::word_transfers!(game, descending, 16; le(palette_buffer, 159) <- rom(0xa4b8fd, 15));
        game.clear_enemy_projectiles();
        let mut v8: u16 = 8;
        for _ in 0..8 {
            let k = word!(game.cur_enemy_index);
            v8 = game.spawn_projectile(v8, k, addr_kEproj_CrocomireSpikeWallPieces as u16);
        }
        game.queue_sound(1, 0x30, 6);
        advance_phase(game);
    };
}

pub fn advance_melt_scroll<H: Host + ?Sized>(game: &mut H) {
    emit_lava_plume(game);
    let E0: i32 = 0;
    let E1: i32 = 0x40;
    let E2: i32 = 0x80;
    let mut crocom_var_D: u16 = slot!(game, Crocomire[E2].var_D);
    slot!(game, Crocomire[E1].var_D, |old| old.wrapping_sub(1));
    if (slot!(game, Crocomire[E1].var_D) & 2) != 0 {
        crocom_var_D = (crocom_var_D).wrapping_add(4);
    }
    slot!(game, Crocomire[E0].base.x_pos = crocom_var_D);
    if erase_melt_columns(game) != 0 {
        update_background(word!(game.cur_enemy_index), game);
        let mut v4: u16 = (word!(game.g_word_7E0694)).wrapping_sub(3);
        let skip_tilemap_update: bool = (word!(game.g_word_7E0694) as i32 - 19i32 & 0x8000 != 0)
            && ((((word!(game.g_word_7E0692)) as i16).wrapping_sub(20480) >= 0)
                || ({
                    v4 = 16;
                    false
                }));
        if skip_tilemap_update {
        } else {
            word!(game.g_word_7E0694 = v4);
            let mut v5: u16 = (word!(game.g_word_7E0692)).wrapping_add(384);
            if word!(game.g_word_7E0692) as i32 - 20096i32 & 0x8000 == 0 {
                v5 = 20480;
            }
            word!(game.g_word_7E0692 = v5);
            let mut r18: u16 = 0;
            let mut v6: u16 = (2 * (slot!(game, Crocomire[E1].base.y_pos) as i32 - 72i32)) as u16;
            let mut v7: u16 = word!(game.g_word_7E0694);
            word!(game.message_box_animation_y1 = word!(game.g_word_7E0694));
            do_until! { ((v7) as i16).wrapping_sub((word!(game.g_word_7E0698)) as i16) >= 0 ;
                word!(
                    game.crocomire_bg2_scroll_hdma_data[(v6 >> 1)] =
                        (word!(game.reg_BG2VOFS) as i32 + v7 as i32
                            - word!(game.message_box_animation_y1) as i32)
                            as u16
                );
                let v8: bool = u16::overflowing_add(word!(game.g_word_7E0692), r18).1;
                r18 = (r18).wrapping_add(word!(game.g_word_7E0692));
                if !v8 {
                    v7 = v7.wrapping_add(1);
                }
                word!(game.message_box_animation_y1 += 1);
                v6 = (v6).wrapping_add(2);
            }
            if ((v6) as i16).wrapping_sub(512) < 0 {
                let v9: u16 = word!(game.reg_BG2VOFS);
                do_until! { ((v6) as i16).wrapping_sub(512) >= 0 ;
                    word!(game.crocomire_bg2_scroll_hdma_data[(v6 >> 1)] = v9);
                    v6 = (v6).wrapping_add(2);
                }
            }
            return;
        }
    }
    let EK: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[EK].var_A, |old| (old).wrapping_add(2));
    let mut i: i32 = word!(game.croco_word_7E069A) as i32;
    while native_graphics_word!(game, 0xa49bc5, i) != 0xffff {
        i += 8;
    }
    word!(game.croco_word_7E069A = (i + 2) as u16);
    word!(game.hdma_object_channels_bitmask[(slot!(game, Crocomire[E0].var_1F) >> 1)] = 0);
}

fn erase_melt_columns<H: Host + ?Sized>(game: &mut H) -> u16 {
    let mut n: i32 = game.ram().read_byte(0x68c) as i32;
    loop {
        if word!(game.croco_cur_vline_idx) > 48 {
            return 1;
        }
        let rr: i32 = game.native_image().read(
            sm_data::cartridge::lorom_offset(0xa49697) as usize
                + usize::from(word!(game.croco_cur_vline_idx)),
        ) as i32;
        if (game.ram().read_byte(0x69c + rr as usize) as i8)
            .wrapping_sub((word!(game.croco_target_0688)) as i8)
            < 0
        {
            break;
        }
        if word!(game.croco_cur_vline_idx += 1, reread) >= 128 {
            word!(game.croco_cur_vline_idx = 0);
            return 0;
        }
    }
    // The top-of-loop return dominates this break; intervening reads do not change the index.
    let vline_idx: i32 = game.native_image().read(
        sm_data::cartridge::lorom_offset(0xa49697) as usize
            + usize::from(word!(game.croco_cur_vline_idx)),
    ) as i32;
    let mask: u8 = !(0x80u8 >> (vline_idx & 7));
    do_until! { n == 0 ;
        let q: i32 = game.ram().read_byte(0x69c + vline_idx as usize) as i32;
        let j: i32 = 2 * (q & 7) + ((q & !(7i32)) << 6) + 4 * (vline_idx & !(7i32));
        // Melting tiles span the $4000 graphics buffer, beyond the menu backup prefix.
        for plane_offset in [0, 1, 16, 17] {
            let at = 0x4000 + j as usize + plane_offset;
            game.ram().write_byte(at, game.ram().read_byte(at) & mask);
        }
        if game.ram().read_byte(0x69c + vline_idx as usize) == 48 {
            break;
        }
        let at = 0x69c + vline_idx as usize;
        game.ram().write_byte(at, game.ram().read_byte(at).wrapping_add(1));
        n -= 1;
    }
    let (v7, v8) = loop {
        let v7 = (word!(game.g_word_7E068A)).wrapping_add(word!(game.croco_word_7E069A));
        let v8 = word!(game.vram_write_queue_tail);
        if native_graphics_word!(game, 0xa49bc5, v7) != 0xffff {
            break (v7, v8);
        }
        word!(game.g_word_7E068A = 0);
    };
    queue_melt_graphics(v7, false, game);
    word!(game.vram_write_queue_tail = (v8).wrapping_add(7));
    word!(game.g_word_7E068A += 8);
    1
}

/// Initializes the defeated or active boss room with the original ordered effects.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    word!(game.boss_id = 6);
    crate::word_transfers!(game, ascending, 2048; le(tilemap_stuff, 0) <- value(824));
    if ({
        let area = word!(game.area_index);
        game.boss_word_native(area)
    } & 2)
        != 0
    {
        crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(257));
        crate::word_transfers!(game, ascending, 1; native(0xcd22) <- value(257));
        word!(game.croco_target_0688 = 0);
        let E: i32 = 0;
        slot!(game, Crocomire[E].base.properties, |old| old & 0x7bff
            | 0x400);
        game.spawn_plm(0x20, 0x3, 0xb753);
        game.spawn_plm(0x1e, 0x3, 0xb753);
        game.spawn_plm(0x61, 0xb, 0xb747);
        slot!(game, Crocomire[E].var_A = 84);
        slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E1CC as u16, instruction_timer = 1);
        slot!(game, Crocomire[E].base.x_pos = 576);
        slot!(game, Crocomire[E].base; y_pos = 144, y_height = 28, x_width = 40);
        queue_tilemap_upload(2048, 0xfc, game);
    } else {
        game.disable_minimap_for_boss();
        word!(game.croco_word_7E069A = 0);
        let E_0: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E_0]; var_A = 0, var_E = 0);
        crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(0));
        crate::word_transfers!(game, descending, 17;
            le(target_palettes, 176) <- rom(0xa4b8bd, 16),
            le(target_palettes, 224) <- rom(0xa4b8dd, 16),
        );
        slot!(game, Crocomire[E_0].var_C = 4);
        word! {game; UNUSED_word_7E179E = 16; camera_distance_index = 2;
        enemy_bg2_tilemap_size = 1024; }
        slot!(game, Crocomire[E_0].base; current_instruction = addr_kCrocomire_Ilist_BADE as u16, extra_properties = (slot!(game, Crocomire[E_0].base.extra_properties)) | (4u16), instruction_timer = 1);
    }
}

pub fn complete_skeleton_melt<H: Host + ?Sized>(game: &mut H) {
    select_melt_animation::<true, _>(Key::from_legacy(0), game);
    emit_lava_plume(game);
    melt_down(game);
    let E: i32 = 0;
    if slot!(game, Crocomire[E].var_A) == 62 {
        game.queue_music_delayed(6);
        slot!(game, Crocomire[E].var_A = 88);
        slot!(
            game,
            Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_E1D2 as u16
        );
        crate::word_transfers!(game, ascending, 1; native(0xcd24) <- value(257));
        word!(game.debug_disable_minimap = 0);
        slot!(
            game,
            Crocomire[0x40i32].base.properties =
                slot!(game, Crocomire[0x40i32].base.properties) | (0x200u16)
        );

        game.spawn_plm(0x4e, 0x3, 0xb753);
        word! { game, 0; camera_distance_index, croco_target_0688 }
    }
}

pub fn emit_melt_debris<H: Host + ?Sized>(game: &mut H) {
    let v0: u16 = u16::from_ne_bytes([game.ram().read_byte(0x9016), game.ram().read_byte(0x9017)]);
    if u16::from_ne_bytes([game.ram().read_byte(0x9016), game.ram().read_byte(0x9017)]) as i32
        - 22i32
        & 0x8000
        != 0
    {
        let v1: u16 =
            u16::from_ne_bytes([game.ram().read_byte(0x9016), game.ram().read_byte(0x9017)]);
        checked_word!(game, 0x9016; += 2);
        let parameter = game.crocomire_table_word(BRIDGE_DEBRIS_ADDRESS, (v0 >> 1) as isize);
        game.spawn_projectile(parameter, v1, addr_stru_868F9D as u16);
    }
    melt_rumble_down(game);
}

pub fn release_skeleton_room<H: Host + ?Sized>(game: &mut H) {
    if (word!(game.samus_x_pos) as i16).wrapping_sub(640) >= 0 {
        return;
    }
    game.queue_music_delayed(5);
    game.lease().write_slice_native(0xcd23, 256);
    {
        game.spawn_plm(0x30, 0x3, 0xb757);
    }
    word!(game.camera_distance_index = 6);
    let mut runtime = game.lease();
    // Fixed, disjoint records; no callback or fallible operation between these writes.
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.properties = enemy.properties & 0x7bff | kEnemyProps_Tangible as u16;
        enemy.variables[3] = 4;
        enemy.variables[1] = 0;
        enemy.radius.y = 56;
        enemy.variables[0] = enemy.variables[0].wrapping_add(2);
    });
    runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
        enemy.properties |= 0x500;
        enemy.variables[3] = 0;
    });
    for (key, countdown) in [(0x80, 10), (0xc0, 1)] {
        runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
            enemy.variables[3] = countdown
        });
    }
}

fn accelerated_speed(fraction: u16, whole: u16, acceleration: u32, limit: u16) -> u32 {
    let speed = ((u32::from(whole) << 16) | u32::from(fraction)).wrapping_add(acceleration);
    if ((speed >> 16) as i16).wrapping_sub(limit as i16) >= 0 {
        speed & 0xffff | (u32::from(limit) << 16)
    } else {
        speed
    }
}

pub fn reposition_skeleton<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.x.whole = enemy.x.whole.wrapping_sub(2);
            if (enemy.x.whole as i16).wrapping_sub(480) < 0 {
                enemy.x.whole = 480;
                enemy.y.whole = 54;
                enemy.variables[0] = 62;
            } else {
                enemy.y.whole = 220;
            }
        });
}

pub fn launch_skeleton<H: Host + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(224) < 0 {
            let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 0x8000, 2);
            enemy.variables[4] = speed as u16;
            enemy.pre_instruction = (speed >> 16) as u16;
            enemy.x.advance(speed as i32);
        }
    });
    let finished = runtime.edit_actor(Key::from_legacy(0x80), AccessMode::Native, |enemy| {
        let previous = enemy.variables[3];
        enemy.variables[3] = previous.saturating_sub(1);
        previous == 1
    });
    if finished {
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[4] = 0;
            enemy.instruction = addr_kCrocomire_Ilist_E14A as u16;
            enemy.instruction_delay = 1;
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
    }
}

pub fn land_skeleton<H: Host + ?Sized>(game: &mut H) {
    let impact = game
        .lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 2048, 5);
            enemy.variables[4] = speed as u16;
            enemy.pre_instruction = (speed >> 16) as u16;
            enemy.y.advance(0xe000);
            enemy.x.advance(speed as i32);
            (enemy.x.whole as i16).wrapping_sub(576) >= 0
        });
    if impact {
        game.queue_sound(1, 0x25, 6);
        let mut runtime = game.lease();
        let palette = runtime
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .palette;
        runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.palette = palette
        });
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.instruction = addr_kCrocomire_Ilist_E158 as u16;
            enemy.instruction_delay = 1;
            enemy.variables[0] = enemy.variables[0].wrapping_add(2);
        });
    }
}

pub fn finish_skeleton_escape<H: Host + ?Sized>(game: &mut H) {
    let finished = game
        .lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            if (enemy.instruction as i16).wrapping_add(0x1e3a) < 0 {
                let speed = accelerated_speed(enemy.variables[4], enemy.pre_instruction, 4096, 6);
                enemy.variables[4] = speed as u16;
                enemy.pre_instruction = (speed >> 16) as u16;
                false
            } else {
                enemy.instruction = addr_kCrocomire_Ilist_E1CC as u16;
                enemy.instruction_delay = 1;
                enemy.x.whole = enemy.x.whole.wrapping_add(64);
                enemy.y.whole = enemy.y.whole.wrapping_add(21);
                enemy.radius.y = 28;
                enemy.radius.x = 40;
                true
            }
        });
    if finished {
        {
            game.spawn_plm(0x30, 0x3, 0xb753);
        }
        game.scatter_crocomire_drop(0);
        advance_phase(game);
    }
}

pub fn await_skeleton_script<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            if (enemy.instruction as i16).wrapping_add(0x1e3a) >= 0 {
                enemy.variables[0] = enemy.variables[0].wrapping_add(2);
            }
        });
}

pub fn open_skeleton_exit<H: Host + ?Sized>(game: &mut H) {
    {
        let mut runtime = game.lease();
        runtime.write_slice_native(0xcd20, 257);
        runtime.write_slice_native(0xcd22, 257);
    }
    {
        game.spawn_plm(0x1e, 0x3, 0xb753);
    }
    advance_phase(game);
}

pub fn prepare_scroll_hdma<H: Host + ?Sized>(game: &mut H) {
    let mut v1: u16 = (2 * (slot!(game, Crocomire[0x40i32].base.y_pos) as i32 - 72i32)) as u16;
    let v2: u16 = word!(game.reg_BG2VOFS);
    do_until! { (v1 & 0x8000) != 0 ;
        word!(game.crocomire_bg2_scroll_hdma_data[(v1 >> 1)] = v2);
        v1 = (v1).wrapping_sub(2);
    }
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    slot!(game, Crocomire[E].var_A, |old| old.wrapping_add(1));
    let v4: i32 = 0;
    slot!(
        game,
        Crocomire[0x80i32].var_D = slot!(game, Crocomire[v4].base.x_pos)
    );
    crate::word_transfers!(game, ascending, 1; native(0xcae0) <- value(255));
    crate::word_transfers!(game, ascending, 1; native(0xcae1) <- value(-13584i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(0xcae3) <- value(225));
    crate::word_transfers!(game, ascending, 1; native(0xcae4) <- value(-13330i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(0xcae6) <- value(0));
}

// Returned only for the two synchronous hardware phases of the fixed actor-0 driver.
pub enum ScrollRequest {
    Flesh = 0x77,
    Skeleton = 0x2d,
}
pub fn execute_phase<H: Host + ?Sized>(game: &mut H) -> Option<ScrollRequest> {
    let phases: [Option<fn(&mut H)>; 45] = [
        Some(approach_wall as fn(&mut H)),
        Some(emit_melt_debris as fn(&mut H)),
        Some(rumble_and_wait as fn(&mut H)),
        Some(melt_rumble_up as fn(&mut H)),
        Some(melt_rumble_down as fn(&mut H)),
        Some(rumble_and_wait as fn(&mut H)),
        Some(melt_rumble_up as fn(&mut H)),
        Some(melt_rumble_down as fn(&mut H)),
        Some(prepare_flesh_tilemap as fn(&mut H)),
        Some(prepare_melt_graphics as fn(&mut H)),
        Some(advance_melt_upload as fn(&mut H)),
        Some(melt_flesh_up as fn(&mut H)),
        None,
        Some(advance_melt_scroll as fn(&mut H)),
        Some(reset_melt_tilemap as fn(&mut H)),
        Some(melt_flesh_down as fn(&mut H)),
        Some(wait_for_melt as fn(&mut H)),
        Some(melt_flesh_up as fn(&mut H)),
        Some(melt_flesh_down as fn(&mut H)),
        Some(wait_for_melt as fn(&mut H)),
        Some(melt_flesh_up as fn(&mut H)),
        Some(melt_flesh_down as fn(&mut H)),
        Some(prepare_skeleton_tilemap as fn(&mut H)),
        Some(prepare_melt_graphics as fn(&mut H)),
        Some(advance_melt_upload as fn(&mut H)),
        Some(advance_selected_phase as fn(&mut H)),
        Some(melt_skeleton_up as fn(&mut H)),
        None,
        Some(advance_melt_scroll as fn(&mut H)),
        Some(reset_melt_tilemap as fn(&mut H)),
        Some(complete_skeleton_melt as fn(&mut H)),
        Some(release_skeleton_room as fn(&mut H)),
        Some(wall_step as fn(&mut H)),
        Some(prepare_spike_wall as fn(&mut H)),
        Some(launch_skeleton as fn(&mut H)),
        Some(land_skeleton as fn(&mut H)),
        Some(finish_skeleton_escape as fn(&mut H)),
        Some(open_skeleton_exit as fn(&mut H)),
        Some(await_skeleton_script as fn(&mut H)),
        Some(advance_phase as fn(&mut H)),
        Some(finish_death as fn(&mut H)),
        Some(idle_phase as fn(&mut H)),
        Some(advance_phase as fn(&mut H)),
        Some(reset_scroll as fn(&mut H)),
        Some(reposition_skeleton as fn(&mut H)),
    ];
    let phase = (slot!(game, Crocomire[0i32].var_A) as i32 >> 1) as usize;
    match phases[phase] {
        Some(execute) => {
            execute(game);
            None
        }
        None => Some(if phase == 12 {
            ScrollRequest::Flesh
        } else {
            ScrollRequest::Skeleton
        }),
    }
}
fn idle_phase<H: Host + ?Sized>(_game: &mut H) {}
fn advance_selected_phase<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].var_A, |old| (old).wrapping_add(2));
}
fn reset_melt_tilemap<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E]; var_E = 0, var_F = 0);
    slot!(game, Crocomire[E].var_D = 2048);
    crate::word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
    queue_tilemap_upload(2048, 0xfe, game);
    advance_phase(game);
}
fn reset_scroll<H: Host + ?Sized>(game: &mut H) {
    {
        word! { game, 0; reg_BG2HOFS, reg_BG2VOFS }
    }
}
fn spawn_random_dust<H: Host + ?Sized>(game: &mut H) {
    let random = word!(game.random_number);
    let mut offset = random & 0x1f;
    if (random as i16).wrapping_sub(4096) >= 0 {
        offset = offset.wrapping_neg();
    }
    spawn_dust(EnemyScript::Stop, offset, game);
}
fn spawn_dust<H: Host + ?Sized>(jp: EnemyScript, offset: u16, game: &mut H) -> EnemyScript {
    let (x, y) = {
        let runtime = game.lease();
        let actor = runtime.resolve_actor(Key::from_legacy(0), AccessMode::Native);
        let random = runtime.word(ram::random_number) & 7;
        (
            (actor.x.whole as u32 + offset as u32 + random as u32) as u16,
            actor.radius.y.wrapping_add(actor.y.whole).wrapping_sub(16),
        )
    };
    game.set_projectile_spawn_point(x, y);
    game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
    jp
}

/// True when blocked.
fn walk<H: Host + ?Sized>(k: u16, pixels: i32, game: &mut H) -> bool {
    game.move_horizontal(k, pixels << 16, Slopes::Ignore)
}
fn finish_death<H: Host + ?Sized>(game: &mut H) {
    game.queue_music_delayed(6);
    word!(game.camera_distance_index = 0);
    let area = word!(game.area_index);
    let bits = game.boss_word(area);
    game.set_boss_word(area, bits | 2);
    game.queue_music_delayed(6);
    spawn_dust(EnemyScript::Stop, 0xfff0, game);
    spawn_dust(EnemyScript::Stop, 0x10, game);
    advance_phase(game);
}

pub fn hurt<H: Host + ?Sized>(game: &mut H) {
    update_background(word!(game.cur_enemy_index), game);
    update_palette(game);
}
pub fn initialize_tongue<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    if ({
        let area = word!(game.area_index);
        game.boss_word_native(area)
    } & 2)
        != 0
    {
        slot!(game, Crocomire[E].base.properties, |old| old & 0xdcff
            | 0x300);
    } else {
        slot!(
            game,
            Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_BE56 as u16
        );
        slot!(game, Crocomire[E].base.extra_properties, |old| (old)
            | (0x404u16));
        slot!(game, Crocomire[E].base.instruction_timer = 1);
        slot!(game, Crocomire[E].var_A = 23);
        slot!(game, Crocomire[E].base.palette_index = 3584);
    };
}
const DUST_OFFSETS: [u16; 13] = [
    0xffe0, 0, 0xfff0, 0x10, 0, 8, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0x40,
];
pub fn instruction<H: Host + ?Sized>(
    ea: u32,
    k: u16,
    jp: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    if matches!(ea, 10784762 | 10784767 | 10784911 | 10784916) {
        spawn_random_dust(game);
    }
    match ea {
        10783995 => game.queue_sound(1, 0x74, 6),
        10784007 => game.queue_sound(1, 0x25, 6),
        10784019 => game.queue_sound(1, 0x75, 6),
        10784711 => {
            {
                let mut runtime = game.lease();
                runtime.write_slice_le(ram::earthquake_type, 4);
                runtime.write_slice_le(ram::earthquake_timer, 5);
            }
            game.queue_sound(1, 0x76, 6);
        }
        10784735 | 10784762 | 10784767 => {
            let hit = game
                .lease()
                .resolve_actor(Key::from_legacy(0), AccessMode::Native)
                .variables[1]
                & 0x800
                != 0;
            if !hit {
                walk(word!(game.cur_enemy_index), -4, game);
            }
        }
        10784797 => {
            return if walk(word!(game.cur_enemy_index), -4, game) {
                game.lease()
                    .edit_actor(Key::from_legacy(0), AccessMode::Native, |actor| {
                        actor.variables[2] = 14;
                    });
                EnemyScript::jump(addr_kCrocomire_Ilist_BF3C as u16)
            } else {
                let random = word!(game.random_number);
                let side = if (random as i16).wrapping_sub(2048) >= 0 {
                    (-32i16) as u16
                } else {
                    32
                };
                spawn_dust(jp, side.wrapping_add(random & 0xf), game)
            };
        }
        10784859 | 10784911 => {
            let can_walk = {
                let runtime = game.lease();
                let actor = runtime.resolve_actor(Key::from_legacy(0), AccessMode::Native);
                ((actor.x.whole as i32
                    - actor.radius.x as i32
                    - 260
                    - runtime.word(ram::layer1_x_pos) as i32) as i16)
                    < 0
            };
            if can_walk {
                walk(word!(game.cur_enemy_index), 4, game);
            }
        }

        10784895 | 10784916 => {
            walk(k, 4, game);
        }
        10787483..=10787543 if (ea - 10787483) % 5 == 0 => {
            return spawn_dust(jp, DUST_OFFSETS[((ea - 10787483) / 5) as usize], game);
        }
        10782374 => return dispatch(k, jp, game),
        10782546 => return choose_spit(k, jp, game),
        _ => {
            game.unreachable();
            return EnemyScript::Stop;
        }
    }
    jp
}
