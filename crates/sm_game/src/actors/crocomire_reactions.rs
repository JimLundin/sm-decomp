//! Crocomire, Norfair's lava-wading boss: its death sequence of hops, melting
//! and skeleton, its reactions and its script instructions (bank $A4).
//! Native actor words and captured script images are retained.
//! Hardware effects remain in the visual owner; RAM remains authoritative.
#![forbid(unsafe_code)]
#![allow(non_snake_case)]
use crate::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::actors::crocomire_attack::{Crocomire, flag, lane};
use crate::actors::host::{ActorLease, Death, Slopes};
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

/// Death sequence indices the routines set or test.
pub mod death {
    pub const BEHIND_WALL_WAIT_FOR_SAMUS: u16 = 0x3e;
    pub const SET_INDEX_56: u16 = 0x54;
    pub const FLOWING_DOWN_THE_RIVER: u16 = 0x58;
}

const BANK: u32 = 0xa4;
/// $A4:9BC5: the melting Crocomire tiles loading table: source, destination,
/// size and bank words per entry, ended by FFFFh.
const MELTING_TILES_SOURCE: u32 = 0xa4_9bc5;
const MELTING_TILES_DESTINATION: u32 = 0xa4_9bc7;
const MELTING_TILES_SIZE: u32 = 0xa4_9bc9;
const MELTING_TILES_BANK: u32 = 0xa4_9bcb;
/// $A4:98CA: the rumble table, ended by 8080h; $A4:9697: the melting X offsets.
const RUMBLE_TABLE: u32 = 0xa4_98ca;
const MELTING_X_OFFSETS: u32 = 0xa4_9697;
/// Crocomire palettes ($A4:B87D): BG1/2 palette 7, the target sprite palettes
/// 3 and 6, and sprite palettes 4 and 6.
const PALETTE_BG_7: u32 = 0xa4_b89d;
const PALETTE_TARGET_SPRITE_3: u32 = 0xa4_b8bd;
const PALETTE_TARGET_SPRITE_6: u32 = 0xa4_b8dd;
const PALETTE_SPRITE_4: u32 = 0xa4_b8fd;
const PALETTE_SPRITE_6: u32 = 0xa4_b91d;
/// $7E:C0EE: BG1/2 palette 7 colours 0-7.
const BG_PALETTE_7_RAM: usize = 0xc0ee;
/// Bank $84 PLMs: Crocomire's bridge and invisible walls.
const CLEAR_BRIDGE: u16 = 0xb747;
const CRUMBLE_BRIDGE_BLOCK: u16 = 0xb74b;
const CLEAR_BRIDGE_BLOCK: u16 = 0xb74f;
const CLEAR_INVISIBLE_WALL: u16 = 0xb753;
const CREATE_INVISIBLE_WALL: u16 = 0xb757;
/// $7E:9016: the death sequence's crumbling bridge index.
const CRUMBLING_BRIDGE_INDEX: usize = 0x9016;
/// $7E:069C-$7E:071D: the melting Y offsets, one per pixel column, and
/// $7E:068C: the pixels to erase per column a frame.
const MELTING_Y_OFFSETS: usize = 0x69c;
const PIXELS_TO_ERASE: usize = 0x68c;
/// $7E:CAE0: Crocomire's BG2 Y scroll indirect HDMA table.
const SCROLL_HDMA_TABLE: usize = 0xcae0;
/// The scroll bytes at $7E:CD20, written a pair at a time: 0 is red, 1 blue.
const SCROLLS_0_1: usize = 0xcd20;
const SCROLLS_2_3: usize = 0xcd22;
const SCROLLS_3_4: usize = 0xcd23;
const SCROLLS_4_5: usize = 0xcd24;
/// Block rows of the invisible walls and of the bridge.
const WALL_ROW: u8 = 3;
const BRIDGE_ROW: u8 = 0xb;

/// Crocomire's cry (74h), a big explosion (25h), the skeleton collapsing (75h)
/// and the screen shaking (76h), library 2.
const CRY_SOUND: u8 = 0x74;
const BIG_EXPLOSION_SOUND: u8 = 0x25;
const SKELETON_COLLAPSES_SOUND: u8 = 0x75;
const SHAKE_SOUND: u8 = 0x76;

/// Bank $A4 Crocomire instructions its scripts call.
pub mod instruction {
    pub const FIGHT_AI: u32 = 0xa4_86a6;
    pub const MAYBE_START_PROJECTILE_ATTACK: u32 = 0xa4_8752;
    pub const QUEUE_CRY: u32 = 0xa4_8cfb;
    pub const QUEUE_BIG_EXPLOSION: u32 = 0xa4_8d07;
    pub const QUEUE_SKELETON_COLLAPSES: u32 = 0xa4_8d13;
    pub const SHAKE_SCREEN: u32 = 0xa4_8fc7;
    pub const MOVE_LEFT_4PX: u32 = 0xa4_8fdf;
    pub const MOVE_LEFT_4PX_AND_SPAWN_DUST: u32 = 0xa4_8ffa;
    pub const MOVE_LEFT_4PX_AND_SPAWN_DUST_2: u32 = 0xa4_8fff;
    pub const MOVE_LEFT_4PX_SPAWN_DUST_AND_HANDLE_SPIKE_WALL: u32 = 0xa4_901d;
    pub const MOVE_RIGHT_4PX_IF_ON_SCREEN: u32 = 0xa4_905b;
    pub const MOVE_RIGHT_4PX: u32 = 0xa4_907f;
    pub const MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST: u32 = 0xa4_908f;
    pub const MOVE_RIGHT_4PX_AND_SPAWN_DUST: u32 = 0xa4_9094;
    /// $A4:9A9B-$A4:9AD7: spawn big dust cloud at each of 13 X offsets, 5 bytes apart.
    pub const SPAWN_DUST_FIRST: u32 = 0xa4_9a9b;
    pub const SPAWN_DUST_LAST: u32 = 0xa4_9ad7;
}

