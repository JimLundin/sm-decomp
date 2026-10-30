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
#![forbid(unsafe_code)]
use crate::actors::host::{Death, Shot, Slopes};
use crate::actors::layouts::part_view;
use crate::actors::{kraid_fingernails, kraid_lint, kraid_sinking};
use crate::cartridge::native_record;
use crate::constants::enemy_properties::{
    kEnemyProps_Deleted, kEnemyProps_Invisible, kEnemyProps_Tangible,
};
use crate::enemy_projectiles::drops::DropSource;
use crate::kraid_tables;
use crate::palette_effects::fading::background6;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::cartridge::RomRecord;

/// What Kraid needs beyond the shared boss effects, deaths and lint contact.
pub trait Host:
    crate::actors::host::BossEffects
    + crate::actors::deaths::Deaths
    + kraid_lint::Host
    + crate::records::Records
    + crate::decompression::DecompressionHost
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

part_view! {
    /// The body's view of its slot. `timer` also counts the ceiling platforms
    /// broken while Kraid grows.
    Body: KraidLayout {
        words: [
            function, script, script_delay, var_D, mouth_hit_ready, timer, parameter_1,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [next, field_2, field_4, thinking_timer, min_eject_y, mouth_flags],
        kraid_healths_8ths: [u16; 8],
        words: [field_1C, target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, hurt_frame, hurt_frame_timer],
        field_2E: [u16; 9],
        words: [],
    }
}

part_view! {
    /// A lint's, the arm's or the foot's view of its slot. The foot's `next`
    /// also counts down its walk.
    Part: KraidLayout {
        words: [function, progress, x_offset, var_D, var_E, timer, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [next, field_2, field_4, thinking_timer, min_eject_y, mouth_flags],
        kraid_healths_8ths: [u16; 8],
        words: [field_1C, target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, hurt_frame, hurt_frame_timer],
        field_2E: [u16; 9],
        words: [],
    }
}

part_view! {
    /// A fingernail's view of its slot: its 16.16 velocity. Word 1 of
    /// `kraid_healths_8ths` marks a fingernail held back off screen.
    Fingernail: KraidLayout {
        words: [
            function, x_subspeed, x_speed, y_subspeed, y_speed, timer, launched, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [next, field_2, field_4, thinking_timer, min_eject_y, mouth_flags],
        kraid_healths_8ths: [u16; 8],
        words: [field_1C, target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, hurt_frame, hurt_frame_timer],
        field_2E: [u16; 9],
        words: [],
    }
}

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

/// Whether `value` is below `limit`, as the original's signed 16-bit difference.
fn below(value: u16, limit: u16) -> bool {
    (value.wrapping_sub(limit) as i16) < 0
}

fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.cur_enemy_index) as i32
}

fn kraid_record<H: Host + ?Sized>(address: u16, game: &mut H) -> RomRecord {
    native_record(BANK, address, game)
}

fn set_native_word<H: Host + ?Sized>(game: &mut H, at: usize, value: u16) {
    let bytes = u16::to_ne_bytes(value);
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

fn native_word<H: Host + ?Sized>(game: &H, at: usize) -> u16 {
    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}

/// Health word `index` of the body's eighths, at byte `slot` past the body.
fn health_eighth<H: Host + ?Sized>(game: &H, slot: i32, index: usize) -> u16 {
    native_word(game, slot!(@at Body, slot, kraid_healths_8ths) + index * 2)
}

fn set_health_eighth<H: Host + ?Sized>(game: &mut H, slot: i32, index: usize, value: u16) {
    let at = slot!(@at Body, slot, kraid_healths_8ths) + index * 2;
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
fn queue_vram<H: Host + ?Sized>(
    game: &mut H,
    size: u16,
    source: u16,
    bank: QueueBank,
    destination: impl FnOnce(&H) -> u16,
) {
    let tail: u16 = word!(game.vram_write_queue_tail);
    let entry = VRAM_QUEUE + word!(game.vram_write_queue_tail) as i32;
    let at =
        |offset: isize| usize::try_from(entry as isize + offset).expect("VRAM entry outside RAM");
    set_native_word(game, at(0), size);
    set_native_word(game, at(2), source);
    match bank {
        QueueBank::Byte(value) => game.ram().write_byte(at(4), value),
        QueueBank::Word(value) => set_native_word(game, at(4), value),
    }
    let destination = destination(game);
    set_native_word(game, at(5), destination);
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}

/// The room BG through the BG3 tilemap: Kraid's room background.
fn queue_room_bg<H: Host + ?Sized>(game: &mut H) {
    queue_vram(
        game,
        512,
        animation::ROOM_BG,
        QueueBank::Byte(BANK),
        |game| (((game.ram().read_byte(BG3_TILEMAP_BASE) as i32 & 0xf) << 8) + 16128) as u16,
    );
}

/// Contact with the arm or foot. Was `Kraid_Touch_ArmFoot`.
pub fn touch_arm_or_foot<H: Host + ?Sized>(game: &mut H) {
    game.contact_reaction(Death::Finalize);
}

/// Pushes Samus up and away after a contact. Was `Kraid_Func_1`.
fn push_samus<H: Host + ?Sized>(game: &mut H) {
    word!(game.extra_samus_x_displacement = PUSH_RIGHT);
    word!(game.extra_samus_y_displacement = -8i32 as u16);
}

/// The arm is shot: the projectile explodes. Was `Kraid_Arm_Shot`.
pub fn arm_shot<H: Host + ?Sized>(game: &mut H, j: u16) {
    projectile_explosion(game, j);
    word!(game.projectile_dir[(j >> 1)] |= 0x10);
}

/// Whether Kraid's boss bit is set for this area. Was `Kraid_CheckIfDead`.
pub fn is_dead<H: Host + ?Sized>(game: &mut H) -> u16 {
    let area = word!(game.area_index);
    ((game.boss_word_native(area) & 1) != 0) as u16
}

/// Was `Kraid_SetEnemyPropsToDead`.
fn hide_as_dead<H: Host + ?Sized>(game: &mut H) {
    let me = current(game);
    slot!(
        game,
        Part[me].base.properties = slot!(game, Part[me].base.properties) & 0x50ff | 0x700
    );
}

/// Decompresses BG2's graphics and tilemap and sets the tilemap's priority
/// with `priority`. Was `Kraid_SetupGfxWithTilePrioClear`.
fn load_bg2<H: Host + ?Sized>(game: &mut H, priority: u16) {
    crate::decompression::decompress_to_ram(BG2_TILES, 0x4000, game);
    crate::decompression::decompress_to_ram(BG2_TILEMAP, 0x2000, game);
    slot!(game, Body[BODY]; hurt_frame = 0, hurt_frame_timer = 0);
    for v1 in (0u16..1536).step_by(2) {
        let value = (word!(game.tilemap_stuff[(v1 >> 1)])) & 0xdfff;
        let at = usize::try_from(0x2000 + (((v1 as i32 >> 1) + 1024) as isize) * 2)
            .expect("native word outside RAM");
        set_native_word(game, at, value);
    }
    for v2 in 0u16..1024 {
        word!(game.tilemap_stuff[v2] = priority & native_word(game, 0x4000 + v2 as usize * 2));
    }
}

/// Fades in the room background palette and queues its tilemap. Was
/// `Kraid_DrawRoomBg`.
fn draw_room_bg<H: Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(ROOM_BG_PALETTE, 0));
    word!(game.palette_change_num = 0);
    queue_room_bg(game);
}

/// Was `Kraid_GetsBig_Thinking_Setup`.
fn start_growing_thinking<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Body[BODY].function = function::GETS_BIG_THINKING);
    place_lint_and_think(game);
}

/// Was `Kraid_Mainloop_Thinking_Setup`.
fn start_thinking<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Body[BODY].function = function::MAIN_LOOP_THINKING);
    place_lint_and_think(game);
}

/// Starts lint `k` waiting `timer` frames to fire. Was `Kraid_Lints_Enable`.
fn enable_lint<H: Host + ?Sized>(game: &mut H, k: u16, timer: u16) {
    let me = k as i32;
    slot!(game, Part[me].timer = timer);
    slot!(game, Part[me].function = function::ALIGN_TO_KRAID);
    slot!(game, Part[me].next = function::LINT_PRODUCE);
    slot!(game, Part[me].progress = 0);
}

/// Counts down the thinking timer, then opens the mouth. Was
/// `advance_kraid_thinking_timer!`.
fn advance_thinking<H: Host + ?Sized>(game: &mut H, next: u16) {
    let thinking: u16 = slot!(game, Body[BODY].thinking_timer);
    if thinking != 0 {
        let left: u16 = thinking.wrapping_sub(1);
        slot!(game, Body[BODY].thinking_timer = left);
        if left == 0 {
            slot!(game, Body[BODY].function = next);
            slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
            slot!(
                game,
                Body[BODY].script_delay = kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(0)
            );
        }
    }
}

/// After the mouth closes: a hard hit makes the eye glow. Was
/// `advance_kraid_mouth_hit_response!`.
fn respond_to_mouth_hit<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Body[BODY].script_delay = 90);
    let flags: u16 = slot!(game, Body[BODY].mouth_flags);
    if (flags & 4) != 0 && {
        let left: u16 = flags.wrapping_sub(256);
        slot!(game, Body[BODY].mouth_flags = left);
        (left & 0xff00) != 0
    } {
        slot!(game, Body[BODY].function = function::HANDLE_FUNCTION_TIMER);
        slot!(game, Body[BODY].timer = 64);
        slot!(game, Body[BODY].next = function::INIT_EYE_GLOW);
        slot!(game, Body[BODY].field_2 = 2);
    } else {
        slot!(game, Body[BODY].mouth_flags = 0);
    }
}

/// Steps the body's script; returns its last instruction word, FFFFh at the
/// end. Was `Kraid_ProcessKraidInstr`.
fn process_script<H: Host + ?Sized>(game: &mut H) -> u16 {
    let result: u16 = slot!(game, Body[BODY].script_delay);
    if result != 0 {
        let delay = slot!(game, Body[BODY].script_delay);
        slot!(
            game,
            Body[BODY].script_delay = slot!(game, Body[BODY].script_delay).wrapping_sub(1)
        );
        if delay == 1 {
            return execute_script(game);
        }
    }
    result
}

