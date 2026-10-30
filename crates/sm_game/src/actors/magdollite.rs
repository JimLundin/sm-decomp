//! Magdollite, Norfair's lava creature: a head that surfaces from the lava, an
//! arm that grows towards Samus and a hand that throws lava at her.
//!
//! Was `NorfairLavaMan` in the original decompilation (bank $A8:AC9C-$A8:B40C).
//! The three parts occupy consecutive enemy slots, head first. Each part reads
//! and writes its neighbours' slot fields; the views below name those fields
//! over the same RAM bytes.
#![forbid(unsafe_code)]
use crate::actors::host::{Axis, Death, Shot};
use crate::actors::layouts::part_view;
use crate::constants::enemy_properties::{K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE};
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::enemy_speeds::SpeedTable;
use sm_data::magdollite::{ARM_GROWTH_ADDRESS, ARM_INSTRUCTION_ADDRESS, ARM_Y_OFFSET_ADDRESS};

/// What Magdollite needs beyond the shared actor operations: its tables.
pub trait Host:
    crate::actors::host::ActorHost + crate::actors::host::ActorRuntime + crate::effects::SoundEffects
{
    /// Colours 9-12 of the glow palette whose colour 0 is word `source`.
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4];
    /// Arm step `index`'s field at `address` (growth, instruction list or y offset).
    fn magdollite_arm(&self, address: u32, index: isize) -> u16;
}

/// The size of one enemy spawn record (`EnemySpawnData`).
pub const SPAWN_DATA_BYTES: usize = 64;

/// Bytes between consecutive enemy slots: head, arm, hand.
const SLOT: i32 = 64;

part_view! {
    /// The words every part uses.
    Part: EnemyLayout40 {
        words: [var_a, var_b, shown_animation, animation, var_e, state, kind, parameters],
        pad1: [u8; 26696],
        words: [
            var_00, animation_active, var_02, home_y, cooldown, var_05, var_06, shrink_subspeed,
            shrink_speed, grow_subspeed, grow_speed, var_0b, var_0c, var_0d, var_0e, var_0f, var_10,
            var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19, var_1a, var_1b,
            var_1c, var_1d, var_1e, var_1f,
        ],
    }
}

