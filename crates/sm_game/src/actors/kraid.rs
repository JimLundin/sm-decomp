//! Kraid, Brinstar's giant: he rises through the floor, breaks the ceiling,
//! fires lint and fingernails, lunges, walks, opens his mouth to roar and
//! spit rocks, and sinks back through the floor when killed.
//!
//! Was `Kraid_*`/`KraidsFoot_*`/`KraidLint_*`/`KraidsFingernail_*` in the
//! original decompilation (bank $A7:A8F0-$A7:C9EE). Kraid is eight enemies:
//! the body in slot 0, then the arm, the top, middle and bottom lint, the
//! foot, and the good and bad fingernails. Every part runs the bank $A7
//! function in its `function` word; the views below name the slot words over
//! the same RAM bytes as `KraidLayout`. The pause and unpause hooks are in
//! `kraid_pause`.
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::deaths::Deaths as _;
use crate::actors::host::BossEffects as _;
use crate::actors::host::{Death, Shot, Slopes, below};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::session_slot;
use crate::actors::{kraid_fingernails, kraid_lint, kraid_sinking};
use crate::cartridge::native_record;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE, K_ENEMY_PROPS_TANGIBLE,
};
use crate::enemy_projectiles::drops::DropSource;
use crate::kraid_tables;
use crate::palette_effects::fading::background6;
use crate::ram::CompatibilityRam as _;
use crate::ram::{slot, word};
use crate::runtime_access::RuntimeAccess as _;
use sm_data::cartridge::RomRecord;

/// A lease on a game with Kraid's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

/// What Kraid needs beyond the shared boss effects, deaths and lint contact.
pub trait Host:
    crate::actors::host::BossEffects
    + crate::actors::deaths::Deaths
    + kraid_lint::Host
    + crate::records::Records
    + crate::decompression::DecompressionHost
    + BossLease<Game = Self>
{
    /// Queue a VRAM write of Kraid's tilemap row at bank $A7 `source`, as an
    /// entry at queue `tail`, to BG2's tilemap.
    fn queue_kraid_tilemap(&mut self, tail: u16, source: u16);
    /// Whether actor `k` is touching Samus from above.
    fn touches_player_from_above(&mut self, k: u16) -> bool;
    /// The boss word of `area`, in native byte order.
    fn boss_word_native(&mut self, area: u16) -> u16;
    fn set_boss_word_native(&mut self, area: u16, value: u16);
    /// Set the shot AI of the enemy definition at `enemy_ptr`.
    fn set_shot_ai(&mut self, enemy_ptr: u16, shot_ai: u16);
    /// The original "X undefined" debug message.
    fn print_x_undefined(&mut self);
    /// A verifier observation point in the room-cleared fade.
    fn observe_kraid_flags(&mut self, _step: u8) {}
}

/// The body's view of its slot. `timer` also counts the ceiling platforms
/// broken while Kraid grows.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Body {
    pub base: EnemyBase,
    pub function: u16,
    pub script: u16,
    pub script_delay: u16,
    pad_0fae: [u16; 1],
    pub mouth_hit_ready: u16,
    pub timer: u16,
    pad_0fb4: [u8; 26700],
    pub next: u16,
    pub unread_7802: u16,
    pub unread_7804: u16,
    pub thinking_timer: u16,
    pub min_eject_y: u16,
    pub mouth_flags: u16,
    pub health_eighths: [u16; 8],
    pad_781c: [u16; 1],
    pub target_x: u16,
    pub health_quarters: [u16; 4],
    pad_7828: [u16; 1],
    pub hurt_frame: u16,
    pub hurt_frame_timer: u16,
    pub unread_782e: [u16; 9],
}

same_words!(Body: KraidLayout {
    function = kraid_var_a, script = kraid_var_b, script_delay = kraid_var_c,
    mouth_hit_ready = kraid_var_e, timer = kraid_var_f, next = kraid_next, unread_7802 = field_2,
    unread_7804 = field_4, thinking_timer = kraid_thinking, min_eject_y = kraid_min_y_pos_eject,
    mouth_flags = kraid_mouth_flags, health_eighths = kraid_healths_8ths,
    target_x = kraid_target_x, health_quarters = kraid_healths_4ths,
    hurt_frame = kraid_hurt_frame, hurt_frame_timer = kraid_hurt_frame_timer, unread_782e = field_2e,
});

/// A lint's, the arm's or the foot's view of its slot. The foot's `next`
/// also counts down its walk.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Part {
    pub base: EnemyBase,
    pub function: u16,
    pub progress: u16,
    pub x_offset: u16,
    pad_0fae: [u16; 2],
    pub timer: u16,
    pad_0fb4: [u8; 26700],
    pub next: u16,
    pub unread_7802: u16,
    pub unread_7804: u16,
    pub thinking_timer: u16,
    pub min_eject_y: u16,
    pub mouth_flags: u16,
    pub health_eighths: [u16; 8],
    pad_781c: [u16; 1],
    pub target_x: u16,
    pub health_quarters: [u16; 4],
    pad_7828: [u16; 1],
    pub hurt_frame: u16,
    pub hurt_frame_timer: u16,
    pub unread_782e: [u16; 9],
}

same_words!(Part: KraidLayout {
    function = kraid_var_a, progress = kraid_var_b, x_offset = kraid_var_c, timer = kraid_var_f,
    next = kraid_next, unread_7802 = field_2, unread_7804 = field_4, thinking_timer = kraid_thinking,
    min_eject_y = kraid_min_y_pos_eject, mouth_flags = kraid_mouth_flags,
    health_eighths = kraid_healths_8ths, target_x = kraid_target_x,
    health_quarters = kraid_healths_4ths, hurt_frame = kraid_hurt_frame,
    hurt_frame_timer = kraid_hurt_frame_timer, unread_782e = field_2e,
});

/// A fingernail's view of its slot: its 16.16 velocity. Word 1 of
/// `health_eighths` marks a fingernail held back off screen.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Fingernail {
    pub base: EnemyBase,
    pub function: u16,
    pub x_subspeed: u16,
    pub x_speed: u16,
    pub y_subspeed: u16,
    pub y_speed: u16,
    pub timer: u16,
    pub launched: u16,
    pad_0fb6: [u8; 26698],
    pub next: u16,
    pub unread_7802: u16,
    pub unread_7804: u16,
    pub thinking_timer: u16,
    pub min_eject_y: u16,
    pub mouth_flags: u16,
    pub health_eighths: [u16; 8],
    pad_781c: [u16; 1],
    pub target_x: u16,
    pub health_quarters: [u16; 4],
    pad_7828: [u16; 1],
    pub hurt_frame: u16,
    pub hurt_frame_timer: u16,
    pub unread_782e: [u16; 9],
}

same_words!(Fingernail: KraidLayout {
    function = kraid_var_a, x_subspeed = kraid_var_b, x_speed = kraid_var_c,
    y_subspeed = kraid_var_d, y_speed = kraid_var_e, timer = kraid_var_f,
    launched = kraid_parameter_1, next = kraid_next, unread_7802 = field_2, unread_7804 = field_4,
    thinking_timer = kraid_thinking, min_eject_y = kraid_min_y_pos_eject,
    mouth_flags = kraid_mouth_flags, health_eighths = kraid_healths_8ths,
    target_x = kraid_target_x, health_quarters = kraid_healths_4ths,
    hurt_frame = kraid_hurt_frame, hurt_frame_timer = kraid_hurt_frame_timer, unread_782e = field_2e,
});

const BODY: i32 = 0;
const ARM: i32 = 0x40;
const TOP_LINT: i32 = 0x80;
const MIDDLE_LINT: i32 = 0xc0;
const BOTTOM_LINT: i32 = 0x100;
const FOOT: i32 = 0x140;
const GOOD_FINGERNAIL: i32 = 0x180;
const BAD_FINGERNAIL: i32 = 0x1c0;
const BANK: u8 = 0xa7;

/// Bank $A7 functions, stored in each part's `function` word.
pub mod function {
    pub const GETS_BIG_BREAK_CEILING: u16 = 0xac4d;
    pub const GETS_BIG_SET_BG2_PRIORITY: u16 = 0xad3a;
    pub const GETS_BIG_FINISH_BG2_TILEMAP: u16 = 0xad61;
    pub const GETS_BIG_DRAW_ROOM_BG: u16 = 0xad8e;
    pub const GETS_BIG_FADE_IN_ROOM_BG: u16 = 0xae23;
    pub const MAIN_LOOP_THINKING: u16 = 0xaea4;
    pub const GETS_BIG_THINKING: u16 = 0xaec4;
    pub const MOUTH_OPEN: u16 = 0xaee4;
    pub const INIT_EYE_GLOW: u16 = 0xb6bf;
    pub const GLOW_EYE: u16 = 0xb6d7;
    pub const UNGLOW_EYE: u16 = 0xb73d;
    pub const NOTHING: u16 = 0xb831;
    pub const LINT_PRODUCE: u16 = 0xb832;
    pub const LINT_CHARGE: u16 = 0xb868;
    pub const LINT_FIRE: u16 = 0xb89b;
    pub const FINGERNAIL_WAIT_FOR_LINT: u16 = 0xb907;
    pub const ALIGN_TO_KRAID: u16 = 0xb923;
    pub const HANDLE_FUNCTION_TIMER: u16 = 0xb92d;
    pub const DECREMENT_FUNCTION_TIMER: u16 = 0xb93f;
    pub const FOOT_FIRST_PHASE_THINKING: u16 = 0xb960;
    pub const PROCESS_SCRIPT_AND_TIMER: u16 = 0xb965;
    pub const FOOT_IDLE: u16 = 0xba2d;
    pub const FOOT_SECOND_PHASE_THINKING: u16 = 0xba2e;
    pub const FOOT_WALK_BACKWARDS: u16 = 0xbb45;
    pub const FOOT_WALK_TO_START: u16 = 0xbb6e;
    pub const FOOT_SECOND_PHASE_INIT: u16 = 0xbba4;
    pub const FOOT_WALK_FORWARD: u16 = 0xbbae;
    pub const ATTACK_WITH_MOUTH_OPEN: u16 = 0xbbea;
    pub const FINGERNAIL_INIT: u16 = 0xbd60;
    pub const FINGERNAIL_FIRE: u16 = 0xbe8e;
    pub const FOOT_PREPARE_LUNGE: u16 = 0xbf2d;
    pub const FOOT_LUNGE_FORWARD: u16 = 0xbf5d;
    pub const FOOT_RETREAT_FROM_LUNGE: u16 = 0xbfab;
    pub const GETS_BIG_RELEASE_CAMERA: u16 = 0xc0a1;
    pub const DEATH_INIT: u16 = 0xc360;
    pub const DEATH_FADE_OUT: u16 = 0xc3f9;
    pub const DEATH_BG2_TOP_HALF: u16 = 0xc4a4;
    pub const DEATH_BG2_BOTTOM_HALF: u16 = 0xc4c8;
    pub const DEATH_SINK_THROUGH_FLOOR: u16 = 0xc537;
    pub const FADE_IN_CLEAR_BG2_TOP: u16 = 0xc715;
    pub const FADE_IN_CLEAR_BG2_BOTTOM: u16 = 0xc751;
    pub const FADE_IN_LOAD_BG3_TILES_1: u16 = 0xc777;
    pub const FADE_IN_LOAD_BG3_TILES_2: u16 = 0xc7a3;
    pub const FADE_IN_LOAD_BG3_TILES_3: u16 = 0xc7c9;
    pub const FADE_IN_LOAD_BG3_TILES_4: u16 = 0xc7ef;
    pub const FADE_IN_BG6: u16 = 0xc815;
    pub const FADE_IN_DEAD_WAS_ALIVE: u16 = 0xc843;
    pub const FADE_IN_DEAD_WAS_DEAD: u16 = 0xc851;
    pub const RESTRICT_SAMUS_X: u16 = 0xc865;
    pub const RAISE_THROUGH_FLOOR: u16 = 0xc86b;
    pub const RAISE_LOAD_TILEMAP_BOTTOM: u16 = 0xc89a;
    pub const RAISE_QUAKE_EVERY_16: u16 = 0xc8e0;
    pub const RAISE_QUAKE_EVERY_8: u16 = 0xc902;
    pub const RAISE: u16 = 0xc924;
}

/// The ceiling and death crumble callbacks of `kraid_sinking`.
mod sinking {
    pub const LEFT_PLATFORM_LEFT: u32 = 0xa7_c691;
    pub const NOTHING: u32 = 0xa7_c6a6;
    pub const RIGHT_PLATFORM_MIDDLE: u32 = 0xa7_c6a7;
    pub const RIGHT_PLATFORM_LEFT: u32 = 0xa7_c6bd;
    pub const LEFT_PLATFORM_RIGHT: u32 = 0xa7_c6d3;
    pub const LEFT_PLATFORM_MIDDLE: u32 = 0xa7_c6e9;
    pub const RIGHT_PLATFORM_RIGHT: u32 = 0xa7_c6ff;
}

/// Bank $A7 script instructions: a word with bit 15 set calls one of these.
mod script_call {
    pub const ROAR: u16 = 0xaf94;
    pub const DYING_CRY: u16 = 0xaf9f;
}