/// Runs script calls up to the next tilemap, which it queues. Was
/// `Kraid_ExecuteInstr`.
fn execute_script<H: Host + ?Sized>(game: &mut H) -> u16 {
    loop {
        let script: u16 = slot!(game, Body[BODY].script);
        let record = kraid_record(script, game);
        let result: u16 = record.word(0);
        if record.word(0) != 0xffff {
            if (record.word(0) as i16).wrapping_add(1) < 0 {
                match result {
                    script_call::ROAR => game.queue_sound(1, ROAR_SOUND, 6),
                    script_call::DYING_CRY => game.queue_sound(1, DYING_CRY_SOUND, 15),
                    _ => game.unreachable(),
                }
                slot!(
                    game,
                    Body[BODY].script = (slot!(game, Body[BODY].script)).wrapping_add(2)
                );
            } else {
                slot!(game, Body[BODY].script_delay = result);
                slot!(game, Body[BODY].script = script.wrapping_add(8));
                let tilemap: u16 = record.word(2);
                let tail: u16 = word!(game.vram_write_queue_tail);
                let _size_tail: u16 = word!(game.vram_write_queue_tail);
                game.queue_kraid_tilemap(tail, tilemap);
                return 1;
            }
        } else {
            return result;
        }
    }
}

/// Shots into the open mouth hurt Kraid; a fatal one starts his death. Was
/// `Kraid_Shot_Mouth`.
fn mouth_shots<H: Host + ?Sized>(game: &mut H) {
    // Nothing once the death has reached the sinking (C537h and later).
    if !below(
        slot!(game, Body[BODY].function),
        function::DEATH_SINK_THROUGH_FLOOR,
    ) {
        return;
    }
    let mut hit = false;
    let frame = kraid_record((slot!(game, Body[BODY].script)).wrapping_sub(8), game);
    if frame.word(6) != 0xffff {
        let hitbox_address: u16 = frame.word(6);
        slot!(game, Body[BODY].mouth_hit_ready = 1);
        let hitbox = kraid_record(hitbox_address, game);
        let right: u16 = (slot!(game, Body[BODY].base.x_pos)).wrapping_add(hitbox.word(0));
        let top: u16 = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(2));
        let bottom: u16 = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(6));
        if word!(game.projectile_counter) != 0 {
            let mut index: u16 = 2u16.wrapping_mul(word!(game.projectile_counter));
            loop {
                let p: i32 = index as i32 >> 1;
                if !(word!(game.projectile_y_pos[p]) as i32
                    - word!(game.projectile_y_radius[p]) as i32
                    - 1i32
                    - bottom as i32
                    & 0x8000
                    == 0
                    || word!(game.projectile_y_radius[p]) as i32
                        + word!(game.projectile_y_pos[p]) as i32
                        - top as i32
                        & 0x8000
                        != 0
                    || word!(game.projectile_x_radius[p]) as i32
                        + word!(game.projectile_x_pos[p]) as i32
                        - right as i32
                        & 0x8000
                        != 0)
                {
                    let kind: u16 = word!(game.projectile_type[p]);
                    let skip = (kind & 0xf00) == 0
                        && ((kind & 0x10) == 0 || {
                            slot!(
                                game,
                                Body[BODY].mouth_flags = (slot!(game, Body[BODY].mouth_flags)) | 1
                            );
                            false
                        });
                    if !skip {
                        word!(game.collision_detection_index = index >> 1);
                        game.shot_reaction(Shot::ReactionOnly);
                        word!(game.projectile_dir[(index >> 1)] |= 0x10);
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
        slot!(game, Body[BODY].hurt_frame = 6);
        slot!(game, Body[BODY].hurt_frame_timer = 2);
        let flags: u16 = slot!(game, Body[BODY].mouth_flags);
        if (flags & 2) != 0 {
            slot!(game, Body[BODY].mouth_flags = flags | 4);
        }
        // Out of health and not yet dying (before C360h).
        if below(slot!(game, Body[BODY].base.health), 1)
            && below(slot!(game, Body[BODY].function), function::DEATH_INIT)
        {
            slot!(game, Body[BODY].function = function::DEATH_INIT);
            slot!(game, Body[BODY].mouth_flags = 0);
            slot!(
                game,
                Body[BODY].base.properties =
                    (slot!(game, Body[BODY].base.properties)) | (kEnemyProps_Tangible as u16)
            );
            load_bg2(game, !(0x2000i32) as u16);
            for part in (0u16..384).step_by(64) {
                let p = part as i32;
                slot!(
                    game,
                    Part[p].base.properties =
                        (slot!(game, Part[p].base.properties)) | (kEnemyProps_Tangible as u16)
                );
            }
            if below(slot!(game, Body[BODY].script), animation::SCRIPT_IDLE_END) {
                slot!(
                    game,
                    Body[BODY].script = (slot!(game, Body[BODY].script)).wrapping_add(60)
                );
            }
        }
    }
}

/// A dust cloud where projectile `k` struck. Was `Kraid_SpawnExplosionEproj`.
fn projectile_explosion<H: Host + ?Sized>(game: &mut H, k: u16) {
    let p: i32 = k as i32 >> 1;
    let (x, y) = (
        word!(game.projectile_x_pos[p]),
        word!(game.projectile_y_pos[p]),
    );
    game.set_projectile_spawn_point(x, y);
    let kind: u16 = (if (word!(game.projectile_type[p]) & 0x200) == 0 {
        6
    } else {
        29
    }) as u16;
    game.spawn_room_projectile(DUST_CLOUD, kind);
    game.queue_sound(0, EXPLOSION_SOUND, 6);
}

/// Was `Kraid_HurtFlash_Handling`.
fn hurt_flash<H: Host + ?Sized>(game: &mut H) {
    let mut source: u16 = 0;
    if (slot!(game, Body[BODY].hurt_frame) & 1) == 0 {
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
fn health_palette<H: Host + ?Sized>(game: &mut H) {
    let mut source: u16 = 0;
    if (slot!(game, Body[BODY].hurt_frame) & 1) == 0 {
        let mut eighth: u16 = 14;
        let health: u16 = slot!(game, Body[BODY].base.health);
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
        word!(game.palette_buffer[(c + 112)] = kraid_tables::bg_color(game, s as usize));
        word!(game.palette_buffer[(c + 240)] = kraid_tables::sprite_color(game, s as usize));
        source = source.wrapping_add(2);
    }
}

/// A lint's main: Samus contact, then its function. Was `KraidLintCommon_Main`.
fn lint_main<H: Host + ?Sized>(game: &mut H, k: u16) {
    kraid_lint::lint_touch_samus(k, game);
    run_function(game, slot!(game, Part[k as i32].function));
}

/// The lint slides out of Kraid's belly. Was `KraidLint_ProduceLint`.
fn produce_lint<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(
        game,
        Part[me].base.properties = (slot!(game, Part[me].base.properties)) & 0xfaff
    );
    slot!(
        game,
        Part[me].base.x_pos = (slot!(game, Part[me].x_offset) as i32
            + slot!(game, Part[BODY].base.x_pos) as i32
            - slot!(game, Part[me].progress) as i32) as u16
    );
    let progress: u16 = (slot!(game, Part[me].progress)).wrapping_add(1);
    slot!(game, Part[me].progress = progress);
    if !below(progress, 32) {
        slot!(game, Part[me].function = function::LINT_CHARGE);
        slot!(game, Part[me].timer = 30);
    }
}

/// The lint flashes while it charges. Was `KraidLint_ChargeLint`.
fn charge_lint<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let mut palette: i16 = 0;
    if (slot!(game, Part[me].timer) & 1) != 0 {
        palette = 3584;
    }
    slot!(game, Part[me].base.palette_index = palette as u16);
    slot!(
        game,
        Part[me].base.x_pos = (slot!(game, Part[me].x_offset) as i32
            + slot!(game, Part[BODY].base.x_pos) as i32
            - slot!(game, Part[me].progress) as i32) as u16
    );
    let left = slot!(game, Part[me].timer);
    slot!(
        game,
        Part[me].timer = slot!(game, Part[me].timer).wrapping_sub(1)
    );
    if left == 1 {
        slot!(game, Part[me].function = function::LINT_FIRE);
        game.queue_sound(2, LINT_SOUND, 6);
    }
}

/// The lint flies left, carrying Samus if she stands on it. Was
/// `KraidLint_FireLint`.
fn fire_lint<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(game, Part[me].base(x_pos, x_subpos) += -LINT_SPEED as u32);
    if below(slot!(game, Part[me].base.x_pos), 56) {
        slot!(
            game,
            Part[me].base.properties =
                (slot!(game, Part[me].base.properties)) | (kEnemyProps_Tangible as u16)
        );
    }
    if below(slot!(game, Part[me].base.x_pos), 32) {
        slot!(
            game,
            Part[me].base.properties =
                (slot!(game, Part[me].base.properties)) | (kEnemyProps_Invisible as u16)
        );
        slot!(game, Part[me].function = function::ALIGN_TO_KRAID);
        slot!(game, Part[me].timer = 300);
        slot!(game, Part[me].next = function::LINT_PRODUCE);
        slot!(game, Part[me].progress = 0);
    }
    if 0u16.wrapping_sub(u16::from(game.touches_player_from_above(k))) != 0 {
        let low = word!(game.extra_samus_x_subdisplacement) as u32;
        let high = word!(game.extra_samus_x_displacement) as u32;
        let sum = (low | (high << 16)).wrapping_add(-LINT_SPEED as u32);
        word!(game.extra_samus_x_subdisplacement = sum as u16);
        word!(game.extra_samus_x_displacement = (sum >> 16) as u16);
        if word!(game.extra_samus_x_displacement) as i32 + 16 & 0x8000 != 0 {
            word!(game.extra_samus_x_displacement = -16i32 as u16);
        }
    }
}

/// A fingernail waits for the top lint to clear the screen's left. Was
/// `KraidFingernail_WaitForLintXpos`.
fn fingernail_wait_for_lint<H: Host + ?Sized>(game: &mut H, k: u16) {
    if !below(slot!(game, Part[TOP_LINT].base.x_pos), 256) {
        let me = k as i32;
        slot!(game, Part[me].function = slot!(game, Part[me].next));
        slot!(
            game,
            Part[me].base.properties = (slot!(game, Part[me].base.properties)) & 0xfaff
        );
    }
}

/// Counts down the part's timer, then runs its `next` function. Was
/// `KraidEnemy_HandleFunctionTimer`.
fn handle_function_timer<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    if slot!(game, Part[me].timer) != 0 && {
        let left = slot!(game, Part[me].timer);
        slot!(
            game,
            Part[me].timer = slot!(game, Part[me].timer).wrapping_sub(1)
        );
        left == 1
    } {
        slot!(game, Part[me].function = slot!(game, Part[me].next));
    }
}