/// Instruction - fight AI ($A4:86A6).
pub fn fight_ai<H: Host + ?Sized>(k: u16, jp: EnemyScript, game: &mut H) -> EnemyScript {
    crate::actors::crocomire_attack::fight_ai(k, jp, &mut game.lease())
}
/// Instruction - maybe start projectile attack ($A4:8752).
pub fn maybe_start_projectile_attack<H: Host + ?Sized>(
    k: u16,
    jp: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    crate::actors::crocomire_attack::maybe_start_projectile_attack(k, jp, &mut game.lease())
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
    slot!(game, Crocomire[0].fight_flags, |old| old & !flag::DAMAGED);
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
        let low = slot!(game, Crocomire[key].step_counter) as u8;
        slot!(byte game, Crocomire[key].step_counter[0] = low.wrapping_add(0x80));
        let next = (low.overflowing_add(0x80).1 as u8)
            .wrapping_add(slot!(byte game, Crocomire[key].step_counter[1]))
            .wrapping_add(3);
        let high = cap_byte(next, 48);
        slot!(byte game, Crocomire[key].step_counter[1] = high);
        let carry = slot!(byte game, Crocomire[key].y_velocity[0])
            .overflowing_add(high)
            .1;
        slot!(byte game, Crocomire[key].y_velocity[0] = slot!(byte game, Crocomire[key].y_velocity[0]).wrapping_add(high));
        let high = cap_byte(
            (carry as u8).wrapping_add(slot!(byte game, Crocomire[key].y_velocity[1])),
            3,
        );
        slot!(byte game, Crocomire[key].y_velocity[1] = high);
        let velocity = slot!(game, Crocomire[key].y_velocity) as u8;
        let carry = slot!(byte game, Crocomire[key].projectile_counter[1])
            .overflowing_add(velocity)
            .1;
        slot!(byte game, Crocomire[key].projectile_counter[1] = slot!(byte game, Crocomire[key].projectile_counter[1]).wrapping_add(velocity));
        let (step, carry) =
            (carry as u8).overflowing_add(slot!(byte game, Crocomire[key].y_velocity[1]));
        let carry = carry
            | slot!(byte game, Crocomire[key].base.y_pos[0])
                .overflowing_add(step)
                .1;
        slot!(byte game, Crocomire[key].base.y_pos[0] = slot!(byte game, Crocomire[key].base.y_pos[0]).wrapping_add(step));
        slot!(byte game, Crocomire[key].base.y_pos[1] = slot!(byte game, Crocomire[key].base.y_pos[1]).wrapping_add(carry as u8));
    } else {
        slot!(game, Crocomire[key].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[key].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[0].step_counter = 48);
    }
}
pub fn melt_up<H: Host + ?Sized>(game: &mut H) {
    fill_scroll(game);
    let key = word!(game.cur_enemy_index) as i32;
    if slot!(game, Crocomire[key].base.y_pos) as i32 - 218 & 0x8000 != 0 {
        slot!(game, Crocomire[key].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[key].death_sequence_index, |old| old
            .wrapping_add(1));
    } else {
        update_background(word!(game.cur_enemy_index), game);
        let mut acceleration = (slot!(game, Crocomire[0].step_counter) as i16).wrapping_add(256);
        if slot!(game, Crocomire[0].step_counter) as i32 - 7680 & 0x8000 == 0 {
            acceleration = 7936;
        }
        slot!(game, Crocomire[0].step_counter = acceleration as u16);
        let low = slot!(game, Crocomire[0].y_velocity) as u8;
        let borrow = low < slot!(byte game, Crocomire[0].step_counter[1]);
        slot!(byte game, Crocomire[0].y_velocity[0] = low.wrapping_sub(slot!(byte game, Crocomire[0].step_counter[1])));
        let mut high =
            (slot!(byte game, Crocomire[0].y_velocity[1]) as i8).wrapping_sub(borrow as i8);
        if high < 0 {
            slot!(byte game, Crocomire[0].y_velocity[0] = 0xff);
            high = 0;
        }
        slot!(byte game, Crocomire[0].y_velocity[1] = high as u8);
        let fraction = slot!(byte game, Crocomire[0].projectile_counter[1]);
        let borrow = fraction < slot!(byte game, Crocomire[0].y_velocity[0]);
        slot!(byte game, Crocomire[0].projectile_counter[1] = fraction.wrapping_sub(slot!(byte game, Crocomire[0].y_velocity[0])));
        let y = slot!(game, Crocomire[0].base.y_pos) as u8;
        let step = (borrow as u8).wrapping_add(slot!(byte game, Crocomire[0].y_velocity[1]));
        slot!(byte game, Crocomire[0].base.y_pos[0] = y.wrapping_sub(step));
        slot!(byte game, Crocomire[0].base.y_pos[1] = slot!(byte game, Crocomire[0].base.y_pos[1]).wrapping_sub((y < step) as u8));
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
            crate::word_transfers!(game, descending, 8; native(BG_PALETTE_7_RAM) <- value(0x7fff));
        } else {
            crate::word_transfers!(game, descending, 8; native(BG_PALETTE_7_RAM) <- rom(PALETTE_BG_7, 7));
        }
    }
}
fn wall_counter(runtime: &impl ActorAccess, key: u16) -> u16 {
    runtime
        .resolve_actor(Key::from_legacy(key), AccessMode::Native)
        .variables[lane::STEP_COUNTER]
}
fn set_wall_counter(runtime: &mut impl ActorAccess, key: u16, value: u16) {
    runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
        enemy.variables[lane::STEP_COUNTER] = value
    });
}
/// Crocomire main AI - death sequence index 40h - behind wall - rumbling ($A4:9859). Was
/// `wall_step`.
pub fn behind_wall_rumbling<H: Host + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    let mut cursor = wall_counter(&runtime, 0);
    let image = runtime.leased_image().expect("ROM must be loaded");
    let target = image.table_word(
        sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
        (cursor >> 1) as isize,
    );
    if target == 0x8080 {
        set_wall_counter(&mut runtime, 0x40, 0x8080);
        set_wall_counter(&mut runtime, 0, 128);
        drop(runtime);
        crate::word_transfers!(game, descending, 16; le(palette_buffer, 191) <- rom(PALETTE_SPRITE_6, 15));
        game.lease()
            .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
                enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                    enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
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
            sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0x80, countdown);
        cursor = cursor.wrapping_add(2);
        let step = image.table_word(
            sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(&mut runtime, 0xc0, step);
    }
    set_wall_counter(&mut runtime, 0, cursor.wrapping_add(2));
}
/// Handle playing Crocomire acid damage sound effect ($A4:8D1F). Was `rumble`.
pub fn play_acid_damage_sound<H: Host + ?Sized>(game: &mut H) {
    let remaining = slot!(game, Crocomire[0].acid_damage_sound_timer);
    if remaining != 0 {
        slot!(game, Crocomire[0].acid_damage_sound_timer = remaining - 1);
        if remaining == 1 {
            slot!(game, Crocomire[0].acid_damage_sound_timer = 32);
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
/// Enemy touch - Crocomire - claws ($A4:B93D).
pub fn touch<H: Host + ?Sized>(game: &mut H) {
    contact(game);
    slot!(game, Crocomire[0].fight_flags, |flags| flags
        | flag::SAMUS_HIT_BY_CLAW);
    word!(game.extra_samus_x_displacement = -4i32 as u16);
}
/// Crocomire / Samus collision handling ($A4:8C95). Was `push_player`.
pub fn samus_collision<H: Host + ?Sized>(game: &mut H) {
    if slot!(game, Crocomire[0].death_sequence_index) == 0
        && ((contact_edge(game) - game.player_x() as i32) as i16) < 0
    {
        contact(game);
        let edge = contact_edge(game) as u16;
        game.set_player_x(edge);
        game.set_previous_player_x(game.player_x());
        word!(game.extra_samus_x_displacement = -4i32 as u16);
        word!(game.extra_samus_y_displacement = -1i32 as u16);
    }
}
fn next_hit_count(flags: u16) -> u16 {
    ((flags & 0xf) + 1).min(15)
}
/// Enemy shot - Crocomire - nothing ($A4:B951): count the hit.
pub fn count_hit<H: Host + ?Sized>(game: &mut H) {
    let count = next_hit_count(slot!(game, Crocomire[0].fight_flags));
    slot!(game, Crocomire[0].fight_flags, |flags| flags | count);
}
/// Enemy shot - Crocomire - spawn shot explosion ($A4:B968, $A4:BAB4).
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
/// Enemy shot - Crocomire - open mouth ($A4:BA05).
pub fn hit<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Crocomire[0].base.invincibility_timer = 0);
    let mut advance = ((slot!(game, Crocomire[0].base.x_pos) as i32
        - slot!(game, Crocomire[0].base.x_width) as i32
        - 256
        - crate::camera::current(game).layer1[0] as i32) as i16)
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
            slot!(game, Crocomire[0].step_counter, |old| old
                .wrapping_add(recoil));
        }
    }
    if advance {
        let count = next_hit_count(slot!(game, Crocomire[0].fight_flags));
        if slot!(game, Crocomire[0].fight_flags) & flag::DAMAGED == 0 {
            let mut delay = 8;
            if slot!(game, Crocomire[0].fight_function) == 8 {
                delay = 8;
            }
            slot!(game, Crocomire[0].base.instruction_timer, |old| old
                .wrapping_add(delay));
        }
        slot!(
            game,
            Crocomire[0].fight_flags = count
                | slot!(game, Crocomire[0].fight_flags)
                    & !(flag::HIT_COUNT | flag::DAMAGED | flag::SAMUS_HIT_BY_CLAW)
                | flag::DAMAGED
        );
        slot!(game, Crocomire[0].y_velocity = 10);
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

/// Crocomire main AI - death sequence index 34h - hop 6 - rising ($A4:926E). Was
/// `melt_skeleton_up`.
pub fn hop_6_rising<H: Host + ?Sized>(game: &mut H) {
    select_melt_animation::<true, _>(Key::from_legacy(word!(game.cur_enemy_index)), game);
    emit_lava_plume(game);
    melt_up(game);
}

/// Crocomire main AI - death sequence index 1Eh/24h/2Ah - hop 3/4/5 - sinking ($A4:9108). Was
/// `melt_flesh_down`.
pub fn hop_3_4_5_sinking<H: Host + ?Sized>(game: &mut H) {
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

/// Crocomire main AI - death sequence index 8/Eh - hop 1/2 - sinking ($A4:91BA). Was
/// `melt_rumble_down`.
pub fn hop_1_2_sinking<H: Host + ?Sized>(game: &mut H) {
    play_acid_damage_sound(game);
    emit_lava_plume(game);
    melt_down(game);
}

/// Crocomire main AI - death sequence index 16h/22h/28h - hop 3/4/5 - rising ($A4:929E). Was
/// `melt_flesh_up`.
pub fn hop_3_4_5_rising<H: Host + ?Sized>(game: &mut H) {
    emit_lava_plume(game);
    select_melt_animation::<false, _>(Key::from_legacy(word!(game.cur_enemy_index)), game);
    melt_up(game);
}

/// Crocomire main AI - death sequence index 6/Ch - hop 1/2 - rising ($A4:92CE). Was
/// `melt_rumble_up`.
pub fn hop_1_2_rising<H: Host + ?Sized>(game: &mut H) {
    play_acid_damage_sound(game);
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

/// Crocomire main AI - death sequence index 10h - hop 3 - load melting tilemap ($A4:9341). Was
/// `prepare_flesh_tilemap`.
pub fn hop_3_load_melting_tilemap<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(PIXELS_TO_ERASE, 48);
    word!(game.croco_target_0688 = 48);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
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

/// Crocomire main AI - death sequence index 2Ch - hop 6 - load melting tilemap ($A4:93ED). Was
/// `prepare_skeleton_tilemap`.
pub fn hop_6_load_melting_tilemap<H: Host + ?Sized>(game: &mut H) {
    fill_scroll(game);
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
    slot!(game, Crocomire[E].base.instruction_timer = 1);
    game.ram().write_byte(PIXELS_TO_ERASE, 48);
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
        let value = native_graphics_word!(game, MELTING_TILES_SOURCE + source, {
            if source == 0 && live_size {
                word!(game.g_word_7E068A)
            } else {
                offset
            }
        });
        crate::word_transfers!(game, ascending, 1; native(entry + destination) <- value(value));
    }
}

/// Crocomire main AI - death sequence index 14h/30h - hop 3/6 - uploading to VRAM ($A4:94B6). Was
/// `advance_melt_upload`.
pub fn upload_melting_tiles<H: Host + ?Sized>(game: &mut H) {
    let offset = word!(game.g_word_7E068A);
    let tail = word!(game.vram_write_queue_tail);
    if native_graphics_word!(game, MELTING_TILES_SOURCE, word!(game.g_word_7E068A)) == 0xffff {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[E].death_sequence_index, |old| old
            .wrapping_add(1));
        word!(game.g_word_7E068A = 0);
    } else {
        queue_melt_graphics(offset, true, game);
        word!(game.g_word_7E068A = offset.wrapping_add(8));
        word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    };
}

/// Crocomire main AI - death sequence index 12h/2Eh - hop 3/6 - load melting tiles ($A4:943D). Was
/// `prepare_melt_graphics`.
pub fn load_melting_tiles<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].death_sequence_index, |old| (old)
        .wrapping_add(2));
    word! { game;
        g_word_7E0692 = 256;
        croco_cur_vline_idx = 0;
        g_word_7E0698 = native_graphics_word!(game, MELTING_TILES_SOURCE, word!(game.croco_word_7E069A));
        g_word_7E0694 = word!(game.g_word_7E0698);
        g_word_7E0696 = native_graphics_word!(game, MELTING_TILES_DESTINATION, word!(game.croco_word_7E069A));
        g_word_7E068E = native_graphics_word!(game, MELTING_TILES_SIZE, word!(game.croco_word_7E069A));
    }
    let bank: u8 =
        native_graphics_word!(game, MELTING_TILES_BANK, word!(game.croco_word_7E069A)) as u8;
    let mut i: i32 = word!(game.croco_word_7E069A) as i32 + 8;
    loop {
        let mut v2: u16 = native_graphics_word!(game, MELTING_TILES_SOURCE, i);
        if v2 == 0xffff {
            break;
        }
        let mut v3: u16 = native_graphics_word!(game, MELTING_TILES_DESTINATION, i);
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

/// Power bomb reaction - Crocomire ($A4:B992).
pub fn power_bomb<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    if slot!(game, Crocomire[E].death_sequence_index) == 0 {
        slot!(game, Crocomire[E].step_counter = 3);
        if slot!(game, Crocomire[E].fight_function) != 24 {
            slot!(game, Crocomire[E].fight_flags, |old| old
                & !(flag::HIT_COUNT | flag::SAMUS_HIT_BY_CLAW | flag::AWAKE)
                | flag::AWAKE);
            slot!(game, Crocomire[E].y_velocity = 10);
            slot!(game, Crocomire[E].base.flash_timer, |old| old
                .wrapping_add(4));
            slot!(game, Crocomire[E].base.ai_handler_bits, |old| old | 2u16);
            slot!(game, Crocomire[E].fight_function = 24);
            let mut spritemap_pointer: u16 = slot!(game, Crocomire[E].base.spritemap_pointer);
            let mut n: i32 =
                native_graphics_word!(game, BANK << 16 | u32::from(spritemap_pointer), 0) as i32;
            let instruction = loop {
                let tag = native_graphics_word!(game, BANK << 16 | u32::from(spritemap_pointer), 6);
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

/// Crocomire main AI - death sequence index 0 - death sequence not started ($A4:8C6E). Was
/// `approach_wall`.
pub fn not_started<H: Host + ?Sized>(game: &mut H) {
    update_wall(game);
    crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(257));
    if game.player_x() as i32 - 1312i32 & 0x8000 == 0 {
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(256));
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
            game.spawn_plm(0x61, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK);
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
                game.spawn_plm(0x62, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK);
                game.spawn_plm(0x63, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK);
                game.set_projectile_spawn_point(1584, 176);
                game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
            }
        } else if ((x_pos) as i16).wrapping_sub((1600u16) as i16) >= 0 {
            demolish_wall(game);
            slot!(game, Crocomire[E0].acid_damage_sound_timer = 1);
            word!(game.g_word_7E9018 = 1);
            let _v3: u16 = word!(game.cur_enemy_index);
            let EK: i32 = word!(game.cur_enemy_index) as i32;
            slot!(game, Crocomire[EK].death_sequence_index, |old| (old)
                .wrapping_add(2));
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].unread_7800 = 2);
            }
            for key in (0..=0x140i32).step_by(0x40) {
                slot!(game, Crocomire[key].unread_7802 = 0);
            }
            game.queue_sound(1, 0x3b, 6);
            crate::word_transfers!(game, ascending, 1; native(CRUMBLING_BRIDGE_INDEX) <- value(0));
            game.spawn_plm(0x4e, WALL_ROW, CREATE_INVISIBLE_WALL);
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

            slot!(game, Crocomire[EK]; y_velocity = 0, projectile_counter = 0);
            slot!(game, Crocomire[EK].step_counter = 2048);
            slot!(game, Crocomire[0i32].base.y_height = 16);
        }
    };
}