/// Bank $A7 instruction lists, spritemaps and scripts.
pub mod animation {
    pub const FOOT_IDLE: u16 = 0x86e7;
    pub const FOOT_STAND: u16 = 0x86ed;
    pub const FOOT_WALK_FORWARD: u16 = 0x86f3;
    pub const FOOT_WALK_FORWARD_END: u16 = 0x87bb;
    pub const FOOT_LUNGE: u16 = 0x87bd;
    pub const FOOT_WALK_BACKWARD_END: u16 = 0x8885;
    pub const FOOT_WALK_BACKWARD: u16 = 0x8887;
    /// The last step of the backward walk; the foot waits for its list to reach it.
    pub const FOOT_WALK_BACKWARD_LAST: u16 = 0x8939;
    pub const ARM_IDLE: u16 = 0x89f3;
    /// The idle arm's last frame, before the lunge.
    pub const ARM_IDLE_LAST: u16 = 0x8a37;
    pub const ARM_SWIPE: u16 = 0x8a41;
    pub const ARM_RAISED: u16 = 0x8aa4;
    pub const ARM_DYING: u16 = 0x8af0;
    pub const LINT_HIDDEN: u16 = 0x8afe;
    pub const LINT_SHOWN: u16 = 0x8b04;
    pub const FINGERNAIL: u16 = 0x8b0a;
    pub const LINT_SPRITEMAP: u16 = 0x8c6c;
    pub const LINT_HIDDEN_SPRITEMAP: u16 = 0xa5df;
    /// The body's scripts: idle, eye glow and dying.
    pub const SCRIPT_IDLE: u16 = 0x96da;
    pub const SCRIPT_IDLE_FIRST: u16 = 0x96d2;
    /// A fatal mouth hit with the script before this advances it 60 bytes.
    pub const SCRIPT_IDLE_END: u16 = 0x970e;
    pub const SCRIPT_GLOW: u16 = 0x974a;
    pub const SCRIPT_DYING: u16 = 0x9764;
    /// Tilemaps of the mouth opening, by how far open.
    pub const TILEMAP_0: u16 = 0x97c8;
    pub const TILEMAP_1: u16 = 0x9ac8;
    pub const TILEMAP_2: u16 = 0x9dc8;
    pub const TILEMAP_3: u16 = 0xa0c8;
    pub const ROOM_BG: u16 = 0xa716;
}

/// Bank $A7 palettes.
const ROOM_BG_PALETTE: u32 = 0xa7_86c7;
const DEATH_PALETTE: u32 = 0xa7_b4f3;
const RISING_PALETTE: u32 = 0xa7_aaa6;
/// Bank $B9 compressed BG2 graphics and tilemap.
const BG2_TILES: u32 = 0xb9_fa38;
const BG2_TILEMAP: u32 = 0xb9_fe3e;
/// Bank $86 enemy projectiles.
const DUST_CLOUD: u16 = 0xe509;
const SPIT_ROCKS: u16 = 0x9c45;
const CEILING_ROCKS: u16 = 0x9c53;
const RISING_ROCKS_LEFT: u16 = 0x9c61;
const RISING_ROCKS_RIGHT: u16 = 0x9c6f;
/// Bank $84 PLMs: ceiling pieces, the spikes and the cleared ceiling.
const CLEAR_CEILING_PLM: u16 = 0xb7b7;
const CLEAR_SPIKES_PLM: u16 = 0xb7bb;
const DEATH_SPIKES_PLM: u16 = 0xb7bf;
const CEILING_PLM_A3: u16 = 0xb7a3;
const CEILING_PLM_A7: u16 = 0xb7a7;
const CEILING_PLM_AB: u16 = 0xb7ab;
const CEILING_PLM_AF: u16 = 0xb7af;
const CEILING_PLM_B3: u16 = 0xb7b3;
/// The ceiling pieces Kraid breaks as he grows: block column and PLM.
const CEILING_PLATFORMS: [(u8, u16); 9] = [
    (0x6, CEILING_PLM_B3),
    (0xd, CEILING_PLM_AB),
    (0x2, CEILING_PLM_A3),
    (0xa, CEILING_PLM_B3),
    (0x5, CEILING_PLM_AB),
    (0xc, CEILING_PLM_B3),
    (0x3, CEILING_PLM_AB),
    (0xb, CEILING_PLM_AB),
    (0x4, CEILING_PLM_B3),
];
/// The empty shot AI written into Kraid's definition once he is dead.
pub const NO_SHOT_AI: u16 = 0x804c;
/// The pause and unpause hooks, as bank $A7 addresses.
const UNPAUSE_ALIVE: u16 = 0xc24e;
const UNPAUSE_SINKING: u16 = 0xc2a0;
const UNPAUSE_DEAD: u16 = 0xc1fb;
const PAUSE: u16 = 0xc325;
/// RAM: the pause and unpause hook pointers, the BG tilemap base mirrors,
/// the camera scroll limits and the blank BG2 tile.
const UNPAUSE_HOOK: usize = 0x604;
const PAUSE_HOOK: usize = 0x601;
const BG2_TILEMAP_BASE: usize = 0x59;
const BG3_TILEMAP_BASE: usize = 0x5d;
const SCROLL_LIMITS: usize = 0xcd20;
const VRAM_QUEUE: i32 = 0xd0;
const BLANK_BG2_TILE: u16 = 824;
/// The lint fly left at 3.5 pixels a frame; the foot moves 3 or 4.
const LINT_SPEED: i32 = (3 << 16) | 0x8000;
const FOOT_STEP_BACK: u16 = 3;
const FOOT_STEP_FORWARD: u16 = 3;
const PUSH_RIGHT: u16 = 4;
/// The lint's three heights' fire timers when Kraid gets big.
const LINT_TIMERS: [u16; 3] = [0x120, 0xa0, 0x40];
/// Library sounds.
const ROAR_SOUND: u8 = 0x2d;
const DYING_CRY_SOUND: u8 = 0x2e;
const EXPLOSION_SOUND: u8 = 0x3d;
const LINT_SOUND: u8 = 0x1f;
const ROCKS_SOUND: u8 = 0x1e;
const STOMP_SOUND: u8 = 0x76;

fn current<S: BossState + ?Sized>(game: &S) -> i32 {
    game.current_enemy() as i32
}

fn kraid_record<S: BossState + ?Sized>(address: u16, game: &mut S) -> RomRecord {
    native_record(BANK, address, game)
}

fn set_native_word<S: BossState + ?Sized>(game: &mut S, at: usize, value: u16) {
    let bytes = u16::to_ne_bytes(value);
    game.set_slot_byte(at, bytes[0]);
    game.set_slot_byte(at + 1, bytes[1]);
}

fn native_word<S: BossState + ?Sized>(game: &S, at: usize) -> u16 {
    u16::from_ne_bytes([game.slot_byte(at), game.slot_byte(at + 1)])
}

/// Health word `index` of the body's eighths, at byte `slot` past the body.
fn health_eighth<S: BossState + ?Sized>(game: &S, slot: i32, index: usize) -> u16 {
    native_word(game, slot!(@at Body, slot, health_eighths) + index * 2)
}

fn set_health_eighth<S: BossState + ?Sized>(game: &mut S, slot: i32, index: usize, value: u16) {
    let at = slot!(@at Body, slot, health_eighths) + index * 2;
    set_native_word(game, at, value);
}

/// How a VRAM queue entry's source bank is stored.
enum QueueBank {
    Byte(u8),
    /// A word, which the destination's low byte then overwrites.
    Word(u16),
}

/// Appends a VRAM write queue entry: `size` bytes from `bank:source` to the
/// destination `destination` computes after the bank store.
fn queue_vram<L: Lease + ?Sized>(
    game: &mut L,
    size: u16,
    source: u16,
    bank: QueueBank,
    destination: impl FnOnce(&L) -> u16,
) {
    let tail: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
    let entry = VRAM_QUEUE + word!(game.VRAM_WRITE_QUEUE_TAIL) as i32;
    let at =
        |offset: isize| usize::try_from(entry as isize + offset).expect("VRAM entry outside RAM");
    set_native_word(game, at(0), size);
    set_native_word(game, at(2), source);
    match bank {
        QueueBank::Byte(value) => game.set_slot_byte(at(4), value),
        QueueBank::Word(value) => set_native_word(game, at(4), value),
    }
    let destination = destination(game);
    set_native_word(game, at(5), destination);
    word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(7));
}

/// The room BG through the BG3 tilemap: Kraid's room background.
fn queue_room_bg<L: Lease + ?Sized>(game: &mut L) {
    queue_vram(
        game,
        512,
        animation::ROOM_BG,
        QueueBank::Byte(BANK),
        |game| (((game.slot_byte(BG3_TILEMAP_BASE) as i32 & 0xf) << 8) + 16128) as u16,
    );
}

/// Contact with the arm or foot. Was `Kraid_Touch_ArmFoot`.
pub fn touch_arm_or_foot<L: Lease + ?Sized>(game: &mut L) {
    game.contact_reaction(Death::Finalize);
}

/// Pushes Samus up and away after a contact. Was `Kraid_Func_1`.
fn push_samus<S: BossState + ?Sized>(game: &mut S) {
    word!(game.EXTRA_SAMUS_X_DISPLACEMENT = PUSH_RIGHT);
    word!(game.EXTRA_SAMUS_Y_DISPLACEMENT = -8i32 as u16);
}

/// The arm is shot: the projectile explodes. Was `Kraid_Arm_Shot`.
pub fn arm_shot<L: Lease + ?Sized>(game: &mut L, j: u16) {
    projectile_explosion(game, j);
    word!(game.PROJECTILE_DIR[(j >> 1)] |= 0x10);
}

/// Whether Kraid's boss bit is set for this area. Was `Kraid_CheckIfDead`.
pub fn is_dead<L: Lease + ?Sized>(game: &mut L) -> u16 {
    let area = word!(game.AREA_INDEX);
    ((game.foreign(|game| game.boss_word_native(area)) & 1) != 0) as u16
}

/// Was `Kraid_SetEnemyPropsToDead`.
fn hide_as_dead<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    session_slot!(game, Part[me].base.properties, |old| old & 0x50ff | 0x700);
}

/// Decompresses BG2's graphics and tilemap and sets the tilemap's priority
/// with `priority`. Was `Kraid_SetupGfxWithTilePrioClear`.
fn load_bg2<S: BossState + ?Sized>(game: &mut S, priority: u16) {
    game.decompress_into_ram(BG2_TILES, 0x4000);
    game.decompress_into_ram(BG2_TILEMAP, 0x2000);
    session_slot!(game, Body[BODY]; hurt_frame = 0, hurt_frame_timer = 0);
    for v1 in (0u16..1536).step_by(2) {
        let value = (word!(game.TILEMAP_STUFF[(v1 >> 1)])) & 0xdfff;
        let at = usize::try_from(0x2000 + (((v1 as i32 >> 1) + 1024) as isize) * 2)
            .expect("native word outside RAM");
        set_native_word(game, at, value);
    }
    for v2 in 0u16..1024 {
        word!(game.TILEMAP_STUFF[v2] = priority & native_word(game, 0x4000 + v2 as usize * 2));
    }
}

/// Fades in the room background palette and queues its tilemap. Was
/// `Kraid_DrawRoomBg`.
fn draw_room_bg<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        crate::word_transfers!(game, ascending, 16; le(TARGET_PALETTES, 96) <- rom(ROOM_BG_PALETTE, 0));
    });
    word!(game.PALETTE_CHANGE_NUM = 0);
    queue_room_bg(game);
}

/// Was `Kraid_GetsBig_Thinking_Setup`.
fn start_growing_thinking<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].function = function::GETS_BIG_THINKING);
    place_lint_and_think(game);
}

/// Was `Kraid_Mainloop_Thinking_Setup`.
fn start_thinking<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].function = function::MAIN_LOOP_THINKING);
    place_lint_and_think(game);
}

/// Starts lint `k` waiting `timer` frames to fire. Was `Kraid_Lints_Enable`.
fn enable_lint<L: Lease + ?Sized>(game: &mut L, k: u16, timer: u16) {
    let me = k as i32;
    session_slot!(game, Part[me].timer = timer);
    session_slot!(game, Part[me].function = function::ALIGN_TO_KRAID);
    session_slot!(game, Part[me].next = function::LINT_PRODUCE);
    session_slot!(game, Part[me].progress = 0);
}

/// Counts down the thinking timer, then opens the mouth. Was
/// `advance_kraid_thinking_timer!`.
fn advance_thinking<L: Lease + ?Sized>(game: &mut L, next: u16) {
    let thinking: u16 = session_slot!(game, Body[BODY].thinking_timer);
    if thinking != 0 {
        let left: u16 = thinking.wrapping_sub(1);
        session_slot!(game, Body[BODY].thinking_timer = left);
        if left == 0 {
            session_slot!(game, Body[BODY].function = next);
            session_slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
            session_slot!(
                game,
                Body[BODY].script_delay = kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(0)
            );
        }
    }
}

/// After the mouth closes: a hard hit makes the eye glow. Was
/// `advance_kraid_mouth_hit_response!`.
fn respond_to_mouth_hit<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].script_delay = 90);
    let flags: u16 = session_slot!(game, Body[BODY].mouth_flags);
    if (flags & 4) != 0 && {
        let left: u16 = flags.wrapping_sub(256);
        session_slot!(game, Body[BODY].mouth_flags = left);
        (left & 0xff00) != 0
    } {
        session_slot!(game, Body[BODY].function = function::HANDLE_FUNCTION_TIMER);
        session_slot!(game, Body[BODY].timer = 64);
        session_slot!(game, Body[BODY].next = function::INIT_EYE_GLOW);
        session_slot!(game, Body[BODY].unread_7802 = 2);
    } else {
        session_slot!(game, Body[BODY].mouth_flags = 0);
    }
}