/// Keeps a lint at Kraid's belly. Was `Kraid_AlignEnemyToKraid`.
fn align_to_kraid<H: Host + ?Sized>(game: &mut H, k: u16) {
    let x: u16 = slot!(game, Part[BODY].base.x_pos);
    let me = k as i32;
    slot!(
        game,
        Part[me].base.x_pos = x.wrapping_sub(slot!(game, Part[me].base.x_width))
    );
    handle_function_timer(game, k);
}

/// Was `Kraid_SetWalkingBackwards`.
fn walk_backwards<H: Host + ?Sized>(game: &mut H, frames: u16, target: u16) {
    slot!(game, Body[BODY].target_x = target);
    slot!(game, Part[FOOT].next = frames);
    slot!(game, Part[FOOT].function = function::FOOT_WALK_BACKWARDS);
    slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_BACKWARD);
}

/// Was `Kraid_SetWalkingForwards`.
fn walk_forwards<H: Host + ?Sized>(game: &mut H, frames: u16, target: u16) {
    slot!(game, Body[BODY].target_x = target);
    slot!(game, Part[FOOT].next = frames);
    slot!(game, Part[FOOT].function = function::FOOT_WALK_FORWARD);
    slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_FORWARD);
}

/// The fingernail flies, bouncing off walls and Kraid. Was
/// `KraidsFingernail_Fire`.
fn fire_fingernail<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let x_speed = (((slot!(game, Fingernail[me].x_speed) as u32) << 16)
        | slot!(game, Fingernail[me].x_subspeed) as u32) as i32;
    if game.move_horizontal(k, x_speed, Slopes::Ignore) {
        slot!(
            game,
            Fingernail[me].x_subspeed = -(slot!(game, Fingernail[me].x_subspeed) as i32) as u16
        );
        slot!(
            game,
            Fingernail[me].x_speed = -(slot!(game, Fingernail[me].x_speed) as i32) as u16
        );
    } else {
        let mut row: i32 = 0;
        loop {
            if ((kraid_tables::fingernail_boundary(game, (row >> 2) as usize).top as i32
                + slot!(game, Fingernail[BODY].base.y_pos) as i32
                - slot!(game, Fingernail[me].base.y_pos) as i32) as i16)
                < 0
            {
                break;
            }
            row += 4;
        }
        let left: u16 = (kraid_tables::fingernail_boundary(game, (row >> 2) as usize).left)
            .wrapping_add(slot!(game, Fingernail[BODY].base.x_pos));
        if slot!(game, Fingernail[me].base.x_width) as i32
            + slot!(game, Fingernail[me].base.x_pos) as i32
            - left as i32
            & 0x8000
            == 0
            && (slot!(game, Fingernail[me].x_speed) & 0x8000) == 0
        {
            slot!(
                game,
                Fingernail[me].x_subspeed = -(slot!(game, Fingernail[me].x_subspeed) as i32) as u16
            );
            slot!(
                game,
                Fingernail[me].x_speed = -(slot!(game, Fingernail[me].x_speed) as i32) as u16
            );
        }
    }
    let y_speed = (((slot!(game, Fingernail[me].y_speed) as u32) << 16)
        | slot!(game, Fingernail[me].y_subspeed) as u32) as i32;
    if game.move_vertical(k, y_speed) {
        let speed = ((slot!(game, Fingernail[me].y_speed) as i32) << 16)
            | slot!(game, Fingernail[me].y_subspeed) as i32;
        slot!(game, Fingernail[me](y_speed, y_subspeed) = -speed as u32);
    }
}

/// Spawns a ceiling rock from bank $A7 `crumble`'s column and its PLM.
fn crumble<H: Host + ?Sized>(game: &mut H, rock: u16, block_x: u8, plm: u16) {
    let k = word!(game.cur_enemy_index);
    game.spawn_projectile(rock, k, CEILING_ROCKS);
    game.spawn_plm(block_x, 0x12, plm);
}

fn sound_every_half_second<H: Host + ?Sized>(game: &mut H) {
    word!(game.kraid_unk9000 = word!(game.kraid_unk9000).wrapping_sub(1));
    if word!(game.kraid_unk9000) == 0 {
        game.queue_sound(2, ROCKS_SOUND, 6);
        word!(game.kraid_unk9000 = 30);
    }
}

/// Crumbles the ceiling platforms as Kraid sinks. Was `Kraid_HandleSinking`.
fn handle_sinking<H: Host + ?Sized>(game: &mut H) {
    if let Some(callback) = kraid_sinking::handle_sinking(game) {
        match callback {
            sinking::LEFT_PLATFORM_LEFT => crumble(game, 0x70, 0x7, CEILING_PLM_A7),
            sinking::RIGHT_PLATFORM_MIDDLE => crumble(game, 0xf0, 0xf, CEILING_PLM_A7),
            sinking::RIGHT_PLATFORM_LEFT => crumble(game, 0xe0, 0xe, CEILING_PLM_AF),
            sinking::LEFT_PLATFORM_RIGHT => crumble(game, 0x90, 0x9, CEILING_PLM_A7),
            sinking::LEFT_PLATFORM_MIDDLE => crumble(game, 0x80, 0x8, CEILING_PLM_AF),
            sinking::RIGHT_PLATFORM_RIGHT => crumble(game, 0x100, 0x10, CEILING_PLM_AF),
            sinking::NOTHING => {}
            _ => game.unreachable(),
        }
    }
}

/// Queues BG2's tilemap top half. Was `Kraid_UpdateBg2TilemapTopHalf`.
fn queue_bg2_top<H: Host + ?Sized>(game: &mut H) {
    queue_vram(game, 2048, 0x2000, QueueBank::Byte(126), |game| {
        ((game.ram().read_byte(BG2_TILEMAP_BASE) as i32 & 0xfc) << 8) as u16
    });
}

/// Queues BG2's tilemap bottom half. Was `Kraid_UpdateBG2TilemapBottomHalf`.
fn queue_bg2_bottom<H: Host + ?Sized>(game: &mut H) {
    queue_vram(game, 2048, 0x2800, QueueBank::Byte(126), |game| {
        (((game.ram().read_byte(BG2_TILEMAP_BASE) as i32 & 0xfc) << 8) + 2048) as u16
    });
}

/// Rocks and dust while Kraid rises. Was `Kraid_SpawnRandomQuakeProjs`.
fn quake_rocks<H: Host + ?Sized>(game: &mut H) {
    let mut offset: u16 = (word!(game.random_number)) & 0x3f;
    if (word!(game.random_number) & 2) == 0 {
        offset = !(offset as i32) as u16;
    }
    let x: u16 = (slot!(game, Body[BODY].base.x_pos)).wrapping_add(offset);
    let y: u16 = (448 - ((word!(game.random_number) & 0x3f00) as i32 >> 8)) as u16;
    game.create_sprite(x, y, 21, 0);
    let rocks: u16 = if (word!(game.random_number) & 0x10) != 0 {
        RISING_ROCKS_RIGHT
    } else {
        RISING_ROCKS_LEFT
    };
    let parameter = (word!(game.random_number)) & 0x3f0;
    let k = word!(game.cur_enemy_index);
    game.spawn_projectile(parameter, k, rocks);
}

/// Keeps Samus on the first screen while Kraid rises. Was
/// `Kraid_RestrictSamusXtoFirstScreen_2`.
fn restrict_samus_x<H: Host + ?Sized>(game: &mut H) {
    if ((word!(game.samus_x_pos)) as i16).wrapping_sub(256) >= 0 {
        word!(game.samus_x_pos = 256);
        word!(game.samus_prev_x_pos = 256);
    }
}

/// Counts down the body's timer, spawning rocks every `interval + 1` frames.
fn quake_until_timer<H: Host + ?Sized>(game: &mut H, interval: u16, next: u16, next_timer: u16) {
    restrict_samus_x(game);
    let left: u16 = (slot!(game, Body[BODY].timer)).wrapping_sub(1);
    slot!(game, Body[BODY].timer = left);
    if left != 0 {
        if (left & interval) == 0 {
            quake_rocks(game);
        }
    } else {
        slot!(game, Body[BODY].function = next);
        slot!(game, Body[BODY].timer = next_timer);
    }
}

