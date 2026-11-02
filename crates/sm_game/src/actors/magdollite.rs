//! Magdollite, Norfair's lava creature: a head that surfaces from the lava, an
//! arm that grows towards Samus and a hand that throws lava at her.
//!
//! Was `NorfairLavaMan` in the original decompilation (bank $A8:AC9C-$A8:B40C).
//! The three parts occupy consecutive enemy slots, head first. Each part reads
//! and writes its neighbours' slot fields; the view below names them.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Axis, BossEffects, Death, Shot};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::constants::enemy_properties::{K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE};
use crate::enemy_script::EnemyScript;
use crate::ram::word;
use sm_data::enemy_speeds::SpeedTable;
use sm_data::magdollite::{ARM_GROWTH_ADDRESS, ARM_INSTRUCTION_ADDRESS, ARM_Y_OFFSET_ADDRESS};

/// A lease on a game with Magdollite's effects.
pub trait Lease: BossLease<Game: Host + BossEffects> + SlotState {}
impl<T: BossLease<Game: Host + BossEffects> + SlotState + ?Sized> Lease for T {}

/// What Magdollite needs beyond the shared actor operations: its tables.
pub trait Host {
    /// Colours 9-12 of the glow palette whose colour 0 is word `source`.
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4];
    /// Arm step `index`'s field at `address` (growth, instruction list or y offset).
    fn magdollite_arm(&self, address: u32, index: isize) -> u16;
    /// The original diagnostic for an undefined lava height.
    fn warn_lava_height(&mut self, value: i32);
}

/// The size of one enemy spawn record (`EnemySpawnData`).
pub const SPAWN_DATA_BYTES: usize = 64;

/// Bytes between consecutive enemy slots: head, arm, hand.
const SLOT: u16 = 64;