/// Steps the body's script; returns its last instruction word, FFFFh at the
/// end. Was `Kraid_ProcessKraidInstr`.
fn process_script<L: Lease + ?Sized>(game: &mut L) -> u16 {
    let result: u16 = session_slot!(game, Body[BODY].script_delay);
    if result != 0 {
        let delay = session_slot!(game, Body[BODY].script_delay);
        session_slot!(game, Body[BODY].script_delay, |old| old.wrapping_sub(1));
        if delay == 1 {
            return execute_script(game);
        }
    }
    result
}

/// Runs script calls up to the next tilemap, which it queues. Was
/// `Kraid_ExecuteInstr`.
fn execute_script<L: Lease + ?Sized>(game: &mut L) -> u16 {
    loop {
        let script: u16 = session_slot!(game, Body[BODY].script);
        let record = kraid_record(script, game);
        let result: u16 = record.word(0);
        if record.word(0) != 0xffff {
            if (record.word(0) as i16).wrapping_add(1) < 0 {
                match result {
                    script_call::ROAR => game.queue_sound(1, ROAR_SOUND, 6),
                    script_call::DYING_CRY => game.queue_sound(1, DYING_CRY_SOUND, 15),
                    _ => game.foreign(|game| game.unreachable()),
                }
                session_slot!(game, Body[BODY].script, |old| old.wrapping_add(2));
            } else {
                session_slot!(game, Body[BODY].script_delay = result);
                session_slot!(game, Body[BODY].script = script.wrapping_add(8));
                let tilemap: u16 = record.word(2);
                let tail: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
                let _size_tail: u16 = word!(game.VRAM_WRITE_QUEUE_TAIL);
                game.foreign(|game| game.queue_kraid_tilemap(tail, tilemap));
                return 1;
            }
        } else {
            return result;
        }
    }
}

/// Shots into the open mouth hurt Kraid; a fatal one starts his death. Was
/// `Kraid_Shot_Mouth`.
fn mouth_shots<S: BossState + ?Sized>(game: &mut S) {
    // Nothing once the death has reached the sinking (C537h and later).
    if !below(
        session_slot!(game, Body[BODY].function),
        function::DEATH_SINK_THROUGH_FLOOR,
    ) {
        return;
    }
    let mut hit = false;
    let frame = kraid_record(
        (session_slot!(game, Body[BODY].script)).wrapping_sub(8),
        game,
    );
    if frame.word(6) != 0xffff {
        let hitbox_address: u16 = frame.word(6);
        session_slot!(game, Body[BODY].mouth_hit_ready = 1);
        let hitbox = kraid_record(hitbox_address, game);
        let right: u16 = (session_slot!(game, Body[BODY].base.x_pos)).wrapping_add(hitbox.word(0));
        let top: u16 = (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(2));
        let bottom: u16 = (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(6));
        if word!(game.PROJECTILE_COUNTER) != 0 {
            let mut index: u16 = 2u16.wrapping_mul(word!(game.PROJECTILE_COUNTER));
            loop {
                let p: i32 = index as i32 >> 1;
                if !((word!(game.PROJECTILE_Y_POS[p]) as i32
                    - word!(game.PROJECTILE_Y_RADIUS[p]) as i32
                    - 1i32
                    - bottom as i32)
                    & 0x8000
                    == 0
                    || (word!(game.PROJECTILE_Y_RADIUS[p]) as i32
                        + word!(game.PROJECTILE_Y_POS[p]) as i32
                        - top as i32)
                        & 0x8000
                        != 0
                    || (word!(game.PROJECTILE_X_RADIUS[p]) as i32
                        + word!(game.PROJECTILE_X_POS[p]) as i32
                        - right as i32)
                        & 0x8000
                        != 0)
                {
                    let kind: u16 = word!(game.PROJECTILE_TYPE[p]);
                    let skip = (kind & 0xf00) == 0
                        && ((kind & 0x10) == 0 || {
                            session_slot!(
                                game,
                                Body[BODY].mouth_flags =
                                    (session_slot!(game, Body[BODY].mouth_flags)) | 1
                            );
                            false
                        });
                    if !skip {
                        word!(game.COLLISION_DETECTION_INDEX = index >> 1);
                        game.react_to_shot(Shot::ReactionOnly);
                        word!(game.PROJECTILE_DIR[(index >> 1)] |= 0x10);
                        hit = true;
                        index = 0;
                    }
                }
                index = index.wrapping_sub(2);
                if (index & 0x8000) != 0 {
                    break;
                }
            }
        }
    }
    if hit {
        session_slot!(game, Body[BODY].hurt_frame = 6);
        session_slot!(game, Body[BODY].hurt_frame_timer = 2);
        let flags: u16 = session_slot!(game, Body[BODY].mouth_flags);
        if (flags & 2) != 0 {
            session_slot!(game, Body[BODY].mouth_flags = flags | 4);
        }
        // Out of health and not yet dying (before C360h).
        if below(session_slot!(game, Body[BODY].base.health), 1)
            && below(
                session_slot!(game, Body[BODY].function),
                function::DEATH_INIT,
            )
        {
            session_slot!(game, Body[BODY].function = function::DEATH_INIT);
            session_slot!(game, Body[BODY].mouth_flags = 0);
            session_slot!(
                game,
                Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties))
                    | (K_ENEMY_PROPS_TANGIBLE as u16)
            );
            load_bg2(game, !(0x2000i32) as u16);
            for part in (0u16..384).step_by(64) {
                let p = part as i32;
                session_slot!(
                    game,
                    Part[p].base.properties = (session_slot!(game, Part[p].base.properties))
                        | (K_ENEMY_PROPS_TANGIBLE as u16)
                );
            }
            if below(
                session_slot!(game, Body[BODY].script),
                animation::SCRIPT_IDLE_END,
            ) {
                session_slot!(game, Body[BODY].script, |old| old.wrapping_add(60));
            }
        }
    }
}

/// A dust cloud where projectile `k` struck. Was `Kraid_SpawnExplosionEproj`.
fn projectile_explosion<S: BossState + ?Sized>(game: &mut S, k: u16) {
    let p: i32 = k as i32 >> 1;
    let (x, y) = (
        word!(game.PROJECTILE_X_POS[p]),
        word!(game.PROJECTILE_Y_POS[p]),
    );
    game.place_projectile_spawn(x, y);
    let kind: u16 = (if (word!(game.PROJECTILE_TYPE[p]) & 0x200) == 0 {
        6
    } else {
        29
    }) as u16;
    game.spawn_room_eproj(DUST_CLOUD, kind);
    game.queue_sound(0, EXPLOSION_SOUND, 6);
}

/// Was `Kraid_HurtFlash_Handling`.
fn hurt_flash<S: BossState + ?Sized>(game: &mut S) {
    let mut source: u16 = 0;
    if (session_slot!(game, Body[BODY].hurt_frame) & 1) == 0 {
        source = 32;
    }
    let mut colour: u16 = 0;
    loop {
        let value = kraid_tables::sprite_color(game, (source >> 1) as usize);
        let at = usize::try_from(0xc000 + (((colour as i32 >> 1) + 240) as isize) * 2)
            .expect("native word outside RAM");
        set_native_word(game, at, value);
        colour = colour.wrapping_add(2);
        source = source.wrapping_add(2);
        if (colour as i16).wrapping_sub(32) >= 0 {
            break;
        }
    }
}

/// The BG and sprite palettes for the health remaining. Was
/// `Kraid_HealthBasedPaletteHandling`.
fn health_palette<S: BossState + ?Sized>(game: &mut S) {
    let mut source: u16 = 0;
    if (session_slot!(game, Body[BODY].hurt_frame) & 1) == 0 {
        let mut eighth: u16 = 14;
        let health: u16 = session_slot!(game, Body[BODY].base.health);
        while (health as i16).wrapping_sub(health_eighth(game, eighth as i32, 0) as i16) < 0 {
            eighth = eighth.wrapping_sub(2);
            if eighth == 0 {
                break;
            }
        }
        source = (16 * (eighth as i32 + 2)) as u16;
    }
    for colour in (0u16..32).step_by(2) {
        let s: i32 = source as i32 >> 1;
        let c: i32 = colour as i32 >> 1;
        word!(game.PALETTE_BUFFER[(c + 112)] = kraid_tables::bg_color(game, s as usize));
        word!(game.PALETTE_BUFFER[(c + 240)] = kraid_tables::sprite_color(game, s as usize));
        source = source.wrapping_add(2);
    }
}

/// A lint's main: Samus contact, then its function. Was `KraidLintCommon_Main`.
fn lint_main<L: Lease + ?Sized>(game: &mut L, k: u16) {
    game.foreign(|game| kraid_lint::lint_touch_samus(k, game));
    run_function(game, session_slot!(game, Part[k as i32].function));
}

/// The lint slides out of Kraid's belly. Was `KraidLint_ProduceLint`.
fn produce_lint<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    session_slot!(game, Part[me].base.properties, |old| old & 0xfaff);
    session_slot!(
        game,
        Part[me].base.x_pos = (session_slot!(game, Part[me].x_offset) as i32
            + session_slot!(game, Part[BODY].base.x_pos) as i32
            - session_slot!(game, Part[me].progress) as i32) as u16
    );
    let progress: u16 = (session_slot!(game, Part[me].progress)).wrapping_add(1);
    session_slot!(game, Part[me].progress = progress);
    if !below(progress, 32) {
        session_slot!(game, Part[me].function = function::LINT_CHARGE);
        session_slot!(game, Part[me].timer = 30);
    }
}

/// The lint flashes while it charges. Was `KraidLint_ChargeLint`.
fn charge_lint<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    let mut palette: i16 = 0;
    if (session_slot!(game, Part[me].timer) & 1) != 0 {
        palette = 3584;
    }
    session_slot!(game, Part[me].base.palette_index = palette as u16);
    session_slot!(
        game,
        Part[me].base.x_pos = (session_slot!(game, Part[me].x_offset) as i32
            + session_slot!(game, Part[BODY].base.x_pos) as i32
            - session_slot!(game, Part[me].progress) as i32) as u16
    );
    let left = session_slot!(game, Part[me].timer);
    session_slot!(game, Part[me].timer, |old| old.wrapping_sub(1));
    if left == 1 {
        session_slot!(game, Part[me].function = function::LINT_FIRE);
        game.queue_sound(2, LINT_SOUND, 6);
    }
}

/// The lint flies left, carrying Samus if she stands on it. Was
/// `KraidLint_FireLint`.
fn fire_lint<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    session_slot!(game, Part[me].base(x_pos, x_subpos) += -LINT_SPEED as u32);
    if below(session_slot!(game, Part[me].base.x_pos), 56) {
        session_slot!(
            game,
            Part[me].base.properties =
                (session_slot!(game, Part[me].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16)
        );
    }
    if below(session_slot!(game, Part[me].base.x_pos), 32) {
        session_slot!(
            game,
            Part[me].base.properties =
                (session_slot!(game, Part[me].base.properties)) | (K_ENEMY_PROPS_INVISIBLE as u16)
        );
        session_slot!(game, Part[me].function = function::ALIGN_TO_KRAID);
        session_slot!(game, Part[me].timer = 300);
        session_slot!(game, Part[me].next = function::LINT_PRODUCE);
        session_slot!(game, Part[me].progress = 0);
    }
    if 0u16.wrapping_sub(u16::from(
        game.foreign(|game| game.touches_player_from_above(k)),
    )) != 0
    {
        let low = word!(game.EXTRA_SAMUS_X_SUBDISPLACEMENT) as u32;
        let high = word!(game.EXTRA_SAMUS_X_DISPLACEMENT) as u32;
        let sum = (low | (high << 16)).wrapping_add(-LINT_SPEED as u32);
        word!(game.EXTRA_SAMUS_X_SUBDISPLACEMENT = sum as u16);
        word!(game.EXTRA_SAMUS_X_DISPLACEMENT = (sum >> 16) as u16);
        if (word!(game.EXTRA_SAMUS_X_DISPLACEMENT) as i32 + 16) & 0x8000 != 0 {
            word!(game.EXTRA_SAMUS_X_DISPLACEMENT = -16i32 as u16);
        }
    }
}

/// A fingernail waits for the top lint to clear the screen's left. Was
/// `KraidFingernail_WaitForLintXpos`.
fn fingernail_wait_for_lint<L: Lease + ?Sized>(game: &mut L, k: u16) {
    if !below(session_slot!(game, Part[TOP_LINT].base.x_pos), 256) {
        let me = k as i32;
        session_slot!(game, Part[me].function = session_slot!(game, Part[me].next));
        session_slot!(game, Part[me].base.properties, |old| old & 0xfaff);
    }
}

/// Counts down the part's timer, then runs its `next` function. Was
/// `KraidEnemy_HandleFunctionTimer`.
fn handle_function_timer<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    if session_slot!(game, Part[me].timer) != 0 && {
        let left = session_slot!(game, Part[me].timer);
        session_slot!(game, Part[me].timer, |old| old.wrapping_sub(1));
        left == 1
    } {
        session_slot!(game, Part[me].function = session_slot!(game, Part[me].next));
    }
}