fn demolish_wall<H: Host + ?Sized>(game: &mut H) {
    for x in 0x61..=0x6a {
        game.spawn_plm(x, BRIDGE_ROW, CLEAR_BRIDGE_BLOCK);
    }
    for (offset, y) in [176, 192, 176, 192, 192, 192, 192].into_iter().enumerate() {
        game.set_projectile_spawn_point(1536 + 16 * offset as u16, y);
        game.spawn_room_projectile(addr_kEproj_DustCloudExplosion as u16, 0x15);
    }
}

/// Crocomire death sequence index += 2 ($A4:9BB3). Was `advance_phase`.
pub fn advance_death_sequence<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
}

/// Crocomire main AI - death sequence index 20h/26h - hop 4/5 - resting ($A4:8D47). Was
/// `wait_for_melt`.
pub fn hop_4_5_resting<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    let counter: u16 = slot!(game, Crocomire[E].step_counter);
    if counter != 0 {
        slot!(game, Crocomire[E].step_counter = (counter).wrapping_sub(1));
    } else {
        slot!(game, Crocomire[E].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[E].death_sequence_index, |old| old
            .wrapping_add(1));
        slot!(game, Crocomire[E].y_velocity = 768);
    };
}

/// Crocomire main AI - death sequence index 4/Ah - hop 1/2 - resting ($A4:8D3F). Was
/// `rumble_and_wait`.
pub fn hop_1_2_resting<H: Host + ?Sized>(game: &mut H) {
    play_acid_damage_sound(game);
    game.align_crocomire_background();
    hop_4_5_resting(game);
}

