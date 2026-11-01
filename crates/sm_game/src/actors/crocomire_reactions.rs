//! Crocomire, Norfair's lava-wading boss: its death sequence of hops, melting
//! and skeleton, its reactions and its script instructions (bank $A4).
//! Native actor words and captured script images are retained.
//! Hardware effects remain in the visual owner; RAM remains authoritative.
use crate::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use crate::actors::boss_interaction as interaction;
use crate::actors::boss_lease::BossLease;
use crate::actors::boss_lease::bg2_staging;
use crate::actors::crocomire_attack::{Crocomire, flag, lane};
use crate::actors::host::{BossEffects, Death, Slopes};
use crate::actors::session_slots::session_slot;
use crate::carry::{adc, sbc};
use crate::constants::{
    bank_86::{
        ADDR_K_EPROJ_CROCOMIRE_SPIKE_WALL_PIECES, ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION,
        ADDR_STRU_868F9D,
    },
    bank_a4::*,
    enemy_properties::K_ENEMY_PROPS_TANGIBLE,
};
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::enemy_script::EnemyScript;
use crate::menus::prelude::battle;
use crate::menus::prelude::{display_scroll, do_until, owned};
use crate::ram::{checked_word, word, words as ram};
use sm_data::crocomire::{
    BRIDGE_DEBRIS_ADDRESS, FLESH_TILEMAP_ADDRESS, MELT_TILES, SCROLL_SPRITEMAP_ADDRESS,
    SKELETON_TILEMAP_ADDRESS, SPIKE_WALL_DESTINATION_ADDRESS, SPIKE_WALL_TILE_ADDRESS,
};

/// A lease on a game with Crocomire's effects.
/// Crocomire's melt words, which the Session holds as `actors.crocomire_melt`,
/// read and stored through the host's projected bytes.

pub trait Lease:
    BossLease<Game: Host> + ActorAccess + crate::actors::boss_interaction::Access
{
}
impl<T: BossLease<Game: Host> + ActorAccess + crate::actors::boss_interaction::Access + ?Sized>
    Lease for T
{
}