/// Runs a Kraid part's bank $A7 function. Was `CallKraidFunc`.
pub fn run_function<H: Host + ?Sized>(game: &mut H, function: u16) {
    let k: u16 = word!(game.cur_enemy_index);
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
            slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_1
            );
        }
        function::FADE_IN_LOAD_BG3_TILES_1 => {
            set_native_word(game, UNPAUSE_HOOK, UNPAUSE_DEAD);
            slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_2
            );
            queue_vram(game, 1024, 0xb200, QueueBank::Word(154), |_| 0x4000);
        }
        function::FADE_IN_LOAD_BG3_TILES_2 => {
            slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_3
            );
            queue_vram(game, 1024, 0xb600, QueueBank::Word(154), |_| 16896);
        }
        function::FADE_IN_LOAD_BG3_TILES_3 => {
            slot!(
                game,
                Body[BODY].function = function::FADE_IN_LOAD_BG3_TILES_4
            );
            queue_vram(game, 1024, 0xba00, QueueBank::Word(154), |_| 17408);
        }
        function::FADE_IN_LOAD_BG3_TILES_4 => {
            slot!(game, Body[BODY].function = function::FADE_IN_BG6);
            queue_vram(game, 1024, 0xbe00, QueueBank::Word(154), |_| 17920);
        }
        function::FADE_IN_BG6 => fade_in_bg6(game),
        function::FADE_IN_DEAD_WAS_ALIVE => {
            if is_dead(game) == 0 || word!(game.layer1_x_pos) != 0 {
                hide_as_dead(game);
            }
        }
        function::FADE_IN_DEAD_WAS_DEAD => {
            if is_dead(game) == 0 || word!(game.layer1_x_pos) != 0 {
                hide_as_dead(game);
            }
            word!(game.previous_layer1_x_block = -1i32 as u16);
        }
        function::RAISE_LOAD_TILEMAP_BOTTOM => {
            restrict_samus_x(game);
            slot!(game, Body[BODY].function = function::RAISE_QUAKE_EVERY_16);
            slot!(game, Body[BODY].timer = 120);
            word!(game.earthquake_timer = 496);
            game.queue_music_delayed(5);
            queue_bg2_bottom(game);
        }
        function::RAISE_QUAKE_EVERY_16 => {
            quake_until_timer(game, 0xf, function::RAISE_QUAKE_EVERY_8, 96)
        }
        function::RAISE_QUAKE_EVERY_8 => quake_until_timer(game, 7, function::RAISE, 288),
        function::RAISE => raise(game),
        function::GETS_BIG_DRAW_ROOM_BG => {
            slot!(
                game,
                Body[BODY].function = function::GETS_BIG_FADE_IN_ROOM_BG
            );
            slot!(game, Body[BODY]; mouth_hit_ready = 0, timer = 0);
            draw_room_bg(game);
        }
        function::MAIN_LOOP_THINKING => advance_thinking(game, function::ATTACK_WITH_MOUTH_OPEN),
        function::GETS_BIG_THINKING => advance_thinking(game, function::MOUTH_OPEN),
        function::MOUTH_OPEN => {
            if process_script(game) == 0xffff {
                slot!(game, Body[BODY].function = function::MAIN_LOOP_THINKING);
                respond_to_mouth_hit(game);
            }
        }
        function::INIT_EYE_GLOW => {
            slot!(game, Body[BODY].function = function::MOUTH_OPEN);
            slot!(game, Body[BODY].function = function::GLOW_EYE);
            slot!(game, Body[BODY].script = animation::SCRIPT_GLOW + 8);
            slot!(
                game,
                Body[BODY].script_delay = kraid_record(animation::SCRIPT_GLOW, game).word(0)
            );
            glow_eye(game);
        }
        function::UNGLOW_EYE => unglow_eye(game),
        function::ATTACK_WITH_MOUTH_OPEN => attack_with_mouth_open(game),
        function::GETS_BIG_RELEASE_CAMERA => {
            slot!(game, Body[BODY].function = function::GETS_BIG_BREAK_CEILING);
            set_native_word(game, SCROLL_LIMITS, 514);
            set_native_word(game, SCROLL_LIMITS + 2, 257);
            slot!(game, Body[BODY].min_eject_y = 164);
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
        function::DEATH_INIT => death_init(game),
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
            slot!(
                game,
                Body[BODY].function = function::RAISE_LOAD_TILEMAP_BOTTOM
            );
            queue_bg2_top(game);
        }
        function::LINT_CHARGE => charge_lint(game, k),
        function::FOOT_IDLE | function::NOTHING => {}
        _ => game.unreachable(),
    }
}

/// Kraid grows, shaking, and breaks the ceiling platforms. Was
/// `Kraid_GetsBig_BreakCeilingPlatforms`.
fn break_ceiling<H: Host + ?Sized>(game: &mut H) {
    if (word!(game.nmi_frame_counter_word) & 7) == 0 {
        quake_rocks(game);
    }
    let mut step: u16 = 1;
    if (slot!(game, Body[BODY].base.y_pos) & 2) != 0 {
        step = -1i32 as u16;
    }
    slot!(
        game,
        Body[BODY].base.x_pos = (slot!(game, Body[BODY].base.x_pos)).wrapping_add(step)
    );
    slot!(
        game,
        Body[BODY].base.y_pos = slot!(game, Body[BODY].base.y_pos).wrapping_sub(1)
    );
    // A platform breaks every four pixels risen, nine in all.
    if (slot!(game, Body[BODY].base.y_pos) & 3) == 0 {
        let platform: u16 = slot!(game, Body[BODY].timer);
        if (platform as i16).wrapping_sub(18) < 0 {
            let rock = crate::boss_lists::kraid_rock_x(game, (platform >> 1) as isize);
            let k = word!(game.cur_enemy_index);
            game.spawn_projectile(rock, k, CEILING_ROCKS);
            let broken = (slot!(game, Body[BODY].timer) >> 1) as usize;
            if let Some(&(block_x, plm)) = CEILING_PLATFORMS.get(broken) {
                game.spawn_plm(block_x, 0x12, plm);
            }
            slot!(
                game,
                Body[BODY].timer = (slot!(game, Body[BODY].timer)).wrapping_add(2)
            );
        }
    }
    if below(slot!(game, Body[BODY].base.y_pos), 296) {
        slot!(
            game,
            Body[BODY].function = function::GETS_BIG_SET_BG2_PRIORITY
        );
    }
}

/// Was `Kraid_GetsBig_SetBG2TilemapPrioBits`.
fn set_bg2_priority<H: Host + ?Sized>(game: &mut H) {
    for v0 in (0u16..4096).step_by(2) {
        word!(game.tilemap_stuff[(v0 >> 1)] |= 0x2000);
    }
    slot!(
        game,
        Part[ARM].base.properties = slot!(game, Part[ARM].base.properties) & !(0x400)
    );
    slot!(
        game,
        Body[BODY].function = function::GETS_BIG_FINISH_BG2_TILEMAP
    );
    queue_bg2_top(game);
}