/// Crocomire main AI - death sequence index 42h - behind wall - no more rumbling ($A4:990A). Was
/// `prepare_spike_wall`.
pub fn behind_wall_no_more_rumbling<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = 0;
    let counter: u16 = slot!(game, Crocomire[E].step_counter);
    if counter != 0 {
        slot!(game, Crocomire[E].step_counter = (counter).wrapping_sub(1));
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
        slot!(game, Crocomire[0x80i32].step_counter = 80);
        slot!(game, Crocomire[E]; y_velocity = 0, projectile_counter = 0);
        game.spawn_plm(0x20, WALL_ROW, CLEAR_INVISIBLE_WALL);
        game.spawn_plm(0x1e, WALL_ROW, CREATE_INVISIBLE_WALL);
        game.spawn_plm(0x70, BRIDGE_ROW, CLEAR_BRIDGE);
        game.queue_sound(1, 0x29, 6);
        slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E158 as u16, instruction_timer = 1);
        slot!(game, Crocomire[E].base.palette_index = 0);
        crate::word_transfers!(game, descending, 16; le(palette_buffer, 159) <- rom(PALETTE_SPRITE_4, 15));
        game.clear_enemy_projectiles();
        let mut v8: u16 = 8;
        for _ in 0..8 {
            let k = word!(game.cur_enemy_index);
            v8 = game.spawn_projectile(v8, k, addr_kEproj_CrocomireSpikeWallPieces as u16);
        }
        game.queue_sound(1, 0x30, 6);
        advance_death_sequence(game);
    };
}