/// What Crocomire needs beyond the shared boss effects: its imported tables,
/// the BG2 alignment, the native boss word, projectile clearing and its drop.
pub trait Host: BossEffects + crate::menus::host::Host {
    /// The word at `index` of the Crocomire table at `address` (scroll
    /// spritemaps, spike wall tiles and destinations, bridge debris).
    fn crocomire_table_word(&self, address: u32, index: isize) -> u16;
    /// The 256 tiles of the melt tilemap at `table` while it is imported.
    fn crocomire_melt_tilemap(&self, table: u32) -> Option<[u16; MELT_TILES]>;
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
/// $A4:98CA: the rumble table, ended by 8080h.
const RUMBLE_TABLE: u32 = 0xa4_98ca;
/// $A4:9697: the melting X offsets, one per pixel column in melting order, and
/// $A4:9BBD: the tile pixel column bitmasks, ~80h >> column. Erase melting
/// Crocomire pixel column reads both, and the melting Y offsets, through the
/// data bank $A4; each `_BUS` is the load's high operand byte, which the read
/// leaves on the data bus.
const MELTING_X_OFFSETS: u16 = 0x9697;
const MELTING_X_OFFSETS_BUS: u8 = 0x96;
const PIXEL_COLUMN_MASKS: u16 = 0x9bbd;
const PIXEL_COLUMN_MASKS_BUS: u8 = 0x9b;
const MELTING_Y_OFFSETS_OPERAND: u16 = 0x069c;
const MELTING_Y_OFFSETS_BUS: u8 = 0x06;
/// $7E:4000: the melting tiles' graphics buffer.
const MELTING_TILES_BUFFER: usize = 0x4000;
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

/// The death sequence's next step: its index is a word offset, raised a byte
/// at a time as the original does.
fn next_death_step<L: Lease + ?Sized>(k: i32, game: &mut L) {
    for _ in 0..2 {
        session_slot!(game, Crocomire[k].death_sequence_index, |old| old
            .wrapping_add(1));
    }
}
/// The death sequence's next step in one store.
fn skip_death_step<L: Lease + ?Sized>(k: i32, game: &mut L) {
    session_slot!(game, Crocomire[k].death_sequence_index, |old| old
        .wrapping_add(2));
}
fn cap_byte(value: u8, limit: u8) -> u8 {
    if value.wrapping_sub(limit) as i8 >= 0 {
        limit
    } else {
        value
    }
}
pub fn fill_scroll<L: Lease + ?Sized>(game: &mut L) {
    let scroll = display_scroll!(game.Bg2V);
    crate::word_transfers!(game, descending, 256; le(CROCOMIRE_BG2_SCROLL_HDMA_DATA, 255) <- value(scroll));
}
pub fn melt_down<L: Lease + ?Sized>(game: &mut L) {
    fill_scroll(game);
    session_slot!(game, Crocomire[0].fight_flags, |old| old & !flag::DAMAGED);
    update_background(game.current_enemy(), game);
    let key = game.current_enemy() as i32;
    if (session_slot!(game, Crocomire[key].base.y_pos) as i32 - 280) & 0x8000 != 0 {
        let mut cursor = (8 * ((display_scroll!(game.Bg2V) as i32 + 288) & 0xfff8)) as u16;
        for _ in 0..32 {
            bg2_staging::set(game, usize::from(cursor >> 1), 824);
            cursor = cursor.wrapping_add(2);
        }
        battle!(game.EnemyTilemapPending = 1);
        session_slot!(game, Crocomire[0].base.extra_properties, |old| old
            & !0x8000);
        let low = session_slot!(game, Crocomire[key].step_counter) as u8;
        session_slot!(byte game, Crocomire[key].step_counter[0] = low.wrapping_add(0x80));
        let next = (low.overflowing_add(0x80).1 as u8)
            .wrapping_add(session_slot!(byte game, Crocomire[key].step_counter[1]))
            .wrapping_add(3);
        let high = cap_byte(next, 48);
        session_slot!(byte game, Crocomire[key].step_counter[1] = high);
        let carry = session_slot!(byte game, Crocomire[key].y_velocity[0])
            .overflowing_add(high)
            .1;
        session_slot!(byte game, Crocomire[key].y_velocity[0] = session_slot!(byte game, Crocomire[key].y_velocity[0]).wrapping_add(high));
        let high = cap_byte(
            (carry as u8).wrapping_add(session_slot!(byte game, Crocomire[key].y_velocity[1])),
            3,
        );
        session_slot!(byte game, Crocomire[key].y_velocity[1] = high);
        let velocity = session_slot!(game, Crocomire[key].y_velocity) as u8;
        let carry = session_slot!(byte game, Crocomire[key].projectile_counter[1])
            .overflowing_add(velocity)
            .1;
        session_slot!(byte game, Crocomire[key].projectile_counter[1] = session_slot!(byte game, Crocomire[key].projectile_counter[1]).wrapping_add(velocity));
        let (step, carry) =
            (carry as u8).overflowing_add(session_slot!(byte game, Crocomire[key].y_velocity[1]));
        let carry = carry
            | session_slot!(byte game, Crocomire[key].base.y_pos[0])
                .overflowing_add(step)
                .1;
        session_slot!(byte game, Crocomire[key].base.y_pos[0] = session_slot!(byte game, Crocomire[key].base.y_pos[0]).wrapping_add(step));
        session_slot!(byte game, Crocomire[key].base.y_pos[1] = session_slot!(byte game, Crocomire[key].base.y_pos[1]).wrapping_add(carry as u8));
    } else {
        next_death_step(key, game);
        session_slot!(game, Crocomire[0].step_counter = 48);
    }
}
pub fn melt_up<L: Lease + ?Sized>(game: &mut L) {
    fill_scroll(game);
    let key = game.current_enemy() as i32;
    if (session_slot!(game, Crocomire[key].base.y_pos) as i32 - 218) & 0x8000 != 0 {
        next_death_step(key, game);
    } else {
        update_background(game.current_enemy(), game);
        let mut acceleration =
            (session_slot!(game, Crocomire[0].step_counter) as i16).wrapping_add(256);
        if (session_slot!(game, Crocomire[0].step_counter) as i32 - 7680) & 0x8000 == 0 {
            acceleration = 7936;
        }
        session_slot!(game, Crocomire[0].step_counter = acceleration as u16);
        let low = session_slot!(game, Crocomire[0].y_velocity) as u8;
        let borrow = low < session_slot!(byte game, Crocomire[0].step_counter[1]);
        session_slot!(byte game, Crocomire[0].y_velocity[0] = low.wrapping_sub(session_slot!(byte game, Crocomire[0].step_counter[1])));
        let mut high =
            (session_slot!(byte game, Crocomire[0].y_velocity[1]) as i8).wrapping_sub(borrow as i8);
        if high < 0 {
            session_slot!(byte game, Crocomire[0].y_velocity[0] = 0xff);
            high = 0;
        }
        session_slot!(byte game, Crocomire[0].y_velocity[1] = high as u8);
        let fraction = session_slot!(byte game, Crocomire[0].projectile_counter[1]);
        let borrow = fraction < session_slot!(byte game, Crocomire[0].y_velocity[0]);
        session_slot!(byte game, Crocomire[0].projectile_counter[1] = fraction.wrapping_sub(session_slot!(byte game, Crocomire[0].y_velocity[0])));
        let y = session_slot!(game, Crocomire[0].base.y_pos) as u8;
        let step =
            (borrow as u8).wrapping_add(session_slot!(byte game, Crocomire[0].y_velocity[1]));
        session_slot!(byte game, Crocomire[0].base.y_pos[0] = y.wrapping_sub(step));
        session_slot!(byte game, Crocomire[0].base.y_pos[1] = session_slot!(byte game, Crocomire[0].base.y_pos[1]).wrapping_sub((y < step) as u8));
    }
}
pub fn update_background<L: Lease + ?Sized>(k: u16, game: &mut L) {
    display_scroll!(
        game.Bg2V = 67u16.wrapping_sub(session_slot!(game, Crocomire[k as i32].base.y_pos))
    );
    let mut index = 32u16;
    let spritemap = loop {
        let candidate = session_slot!(game, Crocomire[0].base.spritemap_pointer);
        let spritemap = game.foreign(|game| {
            game.crocomire_table_word(SCROLL_SPRITEMAP_ADDRESS, (index >> 1) as isize)
        });
        if candidate == spritemap {
            break candidate;
        }
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            align_background(game);
            return;
        }
    };
    let offset = game
        .foreign(|game| crate::cartridge::native_record(0xa4, spritemap, game))
        .table_word(14);
    display_scroll!(game.Bg2V += offset);
    align_background(game);
}
/// Position Crocomire's second body part and align its BG2 image with the camera.
fn align_background<L: Lease + ?Sized>(game: &mut L) {
    use crate::actors::layouts::EnemyLayout72 as Layout;
    let primary = i32::from(game.current_enemy());
    let secondary = i32::from(game.current_enemy()) + 64;
    session_slot!(
        game,
        Layout[secondary].base.x_pos = session_slot!(game, Layout[secondary].var_a)
            .wrapping_add(session_slot!(game, Layout[primary].base.x_pos))
    );
    session_slot!(
        game,
        Layout[secondary].base.y_pos = session_slot!(game, Layout[primary].base.y_pos)
    );
    let relative = session_slot!(game, Layout[primary].base.x_pos)
        .wrapping_sub(game.camera().layer1[0]) as i16;
    let offset = 51u16.wrapping_sub(relative as u16);
    crate::menus::prelude::display_scroll!(
        game.Bg2H = if (-128..384).contains(&relative) && (offset as i16).unsigned_abs() < 284 {
            offset
        } else {
            256
        }
    );
}
pub fn update_palette<L: Lease + ?Sized>(game: &mut L) {
    if battle!(game.DoorTransitionEnemies) == 0 {
        if session_slot!(game, Crocomire[0].base.flash_timer) != 0
            && (battle!(game.RandomEnemyCounter) & 2) != 0
        {
            crate::word_transfers!(game, descending, 8; native(BG_PALETTE_7_RAM) <- value(0x7fff));
        } else {
            crate::word_transfers!(game, descending, 8; native(BG_PALETTE_7_RAM) <- rom(PALETTE_BG_7, 7));
        }
    }
}
fn wall_counter(runtime: &(impl ActorAccess + ?Sized), key: u16) -> u16 {
    runtime
        .resolve_actor(Key::from_legacy(key), AccessMode::Native)
        .variables[lane::STEP_COUNTER]
}
fn set_wall_counter(runtime: &mut (impl ActorAccess + ?Sized), key: u16, value: u16) {
    runtime.edit_actor(Key::from_legacy(key), AccessMode::Native, |enemy| {
        enemy.variables[lane::STEP_COUNTER] = value
    });
}
/// Crocomire main AI - death sequence index 40h - behind wall - rumbling ($A4:9859). Was
/// `wall_step`.
pub fn behind_wall_rumbling<L: Lease + ?Sized>(game: &mut L) {
    let mut cursor = wall_counter(game, 0);
    let image = game.loaded_image().clone();
    let target = image.table_word(
        sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
        (cursor >> 1) as isize,
    );
    if target == 0x8080 {
        set_wall_counter(game, 0x40, 0x8080);
        set_wall_counter(game, 0, 128);
        crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 191) <- rom(PALETTE_SPRITE_6, 15));
        game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
        return;
    }
    let current = wall_counter(game, 0x40);
    if current != target {
        let step = wall_counter(game, 0xc0);
        let next = if (current as i16).wrapping_sub(target as i16) >= 0 {
            current.wrapping_sub(step)
        } else {
            current.wrapping_add(step)
        };
        set_wall_counter(game, 0x40, next);
        return;
    }
    if target & 0x8000 != 0 {
        let countdown = wall_counter(game, 0x80);
        if countdown != 0 {
            set_wall_counter(game, 0x80, countdown.wrapping_sub(1));
            set_wall_counter(game, 0, cursor.wrapping_sub(2));
            game.queue_sound(1, 0x2b, 6);
            return;
        }
        cursor = cursor.wrapping_add(2);
        let countdown = image.table_word(
            sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(game, 0x80, countdown);
        cursor = cursor.wrapping_add(2);
        let step = image.table_word(
            sm_data::cartridge::lorom_offset(RUMBLE_TABLE) as usize,
            (cursor >> 1) as isize,
        );
        set_wall_counter(game, 0xc0, step);
    }
    set_wall_counter(game, 0, cursor.wrapping_add(2));
}
/// Handle playing Crocomire acid damage sound effect ($A4:8D1F). Was `rumble`.
pub fn play_acid_damage_sound<L: Lease + ?Sized>(game: &mut L) {
    let remaining = session_slot!(game, Crocomire[0].acid_damage_sound_timer);
    if remaining != 0 {
        session_slot!(game, Crocomire[0].acid_damage_sound_timer = remaining - 1);
        if remaining == 1 {
            session_slot!(game, Crocomire[0].acid_damage_sound_timer = 32);
            game.queue_sound(2, 0x22, 6);
        }
    }
}
fn contact_edge<L: Lease + ?Sized>(game: &mut L) -> i32 {
    session_slot!(game, Crocomire[0].base.x_pos) as i32
        - session_slot!(game, Crocomire[0].base.x_width) as i32
        - battle!(game.SamusXRadius) as i32
}
fn contact<L: Lease + ?Sized>(game: &mut L) {
    game.contact_reaction(Death::Finalize);
}
/// Enemy touch - Crocomire - claws ($A4:B93D).
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    contact(game);
    session_slot!(game, Crocomire[0].fight_flags, |flags| flags
        | flag::SAMUS_HIT_BY_CLAW);
    battle!(game.ExtraSamusXDisplacement = -4i32 as u16);
}
/// Crocomire / Samus collision handling ($A4:8C95). Was `push_player`.
pub fn samus_collision<L: Lease + ?Sized>(game: &mut L) {
    // The test's subtracts carry their borrows on, as the original's do ($A4:8C9A).
    let (edge, carry) = sbc(
        session_slot!(game, Crocomire[0].base.x_pos),
        session_slot!(game, Crocomire[0].base.x_width),
        true,
    );
    let (edge, carry) = sbc(edge, battle!(game.SamusXRadius), carry);
    if session_slot!(game, Crocomire[0].death_sequence_index) == 0
        && (sbc(edge, game.player_x(), carry).0 as i16) < 0
    {
        contact(game);
        let edge = contact_edge(game) as u16;
        game.set_player_x(edge);
        game.set_previous_player_x(game.player_x());
        battle!(game.ExtraSamusXDisplacement = -4i32 as u16);
        battle!(game.ExtraSamusYDisplacement = -1i32 as u16);
    }
}
fn next_hit_count(flags: u16) -> u16 {
    ((flags & 0xf) + 1).min(15)
}
/// Enemy shot - Crocomire - nothing ($A4:B951): count the hit.
pub fn count_hit<L: Lease + ?Sized>(game: &mut L) {
    let count = next_hit_count(session_slot!(game, Crocomire[0].fight_flags));
    session_slot!(game, Crocomire[0].fight_flags, |flags| flags | count);
}
/// Enemy shot - Crocomire - spawn shot explosion ($A4:B968, $A4:BAB4).
pub fn impact<L: Lease + ?Sized>(game: &mut L) {
    let index = battle!(game.CollisionDetectionIndex) as i32;
    let x = interaction::get_at(game, interaction::Array::ProjectileXPos, index);
    let y = interaction::get_at(game, interaction::Array::ProjectileYPos, index);
    game.place_projectile_spawn(x, y);
    let appearance =
        if interaction::get_at(game, interaction::Array::ProjectileType, index) & 0x200 == 0 {
            6
        } else {
            29
        };
    game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, appearance);
}
/// Enemy shot - Crocomire - open mouth ($A4:BA05).
pub fn hit<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Crocomire[0].base.invincibility_timer = 0);
    let mut advance = ((session_slot!(game, Crocomire[0].base.x_pos) as i32
        - session_slot!(game, Crocomire[0].base.x_width) as i32
        - 256
        - game.camera().layer1[0] as i32) as i16)
        >= 0;
    if !advance {
        let projectile = interaction::get_at(
            game,
            interaction::Array::ProjectileType,
            battle!(game.CollisionDetectionIndex),
        );
        if projectile & 0xf10 == 0 {
            session_slot!(game, Crocomire[0].base.instruction_timer = 8);
            impact(game);
            return;
        }
        let recoil = [2u16, 1, 3]
            .get(((projectile & 0xf00) >> 8) as usize)
            .copied()
            .unwrap_or(0);
        advance = recoil != 0;
        if advance {
            session_slot!(game, Crocomire[0].step_counter, |old| old
                .wrapping_add(recoil));
        }
    }
    if advance {
        let count = next_hit_count(session_slot!(game, Crocomire[0].fight_flags));
        if session_slot!(game, Crocomire[0].fight_flags) & flag::DAMAGED == 0 {
            let mut delay = 8;
            if session_slot!(game, Crocomire[0].fight_function) == 8 {
                delay = 8;
            }
            session_slot!(game, Crocomire[0].base.instruction_timer, |old| old
                .wrapping_add(delay));
        }
        session_slot!(game, Crocomire[0].fight_flags, |old| count
            | old & !(flag::HIT_COUNT | flag::DAMAGED | flag::SAMUS_HIT_BY_CLAW)
            | flag::DAMAGED);
        session_slot!(game, Crocomire[0].y_velocity = 10);
    }
    session_slot!(game, Crocomire[0].base.flash_timer, |old| old
        .wrapping_add(14));
    session_slot!(game, Crocomire[0].base.ai_handler_bits, |old| old | 2);
}