/// Was `Kraid_GetsBig_FinishUpdateBg2Tilemap`.
fn finish_bg2_tilemap<H: Host + ?Sized>(game: &mut H) {
    slot!(game, Body[BODY].function = function::GETS_BIG_DRAW_ROOM_BG);
    slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
    slot!(
        game,
        Part[TOP_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    slot!(
        game,
        Part[MIDDLE_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    slot!(
        game,
        Part[BOTTOM_LINT].base.current_instruction = animation::LINT_SHOWN
    );
    slot!(
        game,
        Part[TOP_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    slot!(
        game,
        Part[MIDDLE_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    slot!(
        game,
        Part[BOTTOM_LINT].base.spritemap_pointer = animation::LINT_SPRITEMAP
    );
    queue_bg2_bottom(game);
}

/// Fades in the room, then starts the fight. Was `Kraid_GetsBig_FadeInRoomBg`.
fn fade_in_room_bg<H: Host + ?Sized>(game: &mut H) {
    if background6(game) & 1 != 0 {
        start_growing_thinking(game);
        enable_lint(game, TOP_LINT as u16, LINT_TIMERS[0]);
        enable_lint(game, MIDDLE_LINT as u16, LINT_TIMERS[1]);
        enable_lint(game, BOTTOM_LINT as u16, LINT_TIMERS[2]);
        slot!(game, Part[GOOD_FINGERNAIL].next = function::FINGERNAIL_INIT);
        slot!(game, Part[BAD_FINGERNAIL].next = function::FINGERNAIL_INIT);
        slot!(
            game,
            Part[GOOD_FINGERNAIL].function = function::HANDLE_FUNCTION_TIMER
        );
        slot!(
            game,
            Part[BAD_FINGERNAIL].function = function::HANDLE_FUNCTION_TIMER
        );
        slot!(game, Part[GOOD_FINGERNAIL].timer = 64);
        slot!(game, Part[BAD_FINGERNAIL].timer = 128);
        slot!(game, Part[ARM].x_offset = 1);
        slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        slot!(game, Body[BODY].target_x = 288);
        slot!(game, Part[FOOT].function = function::FOOT_WALK_TO_START);
        slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_WALK_BACKWARD);
    }
}

/// The eye glows until fully lit, then fades back. Was `Kraid_Shot_GlowHisEye`.
fn glow_eye<H: Host + ?Sized>(game: &mut H) {
    process_script(game);
    let mut lit: i16 = 0;
    for v0 in (226u16..232).step_by(2) {
        let c: i32 = v0 as i32 >> 1;
        let mut red: u16 = ((word!(game.palette_buffer[c]) as i32 & 0x1f) + 1) as u16;
        if (word!(game.palette_buffer[c]) as i32 & 0x1f) - 30i32 & 0x8000 == 0 {
            lit += 1;
            red = 31;
        }
        let mut green: u16 = ((word!(game.palette_buffer[c]) as i32 & 0x3e0) + 32) as u16;
        if (word!(game.palette_buffer[c]) as i32 & 0x3e0) - 960i32 & 0x8000 == 0 {
            lit += 1;
            green = 992;
        }
        word!(game.palette_buffer[c] = green | red | word!(game.palette_buffer[c]) & 0xfc00);
    }
    if lit.wrapping_sub(6) >= 0 {
        slot!(game, Body[BODY].function = function::UNGLOW_EYE);
    }
}

/// Counts down the part's timer, then runs `next` with the foot's walk. Was
/// `KraidEnemy_DecrementEnemyFunctionTimer`.
fn decrement_function_timer<H: Host + ?Sized>(game: &mut H) {
    let me = current(game);
    if slot!(game, Part[me].timer) != 0 {
        let left = slot!(game, Part[me].timer);
        slot!(
            game,
            Part[me].timer = slot!(game, Part[me].timer).wrapping_sub(1)
        );
        if left == 1 {
            slot!(game, Part[me].function = slot!(game, Part[me].next));
            slot!(game, Part[me].base; current_instruction = animation::FOOT_WALK_BACKWARD, instruction_timer = 1);
        }
    }
}

/// When the walk ends, picks a new target and walks toward it. Was
/// `KraidsFoot_SecondPhase_Thinking`.
fn foot_second_phase_thinking<H: Host + ?Sized>(game: &mut H) {
    let left: u16 = (slot!(game, Part[FOOT].next)).wrapping_sub(1);
    slot!(game, Part[FOOT].next = left);
    if left == 0 {
        let mut row: u16 = 0;
        loop {
            if slot!(game, Body[BODY].base.x_pos)
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
        let mut choice: i16 = (word!(game.random_number) as i16) & 0x1c;
        if !below(choice as u16, 16) {
            choice = 16;
        }
        let movement = (kraid_tables::foot_target(game, (row >> 2) as usize).movement)
            .wrapping_add(choice as u16);
        let walk = native_record(BANK, movement, game);
        let frames: u16 = walk.word(2);
        if (walk.word(0) as i16).wrapping_sub((slot!(game, Body[BODY].base.x_pos)) as i16) >= 0 {
            walk_backwards(game, frames, walk.word(0));
        } else {
            walk_forwards(game, frames, walk.word(0));
        }
    }
}

/// Launches a fingernail on a random pattern, from Kraid or held back off
/// screen. Was `KraidsFingernail_Init`.
fn initialize_fingernail_attack<H: Host + ?Sized>(game: &mut H) {
    let mut direction: u16 = slot!(game, Fingernail[GOOD_FINGERNAIL].y_speed);
    if word!(game.cur_enemy_index) == 384 {
        direction = slot!(game, Fingernail[BAD_FINGERNAIL].y_speed);
    }
    let pattern: u16 = if direction & 0x8000 != 0 {
        crate::boss_lists::kraid_nail_velocity(
            game,
            true,
            ((word!(game.random_number) as i32 & 6) >> 1) as isize,
        )
    } else {
        crate::boss_lists::kraid_nail_velocity(
            game,
            false,
            ((word!(game.random_number) as i32 & 6) >> 1) as isize,
        )
    };
    let speeds = native_record(BANK, pattern, game);
    let me = current(game);
    slot!(game, Fingernail[me]; x_subspeed = speeds.word(0), x_speed = speeds.word(2), y_subspeed = speeds.word(4), y_speed = speeds.word(6), launched = 1);
    slot!(
        game,
        Fingernail[me].base.properties = (slot!(game, Fingernail[me].base.properties) as i32
            & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16
    );
    slot!(game, Fingernail[me].base; instruction_timer = 1, current_instruction = animation::FINGERNAIL);
    slot!(game, Fingernail[me].function = function::FINGERNAIL_FIRE);
    let at_kraid: bool = ((word!(game.random_number) & 1) == 0)
        || ({
            let mut other: u16 = health_eighth(game, GOOD_FINGERNAIL, 1);
            if word!(game.cur_enemy_index) != 448 {
                other = health_eighth(game, BAD_FINGERNAIL, 1);
            }
            (other == 1)
                || ({
                    set_health_eighth(game, me, 1, 1);
                    slot!(game, Fingernail[me].base.x_pos = 50);
                    slot!(game, Fingernail[me].base.y_pos = 240);
                    slot!(game, Fingernail[me].x_subspeed = 0);
                    slot!(game, Fingernail[me].x_speed = 1);
                    slot!(game, Fingernail[me]; y_subspeed = 0, y_speed = 0);
                    slot!(
                        game,
                        Fingernail[me].function = function::FINGERNAIL_WAIT_FOR_LINT
                    );
                    slot!(game, Fingernail[me].next = function::FINGERNAIL_FIRE);
                    slot!(
                        game,
                        Fingernail[me].base.properties =
                            (slot!(game, Fingernail[me].base.properties) as i32
                                | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                                as u16
                    );
                    false
                })
        });
    if at_kraid {
        set_health_eighth(game, me, 1, 0);
        slot!(
            game,
            Fingernail[me].base.x_pos = (slot!(game, Fingernail[BODY].base.x_pos) as i32
                - slot!(game, Fingernail[BODY].base.x_width) as i32
                - slot!(game, Fingernail[me].base.x_width) as i32
                & 0xfff0) as u16
        );
        slot!(
            game,
            Fingernail[me].base.y_pos = (slot!(game, Fingernail[ARM].base.y_pos)).wrapping_add(128)
        );
    }
}

/// Was `KraidsFoot_PrepareToLungeForward`.
fn foot_prepare_lunge<H: Host + ?Sized>(game: &mut H) {
    first_phase(game);
    if !below(
        slot!(game, Part[ARM].base.current_instruction),
        animation::ARM_IDLE_LAST,
    ) {
        slot!(game, Part[ARM].base; current_instruction = animation::ARM_DYING, instruction_timer = 1);
        slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_LUNGE);
        slot!(game, Part[FOOT].function = function::FOOT_LUNGE_FORWARD);
        slot!(game, Part[FOOT].timer = 0);
    }
}

/// Was `KraidsFoot_FirstPhase_LungeForward`.
fn foot_lunge_forward<H: Host + ?Sized>(game: &mut H) {
    first_phase(game);
    if below(slot!(game, Body[BODY].base.x_pos), 92) {
        slot!(game, Body[BODY].base.x_pos = 92);
    }
    if slot!(game, Part[FOOT].base.current_instruction) == animation::FOOT_WALK_BACKWARD_END {
        if slot!(game, Body[BODY].base.x_pos) == 92 {
            slot!(game, Part[FOOT].next = function::FOOT_RETREAT_FROM_LUNGE);
            slot!(
                game,
                Part[FOOT].function = function::DECREMENT_FUNCTION_TIMER
            );
            slot!(game, Part[FOOT].timer = 1);
            slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
        } else {
            slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_LUNGE);
        }
    }
}

/// Was `KraidsFoot_FirstPhase_RetreatFromLunge`.
fn foot_retreat_from_lunge<H: Host + ?Sized>(game: &mut H) {
    first_phase(game);
    if !below(slot!(game, Body[BODY].base.x_pos), 176) {
        slot!(game, Body[BODY].base.x_pos = 176);
    }
    if !below(
        slot!(game, Part[FOOT].base.current_instruction),
        animation::FOOT_WALK_BACKWARD_LAST,
    ) {
        if slot!(game, Body[BODY].base.x_pos) == 176 {
            slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
            slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
            slot!(
                game,
                Part[FOOT].function = function::FOOT_FIRST_PHASE_THINKING
            );
            slot!(game, Part[FOOT].timer = 300);
            slot!(game, Part[FOOT].next = function::FOOT_PREPARE_LUNGE);
        } else {
            game.print_x_undefined();
            let me = current(game);
            slot!(game, Part[me].base; current_instruction = animation::FOOT_WALK_BACKWARD, instruction_timer = 1);
        }
    }
}

/// Fades out the room, then queues the dying tilemap. Was `Kraid_Death_Fadeout`.
fn death_fade_out<H: Host + ?Sized>(game: &mut H) {
    process_script(game);
    if background6(game) & 1 != 0 {
        slot!(game, Body[BODY].function = function::DEATH_BG2_TOP_HALF);
        slot!(game, Body[BODY]; hurt_frame_timer = 1, hurt_frame = 1);
        palette(game);
        crate::word_transfers!(game, (4), 128; xray(0) <- value(0), xray(1) <- value(0));
        queue_vram(game, 512, 0x4000, QueueBank::Byte(126), |game| {
            (((game.ram().read_byte(BG3_TILEMAP_BASE) as i32 & 0xf) << 8) + 16128) as u16
        });
    }
}

/// Was `Kraid_Death_UpdateBG2TilemapTopHalf`.
fn death_bg2_top_half<H: Host + ?Sized>(game: &mut H) {
    process_script(game);
    slot!(game, Body[BODY].function = function::DEATH_BG2_BOTTOM_HALF);
    for lint in [TOP_LINT, MIDDLE_LINT, BOTTOM_LINT] {
        slot!(game, Part[lint].function = function::ALIGN_TO_KRAID);
        slot!(game, Part[lint].timer = 0x7fff);
    }
    queue_bg2_top(game);
}

/// Starts sinking: the earthquake, the sinking unpause hook and the parts'
/// last animations. Was `Kraid_Death_UpdateBG2TilemapBottomHalf`.
fn death_bg2_bottom_half<H: Host + ?Sized>(game: &mut H) {
    process_script(game);
    game.ram().write_byte(UNPAUSE_HOOK + 2, BANK);
    set_native_word(game, UNPAUSE_HOOK, UNPAUSE_SINKING);
    slot!(
        game,
        Body[BODY].function = function::DEATH_SINK_THROUGH_FLOOR
    );
    word!(game.kraid_unk9000 = 43);
    slot!(
        game,
        Body[BODY].base.properties = (slot!(game, Body[BODY].base.properties)) | 0x8000
    );
    word!(game.earthquake_type = 1);
    word!(game.earthquake_timer = 256);
    slot!(game, Part[ARM].base; current_instruction = animation::ARM_RAISED, instruction_timer = 1);
    slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
    slot!(game, Part[FOOT].function = function::FOOT_IDLE);
    queue_bg2_bottom(game);
}

/// Sinks a pixel a frame; once gone, drops items and cleans up. Was
/// `Kraid_Death_SinkThroughFloor`.
pub fn sink_through_floor<H: Host + ?Sized>(game: &mut H) {
    process_script(game);
    sound_every_half_second(game);
    handle_sinking(game);
    slot!(
        game,
        Body[BODY].base.y_pos = slot!(game, Body[BODY].base.y_pos).wrapping_add(1)
    );
    if !below(slot!(game, Body[BODY].base.y_pos), 608) {
        slot!(
            game,
            Body[BODY].base.properties = (slot!(game, Body[BODY].base.properties) as i32
                & !(kEnemyProps_Tangible as i32)) as u16
        );
        word!(game.enemy_bg2_tilemap_size = 2);
        let enemy_ptr: u16 = slot!(game, Body[current(game)].base.enemy_ptr);
        game.set_shot_ai(enemy_ptr, NO_SHOT_AI);
        let deleted: u16 = (slot!(game, Part[ARM].base.properties) as i32
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Deleted as i32) as u16;
        slot!(game, Part[ARM].base.properties = deleted);
        let hidden: u16 = deleted & 0x51ff | 0x600;
        slot!(game, Part[TOP_LINT].base.properties = hidden);
        slot!(game, Part[MIDDLE_LINT].base.properties = hidden);
        slot!(game, Part[BOTTOM_LINT].base.properties = hidden);
        slot!(game, Part[FOOT].base.properties = hidden);
        slot!(game, Body[BODY].function = function::FADE_IN_CLEAR_BG2_TOP);
        word!(game.camera_distance_index = 0);
        game.scatter_enemy_drop(DropSource::Kraid, enemy_ptr);
        draw_room_bg(game);
    }
}

/// Was `Kraid_FadeInBg_ClearBg2TilemapTopHalf`.
fn fade_in_clear_bg2_top<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(BG2_TILEMAP_BASE, 72);
    crate::word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(BLANK_BG2_TILE));
    queue_vram(game, 1024, 0x2000, QueueBank::Word(126), |_| 18432);
    slot!(
        game,
        Body[BODY].function = function::FADE_IN_CLEAR_BG2_BOTTOM
    );
}

/// Fades the room back in, then sets the boss bit once. Was
/// `Kraid_FadeInBg_FadeInBp6`.
pub fn fade_in_bg6<H: Host + ?Sized>(game: &mut H) {
    game.observe_kraid_flags(0);
    if background6(game) & 1 != 0 {
        game.observe_kraid_flags(1);
        game.queue_music_delayed(3);
        game.observe_kraid_flags(2);
        let area = word!(game.area_index);
        let bits: i16 = game.boss_word_native(area) as i16;
        if (bits as u16 & 1) != 0 {
            slot!(game, Body[BODY].function = function::FADE_IN_DEAD_WAS_DEAD);
        } else {
            let area = word!(game.area_index);
            game.set_boss_word_native(area, (bits as u16) | 1);
            slot!(game, Body[BODY].function = function::FADE_IN_DEAD_WAS_ALIVE);
        }
    }
    game.observe_kraid_flags(3);
}

/// Kraid rises the rest of the way. Was `Kraid_Raise_Handler`.
fn raise<H: Host + ?Sized>(game: &mut H) {
    restrict_samus_x(game);
    if (word!(game.earthquake_timer) & 5) == 0 {
        quake_rocks(game);
    }
    slot!(
        game,
        Body[BODY].base.x_pos = (slot!(game, Body[BODY].base.x_pos) as i32
            + if (slot!(game, Body[BODY].base.y_pos) & 2) == 0 {
                -1i32
            } else {
                1
            }) as u16
    );
    slot!(game, Body[BODY].base(y_pos, y_subpos) += -0x8000i32 as u32);
    if below(slot!(game, Body[BODY].base.y_pos), 457) {
        slot!(game, Body[BODY].base.x_pos = 176);
        slot!(
            game,
            Part[FOOT].function = function::FOOT_FIRST_PHASE_THINKING
        );
        slot!(game, Part[FOOT].timer = 300);
        slot!(game, Part[FOOT].next = function::FOOT_PREPARE_LUNGE);
        slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        start_thinking(game);
        slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
    }
}

/// Places the lint by Kraid's height and picks a thinking time. Was
/// `Kraid_SetLintYAndRandomThinkTimer`.
fn place_lint_and_think<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Part[TOP_LINT].base.y_pos = (slot!(game, Body[BODY].base.y_pos)).wrapping_sub(20)
    );
    slot!(
        game,
        Part[MIDDLE_LINT].base.y_pos = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(46)
    );
    slot!(
        game,
        Part[BOTTOM_LINT].base.y_pos = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(112)
    );
    let mut think: i16 = (word!(game.random_number) as i16) & 7;
    if (word!(game.random_number) & 7) == 0 {
        think = 2;
    }
    slot!(
        game,
        Body[BODY].thinking_timer = ((think as i32) << 6) as u16
    );
}

/// Samus touching Kraid's body is pushed out and hurt. Was `Kraid_Enemy_Touch`.
fn body_touch<H: Host + ?Sized>(game: &mut H) {
    if below(slot!(game, Body[BODY].function), function::DEATH_INIT) {
        let right: u16 = (word!(game.samus_x_radius)).wrapping_add(word!(game.samus_x_pos));
        let depth: i16 = ((word!(game.samus_y_pos)) as i16)
            .wrapping_sub((slot!(game, Body[BODY].base.y_pos)) as i16);
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
        if ((slot!(game, Body[BODY].base.x_pos) as i32
            + kraid_tables::body_hitbox(game, (row >> 2) as usize).left as i32
            - right as i32) as i16)
            < 0
        {
            if !below(word!(game.samus_x_pos), 40) {
                word!(game.samus_x_pos = word!(game.samus_x_pos).wrapping_sub(8));
                word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
            }
            let mut eject: u16 = (word!(game.samus_y_pos)).wrapping_sub(8);
            if ((word!(game.samus_y_pos) as i32 - 8i32 - slot!(game, Body[BODY].min_eject_y) as i32)
                as i16)
                < 0
            {
                eject = slot!(game, Body[BODY].min_eject_y);
            }
            word!(game.samus_y_pos = eject);
            word!(game.samus_prev_y_pos = eject);
            push_samus(game);
            if word!(game.samus_invincibility_timer) == 0 {
                game.contact_reaction(Death::Finalize);
            }
        }
    }
}

/// Shots striking Kraid's body explode; a beam hit while thinking opens his
/// mouth. Was `Kraid_Shot_Body`.
fn body_shots<H: Host + ?Sized>(game: &mut H) {
    if below(
        slot!(game, Body[BODY].function),
        function::DEATH_SINK_THROUGH_FLOOR,
    ) {
        slot!(game, Body[BODY].mouth_hit_ready = 0);
        slot!(
            game,
            Body[BODY].mouth_flags = slot!(game, Body[BODY].mouth_flags) & !(1)
        );
        let mut hits: u16 = 0;
        let frame = kraid_record((slot!(game, Body[BODY].script)).wrapping_sub(8), game);
        let hitbox = kraid_record(frame.word(4), game);
        let right: u16 = (slot!(game, Body[BODY].base.x_pos)).wrapping_add(hitbox.word(0));
        let top: u16 = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(2));
        let mut bottom: u16 = (slot!(game, Body[BODY].base.y_pos)).wrapping_add(hitbox.word(6));
        if word!(game.projectile_counter) != 0 {
            let mut i: i32 = 2 * word!(game.projectile_counter) as i32;
            while i >= 0 {
                let p: i32 = i >> 1;
                let hit_body: bool = if word!(game.projectile_y_pos[p]) as i32
                    - word!(game.projectile_y_radius[p]) as i32
                    - 1i32
                    - bottom as i32
                    & 0x8000
                    != 0
                {
                    word!(game.projectile_y_radius[p]) as i32
                        + word!(game.projectile_y_pos[p]) as i32
                        - top as i32
                        & 0x8000
                        == 0
                        && word!(game.projectile_x_radius[p]) as i32
                            + word!(game.projectile_x_pos[p]) as i32
                            - right as i32
                            & 0x8000
                            == 0
                } else {
                    let q: i32 = i >> 1;
                    bottom = (word!(game.projectile_x_radius[q]))
                        .wrapping_add(word!(game.projectile_x_pos[q]));
                    let depth: i16 = ((word!(game.projectile_y_pos[q])) as i16)
                        .wrapping_sub((slot!(game, Body[BODY].base.y_pos)) as i16);
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
                    ((slot!(game, Body[BODY].base.x_pos) as i32
                        + kraid_tables::body_hitbox(game, (j >> 2) as usize).left as i32
                        - bottom as i32) as i16)
                        < 0
                };
                if hit_body {
                    projectile_explosion(game, i as u16);
                    let p: i32 = i >> 1;
                    word!(game.projectile_dir[p] |= 0x10);
                    if (word!(game.projectile_type[p]) & 0x10) != 0 {
                        slot!(
                            game,
                            Body[BODY].mouth_flags = (slot!(game, Body[BODY].mouth_flags)) | 1
                        );
                    }
                    hits = hits.wrapping_add(1);
                }
                i -= 2;
            }
        }
        if hits != 0 && slot!(game, Body[BODY].function) == function::MAIN_LOOP_THINKING {
            slot!(game, Body[BODY].function = function::INIT_EYE_GLOW);
            let flags: u16 = slot!(game, Body[BODY].mouth_flags);
            if (flags & 1) != 0 {
                slot!(game, Body[BODY].mouth_flags = flags | 0x302);
            }
        }
    }
}

/// Flashes when hurt and shows the health palette. Was `Kraid_Palette_Handling`.
fn palette<H: Host + ?Sized>(game: &mut H) {
    if below(slot!(game, Body[BODY].base.health), 1) {
        slot!(
            game,
            Body[BODY].hurt_frame = slot!(game, Body[BODY].base.health)
        );
    } else {
        let update: bool = (slot!(game, Body[BODY].hurt_frame) != 0)
            && ({
                let left: u16 = (slot!(game, Body[BODY].hurt_frame_timer)).wrapping_sub(1);
                slot!(game, Body[BODY].hurt_frame_timer = left);
                (left == 0)
                    && ({
                        slot!(game, Body[BODY].hurt_frame_timer = 2);
                        slot!(
                            game,
                            Body[BODY].hurt_frame =
                                slot!(game, Body[BODY].hurt_frame).wrapping_sub(1)
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
fn unglow_eye<H: Host + ?Sized>(game: &mut H) {
    let mut eighth: u16 = 14;
    let health: u16 = slot!(game, Body[BODY].base.health);
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
        let mut part: u16 = (word!(game.palette_buffer[c])) & 0x1f;
        let s: i32 = source as i32 >> 1;
        if kraid_tables::bg_color(game, (s + 1) as usize) & 0x1f != part {
            fading = fading.wrapping_add(1);
            word!(game.palette_buffer[c] = word!(game.palette_buffer[c]).wrapping_sub(1));
        }
        part = (word!(game.palette_buffer[c])) & 0x3e0;
        if kraid_tables::bg_color(game, (s + 1) as usize) & 0x3e0 != part {
            fading = fading.wrapping_add(1);
            word!(game.palette_buffer[c] = word!(game.palette_buffer[c]).wrapping_sub(32));
        }
        colour = colour.wrapping_add(2);
        source = source.wrapping_add(2);
        if (colour as i16).wrapping_sub(232) >= 0 {
            break;
        }
    }
    if fading == 0 {
        slot!(game, Body[BODY].function = function::MOUTH_OPEN);
        slot!(game, Body[BODY].script = animation::SCRIPT_IDLE);
        slot!(
            game,
            Body[BODY].script_delay = kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(0)
        );
    }
}

/// Roars with the mouth open, spitting rocks when it is widest. Was
/// `Kraid_Main_AttackWithMouthOpen`.
fn attack_with_mouth_open<H: Host + ?Sized>(game: &mut H) {
    if process_script(game) == 0xffff {
        start_thinking(game);
        respond_to_mouth_hit(game);
    } else if kraid_record((slot!(game, Body[BODY].script)).wrapping_sub(8), game).word(2)
        == animation::TILEMAP_3
        && (slot!(game, Body[BODY].script_delay) & 0xf) == 0
    {
        let parameter = crate::boss_lists::kraid_rock_spit_x_velocity(
            game,
            ((word!(game.random_number) as i32 & 0xe) >> 1) as isize,
        );
        let k = (word!(game.random_number)) & 0xe;
        game.spawn_projectile(parameter, k, SPIT_ROCKS);
        game.queue_sound(2, ROCKS_SOUND, 6);
    }
}

/// Walks back to the target, then stops: thinking again, or after `wait`
/// frames the second phase. Was `KraidsFoot_SecondPhase_WalkingBackwards` and
/// `KraidsFoot_SecondPhaseSetup_WalkToStartPt`.
fn foot_walk_backwards<H: Host + ?Sized>(game: &mut H, next: u16, wait: Option<(u16, u16)>) {
    let target: u16 = slot!(game, Body[BODY].target_x);
    if target != slot!(game, Body[BODY].base.x_pos) {
        if (target as i16).wrapping_sub((slot!(game, Body[BODY].base.x_pos)) as i16) >= 0 {
            return;
        }
        slot!(game, Body[BODY].base.x_pos = target);
    }
    if !below(
        slot!(game, Part[FOOT].base.current_instruction),
        animation::FOOT_WALK_BACKWARD_LAST,
    ) {
        slot!(game, Part[FOOT].function = next);
        if let Some((timer, then)) = wait {
            slot!(game, Part[FOOT].timer = timer);
            slot!(game, Part[FOOT].next = then);
        }
        slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
    }
}

/// Was `KraidsFoot_SecondPhase_WalkForward`.
fn foot_walk_forward<H: Host + ?Sized>(game: &mut H) {
    let target: u16 = slot!(game, Body[BODY].target_x);
    if (target as i16).wrapping_sub((slot!(game, Body[BODY].base.x_pos)) as i16) < 0 {
        if slot!(game, Part[FOOT].base.current_instruction) == animation::FOOT_WALK_FORWARD_END {
            slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_WALK_FORWARD, instruction_timer = 1);
        }
    } else {
        slot!(game, Body[BODY].base.x_pos = target);
        if slot!(game, Part[FOOT].base.current_instruction) == animation::FOOT_WALK_FORWARD_END {
            slot!(
                game,
                Part[FOOT].function = function::FOOT_SECOND_PHASE_THINKING
            );
            slot!(game, Part[FOOT].base; instruction_timer = 1, current_instruction = animation::FOOT_STAND);
        }
    }
}

/// Below six eighths of his health, Kraid stands up for the second phase. Was
/// `Kraid_HandleFirstPhase`.
fn first_phase<H: Host + ?Sized>(game: &mut H) {
    if ((slot!(game, Body[BODY].base.health)) as i16)
        .wrapping_sub(health_eighth(game, BODY, 6) as i16)
        < 0
    {
        slot!(
            game,
            Body[BODY].function = function::PROCESS_SCRIPT_AND_TIMER
        );
        slot!(game, Body[BODY].timer = 180);
        slot!(game, Body[BODY].next = function::GETS_BIG_RELEASE_CAMERA);
        let tilemap: u16 = kraid_record(slot!(game, Body[BODY].script), game).word(2);
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
        slot!(
            game,
            Body[BODY].script = animation::SCRIPT_IDLE.wrapping_add(frame)
        );
        slot!(
            game,
            Body[BODY].script_delay =
                kraid_record(animation::SCRIPT_IDLE_FIRST, game).word(usize::from(frame))
        );
        word!(game.earthquake_type = 4);
        word!(game.earthquake_timer = 340);
        slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
        slot!(game, Part[FOOT].function = function::FOOT_IDLE);
        slot!(game, Part[ARM].base; current_instruction = animation::ARM_IDLE, instruction_timer = 1);
        for lint in [TOP_LINT, MIDDLE_LINT, BOTTOM_LINT] {
            slot!(
                game,
                Part[lint].base.properties = (slot!(game, Part[lint].base.properties)) | 0x100
            );
        }
        slot!(
            game,
            Part[ARM].base.properties = (slot!(game, Part[ARM].base.properties)) | 0x400
        );
    }
}

/// Once the hurt flash ends: the dying palette, the parts die and the spikes
/// clear. Was `Kraid_Death_Init`.
fn death_init<H: Host + ?Sized>(game: &mut H) {
    if slot!(game, Body[BODY].hurt_frame) == 0 {
        crate::word_transfers!(game, ascending, 16; le(target_palettes, 96) <- value(0));
        crate::word_transfers!(game, descending, 16; le(palette_buffer, 127) <- rom(DEATH_PALETTE, 15));
        slot!(game, Part[ARM].base; current_instruction = animation::ARM_DYING, instruction_timer = 1);
        slot!(game, Body[BODY].function = function::DEATH_FADE_OUT);
        slot!(game, Body[BODY].script = animation::SCRIPT_DYING + 8);
        slot!(
            game,
            Body[BODY].script_delay = kraid_record(animation::SCRIPT_DYING, game).word(0)
        );
        let _actor: u16 = word!(game.cur_enemy_index);
        let saved: u16 = word!(game.cur_enemy_index);
        slot!(
            game,
            Part[GOOD_FINGERNAIL].base.properties =
                slot!(game, Part[GOOD_FINGERNAIL].base.properties) & !(0x4000)
        );
        word!(game.cur_enemy_index = 384);
        game.normal_death(0x180);
        slot!(
            game,
            Part[BAD_FINGERNAIL].base.properties =
                slot!(game, Part[BAD_FINGERNAIL].base.properties) & !(0x4000)
        );
        word!(game.cur_enemy_index = 448);
        game.normal_death(0x1c0);
        word!(game.cur_enemy_index = 128);
        game.normal_death(0x80);
        word!(game.cur_enemy_index = 192);
        game.normal_death(0xc0);
        word!(game.cur_enemy_index = 256);
        game.normal_death(0x100);
        word!(game.cur_enemy_index = saved);
        game.spawn_plm(0x5, 0x1b, DEATH_SPIKES_PLM);
    }
}

/// Arm contact: Samus is pushed back and the bottom lint fires. Was
/// `KraidsArm_Touch`.
pub fn arm_touch<H: Host + ?Sized>(game: &mut H) {
    if word!(game.samus_invincibility_timer) == 0 {
        push_samus(game);
        slot!(game, Part[BOTTOM_LINT].function = function::LINT_FIRE);
        touch(game);
    }
}

/// Body contact. Was `Kraid_Touch`.
pub fn touch<H: Host + ?Sized>(game: &mut H) {
    game.contact_reaction(Death::LeavePending);
}

/// Body shot. Was `Kraid_Shot`.
pub fn shot<H: Host + ?Sized>(game: &mut H) {
    game.shot_reaction(Shot::Finalize);
}

/// Sets up the room: the hooks, and either the empty room for a dead Kraid or
/// his health eighths, BG2 and the rise. Was `Kraid_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(UNPAUSE_HOOK + 2, BANK);
    game.ram().write_byte(PAUSE_HOOK + 2, BANK);
    set_native_word(game, UNPAUSE_HOOK, UNPAUSE_ALIVE);
    set_native_word(game, PAUSE_HOOK, PAUSE);
    if is_dead(game) != 0 {
        crate::word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(ROOM_BG_PALETTE, 0));
        crate::word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(BLANK_BG2_TILE));
        slot!(game, Body[BODY].field_4 = 0);
        queue_room_bg(game);
        game.spawn_plm(0x2, 0x12, CLEAR_CEILING_PLM);
        game.spawn_plm(0x5, 0x1b, CLEAR_SPIKES_PLM);
        slot!(game, Body[BODY].function = function::FADE_IN_CLEAR_BG2_TOP);
    } else {
        game.ram().write_byte(BG2_TILEMAP_BASE, 67);
        word!(game.camera_distance_index = 2);
        set_native_word(game, SCROLL_LIMITS, 0);
        set_native_word(game, SCROLL_LIMITS + 2, 1);
        slot!(game, Body[BODY].min_eject_y = 324);
        let mut eighth: u16 = slot!(game, Body[BODY].base.health) >> 3;
        let step: u16 = eighth;
        for slot in (0u16..16).step_by(2) {
            set_health_eighth(game, slot as i32, 0, eighth);
            eighth = eighth.wrapping_add(step);
        }
        load_bg2(game, 0xdfff);
        let quarter_at = |index: usize| slot!(@at Body, BODY, kraid_healths_4ths) + index * 2;
        let quarter: u16 = slot!(game, Body[BODY].base.health) >> 2;
        set_native_word(game, quarter_at(0), quarter);
        let half: u16 = native_word(game, quarter_at(0)).wrapping_add(quarter);
        set_native_word(game, quarter_at(1), half);
        let three: u16 = native_word(game, quarter_at(0)).wrapping_add(half);
        set_native_word(game, quarter_at(2), three);
        let all = native_word(game, quarter_at(0)).wrapping_add(three);
        set_native_word(game, quarter_at(3), all);
        set_native_word(game, slot!(@at Body, BODY, field_2E) + 8 * 2, 0);
        slot!(game, Body[BODY].base; x_pos = 176, y_pos = 592, properties = (slot!(game, Body[BODY].base.properties)) | (kEnemyProps_Tangible as u16));
        slot!(game, Body[BODY]; function = function::RESTRICT_SAMUS_X, timer = 300, next = function::RAISE_THROUGH_FLOOR, script_delay = 64);
        game.disable_minimap_for_boss();
        crate::word_transfers!(game, descending, 32; native(0x2ffe) <- value(BLANK_BG2_TILE));
        word!(game.earthquake_type = 5);
        crate::word_transfers!(game, ascending, 16; native(0xc360) <- rom(RISING_PALETTE, 0));
    }
}

/// Was `KraidsArm_Init`.
pub fn initialize_arm<H: Host + ?Sized>(game: &mut H) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        slot!(
            game,
            Part[ARM].base.palette_index = slot!(game, Part[BODY].base.palette_index)
        );
        slot!(game, Part[ARM].function = function::FOOT_IDLE);
        slot!(game, Part[ARM].base; current_instruction = animation::ARM_RAISED, instruction_timer = 1);
        slot!(game, Part[ARM].progress = 0);
    }
}

/// Sets up lint `lint` hidden in the belly at `x_offset`. Was
/// `KraidsTopLint_Init`, `KraidsMiddleLint_Init` and `KraidsBottomLint_Init`.
pub fn initialize_lint<H: Host + ?Sized>(game: &mut H, lint: i32) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        slot!(
            game,
            Part[lint].base.palette_index = slot!(game, Part[BODY].base.palette_index)
        );
        slot!(game, Part[lint].base; instruction_timer = 0x7fff, current_instruction = animation::LINT_HIDDEN);
        slot!(
            game,
            Part[lint].base.spritemap_pointer = animation::LINT_HIDDEN_SPRITEMAP
        );
        if lint == TOP_LINT {
            slot!(game, Part[lint]; function = function::NOTHING, timer = 0x7fff, x_offset = 0);
        } else {
            slot!(game, Part[lint].function = function::NOTHING);
            slot!(game, Part[lint].x_offset = -16i32 as u16);
        }
    }
}

/// Was `KraidsFoot_Init`.
pub fn initialize_foot<H: Host + ?Sized>(game: &mut H) {
    if is_dead(game) != 0 {
        hide_as_dead(game);
    } else {
        let palette: u16 = slot!(game, Body[BODY].base.palette_index);
        slot!(game, Part[FOOT].base.palette_index = palette);
        slot!(game, Part[FOOT].base; current_instruction = animation::FOOT_IDLE, instruction_timer = 1);
        slot!(game, Part[FOOT].function = function::FOOT_IDLE);
        slot!(game, Part[FOOT].next = 0);
    }
}

/// Mouth and body shots, the palette, contact and BG2's scroll, then the
/// body's function. Was `Kraid_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) {
    mouth_shots(game);
    palette(game);
    body_shots(game);
    body_touch(game);
    word!(
        game.reg_BG2HOFS = (slot!(game, Body[BODY].base.x_width) as i32
            + word!(game.reg_BG1HOFS) as i32
            - word!(game.bg1_x_offset) as i32
            - slot!(game, Body[BODY].base.x_pos) as i32) as u16
    );
    word!(
        game.reg_BG2VOFS = (word!(game.layer1_y_pos) as i32
            - slot!(game, Body[BODY].base.y_pos) as i32
            + 152) as u16
    );
    run_function(game, slot!(game, Body[BODY].function));
}