/// Keeps a lint at Kraid's belly. Was `Kraid_AlignEnemyToKraid`.
fn align_to_kraid<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let x: u16 = session_slot!(game, Part[BODY].base.x_pos);
    let me = k as i32;
    session_slot!(
        game,
        Part[me].base.x_pos = x.wrapping_sub(session_slot!(game, Part[me].base.x_width))
    );
    handle_function_timer(game, k);
}

/// Was `Kraid_SetWalkingBackwards`.
fn walk_backwards<L: Lease + ?Sized>(game: &mut L, frames: u16, target: u16) {
    session_slot!(game, Body[BODY].target_x = target);
    session_slot!(game, Part[FOOT].next = frames);
    session_slot!(game, Part[FOOT].function = function::FOOT_WALK_BACKWARDS);
    session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_BACKWARD);
}

/// Was `Kraid_SetWalkingForwards`.
fn walk_forwards<L: Lease + ?Sized>(game: &mut L, frames: u16, target: u16) {
    session_slot!(game, Body[BODY].target_x = target);
    session_slot!(game, Part[FOOT].next = frames);
    session_slot!(game, Part[FOOT].function = function::FOOT_WALK_FORWARD);
    session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_FORWARD);
}

/// The fingernail flies, bouncing off walls and Kraid. Was
/// `KraidsFingernail_Fire`.
fn fire_fingernail<L: Lease + ?Sized>(game: &mut L, k: u16) {
    let me = k as i32;
    let x_speed = (((session_slot!(game, Fingernail[me].x_speed) as u32) << 16)
        | session_slot!(game, Fingernail[me].x_subspeed) as u32) as i32;
    if game.move_horizontal(k, x_speed, Slopes::Ignore) {
        session_slot!(
            game,
            Fingernail[me].x_subspeed =
                -(session_slot!(game, Fingernail[me].x_subspeed) as i32) as u16
        );
        session_slot!(
            game,
            Fingernail[me].x_speed = -(session_slot!(game, Fingernail[me].x_speed) as i32) as u16
        );
    } else {
        let mut row: i32 = 0;
        loop {
            if ((kraid_tables::fingernail_boundary(game, (row >> 2) as usize).top as i32
                + session_slot!(game, Fingernail[BODY].base.y_pos) as i32
                - session_slot!(game, Fingernail[me].base.y_pos) as i32) as i16)
                < 0
            {
                break;
            }
            row += 4;
        }
        let left: u16 = (kraid_tables::fingernail_boundary(game, (row >> 2) as usize).left)
            .wrapping_add(session_slot!(game, Fingernail[BODY].base.x_pos));
        if (session_slot!(game, Fingernail[me].base.x_width) as i32
            + session_slot!(game, Fingernail[me].base.x_pos) as i32
            - left as i32)
            & 0x8000
            == 0
            && (session_slot!(game, Fingernail[me].x_speed) & 0x8000) == 0
        {
            session_slot!(
                game,
                Fingernail[me].x_subspeed =
                    -(session_slot!(game, Fingernail[me].x_subspeed) as i32) as u16
            );
            session_slot!(
                game,
                Fingernail[me].x_speed =
                    -(session_slot!(game, Fingernail[me].x_speed) as i32) as u16
            );
        }
    }
    let y_speed = (((session_slot!(game, Fingernail[me].y_speed) as u32) << 16)
        | session_slot!(game, Fingernail[me].y_subspeed) as u32) as i32;
    if game.move_vertical(k, y_speed) {
        let speed = ((session_slot!(game, Fingernail[me].y_speed) as i32) << 16)
            | session_slot!(game, Fingernail[me].y_subspeed) as i32;
        session_slot!(game, Fingernail[me](y_speed, y_subspeed) = -speed as u32);
    }
}

/// Spawns a ceiling rock from bank $A7 `crumble`'s column and its PLM.
fn crumble<L: Lease + ?Sized>(game: &mut L, rock: u16, block_x: u8, plm: u16) {
    let k = game.current_enemy();
    game.spawn_projectile(rock, k, CEILING_ROCKS);
    game.foreign(|game| game.spawn_plm(block_x, 0x12, plm));
}

fn sound_every_half_second<L: Lease + ?Sized>(game: &mut L) {
    word!(
        game.BOSS_DEATH_QUAKE_SOUND_TIMER =
            word!(game.BOSS_DEATH_QUAKE_SOUND_TIMER).wrapping_sub(1)
    );
    if word!(game.BOSS_DEATH_QUAKE_SOUND_TIMER) == 0 {
        game.queue_sound(2, ROCKS_SOUND, 6);
        word!(game.BOSS_DEATH_QUAKE_SOUND_TIMER = 30);
    }
}

/// Crumbles the ceiling platforms as Kraid sinks. Was `Kraid_HandleSinking`.
fn handle_sinking<L: Lease + ?Sized>(game: &mut L) {
    if let Some(callback) = game.foreign(kraid_sinking::handle_sinking) {
        match callback {
            sinking::LEFT_PLATFORM_LEFT => crumble(game, 0x70, 0x7, CEILING_PLM_A7),
            sinking::RIGHT_PLATFORM_MIDDLE => crumble(game, 0xf0, 0xf, CEILING_PLM_A7),
            sinking::RIGHT_PLATFORM_LEFT => crumble(game, 0xe0, 0xe, CEILING_PLM_AF),
            sinking::LEFT_PLATFORM_RIGHT => crumble(game, 0x90, 0x9, CEILING_PLM_A7),
            sinking::LEFT_PLATFORM_MIDDLE => crumble(game, 0x80, 0x8, CEILING_PLM_AF),
            sinking::RIGHT_PLATFORM_RIGHT => crumble(game, 0x100, 0x10, CEILING_PLM_AF),
            sinking::NOTHING => {}
            _ => game.foreign(|game| game.unreachable()),
        }
    }
}

/// Queues BG2's tilemap top half. Was `Kraid_UpdateBg2TilemapTopHalf`.
fn queue_bg2_top<L: Lease + ?Sized>(game: &mut L) {
    queue_vram(game, 2048, 0x2000, QueueBank::Byte(126), |game| {
        ((game.slot_byte(BG2_TILEMAP_BASE) as i32 & 0xfc) << 8) as u16
    });
}

/// Queues BG2's tilemap bottom half. Was `Kraid_UpdateBG2TilemapBottomHalf`.
fn queue_bg2_bottom<L: Lease + ?Sized>(game: &mut L) {
    queue_vram(game, 2048, 0x2800, QueueBank::Byte(126), |game| {
        (((game.slot_byte(BG2_TILEMAP_BASE) as i32 & 0xfc) << 8) + 2048) as u16
    });
}

/// Rocks and dust while Kraid rises. Was `Kraid_SpawnRandomQuakeProjs`.
fn quake_rocks<L: Lease + ?Sized>(game: &mut L) {
    let mut offset: u16 = (word!(game.RANDOM_NUMBER)) & 0x3f;
    if (word!(game.RANDOM_NUMBER) & 2) == 0 {
        offset = !(offset as i32) as u16;
    }
    let x: u16 = (session_slot!(game, Body[BODY].base.x_pos)).wrapping_add(offset);
    let y: u16 = (448 - ((word!(game.RANDOM_NUMBER) & 0x3f00) as i32 >> 8)) as u16;
    game.foreign(|game| game.create_sprite(x, y, 21, 0));
    let rocks: u16 = if (word!(game.RANDOM_NUMBER) & 0x10) != 0 {
        RISING_ROCKS_RIGHT
    } else {
        RISING_ROCKS_LEFT
    };
    let parameter = (word!(game.RANDOM_NUMBER)) & 0x3f0;
    let k = game.current_enemy();
    game.spawn_projectile(parameter, k, rocks);
}

/// Keeps Samus on the first screen while Kraid rises. Was
/// `Kraid_RestrictSamusXtoFirstScreen_2`.
fn restrict_samus_x<L: Lease + ?Sized>(game: &mut L) {
    if ((game.player_x()) as i16).wrapping_sub(256) >= 0 {
        game.set_player_x(256);
        game.set_previous_player_x(256);
    }
}

/// Counts down the body's timer, spawning rocks every `interval + 1` frames.
fn quake_until_timer<L: Lease + ?Sized>(game: &mut L, interval: u16, next: u16, next_timer: u16) {
    restrict_samus_x(game);
    let left: u16 = (session_slot!(game, Body[BODY].timer)).wrapping_sub(1);
    session_slot!(game, Body[BODY].timer = left);
    if left != 0 {
        if (left & interval) == 0 {
            quake_rocks(game);
        }
    } else {
        session_slot!(game, Body[BODY].function = next);
        session_slot!(game, Body[BODY].timer = next_timer);
    }
}

/// Runs a Kraid part's bank $A7 function. Was `CallKraidFunc`.
pub fn run_function<L: Lease + ?Sized>(game: &mut L, function: u16) {
    let k: u16 = game.current_enemy();
    match function {
        function::GETS_BIG_BREAK_CEILING => break_ceiling(game),
        function::GETS_BIG_SET_BG2_PRIORITY => set_bg2_priority(game),
        function::GETS_BIG_FINISH_BG2_TILEMAP => finish_bg2_tilemap(game),
        function::GETS_BIG_FADE_IN_ROOM_BG => fade_in_room_bg(game),
        function::GLOW_EYE => glow_eye(game),
        function::DECREMENT_FUNCTION_TIMER => decrement_function_timer(game),
        function::FOOT_SECOND_PHASE_THINKING => foot_second_phase_thinking(game),
        function::FOOT_SECOND_PHASE_INIT => walk_backwards(game, 0xb4, 0x160),
        function::FINGERNAIL_INIT => initialize_fingernail_attack(game),
        function::FOOT_PREPARE_LUNGE => foot_prepare_lunge(game),
        function::FOOT_LUNGE_FORWARD => foot_lunge_forward(game),
        function::FOOT_RETREAT_FROM_LUNGE => foot_retreat_from_lunge(game),
        function::DEATH_FADE_OUT => death_fade_out(game),
        function::DEATH_BG2_TOP_HALF => death_bg2_top_half(game),
        function::DEATH_BG2_BOTTOM_HALF => death_bg2_bottom_half(game),
        function::DEATH_SINK_THROUGH_FLOOR => sink_through_floor(game),
        function::FADE_IN_CLEAR_BG2_TOP => fade_in_clear_bg2_top(game),
        function::FADE_IN_CLEAR_BG2_BOTTOM => {
            queue_vram(game, 1024, 0x2000, QueueBank::Word(126), |_| 18944);
            session_slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_1
            );
        }
        function::FADE_IN_LOAD_BG3_TILES_1 => {
            set_native_word(game, UNPAUSE_HOOK, UNPAUSE_DEAD);
            session_slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_2
            );
            queue_vram(game, 1024, 0xb200, QueueBank::Word(154), |_| 0x4000);
        }
        function::FADE_IN_LOAD_BG3_TILES_2 => {
            session_slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_3
            );
            queue_vram(game, 1024, 0xb600, QueueBank::Word(154), |_| 16896);
        }
        function::FADE_IN_LOAD_BG3_TILES_3 => {
            session_slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_4
            );
            queue_vram(game, 1024, 0xba00, QueueBank::Word(154), |_| 17408);
        }
        function::FADE_IN_LOAD_BG3_TILES_4 => {
            session_slot!(game, Body[BODY].function = function::FADE_IN_BG6);
            queue_vram(game, 1024, 0xbe00, QueueBank::Word(154), |_| 17920);
        }
        // Mostly the game's flags, music and palette fade: run on it.
        function::FADE_IN_BG6 => game.foreign(fade_in_bg6),
        function::FADE_IN_DEAD_WAS_ALIVE => {
            if is_dead(game) == 0 || game.camera().layer1[0] != 0 {
                hide_as_dead(game);
            }
        }
        function::FADE_IN_DEAD_WAS_DEAD => {
            if is_dead(game) == 0 || game.camera().layer1[0] != 0 {
                hide_as_dead(game);
            }
            game.foreign(|game| {
                crate::session_access::SessionAccess::session_mut(&mut game.runtime())
                    .room
                    .camera
                    .previous_layer1_block[0] = u16::MAX;
            });
        }
        function::RAISE_LOAD_TILEMAP_BOTTOM => {
            restrict_samus_x(game);
            session_slot!(game, Body[BODY].function = function::RAISE_QUAKE_EVERY_16);
            session_slot!(game, Body[BODY].timer = 120);
            word!(game.EARTHQUAKE_TIMER = 496);
            game.foreign(|game| game.queue_music_delayed(5));
            queue_bg2_bottom(game);
        }
        function::RAISE_QUAKE_EVERY_16 => {
            quake_until_timer(game, 0xf, function::RAISE_QUAKE_EVERY_8, 96)
        }
        function::RAISE_QUAKE_EVERY_8 => quake_until_timer(game, 7, function::RAISE, 288),
        function::RAISE => raise(game),
        function::GETS_BIG_DRAW_ROOM_BG => {
            session_slot!(
                game,
                Body[BODY].function = function::GETS_BIG_FADE_IN_ROOM_BG
            );
            session_slot!(game, Body[BODY]; mouth_hit_ready = 0, timer = 0);
            draw_room_bg(game);
        }
        function::MAIN_LOOP_THINKING => advance_thinking(game, function::ATTACK_WITH_MOUTH_OPEN),
        function::GETS_BIG_THINKING => advance_thinking(game, function::MOUTH_OPEN),
        function::MOUTH_OPEN => {
            if process_script(game) == 0xffff {
                session_slot!(game, Body[BODY].function = function::MAIN_LOOP_THINKING);
                respond_to_mouth_hit(game);
            }
        }
        function::INIT_EYE_GLOW => {
            session_slot!(game, Body[BODY].function = function::MOUTH_OPEN);
            session_slot!(game, Body[BODY].function = function::GLOW_EYE);
            session_slot!(game, Body[BODY].script = animation::SCRIPT_GLOW + 8);
            session_slot!(
                game,
                Body[BODY].script_delay = kraid_record(animation::SCRIPT_GLOW, game).word(0)
            );
            glow_eye(game);
        }
        function::UNGLOW_EYE => unglow_eye(game),
        function::ATTACK_WITH_MOUTH_OPEN => attack_with_mouth_open(game),
        function::GETS_BIG_RELEASE_CAMERA => {
            session_slot!(game, Body[BODY].function = function::GETS_BIG_BREAK_CEILING);
            set_native_word(game, SCROLL_LIMITS, 514);
            set_native_word(game, SCROLL_LIMITS + 2, 257);
            session_slot!(game, Body[BODY].min_eject_y = 164);
        }
        function::FOOT_WALK_BACKWARDS => {
            foot_walk_backwards(game, function::FOOT_SECOND_PHASE_THINKING, None)
        }
        function::FOOT_WALK_TO_START => foot_walk_backwards(
            game,
            function::HANDLE_FUNCTION_TIMER,
            Some((180, function::FOOT_SECOND_PHASE_INIT)),
        ),
        function::FOOT_WALK_FORWARD => foot_walk_forward(game),
        function::DEATH_INIT => game.foreign(death_init),
        function::LINT_PRODUCE => produce_lint(game, k),
        function::LINT_FIRE => fire_lint(game, k),
        function::FINGERNAIL_WAIT_FOR_LINT => fingernail_wait_for_lint(game, k),
        function::ALIGN_TO_KRAID => align_to_kraid(game, k),
        function::HANDLE_FUNCTION_TIMER => handle_function_timer(game, k),
        function::FOOT_FIRST_PHASE_THINKING => {
            first_phase(game);
            handle_function_timer(game, k);
        }
        function::PROCESS_SCRIPT_AND_TIMER => {
            process_script(game);
            handle_function_timer(game, k);
        }
        function::FINGERNAIL_FIRE => fire_fingernail(game, k),
        function::RESTRICT_SAMUS_X => {
            restrict_samus_x(game);
            handle_function_timer(game, k);
        }
        function::RAISE_THROUGH_FLOOR => {
            restrict_samus_x(game);
            session_slot!(
                game,
                Body[BODY].function = function::RAISE_LOAD_TILEMAP_BOTTOM
            );
            queue_bg2_top(game);
        }
        function::LINT_CHARGE => charge_lint(game, k),
        function::FOOT_IDLE | function::NOTHING => {}
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Kraid grows, shaking, and breaks the ceiling platforms. Was
/// `Kraid_GetsBig_BreakCeilingPlatforms`.
fn break_ceiling<L: Lease + ?Sized>(game: &mut L) {
    if (word!(game.NMI_FRAME_COUNTER_WORD) & 7) == 0 {
        quake_rocks(game);
    }
    let mut step: u16 = 1;
    if (session_slot!(game, Body[BODY].base.y_pos) & 2) != 0 {
        step = -1i32 as u16;
    }
    session_slot!(game, Body[BODY].base.x_pos, |old| old.wrapping_add(step));
    session_slot!(game, Body[BODY].base.y_pos, |old| old.wrapping_sub(1));
    // A platform breaks every four pixels risen, nine in all.
    if (session_slot!(game, Body[BODY].base.y_pos) & 3) == 0 {
        let platform: u16 = session_slot!(game, Body[BODY].timer);
        if (platform as i16).wrapping_sub(18) < 0 {
            let rock = crate::boss_lists::kraid_rock_x(game, (platform >> 1) as isize);
            let k = game.current_enemy();
            game.spawn_projectile(rock, k, CEILING_ROCKS);
            let broken = (session_slot!(game, Body[BODY].timer) >> 1) as usize;
            if let Some(&(block_x, plm)) = CEILING_PLATFORMS.get(broken) {
                game.foreign(|game| game.spawn_plm(block_x, 0x12, plm));
            }
            session_slot!(game, Body[BODY].timer, |old| old.wrapping_add(2));
        }
    }
    if below(session_slot!(game, Body[BODY].base.y_pos), 296) {
        session_slot!(
            game,
            Body[BODY].function = function::GETS_BIG_SET_BG2_PRIORITY
        );
    }
}