/// Crocomire main AI - death sequence index 1Ah/38h - hop 3/6 - melting ($A4:9580). Was
/// `advance_melt_scroll`.
pub fn melting<H: Host + ?Sized>(game: &mut H) {
    emit_lava_plume(game);
    let E0: i32 = 0;
    let E1: i32 = 0x40;
    let E2: i32 = 0x80;
    let mut counter: u16 = slot!(game, Crocomire[E2].step_counter);
    slot!(game, Crocomire[E1].step_counter, |old| old.wrapping_sub(1));
    if (slot!(game, Crocomire[E1].step_counter) & 2) != 0 {
        counter = (counter).wrapping_add(4);
    }
    slot!(game, Crocomire[E0].base.x_pos = counter);
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
    slot!(game, Crocomire[EK].death_sequence_index, |old| (old)
        .wrapping_add(2));
    let mut i: i32 = word!(game.croco_word_7E069A) as i32;
    while native_graphics_word!(game, MELTING_TILES_SOURCE, i) != 0xffff {
        i += 8;
    }
    word!(game.croco_word_7E069A = (i + 2) as u16);
    word!(
        game.hdma_object_channels_bitmask[(slot!(game, Crocomire[E0].melting_hdma_object) >> 1)] =
            0
    );
}

fn erase_melt_columns<H: Host + ?Sized>(game: &mut H) -> u16 {
    let mut n: i32 = game.ram().read_byte(PIXELS_TO_ERASE) as i32;
    loop {
        if word!(game.croco_cur_vline_idx) > 48 {
            return 1;
        }
        let rr: i32 = game.native_image().read(
            sm_data::cartridge::lorom_offset(MELTING_X_OFFSETS) as usize
                + usize::from(word!(game.croco_cur_vline_idx)),
        ) as i32;
        if (game.ram().read_byte(MELTING_Y_OFFSETS + rr as usize) as i8)
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
        sm_data::cartridge::lorom_offset(MELTING_X_OFFSETS) as usize
            + usize::from(word!(game.croco_cur_vline_idx)),
    ) as i32;
    let mask: u8 = !(0x80u8 >> (vline_idx & 7));
    do_until! { n == 0 ;
        let q: i32 = game.ram().read_byte(MELTING_Y_OFFSETS + vline_idx as usize) as i32;
        let j: i32 = 2 * (q & 7) + ((q & !(7i32)) << 6) + 4 * (vline_idx & !(7i32));
        // Melting tiles span the $4000 graphics buffer, beyond the menu backup prefix.
        for plane_offset in [0, 1, 16, 17] {
            let at = 0x4000 + j as usize + plane_offset;
            game.ram().write_byte(at, game.ram().read_byte(at) & mask);
        }
        if game.ram().read_byte(MELTING_Y_OFFSETS + vline_idx as usize) == 48 {
            break;
        }
        let at = MELTING_Y_OFFSETS + vline_idx as usize;
        game.ram().write_byte(at, game.ram().read_byte(at).wrapping_add(1));
        n -= 1;
    }
    let (v7, v8) = loop {
        let v7 = (word!(game.g_word_7E068A)).wrapping_add(word!(game.croco_word_7E069A));
        let v8 = word!(game.vram_write_queue_tail);
        if native_graphics_word!(game, MELTING_TILES_SOURCE, v7) != 0xffff {
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
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_0_1) <- value(257));
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_2_3) <- value(257));
        word!(game.croco_target_0688 = 0);
        let E: i32 = 0;
        slot!(game, Crocomire[E].base.properties, |old| old & 0x7bff
            | 0x400);
        game.spawn_plm(0x20, WALL_ROW, CLEAR_INVISIBLE_WALL);
        game.spawn_plm(0x1e, WALL_ROW, CLEAR_INVISIBLE_WALL);
        game.spawn_plm(0x61, BRIDGE_ROW, CLEAR_BRIDGE);
        slot!(
            game,
            Crocomire[E].death_sequence_index = death::SET_INDEX_56
        );
        slot!(game, Crocomire[E].base; current_instruction = addr_kCrocomire_Ilist_E1CC as u16, instruction_timer = 1);
        slot!(game, Crocomire[E].base.x_pos = 576);
        slot!(game, Crocomire[E].base; y_pos = 144, y_height = 28, x_width = 40);
        queue_tilemap_upload(2048, 0xfc, game);
    } else {
        game.disable_minimap_for_boss();
        word!(game.croco_word_7E069A = 0);
        let E_0: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Crocomire[E_0]; death_sequence_index = 0, y_velocity = 0);
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_0_1) <- value(0));
        crate::word_transfers!(game, descending, 17;
            le(target_palettes, 176) <- rom(PALETTE_TARGET_SPRITE_3, 16),
            le(target_palettes, 224) <- rom(PALETTE_TARGET_SPRITE_6, 16),
        );
        slot!(game, Crocomire[E_0].fight_function = 4);
        word! {game; UNUSED_word_7E179E = 16; camera_distance_index = 2;
        enemy_bg2_tilemap_size = 1024; }
        slot!(game, Crocomire[E_0].base; current_instruction = addr_kCrocomire_Ilist_BADE as u16, extra_properties = (slot!(game, Crocomire[E_0].base.extra_properties)) | (4u16), instruction_timer = 1);
    }
}