/// The arm follows the body and shows only on screen. Was `KraidsArm_Main`.
pub fn arm_main<H: Host + ?Sized>(game: &mut H) {
    let screen_bottom: u16 = (word!(game.layer1_y_pos)).wrapping_add(224);
    let y: u16 = (slot!(game, Body[BODY].base.y_pos)).wrapping_sub(44);
    slot!(game, Part[ARM].base.y_pos = y);
    let mut properties: u16 = (slot!(game, Part[ARM].base.properties)) | 0x100;
    if (y as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0
        && (y as i16).wrapping_sub(screen_bottom as i16) < 0
    {
        properties = (slot!(game, Part[ARM].base.properties)) & 0xfeff;
    }
    slot!(game, Part[ARM].base.properties = properties);
    slot!(
        game,
        Part[ARM].base.x_pos = slot!(game, Body[BODY].base.x_pos)
    );
    if slot!(byte game, Body[BODY].mouth_flags[1]) != 0 {
        slot!(
            game,
            Part[ARM].base.instruction_timer =
                slot!(game, Part[ARM].base.instruction_timer).wrapping_add(1)
        );
    }
}

/// A lint's main AI. Was `KraidsTopLint_Main`, `KraidsMiddleLint_Main` and
/// `KraidsBottomLint_Main`.
pub fn lint_main_ai<H: Host + ?Sized>(game: &mut H, lint: i32) {
    slot!(game, Part[lint].base.instruction_timer = 0x7fff);
    lint_main(game, lint as u16);
}

/// The foot follows the body and shows only on screen. Was `KraidsFoot_Main`.
pub fn foot_main<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Part[FOOT].base.x_pos = slot!(game, Part[BODY].base.x_pos)
    );
    let y: u16 = (slot!(game, Part[BODY].base.y_pos)).wrapping_add(100);
    slot!(game, Part[FOOT].base.y_pos = y);
    let above: i16 = (y as i16).wrapping_sub(224);
    let mut properties: u16 = (slot!(game, Part[FOOT].base.properties)) & 0xfeff;
    if (y as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
        if above.wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
            properties |= 0x100;
        }
    } else {
        properties |= 0x100;
    }
    slot!(game, Part[FOOT].base.properties = properties);
    run_function(game, slot!(game, Part[FOOT].function));
}