/// Was `Kraid_GetsBig_SetBG2TilemapPrioBits`.
fn set_bg2_priority<L: Lease + ?Sized>(game: &mut L) {
    for v0 in (0u16..4096).step_by(2) {
        word!(game.TILEMAP_STUFF[(v0 >> 1)] |= 0x2000);
    }
    session_slot!(game, Part[ARM].base.properties, |old| old & !(0x400));
    session_slot!(
        game,
        Body[BODY].function = function::GETS_BIG_FINISH_BG2_TILEMAP
    );
    queue_bg2_top(game);
}

/// Was `Kraid_GetsBig_FinishUpdateBg2Tilemap`.
fn finish_bg2_tilemap<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].function = function::GETS_BIG_DRAW_ROOM_BG);
    session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
    session_slot!(
        game,
        Part[TOP_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    session_slot!(
        game,
        Part[MIDDLE_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    session_slot!(
        game,
        Part[BOTTOM_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    session_slot!(
        game,
        Part[TOP_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    session_slot!(
        game,
        Part[MIDDLE_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    session_slot!(
        game,
        Part[BOTTOM_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    queue_bg2_bottom(game);
}

/// Fades in the room, then starts the fight. Was `Kraid_GetsBig_FadeInRoomBg`.
fn fade_in_room_bg<L: Lease + ?Sized>(game: &mut L) {
    if game.foreign(background6) & 1 != 0 {
        start_growing_thinking(game);
        enable_lint(game, TOP_LINT as u16, LINT_TIMERS[0]);
        enable_lint(game, MIDDLE_LINT as u16, LINT_TIMERS[1]);
        enable_lint(game, BOTTOM_LINT as u16, LINT_TIMERS[2]);
        session_slot!(game, Part[GOOD_FINGERNAIL].next = function::FINGERNAIL_INIT);
        session_slot!(game, Part[BAD_FINGERNAIL].next = function::FINGERNAIL_INIT);
        session_slot!(
            game,
            Part[GOOD_FINGERNAIL].function = function::HANDLE_FUNCTION_TIMER
        );
        session_slot!(
            game,
            Part[BAD_FINGERNAIL].function = function::HANDLE_FUNCTION_TIMER
        );
        session_slot!(game, Part[GOOD_FINGERNAIL].timer = 64);
        session_slot!(game, Part[BAD_FINGERNAIL].timer = 128);
        session_slot!(game, Part[ARM].x_offset = 1);
        session_slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        session_slot!(game, Body[BODY].target_x = 288);
        session_slot!(game, Part[FOOT].function = function::FOOT_WALK_TO_START);
        session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_BACKWARD);
    }
}

/// The eye glows until fully lit, then fades back. Was `Kraid_Shot_GlowHisEye`.
fn glow_eye<L: Lease + ?Sized>(game: &mut L) {
    process_script(game);
    let mut lit: i16 = 0;
    for v0 in (226u16..232).step_by(2) {
        let c: i32 = v0 as i32 >> 1;
        let mut red: u16 = ((word!(game.PALETTE_BUFFER[c]) as i32 & 0x1f) + 1) as u16;
        if ((word!(game.PALETTE_BUFFER[c]) as i32 & 0x1f) - 30i32) & 0x8000 == 0 {
            lit += 1;
            red = 31;
        }
        let mut green: u16 = ((word!(game.PALETTE_BUFFER[c]) as i32 & 0x3e0) + 32) as u16;
        if ((word!(game.PALETTE_BUFFER[c]) as i32 & 0x3e0) - 960i32) & 0x8000 == 0 {
            lit += 1;
            green = 992;
        }
        word!(game.PALETTE_BUFFER[c] = green | red | word!(game.PALETTE_BUFFER[c]) & 0xfc00);
    }
    if lit.wrapping_sub(6) >= 0 {
        session_slot!(game, Body[BODY].function = function::UNGLOW_EYE);
    }
}

/// Counts down the part's timer, then runs `next` with the foot's walk. Was
/// `KraidEnemy_DecrementEnemyFunctionTimer`.
fn decrement_function_timer<L: Lease + ?Sized>(game: &mut L) {
    let me = current(game);
    if session_slot!(game, Part[me].timer) != 0 {
        let left = session_slot!(game, Part[me].timer);
        session_slot!(game, Part[me].timer, |old| old.wrapping_sub(1));
        if left == 1 {
            session_slot!(game, Part[me].function = session_slot!(game, Part[me].next));
            session_slot!(game, Part[me].base; current_instruction = animation::FOOT_WALK_BACKWARD, instruction_timer = 1);
        }
    }
}

/// When the walk ends, picks a new target and walks toward it. Was
/// `KraidsFoot_SecondPhase_Thinking`.
fn foot_second_phase_thinking<L: Lease + ?Sized>(game: &mut L) {
    let left: u16 = (session_slot!(game, Part[FOOT].next)).wrapping_sub(1);
    session_slot!(game, Part[FOOT].next = left);
    if left == 0 {
        let mut row: u16 = 0;
        loop {
            if session_slot!(game, Body[BODY].base.x_pos)
                == kraid_tables::foot_target(game, (row >> 2) as usize).x_position
            {
                break;
            }
            row = row.wrapping_add(4);
            if (row as i16).wrapping_sub(24) < 0 {
                continue;
            }
            row = 4;
            break;
        }
        let mut choice: i16 = (word!(game.RANDOM_NUMBER) as i16) & 0x1c;
        if !below(choice as u16, 16) {
            choice = 16;
        }
        let movement = (kraid_tables::foot_target(game, (row >> 2) as usize).movement)
            .wrapping_add(choice as u16);
        let walk = native_record(BANK, movement, game);
        let frames: u16 = walk.word(2);
        if (walk.word(0) as i16).wrapping_sub((session_slot!(game, Body[BODY].base.x_pos)) as i16)
            >= 0
        {
            walk_backwards(game, frames, walk.word(0));
        } else {
            walk_forwards(game, frames, walk.word(0));
        }
    }
}

/// Launches a fingernail on a random pattern, from Kraid or held back off
/// screen. Was `KraidsFingernail_Init`.
fn initialize_fingernail_attack<L: Lease + ?Sized>(game: &mut L) {
    let mut direction: u16 = session_slot!(game, Fingernail[GOOD_FINGERNAIL].y_speed);
    if game.current_enemy() == 384 {
        direction = session_slot!(game, Fingernail[BAD_FINGERNAIL].y_speed);
    }
    let pattern: u16 = if direction & 0x8000 != 0 {
        crate::boss_lists::kraid_nail_velocity(
            game,
            true,
            ((word!(game.RANDOM_NUMBER) as i32 & 6) >> 1) as isize,
        )
    } else {
        crate::boss_lists::kraid_nail_velocity(
            game,
            false,
            ((word!(game.RANDOM_NUMBER) as i32 & 6) >> 1) as isize,
        )
    };
    let speeds = native_record(BANK, pattern, game);
    let me = current(game);
    session_slot!(game, Fingernail[me]; x_subspeed = speeds.word(0), x_speed = speeds.word(2), y_subspeed = speeds.word(4), y_speed = speeds.word(6), launched = 1);
    session_slot!(
        game,
        Fingernail[me].base.properties = (session_slot!(game, Fingernail[me].base.properties)
            as i32
            & !(K_ENEMY_PROPS_TANGIBLE as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
            as u16
    );
    session_slot!(game, Fingernail[me].base; instruction_timer = 1, current_instruction = animation::FINGERNAIL);
    session_slot!(game, Fingernail[me].function = function::FINGERNAIL_FIRE);
    let at_kraid: bool = ((word!(game.RANDOM_NUMBER) & 1) == 0)
        || ({
            let mut other: u16 = health_eighth(game, GOOD_FINGERNAIL, 1);
            if game.current_enemy() != 448 {
                other = health_eighth(game, BAD_FINGERNAIL, 1);
            }
            (other == 1)
                || ({
                    set_health_eighth(game, me, 1, 1);
                    session_slot!(game, Fingernail[me].base.x_pos = 50);
                    session_slot!(game, Fingernail[me].base.y_pos = 240);
                    session_slot!(game, Fingernail[me].x_subspeed = 0);
                    session_slot!(game, Fingernail[me].x_speed = 1);
                    session_slot!(game, Fingernail[me]; y_subspeed = 0, y_speed = 0);
                    session_slot!(
                        game,
                        Fingernail[me].function = function::FINGERNAIL_WAIT_FOR_LINT
                    );
                    session_slot!(game, Fingernail[me].next = function::FINGERNAIL_FIRE);
                    session_slot!(
                        game,
                        Fingernail[me].base.properties =
                            (session_slot!(game, Fingernail[me].base.properties) as i32
                                | (K_ENEMY_PROPS_TANGIBLE as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                                as u16
                    );
                    false
                })
        });
    if at_kraid {
        set_health_eighth(game, me, 1, 0);
        session_slot!(
            game,
            Fingernail[me].base.x_pos = ((session_slot!(game, Fingernail[BODY].base.x_pos) as i32
                - session_slot!(game, Fingernail[BODY].base.x_width) as i32
                - session_slot!(game, Fingernail[me].base.x_width) as i32)
                & 0xfff0) as u16
        );
        session_slot!(
            game,
            Fingernail[me].base.y_pos =
                (session_slot!(game, Fingernail[ARM].base.y_pos)).wrapping_add(128)
        );
    }
}

/// Was `KraidsFoot_PrepareToLungeForward`.
fn foot_prepare_lunge<L: Lease + ?Sized>(game: &mut L) {
    first_phase(game);
    if !below(
        session_slot!(game, Part[ARM].base.current_instruction),
        animation::ARM_IDLE_LAST,
    ) {
        session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_DYING, instruction_timer = 1);
        session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_LUNGE);
        session_slot!(game, Part[FOOT].function = function::FOOT_LUNGE_FORWARD);
        session_slot!(game, Part[FOOT].timer = 0);
    }
}

/// Was `KraidsFoot_FirstPhase_LungeForward`.
fn foot_lunge_forward<L: Lease + ?Sized>(game: &mut L) {
    first_phase(game);
    if below(session_slot!(game, Body[BODY].base.x_pos), 92) {
        session_slot!(game, Body[BODY].base.x_pos = 92);
    }
    if session_slot!(game, Part[FOOT].base.current_instruction) == animation::FOOT_WALK_BACKWARD_END
    {
        if session_slot!(game, Body[BODY].base.x_pos) == 92 {
            session_slot!(game, Part[FOOT].next = function::FOOT_RETREAT_FROM_LUNGE);
            session_slot!(
                game,
                Part[FOOT].function = function::DECREMENT_FUNCTION_TIMER
            );
            session_slot!(game, Part[FOOT].timer = 1);
            session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
        } else {
            session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_LUNGE);
        }
    }
}

/// Was `KraidsFoot_FirstPhase_RetreatFromLunge`.
fn foot_retreat_from_lunge<L: Lease + ?Sized>(game: &mut L) {
    first_phase(game);
    if !below(session_slot!(game, Body[BODY].base.x_pos), 176) {
        session_slot!(game, Body[BODY].base.x_pos = 176);
    }
    if !below(
        session_slot!(game, Part[FOOT].base.current_instruction),
        animation::FOOT_WALK_BACKWARD_LAST,
    ) {
        if session_slot!(game, Body[BODY].base.x_pos) == 176 {
            session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
            session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
            session_slot!(
                game,
                Part[FOOT].function = function::FOOT_FIRST_PHASE_THINKING
            );
            session_slot!(game, Part[FOOT].timer = 300);
            session_slot!(game, Part[FOOT].next = function::FOOT_PREPARE_LUNGE);
        } else {
            game.foreign(|game| game.print_x_undefined());
            let me = current(game);
            session_slot!(game, Part[me].base; current_instruction = animation::FOOT_WALK_BACKWARD, instruction_timer = 1);
        }
    }
}

/// Fades out the room, then queues the dying tilemap. Was `Kraid_Death_Fadeout`.
fn death_fade_out<L: Lease + ?Sized>(game: &mut L) {
    process_script(game);
    if game.foreign(background6) & 1 != 0 {
        session_slot!(game, Body[BODY].function = function::DEATH_BG2_TOP_HALF);
        session_slot!(game, Body[BODY]; hurt_frame_timer = 1, hurt_frame = 1);
        palette(game);
        game.foreign(|game| {
            crate::word_transfers!(game, (4), 128; xray(0) <- value(0), xray(1) <- value(0));
        });
        queue_vram(game, 512, 0x4000, QueueBank::Byte(126), |game| {
            (((game.slot_byte(BG3_TILEMAP_BASE) as i32 & 0xf) << 8) + 16128) as u16
        });
    }
}

/// Was `Kraid_Death_UpdateBG2TilemapTopHalf`.
fn death_bg2_top_half<L: Lease + ?Sized>(game: &mut L) {
    process_script(game);
    session_slot!(game, Body[BODY].function = function::DEATH_BG2_BOTTOM_HALF);
    for lint in [TOP_LINT, MIDDLE_LINT, BOTTOM_LINT] {
        session_slot!(game, Part[lint].function = function::ALIGN_TO_KRAID);
        session_slot!(game, Part[lint].timer = 0x7fff);
    }
    queue_bg2_top(game);
}

/// Starts sinking: the earthquake, the sinking unpause hook and the parts'
/// last animations. Was `Kraid_Death_UpdateBG2TilemapBottomHalf`.
fn death_bg2_bottom_half<L: Lease + ?Sized>(game: &mut L) {
    process_script(game);
    game.set_slot_byte(UNPAUSE_HOOK + 2, BANK);
    set_native_word(game, UNPAUSE_HOOK, UNPAUSE_SINKING);
    session_slot!(
        game,
        Body[BODY].function = function::DEATH_SINK_THROUGH_FLOOR
    );
    word!(game.BOSS_DEATH_QUAKE_SOUND_TIMER = 43);
    session_slot!(game, Body[BODY].base.properties, |old| old | 0x8000);
    word!(game.EARTHQUAKE_TYPE = 1);
    word!(game.EARTHQUAKE_TIMER = 256);
    session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_RAISED, instruction_timer = 1);
    session_slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
    session_slot!(game, Part[FOOT].function = function::FOOT_IDLE);
    queue_bg2_bottom(game);
}

/// Sinks a pixel a frame; once gone, drops items and cleans up. Was
/// `Kraid_Death_SinkThroughFloor`.
pub fn sink_through_floor<L: Lease + ?Sized>(game: &mut L) {
    process_script(game);
    sound_every_half_second(game);
    handle_sinking(game);
    session_slot!(game, Body[BODY].base.y_pos, |old| old.wrapping_add(1));
    if !below(session_slot!(game, Body[BODY].base.y_pos), 608) {
        session_slot!(
            game,
            Body[BODY].base.properties = (session_slot!(game, Body[BODY].base.properties) as i32
                & !(K_ENEMY_PROPS_TANGIBLE as i32)) as u16
        );
        word!(game.ENEMY_BG2_TILEMAP_SIZE = 2);
        let enemy_ptr: u16 = session_slot!(game, Body[current(game)].base.enemy_ptr);
        game.foreign(|game| game.set_shot_ai(enemy_ptr, NO_SHOT_AI));
        let deleted: u16 = (session_slot!(game, Part[ARM].base.properties) as i32
            | K_ENEMY_PROPS_TANGIBLE as i32
            | K_ENEMY_PROPS_DELETED as i32) as u16;
        session_slot!(game, Part[ARM].base.properties = deleted);
        let hidden: u16 = deleted & 0x51ff | 0x600;
        session_slot!(game, Part[TOP_LINT].base.properties = hidden);
        session_slot!(game, Part[MIDDLE_LINT].base.properties = hidden);
        session_slot!(game, Part[BOTTOM_LINT].base.properties = hidden);
        session_slot!(game, Part[FOOT].base.properties = hidden);
        session_slot!(game, Body[BODY].function = function::FADE_IN_CLEAR_BG2_TOP);
        word!(game.CAMERA_DISTANCE_INDEX = 0);
        game.foreign(|game| game.scatter_enemy_drop(DropSource::Kraid, enemy_ptr));
        draw_room_bg(game);
    }
}

/// Was `Kraid_FadeInBg_ClearBg2TilemapTopHalf`.
fn fade_in_clear_bg2_top<L: Lease + ?Sized>(game: &mut L) {
    game.set_slot_byte(BG2_TILEMAP_BASE, 72);
    game.foreign(|game| {
        crate::word_transfers!(game, descending, 1024; le(TILEMAP_STUFF, 1023) <- value(BLANK_BG2_TILE));
    });
    queue_vram(game, 1024, 0x2000, QueueBank::Word(126), |_| 18432);
    session_slot!(
        game,
        Body[BODY].function = function::FADE_IN_CLEAR_BG2_BOTTOM
    );
}

/// Fades the room back in, then sets the boss bit once. Was
/// `Kraid_FadeInBg_FadeInBp6`.
pub fn fade_in_bg6<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| game.observe_kraid_flags(0));
    if game.foreign(background6) & 1 != 0 {
        game.foreign(|game| game.observe_kraid_flags(1));
        game.foreign(|game| game.queue_music_delayed(3));
        game.foreign(|game| game.observe_kraid_flags(2));
        let area = word!(game.AREA_INDEX);
        let bits: i16 = game.foreign(|game| game.boss_word_native(area)) as i16;
        if (bits as u16 & 1) != 0 {
            session_slot!(game, Body[BODY].function = function::FADE_IN_DEAD_WAS_DEAD);
        } else {
            let area = word!(game.AREA_INDEX);
            game.foreign(|game| game.set_boss_word_native(area, (bits as u16) | 1));
            session_slot!(game, Body[BODY].function = function::FADE_IN_DEAD_WAS_ALIVE);
        }
    }
    game.foreign(|game| game.observe_kraid_flags(3));
}

/// Kraid rises the rest of the way. Was `Kraid_Raise_Handler`.
fn raise<L: Lease + ?Sized>(game: &mut L) {
    restrict_samus_x(game);
    if (word!(game.EARTHQUAKE_TIMER) & 5) == 0 {
        quake_rocks(game);
    }
    session_slot!(
        game,
        Body[BODY].base.x_pos = (session_slot!(game, Body[BODY].base.x_pos) as i32
            + if (session_slot!(game, Body[BODY].base.y_pos) & 2) == 0 {
                -1i32
            } else {
                1
            }) as u16
    );
    session_slot!(game, Body[BODY].base(y_pos, y_subpos) += -0x8000i32 as u32);
    if below(session_slot!(game, Body[BODY].base.y_pos), 457) {
        session_slot!(game, Body[BODY].base.x_pos = 176);
        session_slot!(
            game,
            Part[FOOT].function = function::FOOT_FIRST_PHASE_THINKING
        );
        session_slot!(game, Part[FOOT].timer = 300);
        session_slot!(game, Part[FOOT].next = function::FOOT_PREPARE_LUNGE);
        session_slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        start_thinking(game);
        session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
    }
}

/// Places the lint by Kraid's height and picks a thinking time. Was
/// `Kraid_SetLintYAndRandomThinkTimer`.
fn place_lint_and_think<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Part[TOP_LINT].base.y_pos = (session_slot!(game, Body[BODY].base.y_pos)).wrapping_sub(20)
    );
    session_slot!(
        game,
        Part[MIDDLE_LINT].base.y_pos =
            (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(46)
    );
    session_slot!(
        game,
        Part[BOTTOM_LINT].base.y_pos =
            (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(112)
    );
    let mut think: i16 = (word!(game.RANDOM_NUMBER) as i16) & 7;
    if (word!(game.RANDOM_NUMBER) & 7) == 0 {
        think = 2;
    }
    session_slot!(
        game,
        Body[BODY].thinking_timer = ((think as i32) << 6) as u16
    );
}

/// Samus touching Kraid's body is pushed out and hurt. Was `Kraid_Enemy_Touch`.
fn body_touch<S: BossState + ?Sized>(game: &mut S) {
    if below(
        session_slot!(game, Body[BODY].function),
        function::DEATH_INIT,
    ) {
        let right: u16 = (word!(game.SAMUS_X_RADIUS)).wrapping_add(game.player_x());
        let depth: i16 = ((game.player_y()) as i16)
            .wrapping_sub((session_slot!(game, Body[BODY].base.y_pos)) as i16);
        let mut row: i32 = 0;
        loop {
            let band = (row >> 2) as usize;
            if depth.wrapping_sub(kraid_tables::body_hitbox(game, band).bottom as i16) >= 0
                || depth.wrapping_sub(kraid_tables::body_hitbox(game, band + 1).bottom as i16) >= 0
            {
                break;
            }
            row += 4;
        }
        if ((session_slot!(game, Body[BODY].base.x_pos) as i32
            + kraid_tables::body_hitbox(game, (row >> 2) as usize).left as i32
            - right as i32) as i16)
            < 0
        {
            if !below(game.player_x(), 40) {
                game.set_player_x(word!(game.SAMUS_X_POS).wrapping_sub(8));
                game.set_previous_player_x(word!(game.SAMUS_X_POS));
            }
            let mut eject: u16 = (game.player_y()).wrapping_sub(8);
            if ((game.player_y() as i32 - 8i32 - session_slot!(game, Body[BODY].min_eject_y) as i32)
                as i16)
                < 0
            {
                eject = session_slot!(game, Body[BODY].min_eject_y);
            }
            game.set_player_y(eject);
            game.set_previous_player_y(eject);
            push_samus(game);
            if word!(game.SAMUS_INVINCIBILITY_TIMER) == 0 {
                game.react_to_contact(Death::Finalize);
            }
        }
    }
}

/// Shots striking Kraid's body explode; a beam hit while thinking opens his
/// mouth. Was `Kraid_Shot_Body`.
fn body_shots<S: BossState + ?Sized>(game: &mut S) {
    if below(
        session_slot!(game, Body[BODY].function),
        function::DEATH_SINK_THROUGH_FLOOR,
    ) {
        session_slot!(game, Body[BODY].mouth_hit_ready = 0);
        session_slot!(game, Body[BODY].mouth_flags, |old| old & !(1));
        let mut hits: u16 = 0;
        let frame = kraid_record(
            (session_slot!(game, Body[BODY].script)).wrapping_sub(8),
            game,
        );
        let hitbox = kraid_record(frame.word(4), game);
        let right: u16 = (session_slot!(game, Body[BODY].base.x_pos)).wrapping_add(hitbox.word(0));
        let top: u16 = (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(2));
        let mut bottom: u16 =
            (session_slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(6));
        if word!(game.PROJECTILE_COUNTER) != 0 {
            let mut i: i32 = 2 * word!(game.PROJECTILE_COUNTER) as i32;
            while i >= 0 {
                let p: i32 = i >> 1;
                let hit_body: bool = if (word!(game.PROJECTILE_Y_POS[p]) as i32
                    - word!(game.PROJECTILE_Y_RADIUS[p]) as i32
                    - 1i32
                    - bottom as i32)
                    & 0x8000
                    != 0
                {
                    (word!(game.PROJECTILE_Y_RADIUS[p]) as i32
                        + word!(game.PROJECTILE_Y_POS[p]) as i32
                        - top as i32)
                        & 0x8000
                        == 0
                        && (word!(game.PROJECTILE_X_RADIUS[p]) as i32
                            + word!(game.PROJECTILE_X_POS[p]) as i32
                            - right as i32)
                            & 0x8000
                            == 0
                } else {
                    let q: i32 = i >> 1;
                    bottom = (word!(game.PROJECTILE_X_RADIUS[q]))
                        .wrapping_add(word!(game.PROJECTILE_X_POS[q]));
                    let depth: i16 = ((word!(game.PROJECTILE_Y_POS[q])) as i16)
                        .wrapping_sub((session_slot!(game, Body[BODY].base.y_pos)) as i16);
                    let mut j: u16 = 0;
                    loop {
                        let band = (j >> 2) as usize;
                        if depth.wrapping_sub(kraid_tables::body_hitbox(game, band).bottom as i16)
                            >= 0
                            || depth.wrapping_sub(
                                kraid_tables::body_hitbox(game, band + 1).bottom as i16,
                            ) >= 0
                        {
                            break;
                        }
                        j = j.wrapping_add(4);
                    }
                    ((session_slot!(game, Body[BODY].base.x_pos) as i32
                        + kraid_tables::body_hitbox(game, (j >> 2) as usize).left as i32
                        - bottom as i32) as i16)
                        < 0
                };
                if hit_body {
                    projectile_explosion(game, i as u16);
                    let p: i32 = i >> 1;
                    word!(game.PROJECTILE_DIR[p] |= 0x10);
                    if (word!(game.PROJECTILE_TYPE[p]) & 0x10) != 0 {
                        session_slot!(
                            game,
                            Body[BODY].mouth_flags =
                                (session_slot!(game, Body[BODY].mouth_flags)) | 1
                        );
                    }
                    hits = hits.wrapping_add(1);
                }
                i -= 2;
            }
        }
        if hits != 0 && session_slot!(game, Body[BODY].function) == function::MAIN_LOOP_THINKING {
            session_slot!(game, Body[BODY].function = function::INIT_EYE_GLOW);
            let flags: u16 = session_slot!(game, Body[BODY].mouth_flags);
            if (flags & 1) != 0 {
                session_slot!(game, Body[BODY].mouth_flags = flags | 0x302);
            }
        }
    }
}

/// Flashes when hurt and shows the health palette. Was `Kraid_Palette_Handling`.
fn palette<S: BossState + ?Sized>(game: &mut S) {
    if below(session_slot!(game, Body[BODY].base.health), 1) {
        let copied = session_slot!(game, Body[BODY].base.health);
        session_slot!(game, Body[BODY].hurt_frame = copied);
    } else {
        let update: bool = (session_slot!(game, Body[BODY].hurt_frame) != 0)
            && ({
                let left: u16 = (session_slot!(game, Body[BODY].hurt_frame_timer)).wrapping_sub(1);
                session_slot!(game, Body[BODY].hurt_frame_timer = left);
                (left == 0)
                    && ({
                        session_slot!(game, Body[BODY].hurt_frame_timer = 2);
                        session_slot!(
                            game,
                            Body[BODY].hurt_frame =
                                session_slot!(game, Body[BODY].hurt_frame).wrapping_sub(1)
                        );
                        true
                    })
            });
        if !update {
            return;
        }
    }
    hurt_flash(game);
    health_palette(game);
}

/// Fades the eye back to its health colour. Was `Kraid_Shot_UnglowEye`.
fn unglow_eye<L: Lease + ?Sized>(game: &mut L) {
    let mut eighth: u16 = 14;
    let health: u16 = session_slot!(game, Body[BODY].base.health);
    while (health as i16).wrapping_sub(health_eighth(game, eighth as i32, 0) as i16) < 0 {
        eighth = eighth.wrapping_sub(2);
        if eighth == 0 {
            break;
        }
    }
    let mut source: u16 = (16 * (eighth as i32 + 2)) as u16;
    let mut colour: u16 = 226;
    let mut fading: u16 = 0;
    loop {
        let c: i32 = colour as i32 >> 1;
        let mut part: u16 = (word!(game.PALETTE_BUFFER[c])) & 0x1f;
        let s: i32 = source as i32 >> 1;
        if kraid_tables::bg_color(game, (s + 1) as usize) & 0x1f != part {
            fading = fading.wrapping_add(1);
            word!(game.PALETTE_BUFFER[c] = word!(game.PALETTE_BUFFER[c]).wrapping_sub(1));
        }
        part = (word!(game.PALETTE_BUFFER[c])) & 0x3e0;
        if kraid_tables::bg_color(game, (s + 1) as usize) & 0x3e0 != part {
            fading = fading.wrapping_add(1);
            word!(game.PALETTE_BUFFER[c] = word!(game.PALETTE_BUFFER[c]).wrapping_sub(32));
        }
        colour = colour.wrapping_add(2);
        source = source.wrapping_add(2);
        if (colour as i16).wrapping_sub(232) >= 0 {
            break;
        }
    }
    if fading == 0 {
        session_slot!(game, Body[BODY].function = function::MOUTH_OPEN);
        session_slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        session_slot!(
            game,
            Body[BODY].script_delay = kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(0)
        );
    }
}

/// Roars with the mouth open, spitting rocks when it is widest. Was
/// `Kraid_Main_AttackWithMouthOpen`.
fn attack_with_mouth_open<L: Lease + ?Sized>(game: &mut L) {
    if process_script(game) == 0xffff {
        start_thinking(game);
        respond_to_mouth_hit(game);
    } else if kraid_record(
        (session_slot!(game, Body[BODY].script)).wrapping_sub(8),
        game,
    )
    .word(2)
        == animation::TILEMAP_3
        && (session_slot!(game, Body[BODY].script_delay) & 0xf) == 0
    {
        let parameter = crate::boss_lists::kraid_rock_spit_x_velocity(
            game,
            ((word!(game.RANDOM_NUMBER) as i32 & 0xe) >> 1) as isize,
        );
        let k = (word!(game.RANDOM_NUMBER)) & 0xe;
        game.spawn_projectile(parameter, k, SPIT_ROCKS);
        game.queue_sound(2, ROCKS_SOUND, 6);
    }
}

/// Walks back to the target, then stops: thinking again, or after `wait`
/// frames the second phase. Was `KraidsFoot_SecondPhase_WalkingBackwards` and
/// `KraidsFoot_SecondPhaseSetup_WalkToStartPt`.
fn foot_walk_backwards<L: Lease + ?Sized>(game: &mut L, next: u16, wait: Option<(u16, u16)>) {
    let target: u16 = session_slot!(game, Body[BODY].target_x);
    if target != session_slot!(game, Body[BODY].base.x_pos) {
        if (target as i16).wrapping_sub((session_slot!(game, Body[BODY].base.x_pos)) as i16) >= 0 {
            return;
        }
        session_slot!(game, Body[BODY].base.x_pos = target);
    }
    if !below(
        session_slot!(game, Part[FOOT].base.current_instruction),
        animation::FOOT_WALK_BACKWARD_LAST,
    ) {
        session_slot!(game, Part[FOOT].function = next);
        if let Some((timer, then)) = wait {
            session_slot!(game, Part[FOOT].timer = timer);
            session_slot!(game, Part[FOOT].next = then);
        }
        session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
    }
}

/// Was `KraidsFoot_SecondPhase_WalkForward`.
fn foot_walk_forward<L: Lease + ?Sized>(game: &mut L) {
    let target: u16 = session_slot!(game, Body[BODY].target_x);
    if (target as i16).wrapping_sub((session_slot!(game, Body[BODY].base.x_pos)) as i16) < 0 {
        if session_slot!(game, Part[FOOT].base.current_instruction)
            == animation::FOOT_WALK_FORWARD_END
        {
            session_slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_WALK_FORWARD, instruction_timer = 1);
        }
    } else {
        session_slot!(game, Body[BODY].base.x_pos = target);
        if session_slot!(game, Part[FOOT].base.current_instruction)
            == animation::FOOT_WALK_FORWARD_END
        {
            session_slot!(
                game,
                Part[FOOT].function = function::FOOT_SECOND_PHASE_THINKING
            );
            session_slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
        }
    }
}

/// Below six eighths of his health, Kraid stands up for the second phase. Was
/// `Kraid_HandleFirstPhase`.
fn first_phase<L: Lease + ?Sized>(game: &mut L) {
    if ((session_slot!(game, Body[BODY].base.health)) as i16)
        .wrapping_sub(health_eighth(game, BODY, 6) as i16)
        < 0
    {
        session_slot!(
            game,
            Body[BODY].function = function::PROCESS_SCRIPT_AND_TIMER
        );
        session_slot!(game, Body[BODY].timer = 180);
        session_slot!(game, Body[BODY].next = function::GETS_BIG_RELEASE_CAMERA);
        let tilemap: u16 = kraid_record(session_slot!(game, Body[BODY].script), game).word(2);
        let mut frame: u16 = 50;
        if tilemap != animation::TILEMAP_0 {
            frame = 42;
            if tilemap != animation::TILEMAP_1 {
                frame = 34;
                if tilemap != animation::TILEMAP_2 {
                    frame = 26;
                }
            }
        }
        session_slot!(
            game,
            Body[BODY].script = animation::SCRIPT_IDLE.wrapping_add(frame)
        );
        session_slot!(
            game,
            Body[BODY].script_delay =
                kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(usize::from(frame))
        );
        word!(game.EARTHQUAKE_TYPE = 4);
        word!(game.EARTHQUAKE_TIMER = 340);
        session_slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
        session_slot!(game, Part[FOOT].function = function::FOOT_IDLE);
        session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
        for lint in [TOP_LINT, MIDDLE_LINT, BOTTOM_LINT] {
            session_slot!(
                game,
                Part[lint].base.properties =
                    (session_slot!(game, Part[lint].base.properties)) | 0x100
            );
        }
        session_slot!(game, Part[ARM].base.properties, |old| old | 0x400);
    }
}

/// Once the hurt flash ends: the dying palette, the parts die and the spikes
/// clear. Was `Kraid_Death_Init`.
fn death_init<L: Lease + ?Sized>(game: &mut L) {
    if session_slot!(game, Body[BODY].hurt_frame) == 0 {
        game.foreign(|game| {
            crate::word_transfers!(game, ascending, 16; le(TARGET_PALETTES, 96) <- value(0));
        });
        game.foreign(|game| {
            crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 127) <- rom(DEATH_PALETTE, 15));
        });
        session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_DYING, instruction_timer = 1);
        session_slot!(game, Body[BODY].function = function::DEATH_FADE_OUT);
        session_slot!(game, Body[BODY].script = animation::SCRIPT_DYING + 8);
        session_slot!(
            game,
            Body[BODY].script_delay = kraid_record(animation::SCRIPT_DYING, game).word(0)
        );
        let _actor: u16 = game.current_enemy();
        let saved: u16 = game.current_enemy();
        session_slot!(
            game,
            Part[GOOD_FINGERNAIL].base.properties =
                session_slot!(game, Part[GOOD_FINGERNAIL].base.properties) & !(0x4000)
        );
        word!(game.CUR_ENEMY_INDEX = 384);
        game.foreign(|game| game.normal_death(0x180));
        session_slot!(
            game,
            Part[BAD_FINGERNAIL].base.properties =
                session_slot!(game, Part[BAD_FINGERNAIL].base.properties) & !(0x4000)
        );
        word!(game.CUR_ENEMY_INDEX = 448);
        game.foreign(|game| game.normal_death(0x1c0));
        word!(game.CUR_ENEMY_INDEX = 128);
        game.foreign(|game| game.normal_death(0x80));
        word!(game.CUR_ENEMY_INDEX = 192);
        game.foreign(|game| game.normal_death(0xc0));
        word!(game.CUR_ENEMY_INDEX = 256);
        game.foreign(|game| game.normal_death(0x100));
        word!(game.CUR_ENEMY_INDEX = saved);
        game.foreign(|game| game.spawn_plm(0x5, 0x1b, DEATH_SPIKES_PLM));
    }
}

/// Arm contact: Samus is pushed back and the bottom lint fires. Was
/// `KraidsArm_Touch`.
pub fn arm_touch<L: Lease + ?Sized>(game: &mut L) {
    if word!(game.SAMUS_INVINCIBILITY_TIMER) == 0 {
        push_samus(game);
        session_slot!(game, Part[BOTTOM_LINT].function = function::LINT_FIRE);
        touch(game);
    }
}

/// Body contact. Was `Kraid_Touch`.
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    game.contact_reaction(Death::LeavePending);
}

/// Body shot. Was `Kraid_Shot`.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    game.shot_reaction(Shot::Finalize);
}

/// Sets up the room: the hooks, and either the empty room for a dead Kraid or
/// his health eighths, BG2 and the rise. Was `Kraid_Init`.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    game.set_slot_byte(UNPAUSE_HOOK + 2, BANK);
    game.set_slot_byte(PAUSE_HOOK + 2, BANK);
    set_native_word(game, UNPAUSE_HOOK, UNPAUSE_ALIVE);
    set_native_word(game, PAUSE_HOOK, PAUSE);
    if is_dead(game) != 0 {
        game.foreign(|game| {
            crate::word_transfers!(game, ascending, 16; le(TARGET_PALETTES, 96) <- rom(ROOM_BG_PALETTE, 0));
        });
        game.foreign(|game| {
            crate::word_transfers!(game, descending, 1024; le(TILEMAP_STUFF, 1023) <- value(BLANK_BG2_TILE));
        });
        session_slot!(game, Body[BODY].unread_7804 = 0);
        queue_room_bg(game);
        game.foreign(|game| game.spawn_plm(0x2, 0x12, CLEAR_CEILING_PLM));
        game.foreign(|game| game.spawn_plm(0x5, 0x1b, CLEAR_SPIKES_PLM));
        session_slot!(game, Body[BODY].function = function::FADE_IN_CLEAR_BG2_TOP);
    } else {
        game.set_slot_byte(BG2_TILEMAP_BASE, 67);
        word!(game.CAMERA_DISTANCE_INDEX = 2);
        set_native_word(game, SCROLL_LIMITS, 0);
        set_native_word(game, SCROLL_LIMITS + 2, 1);
        session_slot!(game, Body[BODY].min_eject_y = 324);
        let mut eighth: u16 = session_slot!(game, Body[BODY].base.health) >> 3;
        let step: u16 = eighth;
        for slot in (0u16..16).step_by(2) {
            set_health_eighth(game, slot as i32, 0, eighth);
            eighth = eighth.wrapping_add(step);
        }
        load_bg2(game, 0xdfff);
        let quarter_at = |index: usize| slot!(@at Body, BODY, health_quarters) + index * 2;
        let quarter: u16 = session_slot!(game, Body[BODY].base.health) >> 2;
        set_native_word(game, quarter_at(0), quarter);
        let half: u16 = native_word(game, quarter_at(0)).wrapping_add(quarter);
        set_native_word(game, quarter_at(1), half);
        let three: u16 = native_word(game, quarter_at(0)).wrapping_add(half);
        set_native_word(game, quarter_at(2), three);
        let all = native_word(game, quarter_at(0)).wrapping_add(three);
        set_native_word(game, quarter_at(3), all);
        set_native_word(game, slot!(@at Body, BODY, unread_782e) + 8 * 2, 0);
        session_slot!(game, Body[BODY].base; x_pos = 176, y_pos = 592, properties = (session_slot!(game, Body[BODY].base.properties)) | (K_ENEMY_PROPS_TANGIBLE as u16));
        session_slot!(game, Body[BODY]; function = function::RESTRICT_SAMUS_X, timer = 300, next = function::RAISE_THROUGH_FLOOR, script_delay = 64);
        game.foreign(|game| game.disable_minimap_for_boss());
        game.foreign(|game| {
            crate::word_transfers!(game, descending, 32; native(0x2ffe) <- value(BLANK_BG2_TILE));
        });
        word!(game.EARTHQUAKE_TYPE = 5);
        game.foreign(|game| {
            crate::word_transfers!(game, ascending, 16; native(0xc360) <- rom(RISING_PALETTE, 0));
        });
    }
}

/// Was `KraidsArm_Init`.
pub fn initialize_arm<L: Lease + ?Sized>(game: &mut L) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        let copied = session_slot!(game, Part[BODY].base.palette_index);
        session_slot!(game, Part[ARM].base.palette_index = copied);
        session_slot!(game, Part[ARM].function = function::FOOT_IDLE);
        session_slot!(game, Part[ARM].base; current_instruction = animation::ARM_RAISED, instruction_timer = 1);
        session_slot!(game, Part[ARM].progress = 0);
    }
}