pub fn select_melt_animation<const SKELETON: bool, L: Lease + ?Sized>(key: Key, game: &mut L) {
    game.edit_actor(key, AccessMode::Native, |enemy| {
        let instruction = match enemy.y.whole as i16 {
            y if y.wrapping_sub(280) >= 0 => ADDR_K_CROCOMIRE_ILIST_BF64 as u16,
            y if y.wrapping_sub(264) >= 0 => ADDR_K_CROCOMIRE_ILIST_BF6C as u16,
            y if y.wrapping_sub(248) >= 0 => ADDR_K_CROCOMIRE_ILIST_BF72 as u16,
            _ => ADDR_K_CROCOMIRE_ILIST_BF78 as u16,
        };
        // Each skeleton list starts 0x1a bytes after its corresponding flesh list.
        enemy.instruction = instruction + if SKELETON { 0x1a } else { 0 };
        enemy.instruction_delay = 1;
    });
}

/// Crocomire main AI - death sequence index 34h - hop 6 - rising ($A4:926E). Was
/// `melt_skeleton_up`.
pub fn hop_6_rising<L: Lease + ?Sized>(game: &mut L) {
    let key = Key::from_legacy(game.current_enemy());
    select_melt_animation::<true, _>(key, game);
    emit_lava_plume(game);
    melt_up(game);
}

/// Crocomire main AI - death sequence index 1Eh/24h/2Ah - hop 3/4/5 - sinking ($A4:9108). Was
/// `melt_flesh_down`.
pub fn hop_3_4_5_sinking<L: Lease + ?Sized>(game: &mut L) {
    emit_lava_plume(game);
    let key = Key::from_legacy(0);
    select_melt_animation::<false, _>(key, game);
    melt_down(game);
}

pub fn emit_lava_plume<L: Lease + ?Sized>(game: &mut L) {
    if battle!(game.CrocomireAcidSmoke -= 1, reread) == 0 {
        battle!(game.CrocomireAcidSmoke = 6);
        let mut v0: u16 = (battle!(game.Random)) & (0x3fu16);
        if (battle!(game.Random) & 2) == 0 {
            v0 = !(v0 as i32) as u16;
        }
        let x: u16 = (session_slot!(game, Crocomire[0i32].base.x_pos)).wrapping_add(v0);
        let y: u16 = (battle!(game.LavaAcidY) as i32 + 16
            - (((battle!(game.Random)) & (0x1f00u16)) as i32 >> 8)) as u16;
        game.foreign(|game| game.create_sprite(x, y, 21, 0));
    }
}

/// Crocomire main AI - death sequence index 8/Eh - hop 1/2 - sinking ($A4:91BA). Was
/// `melt_rumble_down`.
pub fn hop_1_2_sinking<L: Lease + ?Sized>(game: &mut L) {
    play_acid_damage_sound(game);
    emit_lava_plume(game);
    melt_down(game);
}

/// Crocomire main AI - death sequence index 16h/22h/28h - hop 3/4/5 - rising ($A4:929E). Was
/// `melt_flesh_up`.
pub fn hop_3_4_5_rising<L: Lease + ?Sized>(game: &mut L) {
    emit_lava_plume(game);
    let key = Key::from_legacy(game.current_enemy());
    select_melt_animation::<false, _>(key, game);
    melt_up(game);
}

/// Crocomire main AI - death sequence index 6/Ch - hop 1/2 - rising ($A4:92CE). Was
/// `melt_rumble_up`.
pub fn hop_1_2_rising<L: Lease + ?Sized>(game: &mut L) {
    play_acid_damage_sound(game);
    emit_lava_plume(game);
    melt_up(game);
}

pub fn queue_tilemap_upload<L: Lease + ?Sized>(size: u16, bg_mask: u8, game: &mut L) {
    let vram_address = u16::from(game.slot_byte(0x59) & bg_mask) << 8;
    let source = Source {
        address: 0x2000,
        bank: 0x7e,
    };
    queue_tilemap(
        game,
        TilemapUpload {
            bytes: size,
            source,
            column: false,
            vram_address,
        },
    );
}

