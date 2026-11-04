//! The blue Brinstar face block, which turns to look at Samus once she has the morph
//! ball, and glows (bank $A8:E7AC-$A8:E99A). Native hook words, actor words and
//! little-endian hook variables stay authoritative.
use crate::actors::boss_lease::BossState;
use crate::actors::host::{Axis, abs16};
use crate::actors::slot_view::{SlotState, slot_view};

/// What the face block needs: a boss's state, which holds its glow colours,
/// whose slots are the Session's.
pub trait Host: BossState + SlotState {}
impl<T: BossState + SlotState + ?Sized> Host for T {}

slot_view! {
    /// The face block's view of its slot.
    FaceBlock, FaceBlockEdit {
        record: [
            activated = var_a, direction_to_samus = var_b, range = parameter_1,
            direction_to_not_turn = parameter_2,
        ],
        extended: [],
    }
}

/// Bank $A8 face block instruction lists.
pub mod instruction_list {
    pub const TURNING_LEFT: u16 = 0xe80c;
    pub const TURNING_RIGHT: u16 = 0xe81a;
    pub const FACING_FORWARD: u16 = 0xe828;
}

/// The enemy graphics drawn hook ($7E:178C), a long pointer: the palette cycle
/// ($A8:E86E) once the morph ball is collected, else an RTL ($A8:804C).
const PALETTE_CYCLE_HOOK: u16 = 0xe86e;
const NO_HOOK: u16 = 0x804c;
const BANK: u16 = 0xa8;
/// The graphics drawn hook's variables: the palette's byte offset, the colour
/// set index and the cycle timer.
const HOOK_PALETTE: usize = 0;
const HOOK_COLOUR_SET: usize = 1;
const HOOK_TIMER: usize = 2;
const CYCLE_FRAMES: u16 = 16;
const COLOUR_SETS: u16 = 7;
const COLOURS: usize = 4;
/// Colours 9-12 of the enemy's palette start at colour 137 ($7E:C112).
const FIRST_GLOW_COLOUR: usize = 137;
/// Collected items bit 2: the morph ball.
const MORPH_BALL: u16 = 4;
const SIGN: u16 = 0x8000;

/// Store the drawn hook's address word.
fn set_hook(game: &mut (impl Host + ?Sized), hook: u16) {
    game.session_mut().actors.drawing.hook_address[..2].copy_from_slice(&hook.to_le_bytes());
}

/// Store one of the drawn hook's variables ($7E:1794).
fn set_hook_word(game: &mut (impl Host + ?Sized), index: usize, value: u16) {
    game.session_mut().actors.drawing.hook_work[index] = value;
    game.words_stored();
}
fn publish_palette(block: u16, game: &mut (impl Host + ?Sized)) {
    let palette = FaceBlock::of(game, block).actor().palette;
    set_hook_word(game, HOOK_PALETTE, (palette >> 4) & 0xff);
}
fn within_range(block: u16, delta: u16, game: &mut (impl Host + ?Sized)) -> bool {
    (abs16(delta) as i16).wrapping_sub(FaceBlock::of(game, block).range() as i16) < 0
}
/// Initialisation AI - blue Brinstar face block ($A8:E82E).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let block = game.current_enemy();
    FaceBlockEdit::of(game, block).actor().instruction = instruction_list::FACING_FORWARD;
    let hook: u16 = if game.session().inventory.equipment.owned & MORPH_BALL == 0 {
        NO_HOOK
    } else {
        PALETTE_CYCLE_HOOK
    };
    set_hook(game, hook);
    game.session_mut().actors.drawing.hook_address[2..].copy_from_slice(&BANK.to_le_bytes());
    publish_palette(block, game);
    set_hook_word(game, HOOK_TIMER, CYCLE_FRAMES);
    let mut edit = FaceBlockEdit::of(game, block);
    let old = *edit.direction_to_not_turn();
    *edit.direction_to_not_turn() = (old & 1) << 15;
}
/// Main AI - blue Brinstar face block ($A8:E8AE): once Samus is in range, turn
/// towards her, unless she is on the side it never turns to.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    if game.session().inventory.equipment.owned & MORPH_BALL == 0 {
        return;
    }
    set_hook(game, PALETTE_CYCLE_HOOK);
    let block = game.current_enemy();
    publish_palette(block, game);
    if FaceBlock::of(game, block).activated() != 0 {
        return;
    }
    let k = game.current_enemy();
    let dy = game.player_delta(k, Axis::Y);
    if !within_range(block, dy, game) {
        return;
    }
    let k = game.current_enemy();
    let dx = game.player_delta(k, Axis::X);
    *FaceBlockEdit::of(game, block).direction_to_samus() = dx;
    if within_range(block, dx, game)
        && FaceBlock::of(game, block).direction_to_samus() & SIGN
            != FaceBlock::of(game, block).direction_to_not_turn()
    {
        let instruction = if FaceBlock::of(game, block).direction_to_samus() & SIGN == 0 {
            instruction_list::TURNING_RIGHT
        } else {
            instruction_list::TURNING_LEFT
        };
        {
            let mut edit = FaceBlockEdit::of(game, block);
            edit.actor().instruction = instruction;
            edit.actor().instruction_delay = 1;
        }
        *FaceBlockEdit::of(game, block).activated() = 1;
        set_hook_word(game, HOOK_TIMER, CYCLE_FRAMES);
    }
}
/// Enemy graphics drawn hook - blue Brinstar face block - periodically cycle
/// between palettes ($A8:E86E): every 16 frames, show the next glow colours.
pub fn cycle_palette(game: &mut (impl Host + ?Sized)) {
    if game.session().room.entry.enemy_interlock != 0 {
        return;
    }
    let timer = game.session().actors.drawing.hook_work[HOOK_TIMER].wrapping_sub(1);
    set_hook_word(game, HOOK_TIMER, timer);
    if timer != 0 {
        return;
    }
    set_hook_word(game, HOOK_TIMER, CYCLE_FRAMES);
    let hook_work = game.session().actors.drawing.hook_work;
    let mut destination = hook_work[HOOK_PALETTE];
    let mut source = 8u16.wrapping_mul(hook_work[HOOK_COLOUR_SET]);
    for _ in 0..COLOURS {
        let color = crate::enemy_lists::face_block_glow_colour(game, (source >> 1) as isize);
        let at = FIRST_GLOW_COLOUR + usize::from(destination >> 1);
        game.session_mut().presentation.visual_mut().colors[at] = color;
        source = source.wrapping_add(2);
        destination = destination.wrapping_add(2);
    }
    // The legacy phase increment reads physical byte zero, even on a big-endian host.
    let phase = game.session().actors.drawing.hook_work[HOOK_COLOUR_SET].to_le_bytes()[0];
    set_hook_word(game, HOOK_COLOUR_SET, (u16::from(phase) + 1) & COLOUR_SETS);
}