/// Crocomire main AI - death sequence index 3Ch - hop 6 - sinking ($A4:9099). Was
/// `complete_skeleton_melt`.
pub fn hop_6_sinking<H: Host + ?Sized>(game: &mut H) {
    select_melt_animation::<true, _>(Key::from_legacy(0), game);
    emit_lava_plume(game);
    melt_down(game);
    let E: i32 = 0;
    if slot!(game, Crocomire[E].death_sequence_index) == death::BEHIND_WALL_WAIT_FOR_SAMUS {
        game.queue_music_delayed(6);
        slot!(
            game,
            Crocomire[E].death_sequence_index = death::FLOWING_DOWN_THE_RIVER
        );
        slot!(
            game,
            Crocomire[E].base.current_instruction = addr_kCrocomire_Ilist_E1D2 as u16
        );
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(257));
        word!(game.debug_disable_minimap = 0);
        slot!(
            game,
            Crocomire[0x40i32].base.properties =
                slot!(game, Crocomire[0x40i32].base.properties) | (0x200u16)
        );

        game.spawn_plm(0x4e, WALL_ROW, CLEAR_INVISIBLE_WALL);
        word! { game, 0; camera_distance_index, croco_target_0688 }
    }
}

/// Crocomire main AI - death sequence index 2 - falling ($A4:9136). Was `emit_melt_debris`.
pub fn falling<H: Host + ?Sized>(game: &mut H) {
    let v0: u16 = u16::from_ne_bytes([
        game.ram().read_byte(CRUMBLING_BRIDGE_INDEX),
        game.ram().read_byte(CRUMBLING_BRIDGE_INDEX + 1),
    ]);
    if u16::from_ne_bytes([
        game.ram().read_byte(CRUMBLING_BRIDGE_INDEX),
        game.ram().read_byte(CRUMBLING_BRIDGE_INDEX + 1),
    ]) as i32
        - 22i32
        & 0x8000
        != 0
    {
        let v1: u16 = u16::from_ne_bytes([
            game.ram().read_byte(CRUMBLING_BRIDGE_INDEX),
            game.ram().read_byte(CRUMBLING_BRIDGE_INDEX + 1),
        ]);
        checked_word!(game, CRUMBLING_BRIDGE_INDEX; += 2);
        let parameter = game.crocomire_table_word(BRIDGE_DEBRIS_ADDRESS, (v0 >> 1) as isize);
        game.spawn_projectile(parameter, v1, addr_stru_868F9D as u16);
    }
    hop_1_2_sinking(game);
}

/// Crocomire main AI - death sequence index 3Eh - behind wall - wait for Samus ($A4:97D3). Was
/// `release_skeleton_room`.
pub fn behind_wall_wait_for_samus<H: Host + ?Sized>(game: &mut H) {
    if (game.player_x() as i16).wrapping_sub(640) >= 0 {
        return;
    }
    game.queue_music_delayed(5);
    game.lease().write_slice_native(SCROLLS_3_4, 256);
    {
        game.spawn_plm(0x30, WALL_ROW, CREATE_INVISIBLE_WALL);
    }
    word!(game.camera_distance_index = 6);
    let mut runtime = game.lease();
    // Fixed, disjoint records; no callback or fallible operation between these writes.
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.properties = enemy.properties & 0x7bff | kEnemyProps_Tangible as u16;
        enemy.variables[lane::STEP_COUNTER] = 4;
        enemy.variables[lane::FIGHT_FLAGS] = 0;
        enemy.radius.y = 56;
        enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
            enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
    });
    runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
        enemy.properties |= 0x500;
        enemy.variables[lane::STEP_COUNTER] = 0;
    });
    for (key, countdown) in [(0x80, 10), (0xc0, 1)] {
        runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
            enemy.variables[lane::STEP_COUNTER] = countdown
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

/// Crocomire main AI - death sequence index 58h - flowing down the river ($A4:9830). Was
/// `reposition_skeleton`.
pub fn flowing_down_the_river<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.x.whole = enemy.x.whole.wrapping_sub(2);
            if (enemy.x.whole as i16).wrapping_sub(480) < 0 {
                enemy.x.whole = 480;
                enemy.y.whole = 54;
                enemy.variables[lane::DEATH_SEQUENCE_INDEX] = death::BEHIND_WALL_WAIT_FOR_SAMUS;
            } else {
                enemy.y.whole = 220;
            }
        });
}

/// Crocomire main AI - death sequence index 44h - breaks down wall ($A4:99E5). Was
/// `launch_skeleton`.
pub fn breaks_down_wall<H: Host + ?Sized>(game: &mut H) {
    let mut runtime = game.lease();
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.x.whole as i16).wrapping_sub(224) < 0 {
            let speed = accelerated_speed(
                enemy.variables[lane::Y_VELOCITY],
                enemy.pre_instruction,
                0x8000,
                2,
            );
            enemy.variables[lane::Y_VELOCITY] = speed as u16;
            enemy.pre_instruction = (speed >> 16) as u16;
            enemy.x.advance(speed as i32);
        }
    });
    let finished = runtime.edit_actor(Key::from_legacy(0x80), AccessMode::Native, |enemy| {
        let previous = enemy.variables[lane::STEP_COUNTER];
        enemy.variables[lane::STEP_COUNTER] = previous.saturating_sub(1);
        previous == 1
    });
    if finished {
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[lane::Y_VELOCITY] = 0;
            enemy.instruction = addr_kCrocomire_Ilist_E14A as u16;
            enemy.instruction_delay = 1;
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
    }
}