/// Sets up lint `lint` hidden in the belly at `x_offset`. Was
/// `KraidsTopLint_Init`, `KraidsMiddleLint_Init` and `KraidsBottomLint_Init`.
pub fn initialize_lint<L: Lease + ?Sized>(game: &mut L, lint: i32) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        let copied = session_slot!(game, Part[BODY].base.palette_index);
        session_slot!(game, Part[lint].base.palette_index = copied);
        session_slot!(game, Part[lint].base; instruction_timer = 0x7fff, current_instruction = animation::LINT_HIDDEN);
        session_slot!(
            game,
            Part[lint].base.spritemap_pointer = animation::LINT_HIDDEN_SPRITEMAP
        );
        if lint == TOP_LINT {
            session_slot!(game, Part[lint]; function = function::NOTHING, timer = 0x7fff, x_offset = 0);
        } else {
            session_slot!(game, Part[lint].function = function::NOTHING);
            session_slot!(game, Part[lint].x_offset = -16i32 as u16);
        }
    }
}

/// Was `KraidsFoot_Init`.
pub fn initialize_foot<L: Lease + ?Sized>(game: &mut L) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        let palette: u16 = session_slot!(game, Body[BODY].base.palette_index);
        session_slot!(game, Part[FOOT].base.palette_index = palette);
        session_slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
        session_slot!(game, Part[FOOT].function = function::FOOT_IDLE);
        session_slot!(game, Part[FOOT].next = 0);
    }
}

