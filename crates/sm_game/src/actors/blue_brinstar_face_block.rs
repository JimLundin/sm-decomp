//! The blue Brinstar face block, which turns to look at Samus once she has the
//! morph ball, and glows (bank $A8:E7AC-$A8:E99A). Was `blue_face_block`.
//! Native hook words, actor words and little-endian hook variables stay authoritative.
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Axis, abs16};
use crate::actors::layouts::part_view;
use crate::inventory::{self, Word};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use crate::records::Records;

/// What the face block needs: the shared actor operations and its glow colours.
pub trait Host: ActorHost + ActorRuntime + Records {}
impl<T: ActorHost + ActorRuntime + Records + ?Sized> Host for T {}

part_view! {
    /// The face block's view of its slot.
    FaceBlock: EnemyLayout8 {
        words: [activated, direction_to_samus, var_c, var_d, var_e, var_f, range, direction_to_not_turn],
    }
}

/// Bank $A8 face block instruction lists.
pub mod instruction_list {
    pub const TURNING_LEFT: u16 = 0xe80c;
    pub const TURNING_RIGHT: u16 = 0xe81a;
    pub const FACING_FORWARD: u16 = 0xe828;
}

/// $7E:178C: the enemy graphics drawn hook, a long pointer: the palette cycle
/// ($A8:E86E) once the morph ball is collected, else an RTL ($A8:804C).
const GFX_DRAWN_HOOK: usize = 0x178c;
const GFX_DRAWN_HOOK_BANK: usize = 0x178e;
const PALETTE_CYCLE_HOOK: u16 = 0xe86e;
const NO_HOOK: u16 = 0x804c;
const BANK: u16 = 0xa8;
/// The graphics drawn hook's variables: the palette's byte offset, the colour
/// set index and the cycle timer.
const HOOK_PALETTE: usize = 0;
const HOOK_COLOUR_SET: usize = 1;
const HOOK_TIMER: usize = 2;
/// $7E:1796, the colour set index's low byte.
const HOOK_COLOUR_SET_LOW: usize = 0x1796;
const CYCLE_FRAMES: u16 = 16;
const COLOUR_SETS: u16 = 7;
const COLOURS: usize = 4;
/// Colours 9-12 of the enemy's palette start at $7E:C112.
const FIRST_GLOW_COLOUR: usize = 0xc112;
/// Collected items bit 2: the morph ball.
const MORPH_BALL: u16 = 4;
const SIGN: u16 = 0x8000;

fn publish_palette(block: i32, game: &mut (impl Host + ?Sized)) {
    let palette = slot!(game, FaceBlock[block].base.palette_index);
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_PALETTE] = (palette >> 4) & 0xff);
}
fn within_range(block: i32, delta: u16, game: &mut (impl Host + ?Sized)) -> bool {
    (abs16(delta) as i16).wrapping_sub(slot!(game, FaceBlock[block].range) as i16) < 0
}
/// Initialisation AI - blue Brinstar face block ($A8:E82E).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let block = word!(game.cur_enemy_index) as i32;
    slot!(
        game,
        FaceBlock[block].base.current_instruction = instruction_list::FACING_FORWARD
    );
    let hook: u16 = if inventory::get(game, Word::CollectedItems) & MORPH_BALL == 0 {
        NO_HOOK
    } else {
        PALETTE_CYCLE_HOOK
    };
    game.ram().copy_in(GFX_DRAWN_HOOK, &hook.to_ne_bytes());
    game.ram().copy_in(GFX_DRAWN_HOOK_BANK, &BANK.to_ne_bytes());
    publish_palette(block, game);
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] = CYCLE_FRAMES);
    slot!(game, FaceBlock[block].direction_to_not_turn, |old| (old
        & 1)
        << 15);
}
/// Main AI - blue Brinstar face block ($A8:E8AE): once Samus is in range,
/// turn towards her, unless she is on the side it never turns to. Was `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    if inventory::get(game, Word::CollectedItems) & MORPH_BALL == 0 {
        return;
    }
    game.ram()
        .copy_in(GFX_DRAWN_HOOK, &PALETTE_CYCLE_HOOK.to_ne_bytes());
    let block = word!(game.cur_enemy_index) as i32;
    publish_palette(block, game);
    if slot!(game, FaceBlock[block].activated) != 0 {
        return;
    }
    let k = word!(game.cur_enemy_index);
    let dy = game.player_delta(k, Axis::Y);
    if !within_range(block, dy, game) {
        return;
    }
    let k = word!(game.cur_enemy_index);
    let dx = game.player_delta(k, Axis::X);
    slot!(game, FaceBlock[block].direction_to_samus = dx);
    if within_range(block, dx, game)
        && slot!(game, FaceBlock[block].direction_to_samus) & SIGN
            != slot!(game, FaceBlock[block].direction_to_not_turn)
    {
        let instruction = if slot!(game, FaceBlock[block].direction_to_samus) & SIGN == 0 {
            instruction_list::TURNING_RIGHT
        } else {
            instruction_list::TURNING_LEFT
        };
        slot!(game, FaceBlock[block].base; current_instruction = instruction, instruction_timer = 1);
        slot!(game, FaceBlock[block].activated = 1);
        word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] = CYCLE_FRAMES);
    }
}
/// Enemy graphics drawn hook - blue Brinstar face block - periodically cycle
/// between palettes ($A8:E86E): every 16 frames, show the next glow colours.
/// Was `palette`.
pub fn cycle_palette(game: &mut (impl Host + ?Sized)) {
    if word!(game.door_transition_flag_enemies) != 0
        || word!(
            game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] -= 1,
            reread
        ) != 0
    {
        return;
    }
    word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_TIMER] = CYCLE_FRAMES);
    let mut destination = word!(game.variables_for_enemy_graphics_drawn_hook[HOOK_PALETTE]);
    let mut source = 8u16.wrapping_mul(word!(
        game.variables_for_enemy_graphics_drawn_hook[HOOK_COLOUR_SET]
    ));
    for _ in 0..COLOURS {
        let color = crate::enemy_lists::face_block_glow_colour(game, (source >> 1) as isize);
        let at = FIRST_GLOW_COLOUR + usize::from(destination >> 1) * 2;
        game.ram().copy_in(at, &color.to_ne_bytes());
        source = source.wrapping_add(2);
        destination = destination.wrapping_add(2);
    }
    // The legacy phase increment reads physical byte zero, even on a big-endian host.
    word!(
        game.variables_for_enemy_graphics_drawn_hook[HOOK_COLOUR_SET] =
            (u16::from(game.ram().read_byte(HOOK_COLOUR_SET_LOW)) + 1) & COLOUR_SETS
    );
}