fn load_melt_tilemap<L: Lease + ?Sized>(table: u32, game: &mut L) {
    // The tilemap serves its 256 tiles while the loaded image is the imported
    // allocation; otherwise the original loads scan for the $FFFF end.
    if let Some(tiles) = game.foreign(|game| game.crocomire_melt_tilemap(table)) {
        for (index, tile) in tiles.into_iter().enumerate() {
            bg2_staging::set(game, index + 32, tile);
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
        let tile = crate::cartridge::rom_table_word!(game, table, index as isize);
        bg2_staging::set(
            game,
            usize::try_from(index + 32).expect("melt tile index"),
            tile,
        );
        i += 2;
    }
    queue_tilemap_upload((i + 1024) as u16, 0xfc, game);
}

/// Crocomire main AI - death sequence index 10h - hop 3 - load melting tilemap ($A4:9341). Was
/// `prepare_flesh_tilemap`.
pub fn hop_3_load_melting_tilemap<L: Lease + ?Sized>(game: &mut L) {
    game.set_slot_byte(PIXELS_TO_ERASE, 48);
    battle!(game.MeltEraseTarget = 48);
    let e: i32 = game.current_enemy() as i32;
    next_death_step(e, game);
    session_slot!(game, Crocomire[e].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_BF64 as u16, instruction_timer = 1);
    battle!(game.EnemyTilemapPending = 1);
    let e1: i32 = game.current_enemy() as i32 + 64;
    session_slot!(game, Crocomire[e1].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_BF98 as u16, instruction_timer = 1);
    session_slot!(game, Crocomire[e1].base.properties, |old| old & 0xd2ff
        | 0x2c00);
    session_slot!(
        game,
        Crocomire[e1].base.x_pos = session_slot!(game, Crocomire[e].base.x_pos)
    );
    session_slot!(
        game,
        Crocomire[e1].base.y_pos = (session_slot!(game, Crocomire[e].base.y_pos)).wrapping_add(16)
    );
    crate::word_transfers!(game, (4), 256;
        le(TILEMAP_STUFF, 0) <- value(824),
        le(TILEMAP_STUFF, 1) <- value(824),
    );
    load_melt_tilemap(FLESH_TILEMAP_ADDRESS, game);
}

/// Crocomire main AI - death sequence index 2Ch - hop 6 - load melting tilemap ($A4:93ED). Was
/// `prepare_skeleton_tilemap`.
pub fn hop_6_load_melting_tilemap<L: Lease + ?Sized>(game: &mut L) {
    fill_scroll(game);
    let e: i32 = game.current_enemy() as i32;
    next_death_step(e, game);
    session_slot!(game, Crocomire[e].base.instruction_timer = 1);
    game.set_slot_byte(PIXELS_TO_ERASE, 48);
    battle!(game.MeltEraseTarget = 48);
    session_slot!(
        game,
        Crocomire[e].base.current_instruction = ADDR_K_CROCOMIRE_ILIST_BF7E as u16
    );
    bg2_staging::fill(game, 0..1024, 824);
    load_melt_tilemap(SKELETON_TILEMAP_ADDRESS, game);
}

macro_rules! native_graphics_word {
    ($game:ident, $address:expr, $offset:expr) => {{
        let address = $address;
        let record = crate::cartridge::native_record((address >> 16) as u8, address as u16, $game);
        record.image.table_word(record.base + $offset as usize, 0)
    }};
}

/// Queue the melting tiles record at `offset`: its size (at the current
/// subindex if `live_size`), source and VRAM destination.
pub fn queue_melt_graphics<L: Lease + ?Sized>(offset: u16, live_size: bool, game: &mut L) {
    let size_offset = if live_size {
        battle!(game.MeltSubindex)
    } else {
        offset
    };
    let bytes = native_graphics_word!(game, MELTING_TILES_SOURCE, size_offset);
    let address = native_graphics_word!(game, MELTING_TILES_SOURCE + 6, offset);
    let bank = native_graphics_word!(game, MELTING_TILES_SOURCE + 4, offset) as u8;
    let vram_address = native_graphics_word!(game, MELTING_TILES_SOURCE + 2, offset);
    let source = Source { address, bank };
    queue_tilemap(
        game,
        TilemapUpload {
            bytes,
            source,
            column: false,
            vram_address,
        },
    );
}

/// Crocomire main AI - death sequence index 14h/30h - hop 3/6 - uploading to VRAM ($A4:94B6). Was
/// `advance_melt_upload`.
pub fn upload_melting_tiles<L: Lease + ?Sized>(game: &mut L) {
    let offset = battle!(game.MeltSubindex);
    if native_graphics_word!(game, MELTING_TILES_SOURCE, battle!(game.MeltSubindex)) == 0xffff {
        let e: i32 = game.current_enemy() as i32;
        next_death_step(e, game);
        battle!(game.MeltSubindex = 0);
    } else {
        queue_melt_graphics(offset, true, game);
        battle!(game.MeltSubindex = offset.wrapping_add(8));
    };
}

/// Crocomire main AI - death sequence index 12h/2Eh - hop 3/6 - load melting tiles ($A4:943D). Was
/// `prepare_melt_graphics`.
pub fn load_melting_tiles<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = game.current_enemy() as i32;
    skip_death_step(e, game);
    battle!(game; MeltDisplacement = 256; MeltColumn = 0; MeltMaxY = native_graphics_word!(game, MELTING_TILES_SOURCE, battle!(game.MeltIndex)); MeltYOffset = battle!(game.MeltMaxY); MeltUnread = native_graphics_word!(game, MELTING_TILES_DESTINATION, battle!(game.MeltIndex)); MeltWords = native_graphics_word!(game, MELTING_TILES_SIZE, battle!(game.MeltIndex));
    );
    let bank: u8 = native_graphics_word!(game, MELTING_TILES_BANK, battle!(game.MeltIndex)) as u8;
    let mut i: i32 = battle!(game.MeltIndex) as i32 + 8;
    loop {
        let mut v2: u16 = native_graphics_word!(game, MELTING_TILES_SOURCE, i);
        if v2 == 0xffff {
            break;
        }
        let mut v3: u16 = native_graphics_word!(game, MELTING_TILES_DESTINATION, i);
        for _ in 0..=battle!(game.MeltWords) {
            crate::word_transfers!(game, ascending, 1; native(v3) <- value(native_graphics_word!(game, (u32::from(bank) << 16) | u32::from(v2), 0)));
            v3 = (v3).wrapping_add(2);
            v2 = (v2).wrapping_add(2);
        }
        i = i as u16 as i32 + 4;
    }
    battle!(game; MeltIndex = (i + 2) as u16; MeltSubindex = (i + 2) as u16;);
    crate::word_transfers!(game, descending, 65; native(0x71c) <- value(0));
}

/// Power bomb reaction - Crocomire ($A4:B992).
pub fn power_bomb<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = 0;
    if session_slot!(game, Crocomire[e].death_sequence_index) == 0 {
        session_slot!(game, Crocomire[e].step_counter = 3);
        if session_slot!(game, Crocomire[e].fight_function) != 24 {
            session_slot!(game, Crocomire[e].fight_flags, |old| old
                & !(flag::HIT_COUNT | flag::SAMUS_HIT_BY_CLAW | flag::AWAKE)
                | flag::AWAKE);
            session_slot!(game, Crocomire[e].y_velocity = 10);
            session_slot!(game, Crocomire[e].base.flash_timer, |old| old
                .wrapping_add(4));
            session_slot!(game, Crocomire[e].base.ai_handler_bits, |old| old | 2u16);
            session_slot!(game, Crocomire[e].fight_function = 24);
            let mut spritemap_pointer: u16 =
                session_slot!(game, Crocomire[e].base.spritemap_pointer);
            let mut n: i32 =
                native_graphics_word!(game, BANK << 16 | u32::from(spritemap_pointer), 0) as i32;
            let instruction = loop {
                let tag = native_graphics_word!(game, BANK << 16 | u32::from(spritemap_pointer), 6);
                if u32::from(tag) == ADDR_K_CROCOMIRE_BIG_SPRMAP_D600 {
                    break ADDR_K_CROCOMIRE_ILIST_BDAE as u16;
                }
                if u32::from(tag) == ADDR_K_CROCOMIRE_BIG_SPRMAP_D51C {
                    break ADDR_K_CROCOMIRE_ILIST_BDB2 as u16;
                }
                spritemap_pointer = (spritemap_pointer).wrapping_add(8);
                n -= 1;
                if n == 0 {
                    break ADDR_K_CROCOMIRE_ILIST_BDB6 as u16;
                }
            };
            session_slot!(game, Crocomire[e].base; current_instruction = instruction, instruction_timer = 1);
        }
    }
}

/// Crocomire main AI - death sequence index 0 - death sequence not started ($A4:8C6E). Was
/// `approach_wall`.
pub fn not_started<L: Lease + ?Sized>(game: &mut L) {
    update_wall(game);
    crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(257));
    if (game.player_x() as i32 - 1312i32) & 0x8000 == 0 {
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(256));
    }
    update_background(game.current_enemy(), game);
}