/// Mouth and body shots, the palette, contact and BG2's scroll, then the
/// body's function. Was `Kraid_Main`.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    // The shots, palette, contact, BG2 scroll and the body's function run on
    // the lease, what needs the game in foreign calls.
    mouth_shots(game);
    palette(game);
    body_shots(game);
    body_touch(game);
    word!(
        game.REG_BG2HOFS = (session_slot!(game, Body[BODY].base.x_width) as i32
            + word!(game.REG_BG1HOFS) as i32
            - game.camera().bg1_offset[0] as i32
            - session_slot!(game, Body[BODY].base.x_pos) as i32) as u16
    );
    word!(
        game.REG_BG2VOFS = (game.camera().layer1[1] as i32
            - session_slot!(game, Body[BODY].base.y_pos) as i32
            + 152) as u16
    );
    let function = session_slot!(game, Body[BODY].function);
    run_function(game, function);
}

/// The arm follows the body and shows only on screen. Was `KraidsArm_Main`.
pub fn arm_main<L: Lease + ?Sized>(game: &mut L) {
    let layer1_y = game.camera().layer1[1];
    let screen_bottom: u16 = layer1_y.wrapping_add(224);
    let y: u16 = (session_slot!(game, Body[BODY].base.y_pos)).wrapping_sub(44);
    session_slot!(game, Part[ARM].base.y_pos = y);
    let mut properties: u16 = (session_slot!(game, Part[ARM].base.properties)) | 0x100;
    if (y as i16).wrapping_sub(layer1_y as i16) >= 0
        && (y as i16).wrapping_sub(screen_bottom as i16) < 0
    {
        properties = (session_slot!(game, Part[ARM].base.properties)) & 0xfeff;
    }
    session_slot!(game, Part[ARM].base.properties = properties);
    let copied = session_slot!(game, Body[BODY].base.x_pos);
    session_slot!(game, Part[ARM].base.x_pos = copied);
    if session_slot!(byte game, Body[BODY].mouth_flags[1]) != 0 {
        session_slot!(
            game,
            Part[ARM].base.instruction_timer =
                session_slot!(game, Part[ARM].base.instruction_timer).wrapping_add(1)
        );
    }
}