slot_view! {
    /// A Magdollite part's slot. Every part has its shown and requested
    /// animation, state, kind, parameters (the head's throw range in the low
    /// byte, the speed entry in the high byte), animation flag, home Y,
    /// cooldown and the shrink and grow speeds. The head (which surfaces, idles
    /// facing Samus and submerges) also has its facing and its wait for the arm;
    /// the arm (which grows one segment at a time up to Samus's height) its
    /// growth level, extension and grown and retracted flags; the hand (which
    /// rides the arm's tip and throws lava) its throw side, home X and throw
    /// position.
    Magdollite, MagdolliteEdit {
        record: [
            growth_level = var_b, throw_right = var_b, shown_animation = var_c,
            animation = var_d, extension = var_e, state = var_f, kind = parameter_1,
            parameters = parameter_2,
        ],
        extended: [
            facing_right = var_00, animation_active = var_01, waiting_for_arm = var_02,
            home_y = var_03, cooldown = var_04, shrink_subspeed = var_07, shrink_speed = var_08,
            grow_subspeed = var_09, grow_speed = var_0a, fully_grown = var_0c,
            hold_retracted = var_0d, home_x = var_11, throw_x = var_12, throw_y = var_13,
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

/// The current enemy's slot, read afresh at each use as the original did.
fn current<L: Lease>(game: &L) -> u16 {
    game.current_enemy()
}
fn is_negative(value: u16) -> bool {
    (value as i16) < 0
}
/// Whether the low 16 bits of `value` read as negative, as the original's word test did.
fn negative_16(value: i32) -> bool {
    (value as i16) < 0
}
fn player_left<L: Lease>(game: &mut L) -> bool {
    let k = game.current_enemy();
    is_negative(game.player_delta(k, Axis::X))
}
/// Arm step `index`'s field at `address`, from the game's table.
fn arm_step<L: Lease>(game: &mut L, address: u32, index: isize) -> u16 {
    game.foreign(|game| game.magdollite_arm(address, index))
}
/// Start the part's requested animation if it isn't the one showing.
fn animate<L: Lease>(game: &mut L) {
    let part = current(game);
    let view = Magdollite::of(game, part);
    let requested = view.animation();
    if requested != view.shown_animation() {
        let mut edit = MagdolliteEdit::of(game, part);
        edit.actor().instruction = requested;
        *edit.shown_animation() = requested;
        let actor = edit.actor();
        actor.instruction_delay = 1;
        actor.timer = 0;
    }
}
fn initialize_head<L: Lease>(game: &mut L) {
    let head = current(game);
    {
        let mut edit = MagdolliteEdit::of(game, head);
        *edit.shown_animation() = 0;
        *edit.facing_right() = 0;
        *edit.waiting_for_arm() = 0;
        let copied = edit.actor().y.whole;
        *edit.home_y() = copied;
    }
    // Set from the opposite test; the first idle frame replaces it before use.
    if player_left(game) {
        *MagdolliteEdit::of(game, head).facing_right() = 1;
    }
    {
        let mut edit = MagdolliteEdit::of(game, head);
        *edit.animation() = animation::HEAD_IDLE_LEFT;
        if *edit.facing_right() == 0 {
            *edit.animation() = animation::HEAD_IDLE_RIGHT;
        }
    }
    animate(game);
    *MagdolliteEdit::of(game, head).state() = state::HEAD_IDLE;
}
fn initialize_arm<L: Lease>(game: &mut L) {
    let arm = current(game);
    {
        let mut edit = MagdolliteEdit::of(game, arm);
        let copied = edit.actor().y.whole;
        *edit.home_y() = copied;
        *edit.shown_animation() = 0;
        *edit.fully_grown() = 0;
        *edit.hold_retracted() = 1;
        *edit.animation() = animation::ARM_GROWTH_0;
    }
    animate(game);
    let mut edit = MagdolliteEdit::of(game, arm);
    let actor = edit.actor();
    actor.y.whole = actor.y.whole.wrapping_add(START_DEPTH);
    *edit.state() = state::ARM_WAIT_HEAD_SUBMERGED;
    edit.actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
}
fn initialize_hand<L: Lease>(game: &mut L) {
    let hand = current(game);
    {
        let mut edit = MagdolliteEdit::of(game, hand);
        let copied = edit.actor().y.whole;
        *edit.home_y() = copied;
        let copied = edit.actor().x.whole;
        *edit.home_x() = copied;
        *edit.shown_animation() = 0;
        *edit.cooldown() = 0;
        *edit.animation() = animation::HAND_STUB;
    }
    animate(game);
    let mut edit = MagdolliteEdit::of(game, hand);
    let actor = edit.actor();
    actor.y.whole = actor.y.whole.wrapping_add(START_DEPTH);
    *edit.state() = state::HAND_IDLE;
    edit.actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
}
/// Initialisation AI ($A8:AF8B): set up the part `kind` names, then its speeds
/// and the shared palette cycle.
pub fn initialize<L: Lease>(game: &mut L) {
    let kind = 2u16.wrapping_mul(Magdollite::of(game, current(game)).kind()) >> 1;
    let initialize: [fn(&mut L); 3] = [initialize_head, initialize_arm, initialize_hand];
    initialize[kind as usize](game);
    let part = current(game);
    // The speed entry is the parameters' high byte.
    let speed =
        SPEED_ENTRY_WORDS * isize::from(Magdollite::of(game, part).parameters().to_le_bytes()[1]);
    let shrink_speed = game.enemy_speed(SpeedTable::Linear, speed);
    let shrink_subspeed = game.enemy_speed(SpeedTable::Linear, speed + 1);
    let grow_speed = game.enemy_speed(SpeedTable::Linear, speed + 2);
    let grow_subspeed = game.enemy_speed(SpeedTable::Linear, speed + 3);
    {
        let mut edit = MagdolliteEdit::of(game, part);
        *edit.shrink_speed() = shrink_speed;
        *edit.shrink_subspeed() = shrink_subspeed;
        *edit.grow_speed() = grow_speed;
        *edit.grow_subspeed() = grow_subspeed;
    }
    let part = current(game);
    game.set_slot_word(GFX_DRAWN_HOOK, PALETTE_CYCLE_HOOK);
    game.set_slot_word(GFX_DRAWN_HOOK + 2, AI_BANK);
    let palette = Magdollite::of(game, part).actor().palette;
    word!(
        game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_PALETTE] =
            ((palette & SPRITE_PALETTE_BITS) >> 4) + SPRITE_PALETTES_BYTE
    );
    word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_TIMER] = PALETTE_CYCLE_FRAMES);
    word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_FRAME] = 0);
}
/// Graphics drawn hook ($A8:B0B2): every 8 frames, show the next glow palette.
pub fn cycle_palette<L: Lease>(game: &mut L) {
    if word!(game.DOOR_TRANSITION_FLAG_ENEMIES) != 0
        || word!(
            game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_TIMER] -= 1,
            reread
        ) != 0
    {
        return;
    }
    word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_TIMER] = PALETTE_CYCLE_FRAMES);
    word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_FRAME] += 1);
    let frame = word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_FRAME]);
    let source = COLORS_PER_PALETTE * (frame & (GLOW_PALETTES - 1)) as isize;
    let colors: [u16; 4] = game.foreign(|game| game.magdollite_glow_colors(source));
    let target = word!(game.VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[HOOK_PALETTE]) as usize >> 1;
    for (i, color) in colors.into_iter().enumerate() {
        word!(game.PALETTE_BUFFER[target + FIRST_GLOW_COLOR + i] = color);
    }
}
/// Main AI ($A8:B10A): count down the part's cooldown and run its state
/// function.
pub fn main_ai<L: Lease>(game: &mut L) {
    crate::probe::hit("magdollite.main_ai");
    let part = current(game);
    {
        let mut edit = MagdolliteEdit::of(game, part);
        let cooldown = edit.cooldown();
        *cooldown = cooldown.wrapping_sub(1);
    }
    match Magdollite::of(game, part).state() {
        state::ARM_GROWN => {}
        state::ARM_SHRINKING => shrink_arm(game),
        state::ARM_GROWING => {
            let (actor, diagnostic) = grow_arm(game);
            if let Some(value) = diagnostic {
                game.foreign(|game| game.warn_lava_height(value));
            }
            finish_arm_growth(actor, game);
        }
        function @ (state::HEAD_IDLE
        | state::HEAD_WAIT_SUBMERGED
        | state::HEAD_EMERGE_WHEN_READY
        | state::HEAD_WAIT_EMERGED
        | state::ARM_WAIT_HEAD_SUBMERGED
        | state::HAND_IDLE
        | state::HAND_THROW_AND_SHRINK
        | state::HAND_GROW_AND_THROW
        | state::HAND_FOLLOW_SHRINKING_ARM) => run_state(function, game),
        _ => game.foreign(|game| game.unreachable()),
    }
}
/// The head's reaction to a power bomb, contact or shot, shared with the arm and hand.
pub fn damage<L: Lease>(address: u32, game: &mut L) {
    match address {
        reaction::POWER_BOMB => game.power_bomb_reaction(Death::Finalize),
        reaction::CONTACT => game.contact_reaction(Death::Finalize),
        reaction::SHOT => game.shot_reaction(Shot::Finalize),
        _ => unreachable!("Magdollite reaction {address:06x}"),
    }
    let head = current(game);
    let arm = current(game) + SLOT;
    let hand = current(game) + 2 * SLOT;
    if Magdollite::of(game, head).actor().health == 0 {
        MagdolliteEdit::of(game, arm).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
        MagdolliteEdit::of(game, hand).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
    }
    let frozen = Magdollite::of(game, head).actor().frozen;
    if frozen != 0 {
        for part in [arm, hand] {
            let mut edit = MagdolliteEdit::of(game, part);
            let actor = edit.actor();
            actor.frozen = frozen;
            actor.ai_handlers |= FROZEN_AI;
        }
    }
}
/// Face the head towards Samus with the left or right animation.
fn face_samus<L: Lease>(head: u16, left: u16, right: u16, game: &mut L) {
    *MagdolliteEdit::of(game, head).animation() = left;
    if !player_left(game) {
        *MagdolliteEdit::of(game, head).animation() = right;
    }
    animate(game);
}
/// Stretch the head's hitbox up to the hand.
fn stretch_head_hitbox<L: Lease>(hand: u16, game: &mut L) {
    let head = hand - 2 * SLOT;
    let height = Magdollite::of(game, head)
        .actor()
        .y
        .whole
        .wrapping_sub(Magdollite::of(game, hand).actor().y.whole)
        .wrapping_add(HITBOX_MARGIN);
    let mut edit = MagdolliteEdit::of(game, head);
    edit.actor().radius.y = height;
    if is_negative(height.wrapping_sub(MIN_HITBOX_HEIGHT)) {
        edit.actor().radius.y = MIN_HITBOX_HEIGHT;
    }
}
/// Growing arm ($A8:B204), part one: move up at the head's grow speed. Returns
/// the arm and, when it has reached the warning extension, that extension.
pub fn grow_arm<L: Lease>(game: &mut L) -> (u16, Option<i32>) {
    crate::probe::hit("magdollite.grow_arm");
    let arm = current(game);
    let head = current(game) - SLOT;
    let value = u32::from(Magdollite::of(game, head).grow_subspeed())
        + u32::from(Magdollite::of(game, arm).actor().y.fraction);
    let grow_speed = u32::from(Magdollite::of(game, head).grow_speed());
    {
        let mut edit = MagdolliteEdit::of(game, arm);
        let y = &mut edit.actor().y;
        y.fraction = value as u16;
        y.whole = (u32::from(y.whole) + grow_speed + (value >> 16)) as u16;
        let extension = edit.extension();
        *extension = (u32::from(*extension) + grow_speed + (value >> 16)) as u16;
    }
    let k = game.current_enemy();
    let extension = Magdollite::of(game, k).extension();
    let warning = if !negative_16(-i32::from(extension) - ARM_WARNING_EXTENSION) {
        Some(-i32::from(extension as i16))
    } else {
        None
    };
    (k, warning)
}
/// Growing arm, part two: once at Samus's height (or the warning extension),
/// stop and let the hand throw; otherwise add a segment at each growth level.
pub fn finish_arm_growth<L: Lease>(k: u16, game: &mut L) {
    let arm = k;
    let level = Magdollite::of(game, arm).growth_level();
    let reach = |game: &mut L| -i32::from(Magdollite::of(game, arm).extension());
    let mut step = 0isize;
    let at_samus = !negative_16(reach(game) - ARM_WARNING_EXTENSION) || {
        step = (level >> 1) as isize;
        let y = i32::from(Magdollite::of(game, arm).actor().y.whole);
        ((y - i32::from(arm_step(game, ARM_Y_OFFSET_ADDRESS, step)) - i32::from(game.player_y()))
            as i16)
            < 0
    };
    let next_level = if at_samus {
        {
            let mut edit = MagdolliteEdit::of(game, arm);
            *edit.state() = state::ARM_GROWN;
            *edit.fully_grown() = 1;
        }
        if ((reach(game) - arm_step(game, ARM_GROWTH_ADDRESS, (level >> 1) as isize) as i32) as i16)
            < 0
        {
            return;
        }
        true
    } else {
        (reach(game) - arm_step(game, ARM_GROWTH_ADDRESS, step) as i32) as i16 >= 0
    };
    if next_level {
        let level = {
            let mut edit = MagdolliteEdit::of(game, arm);
            let level = edit.growth_level();
            *level = level.wrapping_add(2);
            let y = &mut edit.actor().y.whole;
            *y = y.wrapping_add(SEGMENT_PIXELS);
            *edit.growth_level()
        };
        let animation = arm_step(game, ARM_INSTRUCTION_ADDRESS, (level >> 1) as isize);
        *MagdolliteEdit::of(game, arm).animation() = animation;
        animate(game);
    }
}
/// Shrinking arm ($A8:B295): move down at the head's shrink speed, removing a
/// segment at each growth level, until retracted; then release the head.
pub fn shrink_arm<L: Lease>(game: &mut L) {
    let arm = current(game);
    let head = current(game) - SLOT;
    let subspeed = Magdollite::of(game, head).shrink_subspeed();
    let (subposition, carry) =
        subspeed.overflowing_add(Magdollite::of(game, arm).actor().y.fraction);
    let speed = Magdollite::of(game, head).shrink_speed();
    {
        let mut edit = MagdolliteEdit::of(game, arm);
        let y = &mut edit.actor().y;
        y.fraction = subposition;
        y.whole = y.whole.wrapping_add(speed).wrapping_add(u16::from(carry));
        let extension = edit.extension();
        *extension = extension.wrapping_add(speed).wrapping_add(u16::from(carry));
    }
    let k = game.current_enemy();
    let arm = k;
    let extension = Magdollite::of(game, arm).extension() as i16;
    if extension >= 0 {
        *MagdolliteEdit::of(game, arm).hold_retracted() = 1;
        *MagdolliteEdit::of(game, arm - SLOT).waiting_for_arm() = 0;
        *MagdolliteEdit::of(game, arm).state() = state::ARM_WAIT_HEAD_SUBMERGED;
        return;
    }
    let previous = (Magdollite::of(game, arm).growth_level().wrapping_sub(2) >> 1) as isize;
    if ((-i32::from(extension) - i32::from(arm_step(game, ARM_GROWTH_ADDRESS, previous))) as i16)
        < 0
    {
        let level = {
            let mut edit = MagdolliteEdit::of(game, arm);
            let level = edit.growth_level();
            *level = level.wrapping_sub(2);
            let y = &mut edit.actor().y.whole;
            *y = y.wrapping_sub(SEGMENT_PIXELS);
            *edit.growth_level()
        };
        let animation = arm_step(game, ARM_INSTRUCTION_ADDRESS, (level >> 1) as isize);
        *MagdolliteEdit::of(game, arm).animation() = animation;
        animate(game);
    }
}
/// Set the hand's Y to the arm's tip: the arm's Y less its growth level's offset.
fn follow_arm<L: Lease>(game: &mut L) {
    let arm = Magdollite::of(game, current(game) - SLOT);
    let (arm_y, level) = (arm.actor().y.whole, arm.growth_level());
    let offset = arm_step(game, ARM_Y_OFFSET_ADDRESS, (level >> 1) as isize);
    let hand = current(game);
    MagdolliteEdit::of(game, hand).actor().y.whole = arm_y.wrapping_sub(offset);
}
/// Run the current part's state function `state` (one of [`state`]).
pub fn run_state<L: Lease>(state: u16, game: &mut L) {
    let part = current(game);
    match state {
        state::HEAD_IDLE => {
            let head = part;
            {
                let mut edit = MagdolliteEdit::of(game, head);
                *edit.facing_right() = 0;
                *edit.animation() = animation::HEAD_IDLE_LEFT;
            }
            if !player_left(game) {
                let mut edit = MagdolliteEdit::of(game, head);
                *edit.animation() = animation::HEAD_IDLE_RIGHT;
                *edit.facing_right() = 1;
            }
            animate(game);
            let hand = current(game) + 2 * SLOT;
            if is_negative(Magdollite::of(game, hand).cooldown()) {
                *MagdolliteEdit::of(game, hand).cooldown() = 0;
                let k = game.current_enemy();
                // The throw range is the parameters' low byte.
                let range = Magdollite::of(game, head).parameters() & 0xff;
                if game.player_within(k, Axis::X, range) {
                    {
                        let mut edit = MagdolliteEdit::of(game, head);
                        *edit.animation() = animation::HEAD_SUBMERGE_LEFT;
                        if *edit.facing_right() != 0 {
                            *edit.animation() = animation::HEAD_SUBMERGE_RIGHT;
                        }
                    }
                    animate(game);
                    *MagdolliteEdit::of(game, head).state() = state::HEAD_WAIT_SUBMERGED;
                }
            }
        }
        state::HEAD_WAIT_SUBMERGED => {
            let head = part;
            if Magdollite::of(game, head).animation_active() == 0 {
                *MagdolliteEdit::of(game, head).waiting_for_arm() = 1;
                let arm = current(game) + SLOT;
                *MagdolliteEdit::of(game, arm).hold_retracted() = 0;
                *MagdolliteEdit::of(game, head).state() = state::HEAD_EMERGE_WHEN_READY;
            }
        }
        state::HEAD_EMERGE_WHEN_READY => {
            let head = part;
            if Magdollite::of(game, head).waiting_for_arm() == 0 {
                face_samus(
                    head,
                    animation::HEAD_EMERGE_LEFT,
                    animation::HEAD_EMERGE_RIGHT,
                    game,
                );
                *MagdolliteEdit::of(game, head).state() = state::HEAD_WAIT_EMERGED;
            }
        }
        state::HEAD_WAIT_EMERGED => {
            let head = part;
            if Magdollite::of(game, head).animation_active() == 0 {
                face_samus(
                    head,
                    animation::HEAD_IDLE_LEFT,
                    animation::HEAD_IDLE_RIGHT,
                    game,
                );
                *MagdolliteEdit::of(game, head).state() = state::HEAD_IDLE;
            }
        }
        state::ARM_WAIT_HEAD_SUBMERGED => {
            let arm = part;
            if Magdollite::of(game, arm).hold_retracted() == 0 {
                let mut edit = MagdolliteEdit::of(game, arm);
                *edit.state() = state::ARM_GROWING;
                *edit.extension() = 0;
                *edit.hold_retracted() = 0;
                *edit.growth_level() = 2;
                let copied = *edit.home_y();
                edit.actor().y.whole = copied;
            }
        }
        state::HAND_IDLE => {
            let head = current(game) - 2 * SLOT;
            if Magdollite::of(game, head).state() == state::HEAD_WAIT_SUBMERGED {
                let hand = current(game);
                *MagdolliteEdit::of(game, hand).state() = state::HAND_GROW_AND_THROW;
            }
        }
        state::HAND_THROW_AND_SHRINK => {
            let hand = part;
            if Magdollite::of(game, hand).animation_active() == 0 {
                *MagdolliteEdit::of(game, hand).animation() = animation::HAND_STUB;
                animate(game);
                let arm = current(game) - SLOT;
                *MagdolliteEdit::of(game, arm).state() = state::ARM_SHRINKING;
                *MagdolliteEdit::of(game, hand).state() = state::HAND_FOLLOW_SHRINKING_ARM;
                let arm = current(game) - SLOT;
                *MagdolliteEdit::of(game, arm).fully_grown() = 0;
                let mut edit = MagdolliteEdit::of(game, hand);
                let copied = *edit.throw_x();
                edit.actor().x.whole = copied;
                let copied = *edit.throw_y();
                edit.actor().y.whole = copied;
            }
            stretch_head_hitbox(game.current_enemy(), game);
        }
        state::HAND_GROW_AND_THROW => {
            if Magdollite::of(game, current(game) - SLOT).fully_grown() != 0 {
                let hand = current(game);
                {
                    let mut edit = MagdolliteEdit::of(game, hand);
                    *edit.animation() = animation::HAND_THROW_LEFT;
                    *edit.throw_right() = 0;
                }
                if !player_left(game) {
                    let mut edit = MagdolliteEdit::of(game, hand);
                    *edit.animation() = animation::HAND_THROW_RIGHT;
                    *edit.throw_right() = 1;
                }
                animate(game);
                let mut edit = MagdolliteEdit::of(game, hand);
                *edit.state() = state::HAND_THROW_AND_SHRINK;
                let copied = edit.actor().x.whole;
                *edit.throw_x() = copied;
                let copied = edit.actor().y.whole;
                *edit.throw_y() = copied;
            } else {
                follow_arm(game);
            }
            stretch_head_hitbox(game.current_enemy(), game);
        }
        state::HAND_FOLLOW_SHRINKING_ARM => {
            let head = current(game) - 2 * SLOT;
            if Magdollite::of(game, head).state() == state::HEAD_IDLE {
                let hand = current(game);
                *MagdolliteEdit::of(game, hand).state() = state::HAND_IDLE;
            } else {
                follow_arm(game);
            }
            stretch_head_hitbox(game.current_enemy(), game);
        }
        _ => unreachable!("Magdollite state {state:04x}"),
    }
}