fn update_wall<L: Lease + ?Sized>(game: &mut L) {
    let e0: i32 = 0;
    if (session_slot!(game, Crocomire[e0].base.x_pos) as i32 - 1536i32) & 0x8000 != 0 {
        battle!(game, 0; QuakeSoundTimer, CrocomireBridge1, CrocomireBridge2, CrocomireUnread);
    } else if (session_slot!(game, Crocomire[e0].base.x_pos) as i32 - 1552i32) & 0x8000 != 0 {
        if battle!(game.QuakeSoundTimer) == 0 {
            battle!(game.QuakeSoundTimer = 1);
            game.place_projectile_spawn(1536, 176);
            game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15);
        }
    } else if (session_slot!(game, Crocomire[e0].base.x_pos) as i32 - 1552i32) & 0x8000 != 0 {
        battle!(game, 0; CrocomireBridge1, CrocomireBridge2, CrocomireUnread);
    } else if (session_slot!(game, Crocomire[e0].base.x_pos) as i32 - 1568i32) & 0x8000 != 0 {
        if battle!(game.CrocomireBridge1) == 0 {
            battle!(game.CrocomireBridge1 = 1);
            game.foreign(|game| game.spawn_plm(0x61, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK));
            game.place_projectile_spawn(1568, 176);
            game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15);
        }
    } else {
        let x_pos: u16 = session_slot!(game, Crocomire[e0].base.x_pos);
        if (x_pos as i32 - 1568i32) & 0x8000 != 0 {
            battle!(game, 0; CrocomireBridge2, CrocomireUnread);
        } else if (x_pos as i32 - 1584i32) & 0x8000 != 0 {
            if battle!(game.CrocomireBridge2) == 0 {
                battle!(game.CrocomireBridge2 = 1);
                game.foreign(|game| {
                    game.spawn_plm(0x62, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK);
                    game.spawn_plm(0x63, BRIDGE_ROW, CRUMBLE_BRIDGE_BLOCK);
                });
                game.place_projectile_spawn(1584, 176);
                game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15);
            }
        } else if ((x_pos) as i16).wrapping_sub((1600u16) as i16) >= 0 {
            demolish_wall(game);
            session_slot!(game, Crocomire[e0].acid_damage_sound_timer = 1);
            battle!(game.CrocomireAcidSmoke = 1);
            let _v3: u16 = game.current_enemy();
            let ek: i32 = game.current_enemy() as i32;
            skip_death_step(ek, game);
            for key in (0..=0x140i32).step_by(0x40) {
                session_slot!(game, Crocomire[key].unread_7800 = 2);
            }
            for key in (0..=0x140i32).step_by(0x40) {
                session_slot!(game, Crocomire[key].unread_7802 = 0);
            }
            game.queue_sound(1, 0x3b, 6);
            crate::word_transfers!(game, ascending, 1; native(CRUMBLING_BRIDGE_INDEX) <- value(0));
            game.foreign(|game| game.spawn_plm(0x4e, WALL_ROW, CREATE_INVISIBLE_WALL));
            session_slot!(game, Crocomire[ek].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_BFB0 as u16, instruction_timer = 1);
            session_slot!(game, Crocomire[ek].base.properties, |old| old
                | ((K_ENEMY_PROPS_TANGIBLE) as u16));
            let e1: i32 = game.current_enemy() as i32 + 64;
            session_slot!(game, Crocomire[e1].base; instruction_timer = 0x7fff, current_instruction = ADDR_K_CROCOMIRE_ILIST_BF62 as u16);
            session_slot!(game, Crocomire[0x40i32].base.properties, |old| old
                | (0x100u16));

            session_slot!(game, Crocomire[ek]; y_velocity = 0, projectile_counter = 0);
            session_slot!(game, Crocomire[ek].step_counter = 2048);
            session_slot!(game, Crocomire[0i32].base.y_height = 16);
        }
    };
}

fn demolish_wall<L: Lease + ?Sized>(game: &mut L) {
    for x in 0x61..=0x6a {
        game.foreign(|game| game.spawn_plm(x, BRIDGE_ROW, CLEAR_BRIDGE_BLOCK));
    }
    for (offset, y) in [176, 192, 176, 192, 192, 192, 192].into_iter().enumerate() {
        game.place_projectile_spawn(1536 + 16 * offset as u16, y);
        game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15);
    }
}

/// Crocomire death sequence index += 2 ($A4:9BB3). Was `advance_phase`.
pub fn advance_death_sequence<L: Lease + ?Sized>(game: &mut L) {
    game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
            enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
    });
}

/// Crocomire main AI - death sequence index 20h/26h - hop 4/5 - resting ($A4:8D47). Was
/// `wait_for_melt`.
pub fn hop_4_5_resting<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = 0;
    let counter: u16 = session_slot!(game, Crocomire[e].step_counter);
    if counter != 0 {
        session_slot!(game, Crocomire[e].step_counter = (counter).wrapping_sub(1));
    } else {
        next_death_step(e, game);
        session_slot!(game, Crocomire[e].y_velocity = 768);
    };
}

/// Crocomire main AI - death sequence index 4/Ah - hop 1/2 - resting ($A4:8D3F). Was
/// `rumble_and_wait`.
pub fn hop_1_2_resting<L: Lease + ?Sized>(game: &mut L) {
    play_acid_damage_sound(game);
    align_background(game);
    hop_4_5_resting(game);
}

/// Crocomire main AI - death sequence index 42h - behind wall - no more rumbling ($A4:990A). Was
/// `prepare_spike_wall`.
pub fn behind_wall_no_more_rumbling<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = 0;
    let counter: u16 = session_slot!(game, Crocomire[e].step_counter);
    if counter != 0 {
        session_slot!(game, Crocomire[e].step_counter = (counter).wrapping_sub(1));
        let v3: i16 = battle!(game.MeltEraseTarget) as i16;
        let v4: i32 = battle!(game.MeltEraseTarget) as i32 >> 1;
        let tile =
            game.foreign(|game| game.crocomire_table_word(SPIKE_WALL_TILE_ADDRESS, v4 as isize));
        if tile != 0xffff {
            let address = game.foreign(|game| {
                game.crocomire_table_word(SPIKE_WALL_DESTINATION_ADDRESS, v4 as isize)
            });
            let vram_address = tile.wrapping_add(u16::from(game.slot_byte(0x52) & 7) << 13);
            let source = Source {
                address,
                bank: 0xad,
            };
            queue_tilemap(
                game,
                TilemapUpload {
                    bytes: 512,
                    source,
                    column: false,
                    vram_address,
                },
            );
            battle!(game.MeltEraseTarget = ((v3) as u16).wrapping_add(2));
        }
    } else {
        session_slot!(game, Crocomire[e].base; x_pos = 480, y_pos = 54);
        session_slot!(game, Crocomire[0x80i32].step_counter = 80);
        session_slot!(game, Crocomire[e]; y_velocity = 0, projectile_counter = 0);
        game.foreign(|game| {
            game.spawn_plm(0x20, WALL_ROW, CLEAR_INVISIBLE_WALL);
            game.spawn_plm(0x1e, WALL_ROW, CREATE_INVISIBLE_WALL);
            game.spawn_plm(0x70, BRIDGE_ROW, CLEAR_BRIDGE);
        });
        game.queue_sound(1, 0x29, 6);
        session_slot!(game, Crocomire[e].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_E158 as u16, instruction_timer = 1);
        session_slot!(game, Crocomire[e].base.palette_index = 0);
        crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 159) <- rom(PALETTE_SPRITE_4, 15));
        game.foreign(|game| game.clear_enemy_projectiles());
        let mut v8: u16 = 8;
        for _ in 0..8 {
            let k = game.current_enemy();
            v8 = game.spawn_projectile(v8, k, ADDR_K_EPROJ_CROCOMIRE_SPIKE_WALL_PIECES as u16);
        }
        game.queue_sound(1, 0x30, 6);
        advance_death_sequence(game);
    };
}

/// Crocomire main AI - death sequence index 1Ah/38h - hop 3/6 - melting ($A4:9580). Was
/// `advance_melt_scroll`.
pub fn melting<L: Lease + ?Sized>(game: &mut L) {
    emit_lava_plume(game);
    let e0: i32 = 0;
    let e1: i32 = 0x40;
    let e2: i32 = 0x80;
    let mut counter: u16 = session_slot!(game, Crocomire[e2].step_counter);
    session_slot!(game, Crocomire[e1].step_counter, |old| old.wrapping_sub(1));
    // A is 0 or the shifted X position here, and the erase keeps its low byte.
    let mut bug = 0;
    if (session_slot!(game, Crocomire[e1].step_counter) & 2) != 0 {
        counter = (counter).wrapping_add(4);
        [bug, _] = counter.to_le_bytes();
    }
    session_slot!(game, Crocomire[e0].base.x_pos = counter);
    if erase_melt_columns(game, bug) != 0 {
        update_background(game.current_enemy(), game);
        let mut v4: u16 = (battle!(game.MeltYOffset)).wrapping_sub(3);
        let skip_tilemap_update: bool = ((battle!(game.MeltYOffset) as i32 - 19i32) & 0x8000 != 0)
            && ((((battle!(game.MeltDisplacement)) as i16).wrapping_sub(20480) >= 0)
                || ({
                    v4 = 16;
                    false
                }));
        if skip_tilemap_update {
        } else {
            battle!(game.MeltYOffset = v4);
            let mut v5: u16 = (battle!(game.MeltDisplacement)).wrapping_add(384);
            if (battle!(game.MeltDisplacement) as i32 - 20096i32) & 0x8000 == 0 {
                v5 = 20480;
            }
            battle!(game.MeltDisplacement = v5);
            let mut r18: u16 = 0;
            let mut v6: u16 =
                (2 * (session_slot!(game, Crocomire[e1].base.y_pos) as i32 - 72i32)) as u16;
            let mut v7: u16 = battle!(game.MeltYOffset);
            battle!(game.MessageBoxScratch1 = battle!(game.MeltYOffset));
            do_until! { ((v7) as i16).wrapping_sub((battle!(game.MeltMaxY)) as i16) >= 0 ;
                word!(
                    game.CROCOMIRE_BG2_SCROLL_HDMA_DATA[(v6 >> 1)] =
                        (display_scroll!(game.Bg2V) as i32 + v7 as i32
                            - battle!(game.MessageBoxScratch1) as i32)
                            as u16
                );
                let v8: bool = u16::overflowing_add(battle!(game.MeltDisplacement), r18).1;
                r18 = (r18).wrapping_add(battle!(game.MeltDisplacement));
                if !v8 {
                    v7 = v7.wrapping_add(1);
                }
                battle!(game.MessageBoxScratch1 += 1);
                v6 = (v6).wrapping_add(2);
            }
            if ((v6) as i16).wrapping_sub(512) < 0 {
                let v9: u16 = display_scroll!(game.Bg2V);
                do_until! { ((v6) as i16).wrapping_sub(512) >= 0 ;
                    word!(game.CROCOMIRE_BG2_SCROLL_HDMA_DATA[(v6 >> 1)] = v9);
                    v6 = (v6).wrapping_add(2);
                }
            }
            return;
        }
    }
    let ek: i32 = game.current_enemy() as i32;
    skip_death_step(ek, game);
    let mut i: i32 = battle!(game.MeltIndex) as i32;
    while native_graphics_word!(game, MELTING_TILES_SOURCE, i) != 0xffff {
        i += 8;
    }
    battle!(game.MeltIndex = (i + 2) as u16);
    let object = session_slot!(game, Crocomire[e0].melting_hdma_object);
    game.foreign(|game| crate::hdma_objects::free_hdma_object(object, game));
}