/// A fingernail touched: it shatters. Was `KraidsGoodFingernail_Touch`.
pub fn fingernail_touch<H: Host + ?Sized>(game: &mut H) {
    game.print_x_undefined();
    game.contact_reaction(Death::Finalize);
    let _actor = word!(game.cur_enemy_index);
    game.normal_death(0);
}

/// A fingernail's main: gone with Kraid, else its function. Was
/// `KraidsGoodFingernail_Main` and `KraidsBadFingernail_Main`.
pub fn fingernail_main<H: Host + ?Sized>(game: &mut H, nail: i32) {
    if below(slot!(game, Part[BODY].base.health), 1) {
        slot!(
            game,
            Part[nail].base.properties = (slot!(game, Part[nail].base.properties) as i32
                | (kEnemyProps_Deleted as i32 | kEnemyProps_Invisible as i32))
                as u16
        );
    } else {
        run_function(game, slot!(game, Part[nail].function));
    }
}

/// Was `KraidsGoodFingernail_Init` and `KraidsBadFingernail_Init`.
pub fn initialize_fingernail<H: Host + ?Sized>(game: &mut H) {
    kraid_fingernails::initialize(word!(game.cur_enemy_index), game);
}

/// Instruction: below five eighths of health the arm swipes. Was `Kraid_Instr_9`.
pub fn arm_swipe_when_hurt<H: Host + ?Sized>(game: &mut H) -> Option<u16> {
    if ((slot!(game, Body[BODY].base.health)) as i16)
        .wrapping_sub(health_eighth(game, BODY, 3) as i16)
        < 0
        && below(
            slot!(game, Part[ARM].base.current_instruction),
            animation::ARM_SWIPE,
        )
    {
        return Some(animation::ARM_SWIPE);
    }
    None
}