part_view! {
    /// The head, which surfaces, idles facing Samus and submerges.
    Head: EnemyLayout40 {
        words: [var_a, var_b, shown_animation, animation, var_e, state, kind, parameters],
        pad1: [u8; 26696],
        words: [
            facing_right, animation_active, waiting_for_arm, home_y, cooldown, var_05, var_06,
            shrink_subspeed, shrink_speed, grow_subspeed, grow_speed, var_0b, var_0c, var_0d,
            var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18,
            var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
    }
}

part_view! {
    /// The arm, which grows one segment at a time up to Samus's height.
    Arm: EnemyLayout40 {
        words: [
            var_a, growth_level, shown_animation, animation, extension, state, kind, parameters,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, animation_active, var_02, home_y, cooldown, var_05, var_06, shrink_subspeed,
            shrink_speed, grow_subspeed, grow_speed, var_0b, fully_grown, hold_retracted, var_0e,
            var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18, var_19,
            var_1a, var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
    }
}

part_view! {
    /// The hand, which rides the arm's tip and throws lava.
    Hand: EnemyLayout40 {
        words: [var_a, throw_right, shown_animation, animation, var_e, state, kind, parameters],
        pad1: [u8; 26696],
        words: [
            var_00, animation_active, var_02, home_y, cooldown, var_05, var_06, shrink_subspeed,
            shrink_speed, grow_subspeed, grow_speed, var_0b, var_0c, var_0d, var_0e, var_0f, var_10,
            home_x, throw_x, throw_y, var_14, var_15, var_16, var_17, var_18, var_19, var_1a,
            var_1b, var_1c, var_1d, var_1e, var_1f,
        ],
    }
}

/// Bank $A8 instruction lists.
mod animation {
    pub const HEAD_IDLE_LEFT: u16 = 0xac9c;
    pub const HAND_THROW_LEFT: u16 = 0xacb0;
    pub const HEAD_SUBMERGE_LEFT: u16 = 0xacde;
    pub const HEAD_EMERGE_LEFT: u16 = 0xad0c;
    pub const HEAD_IDLE_RIGHT: u16 = 0xad3c;
    pub const HAND_THROW_RIGHT: u16 = 0xad50;
    pub const HEAD_SUBMERGE_RIGHT: u16 = 0xad7e;
    pub const HEAD_EMERGE_RIGHT: u16 = 0xadac;
    pub const ARM_GROWTH_0: u16 = 0xaddc;
    pub const HAND_STUB: u16 = 0xae0c;
}

/// Bank $A8 state functions, stored in each part's `state` word.
pub mod state {
    pub const HEAD_IDLE: u16 = 0xb11a;
    pub const HEAD_WAIT_SUBMERGED: u16 = 0xb175;
    pub const HEAD_EMERGE_WHEN_READY: u16 = 0xb193;
    pub const HEAD_WAIT_EMERGED: u16 = 0xb1b8;
    pub const ARM_WAIT_HEAD_SUBMERGED: u16 = 0xb1dd;
    pub const ARM_GROWING: u16 = 0xb204;
    pub const ARM_GROWN: u16 = 0xb291;
    pub const ARM_SHRINKING: u16 = 0xb295;
    pub const HAND_IDLE: u16 = 0xb30d;
    pub const HAND_THROW_AND_SHRINK: u16 = 0xb31f;
    pub const HAND_GROW_AND_THROW: u16 = 0xb356;
    pub const HAND_FOLLOW_SHRINKING_ARM: u16 = 0xb3a7;
}

/// Bank $A8 instructions the head's and hand's scripts call.
pub mod instruction {
    pub const QUEUE_SOUND_IF_ON_SCREEN: u32 = 0xa8_ae12;
    pub const HEAD_DOWN_2PX: u32 = 0xa8_ae26;
    pub const HEAD_UP_2PX: u32 = 0xa8_ae30;
    pub const SET_ANIMATION_ACTIVE: u32 = 0xa8_ae3a;
    pub const CLEAR_ANIMATION_ACTIVE: u32 = 0xa8_ae45;
    pub const HEAD_AND_ARM_UP_1PX: u32 = 0xa8_ae50;
    pub const HEAD_AND_ARM_DOWN_1PX: u32 = 0xa8_ae5a;
    pub const SUBMERGED_POSITION_SHOW_ARM: u32 = 0xa8_ae64;
    pub const RESET_POSITION: u32 = 0xa8_ae88;
    pub const EMERGED_POSITION_HIDE_ARM: u32 = 0xa8_ae96;
    pub const SPAWN_LAVA: u32 = 0xa8_aeba;
    pub const OPEN_HAND_RIGHT: u32 = 0xa8_aeca;
    pub const OPEN_HAND_LEFT: u32 = 0xa8_aee4;
    pub const CLOSED_HAND_RIGHT: u32 = 0xa8_aefe;
    pub const CLOSED_HAND_LEFT: u32 = 0xa8_af18;
    pub const RESET_COOLDOWN: u32 = 0xa8_af44;
}

/// Bank $A8 reactions to Samus's weapons and touch.
pub mod reaction {
    pub const POWER_BOMB: u32 = 0xa8_b400;
    pub const CONTACT: u32 = 0xa8_b406;
    pub const SHOT: u32 = 0xa8_b40c;
}

/// $7E:178C, the enemy graphics drawn hook (a long pointer), and the hook
/// Magdollite installs there: its palette cycle at $A8:B0B2.
const GFX_DRAWN_HOOK: usize = 0x178c;
const PALETTE_CYCLE_HOOK: u16 = 0xb0b2;
const AI_BANK: u16 = 0xa8;

/// The graphics drawn hook's variables: palette buffer byte offset, cycle frame, timer.
const HOOK_PALETTE: usize = 0;
const HOOK_FRAME: usize = 1;
const HOOK_TIMER: usize = 2;
const PALETTE_CYCLE_FRAMES: u16 = 8;
/// `palette_index` bits 9-11 select a sprite palette; sprite palettes start
/// at byte 256 of the palette buffer and are 32 bytes each.
const SPRITE_PALETTE_BITS: u16 = 0x0e00;
const SPRITE_PALETTES_BYTE: u16 = 256;
const GLOW_PALETTES: u16 = 4;
const COLORS_PER_PALETTE: isize = 16;
const FIRST_GLOW_COLOR: usize = 9;

/// `parameters` lanes: the head's throw range and every part's speed entry.
const THROW_RANGE_LANE: usize = 0;
const SPEED_LANE: usize = 1;
const SPEED_ENTRY_WORDS: isize = 4;

/// AI handler bit 2: run the frozen AI.
const FROZEN_AI: u16 = 4;
/// Arm and hand start below the head, under the lava.
const START_DEPTH: u16 = 32;
/// Head heights above `home_y`: submerged with the arm out, and emerged.
const SUBMERGED_DEPTH: u16 = 24;
const EMERGED_DEPTH: u16 = 4;
/// Each arm growth level is one 8-pixel segment, two `growth_level` steps.
const SEGMENT_PIXELS: u16 = 8;
/// Arm extensions from this many pixels report a warning.
const ARM_WARNING_EXTENSION: i32 = 108;
/// The head's hitbox reaches the hand, plus this margin, but no less than the minimum.
const HITBOX_MARGIN: u16 = 2;
const MIN_HITBOX_HEIGHT: u16 = 8;
/// Frames between throws.
const THROW_COOLDOWN: u16 = 256;
/// Hand offsets from its throw position: horizontally, and raised when open and closed.
const HAND_REACH: u16 = 8;
const OPEN_HAND_RAISE: u16 = 4;
const CLOSED_HAND_RAISE: u16 = 8;

// The original reads some neighbour fields by other RAM names. From the hand's
// slot, these word indices reach the head's y position, hitbox height and
// state and the arm's y position (`enemy_drawing_queue`, $7E:0E84) and the
// arm's growth level (`enemy_drawing_queue_sizes`, $7E:0F68).
const HEAD_Y_POS: usize = 61;
const HEAD_Y_HEIGHT: usize = 64;
const HEAD_STATE: usize = 87;
const ARM_Y_POS: usize = 93;
const ARM_GROWTH_LEVEL: usize = 1;
const _: () = {
    use crate::ram::word_arrays::{enemy_drawing_queue, enemy_drawing_queue_sizes};
    use core::mem::offset_of;
    const fn from_hand(neighbour: i32, field: usize) -> usize {
        (0x0f78 + neighbour * SLOT) as usize + field
    }
    assert!(enemy_drawing_queue + 2 * HEAD_Y_POS == from_hand(-2, offset_of!(Head, base.y_pos)));
    assert!(
        enemy_drawing_queue + 2 * HEAD_Y_HEIGHT == from_hand(-2, offset_of!(Head, base.y_height))
    );
    assert!(enemy_drawing_queue + 2 * HEAD_STATE == from_hand(-2, offset_of!(Head, state)));
    assert!(enemy_drawing_queue + 2 * ARM_Y_POS == from_hand(-1, offset_of!(Arm, base.y_pos)));
    assert!(
        enemy_drawing_queue_sizes + 2 * ARM_GROWTH_LEVEL
            == from_hand(-1, offset_of!(Arm, growth_level))
    );
};

/// The current enemy's slot, read afresh at each use as the original did.
fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.cur_enemy_index) as i32
}
fn is_negative(value: u16) -> bool {
    (value as i16) < 0
}
/// Whether the low 16 bits of `value` read as negative, as the original's word test did.
fn negative_16(value: i32) -> bool {
    (value as i16) < 0
}
fn player_left<H: Host + ?Sized>(game: &mut H) -> bool {
    let k = word!(game.cur_enemy_index);
    is_negative(game.player_delta(k, Axis::X))
}
/// Start the part's requested animation if it isn't the one showing.
fn animate<H: Host + ?Sized>(game: &mut H) {
    let part = current(game);
    let requested = slot!(game, Part[part].animation);
    if requested != slot!(game, Part[part].shown_animation) {
        slot!(game, Part[part].base.current_instruction = requested);
        slot!(game, Part[part].shown_animation = requested);
        slot!(game, Part[part].base; instruction_timer = 1, timer = 0);
    }
}
fn initialize_head<H: Host + ?Sized>(game: &mut H) {
    let head = current(game);
    slot!(game, Head[head]; shown_animation = 0, facing_right = 0, waiting_for_arm = 0);
    slot!(game, Head[head].home_y = slot!(game, Head[head].base.y_pos));
    // Set from the opposite test; the first idle frame replaces it before use.
    if player_left(game) {
        slot!(game, Head[head].facing_right = 1);
    }
    slot!(game, Head[head].animation = animation::HEAD_IDLE_LEFT);
    if slot!(game, Head[head].facing_right) == 0 {
        slot!(game, Head[head].animation = animation::HEAD_IDLE_RIGHT);
    }
    animate(game);
    slot!(game, Head[head].state = state::HEAD_IDLE);
}
fn initialize_arm<H: Host + ?Sized>(game: &mut H) {
    let arm = current(game);
    slot!(game, Arm[arm].home_y = slot!(game, Arm[arm].base.y_pos));
    slot!(game, Arm[arm]; shown_animation = 0, fully_grown = 0, hold_retracted = 1,
        animation = animation::ARM_GROWTH_0);
    animate(game);
    slot!(game, Arm[arm].base.y_pos, |old| old
        .wrapping_add(START_DEPTH));
    slot!(game, Arm[arm].state = state::ARM_WAIT_HEAD_SUBMERGED);
    slot!(game, Arm[arm].base.properties, |old| old
        | K_ENEMY_PROPS_INVISIBLE as u16);
}
fn initialize_hand<H: Host + ?Sized>(game: &mut H) {
    let hand = current(game);
    slot!(game, Hand[hand].home_y = slot!(game, Hand[hand].base.y_pos));
    slot!(game, Hand[hand].home_x = slot!(game, Hand[hand].base.x_pos));
    slot!(game, Hand[hand]; shown_animation = 0, cooldown = 0, animation = animation::HAND_STUB);
    animate(game);
    slot!(game, Hand[hand].base.y_pos, |old| old
        .wrapping_add(START_DEPTH));
    slot!(game, Hand[hand].state = state::HAND_IDLE);
    slot!(game, Hand[hand].base.properties, |old| old
        | K_ENEMY_PROPS_INVISIBLE as u16);
}
/// Initialisation AI ($A8:AF8B): set up the part `kind` names, then its speeds
/// and the shared palette cycle.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let kind = 2u16.wrapping_mul(slot!(game, Part[current(game)].kind)) >> 1;
    let initialize: [fn(&mut H); 3] = [initialize_head, initialize_arm, initialize_hand];
    initialize[kind as usize](game);
    let part = current(game);
    let speed = SPEED_ENTRY_WORDS * slot!(byte game, Part[part].parameters[SPEED_LANE]) as isize;
    slot!(game, Part[part];
        shrink_speed = game.enemy_speed(SpeedTable::Linear, speed),
        shrink_subspeed = game.enemy_speed(SpeedTable::Linear, speed + 1),
        grow_speed = game.enemy_speed(SpeedTable::Linear, speed + 2),
        grow_subspeed = game.enemy_speed(SpeedTable::Linear, speed + 3));
    let part = current(game);
    game.ram()
        .copy_in(GFX_DRAWN_HOOK, &PALETTE_CYCLE_HOOK.to_ne_bytes());
    game.ram()
        .copy_in(GFX_DRAWN_HOOK + 2, &AI_BANK.to_ne_bytes());
    word!(
        game.variables_for_enemy_graphics_drawn_hook[HOOK_PALETTE] =
            ((slot!(game, Part[part].base.palette_index) & SPRITE_PALETTE_BITS) >> 4)
                + SPRITE_PALETTES_BYTE
    );
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] = PALETTE_CYCLE_FRAMES);
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_FRAME] = 0);
}
/// Graphics drawn hook ($A8:B0B2): every 8 frames, show the next glow palette.
pub fn cycle_palette<H: Host + ?Sized>(game: &mut H) {
    if word!(game.door_transition_flag_enemies) != 0
        || word!(
            game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] -= 1,
            reread
        ) != 0
    {
        return;
    }
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] = PALETTE_CYCLE_FRAMES);
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_FRAME] += 1);
    let frame = word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_FRAME]);
    let source = COLORS_PER_PALETTE * (frame & (GLOW_PALETTES - 1)) as isize;
    let colors: [u16; 4] = game.magdollite_glow_colors(source);
    let target = word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_PALETTE]) as usize >> 1;
    for (i, color) in colors.into_iter().enumerate() {
        word!(game.palette_buffer[target + FIRST_GLOW_COLOR + i] = color);
    }
}
/// Main AI ($A8:B10A): count down the part's cooldown and name its state function.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) -> u32 {
    let part = current(game);
    slot!(game, Part[part].cooldown, |old| old.wrapping_sub(1));
    u32::from(AI_BANK) << 16 | u32::from(slot!(game, Part[part].state))
}
/// The head's reaction to a power bomb, contact or shot, shared with the arm and hand.
pub fn damage<H: Host + ?Sized>(address: u32, game: &mut H) {
    match address {
        reaction::POWER_BOMB => game.power_bomb_reaction(Death::Finalize),
        reaction::CONTACT => game.contact_reaction(Death::Finalize),
        reaction::SHOT => game.shot_reaction(Shot::Finalize),
        _ => unreachable!("Magdollite reaction {address:06x}"),
    }
    let head = current(game);
    let arm = current(game) + SLOT;
    let hand = current(game) + 2 * SLOT;
    if slot!(game, Head[head].base.health) == 0 {
        slot!(game, Arm[arm].base.properties, |old| old
            | K_ENEMY_PROPS_DELETED as u16);
        slot!(game, Hand[hand].base.properties, |old| old
            | K_ENEMY_PROPS_DELETED as u16);
    }
    let frozen = slot!(game, Head[head].base.frozen_timer);
    if frozen != 0 {
        slot!(game, Arm[arm].base.frozen_timer = frozen);
        slot!(game, Arm[arm].base.ai_handler_bits, |old| old | FROZEN_AI);
        slot!(game, Hand[hand].base.frozen_timer = frozen);
        slot!(game, Hand[hand].base.ai_handler_bits, |old| old | FROZEN_AI);
    }
}
/// Face the head towards Samus with the left or right animation.
fn face_samus<H: Host + ?Sized>(head: i32, left: u16, right: u16, game: &mut H) {
    slot!(game, Head[head].animation = left);
    if !player_left(game) {
        slot!(game, Head[head].animation = right);
    }
    animate(game);
}
/// Stretch the head's hitbox up to the hand, through the original RAM names.
fn stretch_head_hitbox<H: Host + ?Sized>(hand: u16, game: &mut H) {
    let index = hand as usize >> 1;
    let height = word!(game.enemy_drawing_queue[index + HEAD_Y_POS])
        .wrapping_sub(slot!(game, Hand[hand as i32].base.y_pos))
        .wrapping_add(HITBOX_MARGIN);
    word!(game.enemy_drawing_queue[index + HEAD_Y_HEIGHT] = height);
    if is_negative(height.wrapping_sub(MIN_HITBOX_HEIGHT)) {
        word!(game.enemy_drawing_queue[index + HEAD_Y_HEIGHT] = MIN_HITBOX_HEIGHT);
    }
}
/// Growing arm ($A8:B204), part one: move up at the head's grow speed. Returns
/// the arm and, when it has reached the warning extension, that extension.
pub fn grow_arm<H: Host + ?Sized>(game: &mut H) -> (u16, Option<i32>) {
    let arm = current(game);
    let head = current(game) - SLOT;
    let value = u32::from(slot!(game, Head[head].grow_subspeed))
        + u32::from(slot!(game, Arm[arm].base.y_subpos));
    slot!(game, Arm[arm].base.y_subpos = value as u16);
    slot!(game, Arm[arm].base.y_pos, |old| (u32::from(old)
        + u32::from(slot!(game, Head[head].grow_speed))
        + (value >> 16))
        as u16);
    slot!(game, Arm[arm].extension, |old| (u32::from(old)
        + u32::from(slot!(game, Head[head].grow_speed))
        + (value >> 16))
        as u16);
    let k = word!(game.cur_enemy_index);
    let reach = |game: &mut H| -(slot!(game, Arm[k as i32].extension) as i32);
    let warning = if !negative_16(reach(game) - ARM_WARNING_EXTENSION) {
        Some(-(slot!(game, Arm[k as i32].extension) as i16 as i32))
    } else {
        None
    };
    (k, warning)
}
/// Growing arm, part two: once at Samus's height (or the warning extension),
/// stop and let the hand throw; otherwise add a segment at each growth level.
pub fn finish_arm_growth<H: Host + ?Sized>(k: u16, game: &mut H) {
    let arm = k as i32;
    let level = slot!(game, Arm[arm].growth_level);
    let reach = |game: &mut H| -(slot!(game, Arm[arm].extension) as i32);
    let mut step = 0isize;
    let at_samus = !negative_16(reach(game) - ARM_WARNING_EXTENSION) || {
        step = (level >> 1) as isize;
        ((slot!(game, Arm[arm].base.y_pos) as i32
            - game.magdollite_arm(ARM_Y_OFFSET_ADDRESS, step) as i32
            - game.player_y() as i32) as i16)
            < 0
    };
    let next_level = if at_samus {
        slot!(game, Arm[arm]; state = state::ARM_GROWN, fully_grown = 1);
        if ((reach(game) - game.magdollite_arm(ARM_GROWTH_ADDRESS, (level >> 1) as isize) as i32)
            as i16)
            < 0
        {
            return;
        }
        true
    } else {
        (reach(game) - game.magdollite_arm(ARM_GROWTH_ADDRESS, step) as i32) as i16 >= 0
    };
    if next_level {
        slot!(game, Arm[arm].growth_level, |old| old.wrapping_add(1));
        slot!(game, Arm[arm].growth_level, |old| old.wrapping_add(1));
        slot!(game, Arm[arm].base.y_pos, |old| old
            .wrapping_add(SEGMENT_PIXELS));
        slot!(
            game,
            Arm[arm].animation = game.magdollite_arm(
                ARM_INSTRUCTION_ADDRESS,
                (slot!(game, Arm[arm].growth_level) >> 1) as isize
            )
        );
        animate(game);
    }
}
/// Shrinking arm ($A8:B295): move down at the head's shrink speed, removing a
/// segment at each growth level, until retracted; then release the head.
pub fn shrink_arm<H: Host + ?Sized>(game: &mut H) {
    let arm = current(game);
    let head = current(game) - SLOT;
    let carry = slot!(game, Head[head].shrink_subspeed)
        .overflowing_add(slot!(game, Arm[arm].base.y_subpos))
        .1;
    slot!(
        game,
        Arm[arm].base.y_subpos = slot!(game, Head[head].shrink_subspeed)
            .wrapping_add(slot!(game, Arm[arm].base.y_subpos))
    );
    slot!(game, Arm[arm].base.y_pos, |old| (old as i32
        + slot!(game, Head[head].shrink_speed) as i32
        + i32::from(carry))
        as u16);
    slot!(game, Arm[arm].extension, |old| (old as i32
        + slot!(game, Head[head].shrink_speed) as i32
        + i32::from(carry))
        as u16);
    let k = word!(game.cur_enemy_index);
    let arm = k as i32;
    let extension = slot!(game, Arm[arm].extension) as i16;
    if extension >= 0 {
        slot!(game, Arm[arm].hold_retracted = 1);
        let at = crate::ram_slot!(@at Head, arm - SLOT, waiting_for_arm);
        game.ram().copy_in(at, &0u16.to_ne_bytes());
        slot!(game, Arm[arm].state = state::ARM_WAIT_HEAD_SUBMERGED);
    } else if ((-(extension as i32)
        - game.magdollite_arm(
            ARM_GROWTH_ADDRESS,
            (slot!(game, Arm[arm].growth_level).wrapping_sub(2) >> 1) as isize,
        ) as i32) as i16)
        < 0
    {
        slot!(game, Arm[arm].growth_level, |old| old.wrapping_sub(2));
        slot!(game, Arm[arm].base.y_pos, |old| old
            .wrapping_sub(SEGMENT_PIXELS));
        slot!(
            game,
            Arm[arm].animation = game.magdollite_arm(
                ARM_INSTRUCTION_ADDRESS,
                (slot!(game, Arm[arm].growth_level) >> 1) as isize
            )
        );
        animate(game);
    }
}
/// Run the current part's state function `state` (one of [`state`]).
pub fn run_state<H: Host + ?Sized>(state: u16, game: &mut H) {
    let part = current(game);
    match state {
        state::HEAD_IDLE => {
            let head = part;
            slot!(game, Head[head]; facing_right = 0, animation = animation::HEAD_IDLE_LEFT);
            if !player_left(game) {
                slot!(game, Head[head]; animation = animation::HEAD_IDLE_RIGHT, facing_right = 1);
            }
            animate(game);
            let hand = current(game) + 2 * SLOT;
            if is_negative(slot!(game, Hand[hand].cooldown)) {
                slot!(game, Hand[hand].cooldown = 0);
                let k = word!(game.cur_enemy_index);
                let range = slot!(byte game, Head[head].parameters[THROW_RANGE_LANE]) as u16;
                if game.player_within(k, Axis::X, range) {
                    slot!(game, Head[head].animation = animation::HEAD_SUBMERGE_LEFT);
                    if slot!(game, Head[head].facing_right) != 0 {
                        slot!(game, Head[head].animation = animation::HEAD_SUBMERGE_RIGHT);
                    }
                    animate(game);
                    slot!(game, Head[head].state = state::HEAD_WAIT_SUBMERGED);
                }
            }
        }
        state::HEAD_WAIT_SUBMERGED => {
            let head = part;
            if slot!(game, Head[head].animation_active) == 0 {
                slot!(game, Head[head].waiting_for_arm = 1);
                slot!(game, Arm[current(game) + SLOT].hold_retracted = 0);
                slot!(game, Head[head].state = state::HEAD_EMERGE_WHEN_READY);
            }
        }
        state::HEAD_EMERGE_WHEN_READY => {
            let head = part;
            if slot!(game, Head[head].waiting_for_arm) == 0 {
                face_samus(
                    head,
                    animation::HEAD_EMERGE_LEFT,
                    animation::HEAD_EMERGE_RIGHT,
                    game,
                );
                slot!(game, Head[head].state = state::HEAD_WAIT_EMERGED);
            }
        }
        state::HEAD_WAIT_EMERGED => {
            let head = part;
            if slot!(game, Head[head].animation_active) == 0 {
                face_samus(
                    head,
                    animation::HEAD_IDLE_LEFT,
                    animation::HEAD_IDLE_RIGHT,
                    game,
                );
                slot!(game, Head[head].state = state::HEAD_IDLE);
            }
        }
        state::ARM_WAIT_HEAD_SUBMERGED => {
            let arm = part;
            if slot!(game, Arm[arm].hold_retracted) == 0 {
                slot!(game, Arm[arm]; state = state::ARM_GROWING, extension = 0,
                    hold_retracted = 0, growth_level = 2);
                slot!(game, Arm[arm].base.y_pos = slot!(game, Arm[arm].home_y));
            }
        }
        state::HAND_IDLE => {
            let head = current(game) - 2 * SLOT;
            if slot!(game, Head[head].state) == state::HEAD_WAIT_SUBMERGED {
                slot!(game, Hand[current(game)].state = state::HAND_GROW_AND_THROW);
            }
        }
        state::HAND_THROW_AND_SHRINK => {
            let hand = part;
            if slot!(game, Hand[hand].animation_active) == 0 {
                slot!(game, Hand[hand].animation = animation::HAND_STUB);
                animate(game);
                let at = crate::ram_slot!(@at Arm, current(game) - SLOT, state);
                game.ram().copy_in(at, &state::ARM_SHRINKING.to_ne_bytes());
                slot!(game, Hand[hand].state = state::HAND_FOLLOW_SHRINKING_ARM);
                let at = crate::ram_slot!(@at Arm, current(game) - SLOT, fully_grown);
                game.ram().copy_in(at, &0u16.to_ne_bytes());
                slot!(
                    game,
                    Hand[hand].base.x_pos = slot!(game, Hand[hand].throw_x)
                );
                slot!(
                    game,
                    Hand[hand].base.y_pos = slot!(game, Hand[hand].throw_y)
                );
            }
            stretch_head_hitbox(word!(game.cur_enemy_index), game);
        }
        state::HAND_GROW_AND_THROW => {
            if slot!(game, Arm[current(game) - SLOT].fully_grown) != 0 {
                let hand = current(game);
                slot!(game, Hand[hand]; animation = animation::HAND_THROW_LEFT, throw_right = 0);
                if !player_left(game) {
                    slot!(game, Hand[hand]; animation = animation::HAND_THROW_RIGHT, throw_right = 1);
                }
                animate(game);
                slot!(game, Hand[hand]; state = state::HAND_THROW_AND_SHRINK,
                    throw_x = slot!(game, Hand[hand].base.x_pos),
                    throw_y = slot!(game, Hand[hand].base.y_pos));
            } else {
                let arm_y = slot!(game, Arm[current(game) - SLOT].base.y_pos);
                let offset = game.magdollite_arm(ARM_Y_OFFSET_ADDRESS, {
                    (slot!(game, Arm[current(game) - SLOT].growth_level) >> 1) as isize
                });
                slot!(
                    game,
                    Hand[current(game)].base.y_pos = arm_y.wrapping_sub(offset)
                );
            }
            stretch_head_hitbox(word!(game.cur_enemy_index), game);
        }
        state::HAND_FOLLOW_SHRINKING_ARM => {
            // The head's state, the arm's y and its growth level, by their
            // original RAM names (see `HEAD_STATE`).
            let index = word!(game.cur_enemy_index) as usize >> 1;
            if word!(game.enemy_drawing_queue[index + HEAD_STATE]) == state::HEAD_IDLE {
                slot!(game, Hand[current(game)].state = state::HAND_IDLE);
            } else {
                let arm_y = word!(game.enemy_drawing_queue[index + ARM_Y_POS]);
                let offset = game.magdollite_arm(
                    ARM_Y_OFFSET_ADDRESS,
                    (word!(game.enemy_drawing_queue_sizes[index + ARM_GROWTH_LEVEL]) >> 1) as isize,
                );
                slot!(
                    game,
                    Hand[current(game)].base.y_pos = arm_y.wrapping_sub(offset)
                );
            }
            stretch_head_hitbox(word!(game.cur_enemy_index), game);
        }
        _ => unreachable!("Magdollite state {state:04x}"),
    }
}