/// Erase melting Crocomire pixel column ($A4:96C8). `bug` is the low byte of
/// the caller's A, which the routine's XBA leaves as the high byte of Y: the
/// column search and the pixel mask then read $100 * `bug` bytes further on.
/// Returns 0 when the column search runs off the X offset table index range.
fn erase_melt_columns<L: Lease + ?Sized>(game: &mut L, bug: u8) -> u16 {
    let [target, _] = battle!(game.MeltEraseTarget).to_le_bytes();
    let mut column = battle!(game.MeltColumn);
    // Columns melted down to the target are skipped. Past the 49-entry table
    // the X offsets are the routine's own code bytes.
    loop {
        let height = data_bank_byte(
            game,
            MELTING_Y_OFFSETS_OPERAND,
            u16::from_le_bytes([x_offset(game, column), bug]),
            MELTING_Y_OFFSETS_BUS,
        );
        if height.wrapping_sub(target) & 0x80 != 0 {
            break;
        }
        column = column.wrapping_add(1);
        if column.wrapping_sub(0x80) & 0x8000 == 0 {
            battle!(game.MeltColumn = 0);
            return 0;
        }
    }
    battle!(game.MeltColumn = column);
    // The mask's pixel column is the table index % 8, not the X offset's.
    let [low, _] = column.to_le_bytes();
    let mask = data_bank_byte(
        game,
        PIXEL_COLUMN_MASKS,
        u16::from_le_bytes([low & 7, bug]),
        PIXEL_COLUMN_MASKS_BUS,
    );
    let offset = x_offset(game, column);
    let height_at = MELTING_Y_OFFSETS + usize::from(offset);
    let mut pixels = u16::from(game.slot_byte(PIXELS_TO_ERASE));
    loop {
        // The Y offset is read as a word: the next column's low two bits reach
        // bit 14 of the tile offset.
        let height = game.slot_word(height_at);
        let tile = ((u16::from(offset) & 0xf8) << 2)
            .wrapping_add((height & 7) << 1)
            .wrapping_add((height & 0xfff8) << 6);
        for plane in [0, 1, 0x10, 0x11] {
            let at = MELTING_TILES_BUFFER + usize::from(tile) + plane;
            game.set_slot_byte(at, game.slot_byte(at) & mask);
        }
        if game.slot_byte(height_at) == 48 {
            break;
        }
        game.set_slot_word(height_at, game.slot_word(height_at).wrapping_add(1));
        pixels = pixels.wrapping_sub(1);
        if pixels == 0 {
            break;
        }
    }
    let v7 = loop {
        let v7 = (battle!(game.MeltSubindex)).wrapping_add(battle!(game.MeltIndex));
        if native_graphics_word!(game, MELTING_TILES_SOURCE, v7) != 0xffff {
            break v7;
        }
        battle!(game.MeltSubindex = 0);
    };
    queue_melt_graphics(v7, false, game);
    battle!(game.MeltSubindex += 8);
    1
}

/// The byte the CPU reads at `address` + `index` with the data bank at $A4, as
/// Erase melting Crocomire pixel column's indexed loads do. The index carries
/// into bank $A5. Below $2000 is low RAM's mirror and from $8000 the cartridge;
/// between them the read finds the open bus, which still holds `bus`. The
/// registers in that span ($21xx, $4016-$4017, $42xx-$43xx) are only within
/// reach of an X offset read past the table, and read as the open bus here.
fn data_bank_byte<L: Lease + ?Sized>(game: &L, address: u16, index: u16, bus: u8) -> u8 {
    let (offset, carry) = address.overflowing_add(index);
    match offset {
        ..0x2000 => game.slot_byte(usize::from(offset)),
        0x8000.. => {
            let long = ((BANK + u32::from(carry)) << 16) | u32::from(offset);
            game.native_image()
                .read(sm_data::cartridge::lorom_offset(long).unsigned_abs())
        }
        _ => bus,
    }
}

/// The melting X offset at table index `column` ($A4:9697).
fn x_offset<L: Lease + ?Sized>(game: &L, column: u16) -> u8 {
    data_bank_byte(game, MELTING_X_OFFSETS, column, MELTING_X_OFFSETS_BUS)
}

/// Initializes the defeated or active boss room with the original ordered effects.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    battle!(game.BossId = 6);
    bg2_staging::fill(game, 0..2048, 824);
    if ({
        let area = game.area();
        game.foreign(|game| game.boss_word_native(area))
    } & 2)
        != 0
    {
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_0_1) <- value(257));
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_2_3) <- value(257));
        battle!(game.MeltEraseTarget = 0);
        let e: i32 = 0;
        session_slot!(game, Crocomire[e].base.properties, |old| old & 0x7bff
            | 0x400);
        game.foreign(|game| {
            game.spawn_plm(0x20, WALL_ROW, CLEAR_INVISIBLE_WALL);
            game.spawn_plm(0x1e, WALL_ROW, CLEAR_INVISIBLE_WALL);
            game.spawn_plm(0x61, BRIDGE_ROW, CLEAR_BRIDGE);
        });
        session_slot!(
            game,
            Crocomire[e].death_sequence_index = death::SET_INDEX_56
        );
        session_slot!(game, Crocomire[e].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_E1CC as u16, instruction_timer = 1);
        session_slot!(game, Crocomire[e].base.x_pos = 576);
        session_slot!(game, Crocomire[e].base; y_pos = 144, y_height = 28, x_width = 40);
        queue_tilemap_upload(2048, 0xfc, game);
    } else {
        game.foreign(|game| game.disable_minimap_for_boss());
        battle!(game.MeltIndex = 0);
        let e_0: i32 = game.current_enemy() as i32;
        session_slot!(game, Crocomire[e_0]; death_sequence_index = 0, y_velocity = 0);
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_0_1) <- value(0));
        crate::word_transfers!(game, descending, 17;
            le(TARGET_PALETTES, 176) <- rom(PALETTE_TARGET_SPRITE_3, 16),
            le(TARGET_PALETTES, 224) <- rom(PALETTE_TARGET_SPRITE_6, 16),
        );
        session_slot!(game, Crocomire[e_0].fight_function = 4);
        battle!(game; Unused179E = 16; EnemyTilemapSize = 1024;);
        battle!(game.CameraDistance = 2);
        session_slot!(game, Crocomire[e_0].base; current_instruction = ADDR_K_CROCOMIRE_ILIST_BADE as u16, extra_properties = (session_slot!(game, Crocomire[e_0].base.extra_properties)) | (4u16), instruction_timer = 1);
    }
}

/// Crocomire main AI - death sequence index 3Ch - hop 6 - sinking ($A4:9099). Was
/// `complete_skeleton_melt`.
pub fn hop_6_sinking<L: Lease + ?Sized>(game: &mut L) {
    let key = Key::from_legacy(0);
    select_melt_animation::<true, _>(key, game);
    emit_lava_plume(game);
    melt_down(game);
    let e: i32 = 0;
    if session_slot!(game, Crocomire[e].death_sequence_index) == death::BEHIND_WALL_WAIT_FOR_SAMUS {
        game.foreign(|game| game.queue_music_delayed(6));
        session_slot!(
            game,
            Crocomire[e].death_sequence_index = death::FLOWING_DOWN_THE_RIVER
        );
        session_slot!(
            game,
            Crocomire[e].base.current_instruction = ADDR_K_CROCOMIRE_ILIST_E1D2 as u16
        );
        crate::word_transfers!(game, ascending, 1; native(SCROLLS_4_5) <- value(257));
        battle!(game.MinimapDisabled = 0);
        session_slot!(game, Crocomire[0x40i32].base.properties, |old| old
            | (0x200u16));

        game.foreign(|game| game.spawn_plm(0x4e, WALL_ROW, CLEAR_INVISIBLE_WALL));
        battle!(game.CameraDistance = 0);
        battle!(game.MeltEraseTarget = 0);
    }
}