/// Instruction: Kraid steps up a pixel. Was `Kraid_Instr_DecYpos`.
pub fn step_up<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Body[BODY].base.y_pos = slot!(game, Body[BODY].base.y_pos).wrapping_sub(1)
    );
}

/// Instruction: Kraid steps down a pixel, shaking the room. Was
/// `Kraid_Instr_IncrYpos_Shake`.
pub fn step_down_and_shake<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Body[BODY].base.y_pos = slot!(game, Body[BODY].base.y_pos).wrapping_add(1)
    );
    word!(game.earthquake_type = 1);
    word!(game.earthquake_timer = 10);
}

/// Instruction: the stomp. Was `Kraid_Instr_PlaySound_0x76`.
pub fn stomp_sound<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(1, STOMP_SOUND, 6);
}

/// Instruction: Kraid steps back. Was `Kraid_Instr_XposMinus3`/`3b`.
pub fn step_back<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Body[BODY].base.x_pos = (slot!(game, Body[BODY].base.x_pos)).wrapping_sub(FOOT_STEP_BACK)
    );
}

/// Instruction: Kraid steps forward. Was `Kraid_Instr_XposPlus3`.
pub fn step_forward<H: Host + ?Sized>(game: &mut H) {
    slot!(
        game,
        Body[BODY].base.x_pos =
            (slot!(game, Body[BODY].base.x_pos)).wrapping_add(FOOT_STEP_FORWARD)
    );
}

/// Instruction: Kraid lunges right until blocked, shaking the room. Was
/// `Kraid_Instr_MoveHimRight`.
pub fn lunge_right<H: Host + ?Sized>(game: &mut H) {
    if (below(slot!(game, Body[BODY].base.x_pos), 320) || {
        let left: u16 = (slot!(game, Body[BODY].target_x)).wrapping_sub(1);
        slot!(game, Body[BODY].target_x = left);
        slot!(game, Body[BODY].target_x) == 0
    }) && game.move_horizontal(0, (PUSH_RIGHT as i16 as i32) << 16, Slopes::Ignore)
    {
        word!(game.earthquake_type = 0);
        word!(game.earthquake_timer = 7);
        let x: u16 = slot!(game, Body[BODY].base.x_pos);
        slot!(game, Part[FOOT].base.x_pos = x);
    }
}