/// A lint's main AI. Was `KraidsTopLint_Main`, `KraidsMiddleLint_Main` and
/// `KraidsBottomLint_Main`.
pub fn lint_main_ai<L: Lease + ?Sized>(game: &mut L, lint: i32) {
    session_slot!(game, Part[lint].base.instruction_timer = 0x7fff);
    lint_main(game, lint as u16);
}

/// The foot follows the body and shows only on screen. Was `KraidsFoot_Main`.
pub fn foot_main<L: Lease + ?Sized>(game: &mut L) {
    let copied = session_slot!(game, Part[BODY].base.x_pos);
    session_slot!(game, Part[FOOT].base.x_pos = copied);
    let y: u16 = (session_slot!(game, Part[BODY].base.y_pos)).wrapping_add(100);
    session_slot!(game, Part[FOOT].base.y_pos = y);
    let above: i16 = (y as i16).wrapping_sub(224);
    let mut properties: u16 = (session_slot!(game, Part[FOOT].base.properties)) & 0xfeff;
    let layer1_y = game.camera().layer1[1];
    if (y as i16).wrapping_sub(layer1_y as i16) >= 0 {
        if above.wrapping_sub(layer1_y as i16) >= 0 {
            properties |= 0x100;
        }
    } else {
        properties |= 0x100;
    }
    session_slot!(game, Part[FOOT].base.properties = properties);
    run_function(game, session_slot!(game, Part[FOOT].function));
}

/// A fingernail touched: it shatters. Was `KraidsGoodFingernail_Touch`.
pub fn fingernail_touch<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| game.print_x_undefined());
    game.contact_reaction(Death::Finalize);
    let _actor = game.current_enemy();
    game.foreign(|game| game.normal_death(0));
}

/// A fingernail's main: gone with Kraid, else its function. Was
/// `KraidsGoodFingernail_Main` and `KraidsBadFingernail_Main`.
pub fn fingernail_main<L: Lease + ?Sized>(game: &mut L, nail: i32) {
    if below(session_slot!(game, Part[BODY].base.health), 1) {
        session_slot!(
            game,
            Part[nail].base.properties = (session_slot!(game, Part[nail].base.properties) as i32
                | (K_ENEMY_PROPS_DELETED as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                as u16
        );
    } else {
        run_function(game, session_slot!(game, Part[nail].function));
    }
}

/// Was `KraidsGoodFingernail_Init` and `KraidsBadFingernail_Init`.
pub fn initialize_fingernail<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| kraid_fingernails::initialize(game.current_enemy(), game));
}

/// Instruction: below five eighths of health the arm swipes. Was `Kraid_Instr_9`.
pub fn arm_swipe_when_hurt<L: Lease + ?Sized>(game: &mut L) -> Option<u16> {
    if ((session_slot!(game, Body[BODY].base.health)) as i16)
        .wrapping_sub(health_eighth(game, BODY, 3) as i16)
        < 0
        && below(
            session_slot!(game, Part[ARM].base.current_instruction),
            animation::ARM_SWIPE,
        )
    {
        return Some(animation::ARM_SWIPE);
    }
    None
}

/// Instruction: Kraid steps up a pixel. Was `Kraid_Instr_DecYpos`.
pub fn step_up<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].base.y_pos, |old| old.wrapping_sub(1));
}

/// Instruction: Kraid steps down a pixel, shaking the room. Was
/// `Kraid_Instr_IncrYpos_Shake`.
pub fn step_down_and_shake<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(game, Body[BODY].base.y_pos, |old| old.wrapping_add(1));
    word!(game.EARTHQUAKE_TYPE = 1);
    word!(game.EARTHQUAKE_TIMER = 10);
}

/// Instruction: the stomp. Was `Kraid_Instr_PlaySound_0x76`.
pub fn stomp_sound<L: Lease + ?Sized>(game: &mut L) {
    game.queue_sound(1, STOMP_SOUND, 6);
}

/// Instruction: Kraid steps back. Was `Kraid_Instr_XposMinus3`/`3b`.
pub fn step_back<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Body[BODY].base.x_pos =
            (session_slot!(game, Body[BODY].base.x_pos)).wrapping_sub(FOOT_STEP_BACK)
    );
}

/// Instruction: Kraid steps forward. Was `Kraid_Instr_XposPlus3`.
pub fn step_forward<L: Lease + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Body[BODY].base.x_pos =
            (session_slot!(game, Body[BODY].base.x_pos)).wrapping_add(FOOT_STEP_FORWARD)
    );
}

/// Instruction: Kraid lunges right until blocked, shaking the room. Was
/// `Kraid_Instr_MoveHimRight`.
pub fn lunge_right<L: Lease + ?Sized>(game: &mut L) {
    if (below(session_slot!(game, Body[BODY].base.x_pos), 320) || {
        let left: u16 = (session_slot!(game, Body[BODY].target_x)).wrapping_sub(1);
        session_slot!(game, Body[BODY].target_x = left);
        session_slot!(game, Body[BODY].target_x) == 0
    }) && game.move_horizontal(0, (PUSH_RIGHT as i16 as i32) << 16, Slopes::Ignore)
    {
        word!(game.EARTHQUAKE_TYPE = 0);
        word!(game.EARTHQUAKE_TIMER = 7);
        let x: u16 = session_slot!(game, Body[BODY].base.x_pos);
        session_slot!(game, Part[FOOT].base.x_pos = x);
    }
}