/// Crocomire main AI - death sequence index 2 - falling ($A4:9136). Was `emit_melt_debris`.
pub fn falling<L: Lease + ?Sized>(game: &mut L) {
    let v0: u16 = u16::from_ne_bytes([
        game.slot_byte(CRUMBLING_BRIDGE_INDEX),
        game.slot_byte(CRUMBLING_BRIDGE_INDEX + 1),
    ]);
    if (u16::from_ne_bytes([
        game.slot_byte(CRUMBLING_BRIDGE_INDEX),
        game.slot_byte(CRUMBLING_BRIDGE_INDEX + 1),
    ]) as i32
        - 22i32)
        & 0x8000
        != 0
    {
        let v1: u16 = u16::from_ne_bytes([
            game.slot_byte(CRUMBLING_BRIDGE_INDEX),
            game.slot_byte(CRUMBLING_BRIDGE_INDEX + 1),
        ]);
        checked_word!(game, CRUMBLING_BRIDGE_INDEX; += 2);
        let parameter = game
            .foreign(|game| game.crocomire_table_word(BRIDGE_DEBRIS_ADDRESS, (v0 >> 1) as isize));
        game.spawn_projectile(parameter, v1, ADDR_STRU_868F9D as u16);
    }
    hop_1_2_sinking(game);
}

/// Crocomire main AI - death sequence index 3Eh - behind wall - wait for Samus ($A4:97D3). Was
/// `release_skeleton_room`.
pub fn behind_wall_wait_for_samus<L: Lease + ?Sized>(game: &mut L) {
    if (game.player_x() as i16).wrapping_sub(640) >= 0 {
        return;
    }
    game.foreign(|game| game.queue_music_delayed(5));
    game.set_slot_word(SCROLLS_3_4, 256);
    game.foreign(|game| game.spawn_plm(0x30, WALL_ROW, CREATE_INVISIBLE_WALL));
    battle!(game.CameraDistance = 6);
    let runtime = &mut *game;
    // Fixed, disjoint records; no callback or fallible operation between these writes.
    runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        enemy.properties = enemy.properties & 0x7bff | K_ENEMY_PROPS_TANGIBLE as u16;
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
pub fn flowing_down_the_river<L: Lease + ?Sized>(game: &mut L) {
    game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
pub fn breaks_down_wall<L: Lease + ?Sized>(game: &mut L) {
    let runtime = &mut *game;
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
            enemy.instruction = ADDR_K_CROCOMIRE_ILIST_E14A as u16;
            enemy.instruction_delay = 1;
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
    }
}

/// Crocomire main AI - death sequence index 46h - skeleton falls ($A4:9A38). Was `land_skeleton`.
pub fn skeleton_falls<L: Lease + ?Sized>(game: &mut L) {
    let impact = game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
        let runtime = &mut *game;
        let palette = runtime
            .resolve_actor(Key::from_legacy(0), AccessMode::Native)
            .palette;
        runtime.edit_actor(Key::from_legacy(0x40), AccessMode::Native, |enemy| {
            enemy.palette = palette
        });
        runtime.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
            enemy.instruction = ADDR_K_CROCOMIRE_ILIST_E158 as u16;
            enemy.instruction_delay = 1;
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        });
    }
}

/// Crocomire main AI - death sequence index 48h - skeleton falls apart ($A4:9B06). Was
/// `finish_skeleton_escape`.
pub fn skeleton_falls_apart<L: Lease + ?Sized>(game: &mut L) {
    let finished = game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
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
            enemy.instruction = ADDR_K_CROCOMIRE_ILIST_E1CC as u16;
            enemy.instruction_delay = 1;
            enemy.x.whole = enemy.x.whole.wrapping_add(64);
            enemy.y.whole = enemy.y.whole.wrapping_add(21);
            enemy.radius.y = 28;
            enemy.radius.x = 40;
            true
        }
    });
    if finished {
        game.foreign(|game| {
            game.spawn_plm(0x30, WALL_ROW, CLEAR_INVISIBLE_WALL);
            game.scatter_crocomire_drop(0);
        });
        advance_death_sequence(game);
    }
}

/// Crocomire main AI - death sequence index 4Ch - set index 4Eh ($A4:9B7D). Was
/// `await_skeleton_script`.
pub fn set_index_4e<L: Lease + ?Sized>(game: &mut L) {
    game.edit_actor(Key::from_legacy(0), AccessMode::Native, |enemy| {
        if (enemy.instruction as i16).wrapping_add(0x1e3a) >= 0 {
            enemy.variables[lane::DEATH_SEQUENCE_INDEX] =
                enemy.variables[lane::DEATH_SEQUENCE_INDEX].wrapping_add(2);
        }
    });
}

/// Crocomire main AI - death sequence index 4Ah - unlock camera ($A4:9B65). Was
/// `open_skeleton_exit`.
pub fn unlock_camera<L: Lease + ?Sized>(game: &mut L) {
    game.set_slot_word(SCROLLS_0_1, 257);
    game.set_slot_word(SCROLLS_2_3, 257);
    game.foreign(|game| game.spawn_plm(0x1e, WALL_ROW, CLEAR_INVISIBLE_WALL));
    advance_death_sequence(game);
}

pub fn prepare_scroll_hdma<L: Lease + ?Sized>(game: &mut L) {
    let mut v1: u16 =
        (2 * (session_slot!(game, Crocomire[0x40i32].base.y_pos) as i32 - 72i32)) as u16;
    let v2: u16 = display_scroll!(game.Bg2V);
    do_until! { (v1 & 0x8000) != 0 ;
        word!(game.CROCOMIRE_BG2_SCROLL_HDMA_DATA[(v1 >> 1)] = v2);
        v1 = (v1).wrapping_sub(2);
    }
    let e: i32 = game.current_enemy() as i32;
    next_death_step(e, game);
    let v4: i32 = 0;
    session_slot!(
        game,
        Crocomire[0x80i32].step_counter = session_slot!(game, Crocomire[v4].base.x_pos)
    );
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE) <- value(255));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 1) <- value(-13584i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 3) <- value(225));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 4) <- value(-13330i32 as u16));
    crate::word_transfers!(game, ascending, 1; native(SCROLL_HDMA_TABLE + 6) <- value(0));
}

// Returned only for the two synchronous hardware phases of the fixed actor-0 driver.
/// Main AI - Crocomire ($A4:8C04): its phase step, the melting phases' scroll
/// HDMA, Samus's collision and its palette.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    static DEFINITION: crate::menus::prelude::HdmaObjectDefinition =
        crate::menus::prelude::HdmaObjectDefinition::new(0x42, 0x10, 0x9563);
    if let Some(request) = execute_phase(game) {
        game.queue_sound(1, request as u16 as u8, 6);
        prepare_scroll_hdma(game);
        let object = game.foreign(|game| crate::hdma_objects::spawn_hdma(0xa4, &DEFINITION, game));
        use crate::actors::layouts::EnemyLayout72 as Layout;
        session_slot!(game, Layout[0].var_1f = object);
    }
    samus_collision(game);
    update_palette(game);
}