/// Run one of the head's or hand's script instructions (one of [`instruction`]).
pub fn instruction<L: Lease>(address: u32, script: EnemyScript, game: &mut L) -> EnemyScript {
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
            // The 2px moves step the head twice; the 1px moves step the arm too.
            let second = if matches!(address, HEAD_DOWN_2PX | HEAD_UP_2PX) {
                head
            } else {
                current(game) + SLOT
            };
            for moved in [head, second] {
                let mut edit = MagdolliteEdit::of(game, moved);
                let y = &mut edit.actor().y.whole;
                *y = y.wrapping_add(step);
            }
        }
        SET_ANIMATION_ACTIVE | CLEAR_ANIMATION_ACTIVE => {
            *MagdolliteEdit::of(game, part).animation_active() =
                u16::from(address == SET_ANIMATION_ACTIVE);
        }
        SUBMERGED_POSITION_SHOW_ARM | EMERGED_POSITION_HIDE_ARM => {
            let head = part;
            let hide = address == EMERGED_POSITION_HIDE_ARM;
            let depth = if hide { EMERGED_DEPTH } else { SUBMERGED_DEPTH };
            let y = Magdollite::of(game, head).home_y().wrapping_add(depth);
            MagdolliteEdit::of(game, head).actor().y.whole = y;
            let invisible = K_ENEMY_PROPS_INVISIBLE as u16;
            let show_or_hide = |old: u16| {
                if hide {
                    old | invisible
                } else {
                    old & !invisible
                }
            };
            let arm = current(game) + SLOT;
            {
                let mut edit = MagdolliteEdit::of(game, arm);
                let actor = edit.actor();
                actor.y.whole = y;
                actor.properties = show_or_hide(actor.properties);
            }
            let hand = current(game) + 2 * SLOT;
            let mut edit = MagdolliteEdit::of(game, hand);
            let actor = edit.actor();
            actor.properties = show_or_hide(actor.properties);
        }
        RESET_POSITION => {
            let head = part;
            let y = Magdollite::of(game, head).home_y();
            MagdolliteEdit::of(game, head).actor().y.whole = y;
            let arm = current(game) + SLOT;
            MagdolliteEdit::of(game, arm).actor().y.whole = y;
        }
        SPAWN_LAVA => {
            let hand = part;
            game.spawn_projectile(
                Magdollite::of(game, hand).throw_right(),
                game.current_enemy(),
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
            // Only the right closed hand rises further, as in the original.
            let raise = if address == CLOSED_HAND_RIGHT {
                CLOSED_HAND_RAISE
            } else {
                OPEN_HAND_RAISE
            };
            let mut edit = MagdolliteEdit::of(game, hand);
            let copied = edit.throw_x().wrapping_add(reach);
            edit.actor().x.whole = copied;
            let copied = edit.throw_y().wrapping_sub(raise);
            edit.actor().y.whole = copied;
        }
        RESET_COOLDOWN => {
            *MagdolliteEdit::of(game, part).cooldown() = THROW_COOLDOWN;
        }
        _ => unreachable!("Magdollite instruction {address:06x}"),
    }
    script
}