/// Run one of the head's or hand's script instructions (one of [`instruction`]).
pub fn instruction<H: Host + ?Sized>(
    address: u32,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    use instruction::*;
    if address == QUEUE_SOUND_IF_ON_SCREEN {
        if !game.current_outside_screen() {
            game.queue_sound(1, script.word(0) as u8, 6);
        }
        return script.offset(1);
    }
    let part = current(game);
    match address {
        HEAD_DOWN_2PX | HEAD_UP_2PX | HEAD_AND_ARM_UP_1PX | HEAD_AND_ARM_DOWN_1PX => {
            let step = if matches!(address, HEAD_DOWN_2PX | HEAD_AND_ARM_DOWN_1PX) {
                1
            } else {
                u16::MAX
            };
            let head = part;
            slot!(game, Head[head].base.y_pos, |old| old.wrapping_add(step));
            // The 2px moves step the head twice; the 1px moves step the arm too.
            let second = if matches!(address, HEAD_DOWN_2PX | HEAD_UP_2PX) {
                head
            } else {
                current(game) + SLOT
            };
            slot!(game, Part[second].base.y_pos, |old| old.wrapping_add(step));
        }
        SET_ANIMATION_ACTIVE | CLEAR_ANIMATION_ACTIVE => {
            slot!(
                game,
                Part[part].animation_active = u16::from(address == SET_ANIMATION_ACTIVE)
            );
        }
        SUBMERGED_POSITION_SHOW_ARM | EMERGED_POSITION_HIDE_ARM => {
            let head = part;
            let hide = address == EMERGED_POSITION_HIDE_ARM;
            let depth = if hide { EMERGED_DEPTH } else { SUBMERGED_DEPTH };
            let y = slot!(game, Head[head].home_y).wrapping_add(depth);
            slot!(game, Head[head].base.y_pos = y);
            let invisible = K_ENEMY_PROPS_INVISIBLE as u16;
            let show_or_hide = |old: u16| {
                if hide {
                    old | invisible
                } else {
                    old & !invisible
                }
            };
            let arm = current(game) + SLOT;
            slot!(game, Arm[arm].base.y_pos = y);
            slot!(game, Arm[arm].base.properties, |old| show_or_hide(old));
            let hand = current(game) + 2 * SLOT;
            slot!(game, Hand[hand].base.properties, |old| show_or_hide(old));
        }
        RESET_POSITION => {
            let head = part;
            let y = slot!(game, Head[head].home_y);
            slot!(game, Head[head].base.y_pos = y);
            slot!(game, Arm[current(game) + SLOT].base.y_pos = y);
        }
        SPAWN_LAVA => {
            let hand = part;
            game.spawn_projectile(
                slot!(game, Hand[hand].throw_right),
                word!(game.cur_enemy_index),
                crate::constants::bank_86::ADDR_K_EPROJ_LAVA_THROWN_BY_MAGDOLLITE as u16,
            );
        }
        OPEN_HAND_RIGHT | OPEN_HAND_LEFT | CLOSED_HAND_RIGHT | CLOSED_HAND_LEFT => {
            let hand = part;
            let right = matches!(address, OPEN_HAND_RIGHT | CLOSED_HAND_RIGHT);
            let reach = if right {
                HAND_REACH
            } else {
                HAND_REACH.wrapping_neg()
            };
            slot!(
                game,
                Hand[hand].base.x_pos = slot!(game, Hand[hand].throw_x).wrapping_add(reach)
            );
            // Only the right closed hand rises further, as in the original.
            let raise = if address == CLOSED_HAND_RIGHT {
                CLOSED_HAND_RAISE
            } else {
                OPEN_HAND_RAISE
            };
            slot!(
                game,
                Hand[hand].base.y_pos = slot!(game, Hand[hand].throw_y).wrapping_sub(raise)
            );
        }
        RESET_COOLDOWN => {
            slot!(game, Hand[part].cooldown = THROW_COOLDOWN);
        }
        _ => unreachable!("Magdollite instruction {address:06x}"),
    }
    script
}