pub enum ScrollRequest {
    Flesh = 0x77,
    Skeleton = 0x2d,
}
/// Main AI - Crocomire ($A4:8C04): run the death sequence function; the two
/// melting HDMA phases are left to the caller.
pub fn execute_phase<L: Lease + ?Sized>(game: &mut L) -> Option<ScrollRequest> {
    let phases: [Option<fn(&mut L)>; 45] = [
        Some(not_started as fn(&mut L)),
        Some(falling as fn(&mut L)),
        Some(hop_1_2_resting as fn(&mut L)),
        Some(hop_1_2_rising as fn(&mut L)),
        Some(hop_1_2_sinking as fn(&mut L)),
        Some(hop_1_2_resting as fn(&mut L)),
        Some(hop_1_2_rising as fn(&mut L)),
        Some(hop_1_2_sinking as fn(&mut L)),
        Some(hop_3_load_melting_tilemap as fn(&mut L)),
        Some(load_melting_tiles as fn(&mut L)),
        Some(upload_melting_tiles as fn(&mut L)),
        Some(hop_3_4_5_rising as fn(&mut L)),
        None,
        Some(melting as fn(&mut L)),
        Some(clear_tilemap as fn(&mut L)),
        Some(hop_3_4_5_sinking as fn(&mut L)),
        Some(hop_4_5_resting as fn(&mut L)),
        Some(hop_3_4_5_rising as fn(&mut L)),
        Some(hop_3_4_5_sinking as fn(&mut L)),
        Some(hop_4_5_resting as fn(&mut L)),
        Some(hop_3_4_5_rising as fn(&mut L)),
        Some(hop_3_4_5_sinking as fn(&mut L)),
        Some(hop_6_load_melting_tilemap as fn(&mut L)),
        Some(load_melting_tiles as fn(&mut L)),
        Some(upload_melting_tiles as fn(&mut L)),
        Some(set_index_34 as fn(&mut L)),
        Some(hop_6_rising as fn(&mut L)),
        None,
        Some(melting as fn(&mut L)),
        Some(clear_tilemap as fn(&mut L)),
        Some(hop_6_sinking as fn(&mut L)),
        Some(behind_wall_wait_for_samus as fn(&mut L)),
        Some(behind_wall_rumbling as fn(&mut L)),
        Some(behind_wall_no_more_rumbling as fn(&mut L)),
        Some(breaks_down_wall as fn(&mut L)),
        Some(skeleton_falls as fn(&mut L)),
        Some(skeleton_falls_apart as fn(&mut L)),
        Some(unlock_camera as fn(&mut L)),
        Some(set_index_4e as fn(&mut L)),
        Some(advance_death_sequence as fn(&mut L)),
        Some(mark_event_and_start_music as fn(&mut L)),
        Some(dead as fn(&mut L)),
        Some(advance_death_sequence as fn(&mut L)),
        Some(dead_from_room_entry as fn(&mut L)),
        Some(flowing_down_the_river as fn(&mut L)),
    ];
    let phase = (session_slot!(game, Crocomire[0i32].death_sequence_index) as i32 >> 1) as usize;
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
fn dead<L: Lease + ?Sized>(_game: &mut L) {}
/// Crocomire main AI - death sequence index 32h - set index 34h ($A4:9576). Was
/// `advance_selected_phase`.
fn set_index_34<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = game.current_enemy() as i32;
    skip_death_step(e, game);
}
/// Crocomire main AI - death sequence index 1Ch/3Ah - hop 3/6 - clear tilemap ($A4:9653). Was
/// `reset_melt_tilemap`.
fn clear_tilemap<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = game.current_enemy() as i32;
    session_slot!(game, Crocomire[e]; y_velocity = 0, projectile_counter = 0);
    session_slot!(game, Crocomire[e].step_counter = 2048);
    bg2_staging::fill(game, (0..=2047).rev(), 824);
    queue_tilemap_upload(2048, 0xfe, game);
    advance_death_sequence(game);
}
/// Crocomire main AI - death sequence index 56h - dead due to room entry ($A4:8C90). Was
/// `reset_scroll`.
fn dead_from_room_entry<L: Lease + ?Sized>(game: &mut L) {
    display_scroll!(game, 0; Bg2H, Bg2V);
}
fn spawn_random_dust<L: Lease + ?Sized>(game: &mut L) {
    let random = battle!(game.Random);
    let mut offset = random & 0x1f;
    if (random as i16).wrapping_sub(4096) >= 0 {
        offset = offset.wrapping_neg();
    }
    spawn_dust(EnemyScript::Stop, offset, game);
}
fn spawn_dust<L: Lease + ?Sized>(jp: EnemyScript, offset: u16, game: &mut L) -> EnemyScript {
    let (x, y) = {
        let random = game.current_random();
        let actor = game.resolve_actor(Key::from_legacy(0), AccessMode::Native);
        // The offset's add carries in from the random add, as the original's does.
        let (x, carry) = adc(random & 7, actor.x.whole, false);
        (
            adc(x, offset, carry).0,
            actor.radius.y.wrapping_add(actor.y.whole).wrapping_sub(16),
        )
    };
    game.place_projectile_spawn(x, y);
    game.spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 0x15);
    jp
}

/// True when blocked.
fn walk<L: Lease + ?Sized>(k: u16, pixels: i32, game: &mut L) -> bool {
    game.move_horizontal(k, pixels << 16, Slopes::Ignore)
}
/// Crocomire main AI - death sequence index 50h - mark event and start post-death music ($A4:9B86).
/// Was `finish_death`.
fn mark_event_and_start_music<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| game.queue_music_delayed(6));
    battle!(game.CameraDistance = 0);
    let area = game.area();
    let bits = game.foreign(|game| game.boss_word(area));
    game.foreign(|game| {
        game.set_boss_word(area, bits | 2);
        game.queue_music_delayed(6);
    });
    spawn_dust(EnemyScript::Stop, 0xfff0, game);
    spawn_dust(EnemyScript::Stop, 0x10, game);
    advance_death_sequence(game);
}

/// Hurt AI - Crocomire ($A4:8687).
pub fn hurt<L: Lease + ?Sized>(game: &mut L) {
    update_background(game.current_enemy(), game);
    update_palette(game);
}
/// Initialisation AI - Crocomire tongue ($A4:F67A).
pub fn initialize_tongue<L: Lease + ?Sized>(game: &mut L) {
    let e: i32 = game.current_enemy() as i32;
    if ({
        let area = game.area();
        game.foreign(|game| game.boss_word_native(area))
    } & 2)
        != 0
    {
        session_slot!(game, Crocomire[e].base.properties, |old| old & 0xdcff
            | 0x300);
    } else {
        session_slot!(
            game,
            Crocomire[e].base.current_instruction = ADDR_K_CROCOMIRE_ILIST_BE56 as u16
        );
        session_slot!(game, Crocomire[e].base.extra_properties, |old| old
            | (0x404u16));
        session_slot!(game, Crocomire[e].base.instruction_timer = 1);
        session_slot!(game, Crocomire[e].death_sequence_index = 23);
        session_slot!(game, Crocomire[e].base.palette_index = 3584);
    };
}
const DUST_OFFSETS: [u16; 13] = [
    0xffe0, 0, 0xfff0, 0x10, 0, 8, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0x40,
];
/// Run one of Crocomire's script instructions (one of [`instruction`]).
pub fn instruction<L: Lease + ?Sized>(
    ea: u32,
    k: u16,
    jp: EnemyScript,
    game: &mut L,
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
            game.set_slot_word(ram::EARTHQUAKE_TYPE, 4);
            game.set_slot_word(ram::EARTHQUAKE_TIMER, 5);
            game.queue_sound(1, SHAKE_SOUND, 6);
        }
        MOVE_LEFT_4PX | MOVE_LEFT_4PX_AND_SPAWN_DUST | MOVE_LEFT_4PX_AND_SPAWN_DUST_2 => {
            let hit = game
                .resolve_actor(Key::from_legacy(0), AccessMode::Native)
                .variables[lane::FIGHT_FLAGS]
                & flag::DAMAGED
                != 0;
            if !hit {
                walk(game.current_enemy(), -4, game);
            }
        }
        MOVE_LEFT_4PX_SPAWN_DUST_AND_HANDLE_SPIKE_WALL => {
            return if walk(game.current_enemy(), -4, game) {
                game.edit_actor(Key::from_legacy(0), AccessMode::Native, |actor| {
                    actor.variables[lane::FIGHT_FUNCTION] =
                        crate::actors::crocomire_attack::fight::BACK_OFF_FROM_SPIKE_WALL;
                });
                EnemyScript::jump(
                    crate::actors::crocomire_attack::instruction_list::BACK_OFF_FROM_SPIKE_WALL,
                )
            } else {
                let random = battle!(game.Random);
                let side = if (random as i16).wrapping_sub(2048) >= 0 {
                    (-32i16) as u16
                } else {
                    32
                };
                spawn_dust(jp, side.wrapping_add(random & 0xf), game)
            };
        }
        MOVE_RIGHT_4PX_IF_ON_SCREEN | MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST => {
            let camera = game.camera();
            let can_walk = {
                let actor = game.resolve_actor(Key::from_legacy(0), AccessMode::Native);
                // The subtracts carry their borrows on, as the original's do ($A4:9067).
                let (edge, carry) = sbc(actor.x.whole, actor.radius.x, true);
                let (edge, carry) = sbc(edge, 0x100, carry);
                (sbc(edge, 4, carry).0.wrapping_sub(camera.layer1[0]) as i16) < 0
            };
            if can_walk {
                walk(game.current_enemy(), 4, game);
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
        _ => {
            game.foreign(|game| game.unreachable());
            return EnemyScript::Stop;
        }
    }
    jp
}