/// Crocomire main AI - death sequence index 46h - skeleton falls ($A4:9A38). Was `land_skeleton`.
pub fn skeleton_falls<H: Host + ?Sized>(game: &mut H) {
    let impact = game
        .lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            let speed = accelerated_speed(
                enemy.variables[lane::Y_VELOCITY],
                enemy.pre_instruction,
                2048,
                5,
            );
            enemy.variables[lane::Y_VELOCITY] = speed as u16;
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
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
    }
}

/// Crocomire main AI - death sequence index 48h - skeleton falls apart ($A4:9B06). Was
/// `finish_skeleton_escape`.
pub fn skeleton_falls_apart<H: Host + ?Sized>(game: &mut H) {
    let finished = game
        .lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            if (enemy.instruction as i16).wrapping_add(0x1e3a) < 0 {
                let speed = accelerated_speed(
                    enemy.variables[lane::Y_VELOCITY],
                    enemy.pre_instruction,
                    4096,
                    6,
                );
                enemy.variables[lane::Y_VELOCITY] = speed as u16;
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
            game.spawn_plm(0x30, WALL_ROW, CLEAR_INVISIBLE_WALL);
        }
        game.scatter_crocomire_drop(0);
        advance_death_sequence(game);
    }
}

/// Crocomire main AI - death sequence index 4Ch - set index 4Eh ($A4:9B7D). Was
/// `await_skeleton_script`.
pub fn set_index_4e<H: Host + ?Sized>(game: &mut H) {
    game.lease()
        .edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            if (enemy.instruction as i16).wrapping_add(0x1e3a) >= 0 {
                enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                    enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
            }
        });
}

/// Crocomire main AI - death sequence index 4Ah - unlock camera ($A4:9B65). Was
/// `open_skeleton_exit`.
pub fn unlock_camera<H: Host + ?Sized>(game: &mut H) {
    {
        let mut runtime = game.lease();
        runtime.write_slice_native(SCROLLS_0_1, 257);
        runtime.write_slice_native(SCROLLS_2_3, 257);
    }
    {
        game.spawn_plm(0x1e, WALL_ROW, CLEAR_INVISIBLE_WALL);
    }
    advance_death_sequence(game);
}

pub fn prepare_scroll_hdma<H: Host + ?Sized>(game: &mut H) {
    let mut v1: u16 = (2 * (slot!(game, Crocomire[0x40i32].base.y_pos) as i32 - 72i32)) as u16;
    let v2: u16 = word!(game.reg_BG2VOFS);
    do_until! { (v1 & 0x8000) != 0 ;
        word!(game.crocomire_bg2_scroll_hdma_data[(v1 >> 1)] = v2);
        v1 = (v1).wrapping_sub(2);
    }
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
    slot!(game, Crocomire[E].death_sequence_index, |old| old
        .wrapping_add(1));
    let v4: i32 = 0;
    slot!(
        game,
        Crocomire[0x80i32].step_counter = slot!(game, Crocomire[v4].base.x_pos)
    );
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE) <- value(255));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 1) <- value(-13584i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 3) <- value(225));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 4) <- value(-13330i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 6) <- value(0));
}

// Returned only for the two synchronous hardware phases of the fixed actor-0 driver.
pub enum ScrollRequest {
    Flesh = 0x77,
    Skeleton = 0x2d,
}
/// Main AI - Crocomire ($A4:8C04): run the death sequence function; the two
/// melting HDMA phases are left to the caller.
pub fn execute_phase<H: Host + ?Sized>(game: &mut H) -> Option<ScrollRequest> {
    let phases: [Option<fn(&mut H)>; 45] = [
        Some(not_started as fn(&mut H)),
        Some(falling as fn(&mut H)),
        Some(hop_1_2_resting as fn(&mut H)),
        Some(hop_1_2_rising as fn(&mut H)),
        Some(hop_1_2_sinking as fn(&mut H)),
        Some(hop_1_2_resting as fn(&mut H)),
        Some(hop_1_2_rising as fn(&mut H)),
        Some(hop_1_2_sinking as fn(&mut H)),
        Some(hop_3_load_melting_tilemap as fn(&mut H)),
        Some(load_melting_tiles as fn(&mut H)),
        Some(upload_melting_tiles as fn(&mut H)),
        Some(hop_3_4_5_rising as fn(&mut H)),
        None,
        Some(melting as fn(&mut H)),
        Some(clear_tilemap as fn(&mut H)),
        Some(hop_3_4_5_sinking as fn(&mut H)),
        Some(hop_4_5_resting as fn(&mut H)),
        Some(hop_3_4_5_rising as fn(&mut H)),
        Some(hop_3_4_5_sinking as fn(&mut H)),
        Some(hop_4_5_resting as fn(&mut H)),
        Some(hop_3_4_5_rising as fn(&mut H)),
        Some(hop_3_4_5_sinking as fn(&mut H)),
        Some(hop_6_load_melting_tilemap as fn(&mut H)),
        Some(load_melting_tiles as fn(&mut H)),
        Some(upload_melting_tiles as fn(&mut H)),
        Some(set_index_34 as fn(&mut H)),
        Some(hop_6_rising as fn(&mut H)),
        None,
        Some(melting as fn(&mut H)),
        Some(clear_tilemap as fn(&mut H)),
        Some(hop_6_sinking as fn(&mut H)),
        Some(behind_wall_wait_for_samus as fn(&mut H)),
        Some(behind_wall_rumbling as fn(&mut H)),
        Some(behind_wall_no_more_rumbling as fn(&mut H)),
        Some(breaks_down_wall as fn(&mut H)),
        Some(skeleton_falls as fn(&mut H)),
        Some(skeleton_falls_apart as fn(&mut H)),
        Some(unlock_camera as fn(&mut H)),
        Some(set_index_4e as fn(&mut H)),
        Some(advance_death_sequence as fn(&mut H)),
        Some(mark_event_and_start_music as fn(&mut H)),
        Some(dead as fn(&mut H)),
        Some(advance_death_sequence as fn(&mut H)),
        Some(dead_from_room_entry as fn(&mut H)),
        Some(flowing_down_the_river as fn(&mut H)),
    ];
    let phase = (slot!(game, Crocomire[0i32].death_sequence_index) as i32 >> 1) as usize;
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
/// Crocomire main AI - death sequence index 52h - dead, the final state ($A4:8C8F). Was
/// `idle_phase`.
fn dead<H: Host + ?Sized>(_game: &mut H) {}
/// Crocomire main AI - death sequence index 32h - set index 34h ($A4:9576). Was
/// `advance_selected_phase`.
fn set_index_34<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E].death_sequence_index, |old| (old)
        .wrapping_add(2));
}
/// Crocomire main AI - death sequence index 1Ch/3Ah - hop 3/6 - clear tilemap ($A4:9653). Was
/// `reset_melt_tilemap`.
fn clear_tilemap<H: Host + ?Sized>(game: &mut H) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Crocomire[E]; y_velocity = 0, projectile_counter = 0);
    slot!(game, Crocomire[E].step_counter = 2048);
    crate::word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
    queue_tilemap_upload(2048, 0xfe, game);
    advance_death_sequence(game);
}
/// Crocomire main AI - death sequence index 56h - dead due to room entry ($A4:8C90). Was
/// `reset_scroll`.
fn dead_from_room_entry<H: Host + ?Sized>(game: &mut H) {
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
/// Crocomire main AI - death sequence index 50h - mark event and start post-death music ($A4:9B86).
/// Was `finish_death`.
fn mark_event_and_start_music<H: Host + ?Sized>(game: &mut H) {
    game.queue_music_delayed(6);
    word!(game.camera_distance_index = 0);
    let area = word!(game.area_index);
    let bits = game.boss_word(area);
    game.set_boss_word(area, bits | 2);
    game.queue_music_delayed(6);
    spawn_dust(EnemyScript::Stop, 0xfff0, game);
    spawn_dust(EnemyScript::Stop, 0x10, game);
    advance_death_sequence(game);
}

/// Hurt AI - Crocomire ($A4:8687).
pub fn hurt<H: Host + ?Sized>(game: &mut H) {
    update_background(word!(game.cur_enemy_index), game);
    update_palette(game);
}
/// Initialisation AI - Crocomire tongue ($A4:F67A).
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
        slot!(game, Crocomire[E].death_sequence_index = 23);
        slot!(game, Crocomire[E].base.palette_index = 3584);
    };
}
const DUST_OFFSETS: [u16; 13] = [
    0xffe0, 0, 0xfff0, 0x10, 0, 8, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0x40,
];
/// Run one of Crocomire's script instructions (one of [`instruction`]).
pub fn instruction<H: Host + ?Sized>(
    ea: u32,
    k: u16,
    jp: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    use instruction::*;
    if matches!(
        ea,
        MOVE_LEFT_4PX_AND_SPAWN_DUST
            | MOVE_LEFT_4PX_AND_SPAWN_DUST_2
            | MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST
            | MOVE_RIGHT_4PX_AND_SPAWN_DUST
    ) {
        spawn_random_dust(game);
    }
    match ea {
        QUEUE_CRY => game.queue_sound(1, CRY_SOUND, 6),
        QUEUE_BIG_EXPLOSION => game.queue_sound(1, BIG_EXPLOSION_SOUND, 6),
        QUEUE_SKELETON_COLLAPSES => game.queue_sound(1, SKELETON_COLLAPSES_SOUND, 6),
        SHAKE_SCREEN => {
            {
                let mut runtime = game.lease();
                runtime.write_slice_le(ram::earthquake_type, 4);
                runtime.write_slice_le(ram::earthquake_timer, 5);
            }
            game.queue_sound(1, SHAKE_SOUND, 6);
        }
        MOVE_LEFT_4PX | MOVE_LEFT_4PX_AND_SPAWN_DUST | MOVE_LEFT_4PX_AND_SPAWN_DUST_2 => {
            let hit = game
                .lease()
                .resolve_actor(Key::from_legacy(0), AccessMode::Native)
                .variables[lane::FIGHT_FLAGS]
                & flag::DAMAGED
                != 0;
            if !hit {
                walk(word!(game.cur_enemy_index), -4, game);
            }
        }
        MOVE_LEFT_4PX_SPAWN_DUST_AND_HANDLE_SPIKE_WALL => {
            return if walk(word!(game.cur_enemy_index), -4, game) {
                game.lease()
                    .edit_actor(Key::from_legacy(0), AccessMode::Native, |actor| {
                        actor.variables[lane::FIGHT_FUNCTION] =
                            crate::actors::crocomire_attack::fight::BACK_OFF_FROM_SPIKE_WALL;
                    });
                EnemyScript::jump(
                    crate::actors::crocomire_attack::instruction_list::BACK_OFF_FROM_SPIKE_WALL,
                )
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
        MOVE_RIGHT_4PX_IF_ON_SCREEN | MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST => {
            let can_walk = {
                let runtime = game.lease();
                let actor = runtime.resolve_actor(Key::from_legacy(0), AccessMode::Native);
                ((actor.x.whole as i32
                    - actor.radius.x as i32
                    - 260
                    - crate::session_access::SessionAccess::session(&runtime)
                        .room
                        .camera
                        .layer1[0] as i32) as i16)
                    < 0
            };
            if can_walk {
                walk(word!(game.cur_enemy_index), 4, game);
            }
        }

        MOVE_RIGHT_4PX | MOVE_RIGHT_4PX_AND_SPAWN_DUST => {
            walk(k, 4, game);
        }
        SPAWN_DUST_FIRST..=SPAWN_DUST_LAST if (ea - SPAWN_DUST_FIRST).is_multiple_of(5) => {
            return spawn_dust(
                jp,
                DUST_OFFSETS[((ea - SPAWN_DUST_FIRST) / 5) as usize],
                game,
            );
        }
        FIGHT_AI => return fight_ai(k, jp, game),
        MAYBE_START_PROJECTILE_ATTACK => return maybe_start_projectile_attack(k, jp, game),
        _ => {
            game.unreachable();
            return EnemyScript::Stop;
        }
    }
    jp
}
